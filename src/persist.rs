use std::sync::{Arc, Mutex, RwLock};

use std::path::PathBuf;
use std::fs::OpenOptions;
use std::fs::File;
use std::io::Write;

use memmap::Mmap;
use memmap::Protection;

use rustc_serialize::json;

use options::SPOptions;
use options::TargetResults;

enum SPPError {
    IndexFileOpen(PathBuf),
    IndexFileParse(PathBuf),
    IndexFileRead(PathBuf),
    IndexFileWrite(PathBuf),
    DataFileOpen(PathBuf),
    DataFileMap(PathBuf),
    DataFileAppend(PathBuf),
}

struct Index {
    addrs_list: Vec<String>,
    indices_map: HashMap<i32, String>,
}

impl Index {
    fn new() -> Self {
        Index::with_addr_list(Vec::new())
    }

    fn with_addr_list(addrs_list: Vec<String>) -> Self {

    }

    fn get_index(&self, addr: String) {

    }

    fn get_addr(&self, index: i32) {

    }
}

struct Persister {
    index: Arc<RwLock<Index>>,
    index_file: Arc<Mutex<File>>,
    data_file: Arc<RwLock<File>>,
    options: Arc<RwLock<SPOptions>>,
}

impl Persister {
    fn new<'a>(data_path: &'a Path, options: Arc<RwLock<SPOptions>>) -> Result<Self, SPPError> {
        let open_options = OpenOptions::new().read(true).append(true).create(true);

        let path = data_path.to_owned();

        path.push("data.dat");
        let data_file = try!(open_options.open(path)
                             .map_err(|_| SPPError::DataFileOpen(path)));

        path.pop();
        path.push("index.json");
        let index_file = try!(open_options.open(path)
                              .map_err(|_| SPPError::IndexFileOpen(path)));

        let index_data_buf = String::new();


        Persister {
            data_file: Arc::new(RwLock::new(file)),
            options: options,
        }
    }

    fn append(&mut self, nonce: i32, data_res: TargetResults) {
        let file = *self.data_file.lock().unwrap();

    }

    fn read_out(&self, nonce: i32, date_from: i32, date_to: i32) {

    }
}
