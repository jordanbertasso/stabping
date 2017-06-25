use std::fs::{File, OpenOptions};
use std::io::{Write, BufRead, BufReader};

use augmented_file::{AugmentedFile, AugmentedFileError as AFE};
use workers::Options;

pub struct OptionsFile {
    file: File,
    options_path: Mutex<PathBuf>,
    options_path: Mutex<PathBuf>,
    options: RwLock<Options>,
    options: RwLock<Options>,
}

impl OptionsFile {
    fn from_path<'b>(path: &'b Path) -> Result<Self, ManagerError> {
    }
}

