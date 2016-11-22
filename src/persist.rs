use std::sync::{Arc, Mutex, RwLock};

use std::path::{Path, PathBuf};
use std::fs::OpenOptions;
use std::fs::File;
use std::io::{Read, Write};

use std::borrow::Borrow;
use std::collections::HashMap;

use memmap::Mmap;
use memmap::Protection;

use rustc_serialize::json;

use options::SPOptions;
use options::TargetResults;

enum SPPError {
    IndexFileOpen(PathBuf),
    IndexFileParse(PathBuf),
    IndexFileRead(PathBuf),
    IndexFileMetadata(PathBuf),
    IndexFileWrite,
    DataFileOpen(PathBuf),
    DataFileMap(PathBuf),
    DataFileAppend(PathBuf),
}

#[derive(RustcEncodable, RustcDecodable, Debug)]
struct IndexData(Vec<String>);

struct Index {
    file: File,
    data: IndexData,
    map: HashMap<String, i32>,
}

impl Index {
    fn from_path(path: PathBuf) -> Result<Self, SPPError> {
        let mut index_file = try!(OpenOptions::new().read(true).append(true).create(true)
                                  .open(&path)
                                  .map_err(|_| SPPError::IndexFileOpen(path.clone())));

        let meta = try!(index_file.metadata()
                        .map_err(|_| SPPError::IndexFileMetadata(path.clone())));
        if meta.len() > 0 {
            let mut index_data_buf = String::new();
            try!(index_file.read_to_string(&mut index_data_buf)
                 .map_err(|_| SPPError::IndexFileRead(path.clone())));
            let index_data = try!(json::decode(&index_data_buf)
                                  .map_err(|_| SPPError::IndexFileParse(path.clone())));
            Ok(Index::from_data_and_file(index_data, index_file))
        } else {
            Ok(Index::from_data_and_file(IndexData(Vec::new()), index_file))
        }
    }

    fn from_data_and_file(index_data: IndexData, index_file: File) -> Self {
        let mut map = HashMap::new();
        for (i, addr) in index_data.0.iter().enumerate() {
            map.insert(addr.clone(), i as i32);
        }

        Index {
            file: index_file,
            data: index_data,
            map: map,
        }
    }

    fn write_to_file(&mut self) -> Result<(), SPPError> {
        let encoded = json::encode(&self.data).unwrap();
        try!(self.file.write_all(encoded.as_bytes())
             .map_err(|_| SPPError::IndexFileWrite));
        Ok(())
    }

    fn get_index(&self, addr: &str) -> Option<i32> {
        self.map.get(addr).cloned()
    }

    fn get_addr(&self, index: i32) -> Option<&String> {
        self.data.0.get(index as usize)
    }
}

struct Persister {
    index: Arc<RwLock<Index>>,
    data_file: Arc<RwLock<File>>,
    options: Arc<RwLock<SPOptions>>,
}


impl Persister {
    fn new<'b>(data_path: &'b Path, options: Arc<RwLock<SPOptions>>) -> Result<Self, SPPError> {
        let mut path = data_path.to_owned();

        path.push("data.dat");
        let data_file = try!(OpenOptions::new().read(true).append(true).create(true)
                             .open(&path)
                             .map_err(|_| SPPError::DataFileOpen(path.clone())));

        path.pop();
        path.push("index.json");
        let index = try!(Index::from_path(path));


        Ok(Persister {
            index: Arc::new(RwLock::new(index)),
            data_file: Arc::new(RwLock::new(data_file)),
            options: options,
        })
    }

    fn append(&mut self, nonce: i32, data_res: TargetResults) {
        let ref mut file = *self.data_file.write().unwrap();

    }

    fn read_out(&self, nonce: i32, date_from: i32, date_to: i32) {

    }
}
