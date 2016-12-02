use std::fmt;
use std::fmt::Display;
use std::collections::HashMap;
use std::path::Path;
use std::fs::OpenOptions;
use std::fs::File;
use std::io;
use std::io::{Read, Write};
use std::io::BufReader;
use std::sync::{Arc, RwLock, RwLockReadGuard};
use std::ops::Deref;
use std::iter;
use std::iter::Extend;

use helpers::{SPIOError, SPFile, VecIntoRawBytes};
use options::{TargetKind, TargetOptions, TargetResults, SENTINEL_NODATA};

#[derive(Debug)]
pub enum ManagerError {
    IndexFileIO(SPIOError),
    DataFileIO(SPIOError),
    OptionsFileIO(SPIOError),
}

impl ManagerError {
    pub fn description(&self) -> String {
        match *self {
            ManagerError::IndexFileIO(ref e) => format!("{} index file", e.description()),
            ManagerError::DataFileIO(ref e) => format!("{} data file", e.description()),
            ManagerError::OptionsFileIO(ref e) => format!("{} options file", e.description()),
        }
    }
}

impl Display for ManagerError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(&self.description())
    }
}


#[derive(Debug)]
struct AddrIndex {
    file: File,
    data: Vec<String>,
    map: HashMap<String, i32>,
}

impl AddrIndex {
    fn from_path<'b>(path: &'b Path) -> Result<Self, ManagerError> {
        let mut index_file = try!(
            File::open_from(OpenOptions::new().read(true).append(true).create(true), path)
            .map_err(|e| ManagerError::IndexFileIO(e))
        );

        let mut index_data = Vec::new();
        if try!(index_file.length_p(path)
                .map_err(|e| ManagerError::IndexFileIO(e))) > 0 {
            use std::io::BufRead;
            let reader = BufReader::new(&mut index_file);

            for line_res in reader.lines() {
                let line = try!(line_res
                                .map_err(|_| ManagerError::IndexFileIO(
                                             SPIOError::Parse(Some(path.to_owned())))));
                index_data.push(line);
            }
        }

        let mut index_map = HashMap::new();
        for (i, addr) in index_data.iter().enumerate() {
            index_map.insert(addr.clone(), i as i32);
        }

        Ok(AddrIndex {
            file: index_file,
            data: index_data,
            map: index_map,
        })
    }

    fn add_addr(&mut self, addr: &str) -> Result<(), ManagerError> {
        if let None = self.map.get(addr) {
            // only deal with it if we don't already have it
            self.map.insert(addr.to_owned(), self.data.len() as i32);
            self.data.push(addr.to_owned());
            try!(self.file.write_all(format!("{}\n", addr).as_bytes())
                 .map_err(|_| ManagerError::IndexFileIO(
                              SPIOError::Write(None))));
        }
        Ok(())
    }

    fn ensure_for_addrs<'a, I, K>(&mut self, addrs: I) -> Result<(), ManagerError>
            where I: Iterator<Item=&'a K>, K: 'a + Deref<Target=str> {
        for addr in addrs {
            try!(self.add_addr(&addr));
        }
        Ok(())
    }

    fn get_index(&self, addr: &str) -> i32 {
        self.map.get(addr).cloned().expect("Non-existant addr requested from AddrIndex!")
    }

    fn get_addr(&self, index: i32) -> &String {
        self.data.get(index as usize).expect("Non-existant index requested from AddrIndex!")
    }

    fn len(&self) -> usize {
        self.data.len()
    }
}

pub struct TargetManager {
    pub kind: &'static TargetKind,
    index: RwLock<AddrIndex>,
    data_file: RwLock<File>,
    options_file: RwLock<File>,
    options: RwLock<TargetOptions>,
}


impl TargetManager {
    pub fn new<'b>(kind: &'static TargetKind, data_path: &'b Path) -> Result<Self, ManagerError> {
        let mut path = data_path.to_owned();

        path.push(format!("{}.data.dat", kind.compact_name()));
        let data_file = try!(
            File::open_from(OpenOptions::new().read(true).append(true).create(true), &path)
            .map_err(|e| ManagerError::DataFileIO(e))
        );

        path.pop();
        path.push(format!("{}.options.json", kind.compact_name()));
        let mut options_file = try!(
            File::open_from(OpenOptions::new().read(true).write(true).create(true), &path)
            .map_err(|e| ManagerError::OptionsFileIO(e))
        );

        let options = if try!(options_file.length_p(&path)
                              .map_err(|e| ManagerError::OptionsFileIO(e))) > 0 {
            try!(
                options_file.read_json_p(&path)
                .map_err(|e| ManagerError::OptionsFileIO(e))
            )
        } else {
            let default_options = kind.default_options();
            try!(
                options_file.overwrite_json_p(&default_options, &path)
                .map_err(|e| ManagerError::OptionsFileIO(e))
            );
            default_options
        };

        path.pop();
        path.push(format!("{}.index.json", kind.compact_name()));
        let mut index = try!(AddrIndex::from_path(&path));
        try!(index.ensure_for_addrs(options.addrs.iter()));


        Ok(TargetManager {
            kind: kind,
            index: RwLock::new(index),
            data_file: RwLock::new(data_file),
            options_file: RwLock::new(options_file),
            options: RwLock::new(options),
        })
    }

    pub fn options_read<'a>(&'a self) -> RwLockReadGuard<'a, TargetOptions> {
        self.options.read().unwrap()
    }

    pub fn options_update(&self, new_options: TargetOptions) -> Result<(), ManagerError> {
        let mut guard = self.options.write().unwrap();
        let mut options_file = self.options_file.write().unwrap();
        *guard = new_options;
        try!(
            options_file.overwrite_json(&*guard)
            .map_err(|e| ManagerError::OptionsFileIO(e))
        );
        try!(self.index.write().unwrap().ensure_for_addrs(guard.addrs.iter()));
        Ok(())
    }

    pub fn data_file_read<'a>(&'a self) -> RwLockReadGuard<'a, File> {
        self.data_file.read().unwrap()
    }

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
            out_data.push(index.get_index(addr));
            out_data.push(*val);
        }

        let ref mut file = *self.data_file.write().unwrap();
        try!(file.write_all(&out_data.into_raw_bytes())
             .map_err(|_| ManagerError::DataFileIO(
                          SPIOError::Write(None))));
        Ok(())
    }

    /**
     * Get the current addrs in options as (nonce, ordered_list, membership)
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
            let i = index.get_index(addr);
            ordered_list.push(i);
            membership[i as usize] = SENTINEL_NODATA;
        }

        (options.nonce, ordered_list, membership)
    }
}

