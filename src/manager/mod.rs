/*
 * Copyright 2016-2017 icasdri
 *
 * This file is part of stabping. The original source code for stabping can be
 * found at <https://github.com/icasdri/stabping>. See COPYING for licensing
 * details.
 */
mod manager_error;
mod index_file;
mod data_file;

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::fs::OpenOptions;
use std::fs::File;
use std::io::Write;
use std::io::BufReader;
use std::sync::{Mutex, RwLock, RwLockReadGuard};
use std::ops::Deref;
use std::iter;
use std::iter::Extend;

use augmented_file::{AugmentedFile, AugmentedFileError as AFE};
use data::AsBytes;
use data::DataElement;
use workers::{Kind, Options};

pub use manager_error::ManagerError;

use index_file::IndexFile;
use data_file::DataFile;

/**
 * Master control structure managing all I/O backed resources (with the
 * exception of running workers which is handled by `TargetKind` and the main
 * thread directly) of a given target.
 *
 * This is include most notably, the target's data file, address index (and
 * associated index file), and options (and associated options file).
 */
pub struct Manager {
    pub kind: &'static Kind,

    index_file: RwLock<IndexFile>,

    hourly_data_file: RwLock<DataFile>,
    daily_data_file: RwLock<DataFile>,
    weekly_data_file: RwLock<DataFile>,

    options_path: Mutex<PathBuf>,
    options: RwLock<Options>,
}

impl Manager {
    /**
     * Creates a new `TargetManager` for the given target kind that will store
     * persistent data at the given location path.
     */
    pub fn new<'b>(kind: &'static TargetKind, data_path: &'b Path) -> Result<Self, ManagerError> {
        let mut path = data_path.to_owned();

        // attempt to open the target's index file
        path.push(format!("{}.index.json", kind.name()));
        let mut index_file = try!(IndexFile::from_path(&path));
        path.pop();

        // attempt to open the target's data file
        path.push(format!("{}.data.dat", kind.name()));
        path.pop();

        // attempt to open the target's options file
        path.push(format!("{}.options.json", kind.name()));
        let path = path;  // last path is options file path (disallow muts)
        let mut options_file = try!(
            File::open_from(OpenOptions::new().read(true).write(true).create(true), &path)
            .map_err(|e| ME::OptionsFileIO(e))
        );

        // read back existing options from options file, or write out defaults
        let options = if try!(options_file.length_p(&path)
                              .map_err(|e| ME::OptionsFileIO(e))) > 0 {
            try!(
                options_file.read_json_p(&path)
                .map_err(|e| ME::OptionsFileIO(e))
            )
        } else {
            let (addr, interval) = kind.default_options_bootstrap();
            let addr_i = index_file.add_addr(addr);
            let default_options = Options {
                addrs: vec![addr_i],
                interval: interval
            };
            try!(
                options_file.write_json_p(&default_options, &path)
                .map_err(|e| ME::OptionsFileIO(e))
            );
            default_options
        };

        Ok(TargetManager {
            kind: kind,

            index_file: RwLock::new(index_file),

            options_path: Mutex::new(path),
            options: RwLock::new(options),
        })
    }

    /**
     * Acquires a read lock on this target's index.
     */
    pub fn index_read<'a>(&'a self) -> RwLockReadGuard<'a, AddrIndex> {
        self.index.read().unwrap()
    }

    /**
     * Acquires a read lock on this target's options.
     */
    pub fn options_read<'a>(&'a self) -> RwLockReadGuard<'a, TargetOptions> {
        self.options.read().unwrap()
    }

    /**
     * Attempts to update this target's options with the given new options.
     */
    pub fn options_update(&self, new_options: TargetOptions) -> Result<(), ManagerError> {
        let mut guard = self.options.write().unwrap();
        let mut options_path = self.options_path.lock().unwrap();
        *guard = new_options;
        try!(
            overwrite_json(&*guard, &*options_path)
            .map_err(|e| ManagerError::OptionsFileIO(e))
        );
        try!(self.index.write().unwrap().ensure_for_addrs(guard.addrs.iter()));
        println!("Updated {} options: {:?}", self.kind.compact_name(), *guard);
        Ok(())
    }

    /**
     * Acquires a read lock on this target's data file.
     */
    pub fn data_file_read<'a>(&'a self) -> RwLockReadGuard<'a, File> {
        self.data_file.read().unwrap()
    }

    /**
     * Appends the given live-collected data (`TargetResults`) to this target's
     * data file.
     */
    pub fn append_data(&self, data_res: &TargetResults) -> Result<(), ManagerError> {
        let ref in_data = data_res.0;

        assert!(in_data[0] == self.kind.kind_id());

        let nonce = in_data[1];
        if nonce != self.options_read().nonce {
            println!("Nonce mismatch for data append! Silently ignoring.");
            return Ok(());
        }

        let mut out_data: Vec<i32> = Vec::with_capacity((in_data.len() - 3) * 3);
        let time = in_data[2];
        let index = self.index.read().unwrap();
        for (addr, val) in self.options_read().addrs.iter().zip(in_data[3..].iter()) {
            out_data.push(time);
            out_data.push(index.get_index(addr).unwrap());
            out_data.push(*val);
        }

        let ref mut file = *self.data_file.write().unwrap();
        try!(file.write_all(&out_data.into_raw_bytes())
             .map_err(|_| ManagerError::DataFileIO(
                          SPIOError::Write(None))));
        Ok(())
    }

    /**
     * Gets the current addrs in options as (nonce, ordered_list, membership)
     * where 'ordered_list' is the list of address indices in order in which
     * they appear in options, and where 'membership' is the set of indices
     * present (i.e. if membership[i] != 0, then the addr with index i is
     * currently present in options).
     */
    pub fn get_current_indices(&self) -> (i32, Vec<i32>, Vec<i32>) {
        let options = self.options_read();

        let index = self.index.read().unwrap();

        let mut ordered_list = Vec::with_capacity(options.addrs.len());

        let mut membership = {
            let len = index.len();
            let mut v = Vec::with_capacity(len);
            v.extend(iter::repeat(0).take(len));
            v
        };

        for addr in options.addrs.iter() {
            let i = index.get_index(addr).unwrap();
            ordered_list.push(i);
            membership[i as usize] = SENTINEL_NODATA;
        }

        (options.nonce, ordered_list, membership)
    }
}

