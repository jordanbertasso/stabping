/*
 * Copyright 2016-2017 icasdri
 *
 * This file is part of stabping. The original source code for stabping can be
 * found at <https://github.com/icasdri/stabping>. See COPYING for licensing
 * details.
 */
use std::fs::{File, OpenOptions};
use std::io::{Write, BufRead, BufReader};
use std::sync::{Mutex, RwLock, RwLockReadGuard};
use std::collections::HashMap;
use std::collections::hash_map::Entry;
use std::path::{Path, PathBuf};

use augmented_file::{AugmentedFile, AugmentedFileError as AFE};
use workers::AddrId;

use manager::ManagerError as ME;


/**
 * A per-target global persistent mapping of index (an integer) to an address
 * (a string used in `TargetOptions.addrs`) backed by an index file.
 */
#[derive(Debug)]
pub struct IndexFile {
    file: File,
    data: Vec<String>,
    map: HashMap<String, AddrId>,
}

impl IndexFile {
    /**
     * Creates an index backed by the index file residing at the given path.
     */
    pub fn from_path<'b>(path: &'b Path) -> Result<Self, ME> {
        // attempt to open the index file
        let mut file = try!(
            File::open_from(OpenOptions::new().read(true).append(true).create(true), path)
            .map_err(|e| ME::IndexFileIO(e))
        );

        let mut data = Vec::new();
        /*
         * if the index file is non-empty, read the data into a list that will
         * function as the index -> addr mapping
         */
        if try!(file.length_p(path).map_err(|e| ME::IndexFileIO(e))) > 0 {
            let reader = BufReader::new(&mut file);

            for line_res in reader.lines() {
                let line = try!(line_res
                                .map_err(|_| ME::IndexFileIO(
                                             AFE::Parse(Some(path.to_owned())))));
                data.push(line);
            }
        }

        // create the map that will contain the reverse addr -> index mapping
        let mut map = HashMap::new();
        for (addr_i, addr) in data.iter().enumerate() {
            map.insert(addr.clone(), addr_i as AddrId);
        }

        Ok(IndexFile {
            file: file,
            data: data,
            map: map,
        })
    }

    /**
     * Adds an addr into this index as necessary (if it does not already
     * exist in the index). Returns the assigned index of the addr.
     */
    pub fn add_addr(&mut self, addr: &str) -> Result<AddrId, ME> {
        let (addr_i, should_insert) = {
            match self.map.get(addr) {
                Some(i) => (i.clone(), false),
                None => (self.data.len() as AddrId, true)
            }
        };

        if should_insert {
            self.map.insert(addr.to_owned(), addr_i);
            self.data.push(addr.to_owned());
            try!(self.file.write_all(format!("{}\n", addr).as_bytes())
                 .map_err(|_| ME::IndexFileIO(
                              AFE::Write(None))));
        }

        Ok(addr_i)
    }

    /**
     * Retrieves the index associated with the given address.
     */
    pub fn get_index(&self, addr: &str) -> Option<AddrId> {
        self.map.get(addr).cloned()
    }

    /**
     * Retrieves the adress associated with the given index.
     */
    pub fn get_addr(&self, index: u32) -> Option<&String> {
        self.data.get(index as usize)
    }

    /**
     * Returns the length (as in number of unique addresses) in this index.
     */
    pub fn len(&self) -> usize {
        self.data.len()
    }
}
