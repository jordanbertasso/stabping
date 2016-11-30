use std::fmt;
use std::fmt::Display;
use std::collections::HashMap;
use std::collections::hash_map::Entry;
use std::path::{Path, PathBuf};
use std::fs::OpenOptions;
use std::fs::File;
use std::io::{Read, Write};
use std::io::BufReader;
use std::sync::{Arc, RwLock, RwLockReadGuard};

use memmap::{Mmap, Protection};

use helpers::{SPIOError, SPFile, VecIntoRawBytes};
use options::{TargetKind, TargetOptions, TargetResults};

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

    fn add_addr(&mut self, addr: String) -> Result<(), ManagerError> {
        match self.map.entry(addr) {
            Entry::Occupied(_) => {
                // we already have it, so no need to make another index
                Ok(())
            },
            Entry::Vacant(v) => {
                self.data.push(v.key().clone());
                v.insert(self.data.len() as i32);
                Ok(try!(self.file.write_all(self.data.last().unwrap().as_bytes())
                        .map_err(|_| ManagerError::IndexFileIO(
                                     SPIOError::Write(None)))))
            }
        }
    }

    fn get_index(&self, addr: &str) -> i32 {
        self.map.get(addr).cloned().expect("Non-existant addr requested from AddrIndex!")
    }

    fn get_addr(&self, index: i32) -> &String {
        self.data.get(index as usize).expect("Non-existant index requested from AddrIndex!")
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
        let index = try!(AddrIndex::from_path(&path));


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
        Ok(())
    }

    pub fn append_data(&mut self, data_res: TargetResults) -> Result<(), ManagerError> {
        let mut in_data = data_res.0;
        let nonce = in_data.pop().expect("Expecting nonce at end of TargetResults.");
        if nonce != self.options_read().nonce {
            println!("Nonce mismatch for data append! Silently ignoring.");
            return Ok(());
        }

        let mut out_data: Vec<i32> = Vec::with_capacity((in_data.len() - 1) * 3);
        let time = in_data[0];
        let index = self.index.read().unwrap();
        for (addr, val) in self.options_read().addrs.iter().zip(in_data[1..].iter()) {
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

    pub fn read_data_for_body(&self, nonce: i32, date_from: i32, date_to: i32) {

    }
}
