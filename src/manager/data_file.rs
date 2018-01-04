/*
 * Copyright 2016-2017 icasdri
 *
 * This file is part of stabping. The original source code for stabping can be
 * found at <https://github.com/icasdri/stabping>. See COPYING for licensing
 * details.
 */
use std::mem;
use std::slice;
use std::ops::Deref;
use std::fs::{File, OpenOptions};
use std::io::{Write, BufRead, BufReader};
use std::path::Path;

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

pub struct DataFileMapping {
    mapping: Mmap,
    de_len: usize,
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

    fn mmap(&mut self) -> Result<DataFileMapping, ME> {
        // attempt to mmap the target's data file
        let map = try!(
            Mmap::open(&mut self.file, Protection::Read)
            .map_err(|_| ME::DataFileIO(AFE::Read(None)))
        );

        let raw_len = map.len();
        if raw_len % mem::size_of::<DataElement>() != 0 {
            return Err(ME::DataFileIO(AFE::Parse(None)));
        }

        let de_len = raw_len / mem::size_of::<DataElement>();

        Ok(DataFileMapping {
            mapping: map,
            de_len: de_len,
        })
    }
}

impl Deref for DataFileMapping {
    /* Modified from memmap source code for their mapping deref at
       https://github.com/danburkert/memmap-rs/blob/
       cc55727a5a759d2700e8619600c75a935f4440dd/src/lib.rs#L390 */

    type Target = [DataElement];

    fn deref(&self) -> &[DataElement] {
        unsafe {
            // attempt to read the bytes of the mapped data as a series of DataElements
            let raw_slice = self.mapping.as_slice();
            let raw_ptr = raw_slice.as_ptr();

            mem::forget(raw_slice);
            slice::from_raw_parts(raw_ptr as *const DataElement, self.de_len)
        }
    }
}
