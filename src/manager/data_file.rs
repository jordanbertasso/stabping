/*
 * Copyright 2016-2017 icasdri
 *
 * This file is part of stabping. The original source code for stabping can be
 * found at <https://github.com/icasdri/stabping>. See COPYING for licensing
 * details.
 */
use std::mem;
use std::slice;
use std::fs::{File, OpenOptions};
use std::io::{Write, BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::sync::{Mutex, RwLock, RwLockReadGuard};

use memmap::{Mmap, Protection};

use augmented_file::{AugmentedFile, AugmentedFileError as AFE};
use data::DataElement;
use manager::ManagerError as ME;

pub enum DataFileType {
    Raw,
    Hourly,
    Daily,
    Weekly,
}

pub struct DataFile {
    file: File,
}

impl DataFile {
    fn from_path<'b>(path: &'b Path) -> Result<Self, ME> {
        let file = try!(
            File::open_from(OpenOptions::new().read(true).append(true).create(true), &path)
            .map_err(|e| ME::DataFileIO(e))
        );
        Ok(DataFile {
            file: file,
        })
    }

    fn map_slice(&mut self) -> Result<&[DataElement], ME> {
        // attempt to mmap the target's data file
        let map = try!(
            Mmap::open(&mut self.file, Protection::Read)
            .map_err(|e| ME::DataFileIO(AFE::Read(None)))
        );

        // attempt to read the bytes of the mapped data as a series of DataElements
        let data: &[DataElement] = unsafe {
            let orig = map.as_slice();
            let raw_ptr = orig.as_ptr();

            let orig_len = orig.len();
            if orig_len % mem::size_of::<DataElement>() != 0 {
                return Err(ME::DataFileIO(AFE::Parse(None)));
            }
            let new_len = orig.len() / mem::size_of::<DataElement>();

            mem::forget(orig);
            slice::from_raw_parts(raw_ptr as *const DataElement, new_len)
        };

        Ok(data)
    }
}
