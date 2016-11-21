use std::sync::Arc;
use std::sync::RwLock;

use std::path::PathBuf;
use std::fs::OpenOptions;
use std::fs::File;
use std::io::Write;

use memmap::Mmap;

use options::SPOptions;

struct Persister {
    data_path: PathBuf,
    options: Arc<RwLock<SPOptions>>,
}

impl Persister {
    fn new(data_path: PathBuf, options: Arc<RwLock<SPOptions>>) -> Self {
        Persister {
            data_path: data_path,
            options: options,
        }
    }
}
