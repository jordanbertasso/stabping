use std::fmt;
use std::fmt::Display;
use std::collections::HashMap;
use std::collections::hash_map::Entry;
use std::path::{Path, PathBuf};
use std::fs::OpenOptions;
use std::fs::File;
use std::io::{Read, Write};
use std::io::BufReader;
use std::sync::{Arc, Mutex, RwLock, RwLockReadGuard, RwLockWriteGuard};
use std::ops::Deref;

use memmap::{Mmap, Protection};
use rustc_serialize::json;

use options::{TargetKind, TargetOptions, TargetResults, VecIntoRawBytes};

#[derive(Debug)]
pub enum SPIOError {
    Open(Option<PathBuf>),
    Read(Option<PathBuf>),
    Metadata(Option<PathBuf>),
    Write(Option<PathBuf>),
    Parse(Option<PathBuf>),
}

impl SPIOError {
    pub fn description(&self) -> String {
        let (verb, maybe_path) = match *self {
            SPIOError::Open(ref p) => ("open", p),
            SPIOError::Read(ref p) => ("read", p),
            SPIOError::Metadata(ref p) => ("get metadata", p),
            SPIOError::Write(ref p) => ("write", p),
            SPIOError::Parse(ref p) => ("parse", p),
        };

        let path_str = match maybe_path {
            &Some(ref path) => match path.to_str() {
                Some(s) => s,
                None => "",
            },
            &None => "",
        };

        format!("Unable to {} '{}'", verb, path_str)
    }
}

impl Display for SPIOError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(&self.description())
    }
}


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
        let mut index_file = try!(OpenOptions::new().read(true).append(true).create(true)
                                  .open(path)
                                  .map_err(|_| ManagerError::IndexFileIO(
                                               SPIOError::Open(Some(path.to_owned())))));

        let meta = try!(index_file.metadata()
                        .map_err(|_| ManagerError::IndexFileIO(
                                     SPIOError::Metadata(Some(path.to_owned())))));
        let mut index_data = Vec::new();
        if meta.len() > 0 {
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


fn write_options_to_file<T>(options: T, options_file: &mut File) -> Result<(), ManagerError>
                            where T: Deref<Target=TargetOptions> {
    let buf = json::encode(&*options).unwrap();
    try!(options_file.set_len(0)
         .map_err(|_| ManagerError::OptionsFileIO(
                      SPIOError::Write(None))));
    try!(options_file.write_all(buf.as_bytes())
         .map_err(|_| ManagerError::OptionsFileIO(
                      SPIOError::Write(None))));
    Ok(())
}

impl TargetManager {
    pub fn new<'b>(kind: &'static TargetKind, data_path: &'b Path) -> Result<Self, ManagerError> {
        let mut path = data_path.to_owned();

        path.push(format!("{}.data.dat", kind.compact_name()));
        let data_file = try!(OpenOptions::new().read(true).append(true).create(true)
                             .open(&path)
                             .map_err(|_| ManagerError::DataFileIO(
                                          SPIOError::Open(Some(path.clone())))));

        path.pop();
        path.push(format!("{}.options.json", kind.compact_name()));
        let mut options_file = try!(OpenOptions::new().read(true).write(true).create(true)
                                    .open(&path)
                                    .map_err(|_| ManagerError::OptionsFileIO(
                                                 SPIOError::Open(Some(path.clone())))));
        let options_meta = try!(options_file.metadata()
                                .map_err(|_| ManagerError::OptionsFileIO(
                                             SPIOError::Metadata(Some(path.clone())))));
        let options = if options_meta.len() > 0 {
            let mut options_buf = String::new();
            try!(options_file.read_to_string(&mut options_buf)
                 .map_err(|_| ManagerError::OptionsFileIO(
                              SPIOError::Read(Some(path.clone())))));
            try!(json::decode(&options_buf)
                 .map_err(|_| ManagerError::OptionsFileIO(
                              SPIOError::Parse(Some(path.clone())))))
        } else {
            let default_options = kind.default_options();
            try!(write_options_to_file(&default_options, &mut options_file));
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

    pub fn options_write<'a>(&'a self) -> RwLockWriteGuard<'a, TargetOptions> {
        self.options.write().unwrap()
    }

    pub fn append_data(&mut self, data_res: TargetResults) -> Result<(), ManagerError> {
        let mut in_data = data_res.0;
        let nonce = in_data.pop().expect("Expecting nonce at end of TargetResults.");
        if nonce != self.options_read().nonce {
            println!("Nonce mismatch for data append! Silently ignoring.");
            return Ok(());
        }

        let mut out_data: Vec<i32> = Vec::with_capacity((data.len() - 1) * 3);
        let time = data[0];
        let index = self.index.read().unwrap();
        for (addr, val) in self.options_read().addrs.iter().zip(data[1..].iter()) {
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
