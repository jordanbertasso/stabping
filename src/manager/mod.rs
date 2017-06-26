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

use augmented_file::{AugmentedFile, AugmentedFileError as AFE, overwrite_json};
use data::AsBytes;
use data::DataElement;
use workers::{Kind, Options};

pub use self::manager_error::ManagerError;
use self::ManagerError as ME;

use self::index_file::IndexFile;
use self::data_file::DataFile;

/**
 * Master control structure managing all I/O backed resources 
 *
 * This is include most notably, the target's associated data files, index file
 * and options (all backed by their respective files).
 */
pub struct Manager {
    pub kind: Kind,

    index_file: RwLock<IndexFile>,

    // raw_data_file: RwLock<DataFile>,
    // hourly_data_file: RwLock<DataFile>,
    // daily_data_file: RwLock<DataFile>,
    // weekly_data_file: RwLock<DataFile>,

    options_path: Mutex<PathBuf>,
    options: RwLock<Options>,
}

impl Manager {
    /**
     * Creates a new `Manager` for the given target kind that will store
     * persistent data at the given location path.
     */
    pub fn new<'b>(kind: Kind, data_path: &'b Path) -> Result<Self, ManagerError> {
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
            let addr_i = try!(index_file.add_addr(addr));
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

        Ok(Manager {
            kind: kind,

            index_file: RwLock::new(index_file),

            options_path: Mutex::new(path),
            options: RwLock::new(options),
        })
    }

    /**
     * Acquires a read lock on this target's index.
     */
    pub fn index_read<'a>(&'a self) -> RwLockReadGuard<'a, IndexFile> {
        self.index_file.read().unwrap()
    }

    /**
     * Acquires a read lock on this target's options.
     */
    pub fn options_read<'a>(&'a self) -> RwLockReadGuard<'a, Options> {
        self.options.read().unwrap()
    }

    /**
     * Attempts to update this target's options with the given new options.
     */
    pub fn options_update(&self, new_options: Options) -> Result<(), ManagerError> {
        {
            let index_guard = self.index_read();
            for addr_i in new_options.addrs.iter() {
                if index_guard.get_addr(*addr_i).is_none() {
                    return Err(ME::InvalidAddrArgument);
                }
            }
        }

        let mut options_guard = self.options.write().unwrap();
        let mut options_path = self.options_path.lock().unwrap();
        *options_guard = new_options;
        try!(
            overwrite_json(&*options_guard, &*options_path)
            .map_err(|e| ME::OptionsFileIO(e))
        );

        println!("Updated {} options: {:?}", self.kind.name(), *options_guard);
        Ok(())
    }
}

