use std::error::Error;
use std::fmt;
use std::fmt::Display;

use std::sync::{Arc, Mutex, RwLock, RwLockReadGuard, RwLockWriteGuard};

use std::path::{Path, PathBuf};
use std::fs::OpenOptions;
use std::fs::File;
use std::io::{Read, Write};
use std::io::BufReader;

use std::borrow::Borrow;
use std::collections::HashMap;

use memmap::Mmap;
use memmap::Protection;

use rustc_serialize::json;

use options::{TargetKind, TargetOptions, TargetResults};

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
            let mut index_data_buf = String::new();

            use std::io::BufRead;
            let mut reader = BufReader::new(&mut index_file);

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
        self.map.insert(addr.clone(), self.data.len() as i32);
        self.data.push(addr);
        let just_added_addr = self.data.last().unwrap();
        Ok(try!(self.file.write_all(just_added_addr.as_bytes())
                .map_err(|_| ManagerError::IndexFileIO(
                             SPIOError::Write(None)))))
    }

    fn get_index(&self, addr: &str) -> Option<i32> {
        self.map.get(addr).cloned()
    }

    fn get_addr(&self, index: i32) -> Option<&String> {
        self.data.get(index as usize)
    }
}

pub struct TargetManager {
    pub kind: &'static TargetKind,
    index: RwLock<AddrIndex>,
    data_file: RwLock<File>,
    options: RwLock<TargetOptions>,
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
        let mut options_buf = String::new();
        try!(options_file.read_to_string(&mut options_buf)
             .map_err(|_| ManagerError::OptionsFileIO(
                          SPIOError::Read(Some(path.clone())))));
        let options = try!(json::decode(&options_buf)
                           .map_err(|_| ManagerError::OptionsFileIO(
                                        SPIOError::Parse(Some(path.clone())))));

        path.pop();
        path.push(format!("{}.index.json", kind.compact_name()));
        let index = try!(AddrIndex::from_path(&path));


        Ok(TargetManager {
            kind: kind,
            index: RwLock::new(index),
            data_file: RwLock::new(data_file),
            options: RwLock::new(options),
        })
    }

    pub fn options_read<'a>(&'a self) -> RwLockReadGuard<'a, TargetOptions> {
        self.options.read().unwrap()
    }

    pub fn options_write<'a>(&'a self) -> RwLockWriteGuard<'a, TargetOptions> {
        self.options.write().unwrap()
    }

    pub fn append_data(&mut self, data_res: TargetResults) {
        let mut data = data_res.0;
        let nonce = data.pop().unwrap();
        let options = self.options.read().unwrap();
        if nonce != options.nonce {
            println!("Nonce mismatch for data append");
            return;
        }
        let ref mut file = *self.data_file.write().unwrap();
    }

    pub fn read_data_for_body(&self, nonce: i32, date_from: i32, date_to: i32) {

    }
}
