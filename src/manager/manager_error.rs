/*
 * Copyright 2016-2017 icasdri
 *
 * This file is part of stabping. The original source code for stabping can be
 * found at <https://github.com/icasdri/stabping>. See COPYING for licensing
 * details.
 */
use std::fmt;
use std::fmt::Display;

use augmented_file::AugmentedFileError;

/**
 * A stabping-specific error container for errors incurred during TargetManager
 * creation and methods.
 */
#[derive(Debug)]
pub enum ManagerError {
    IndexFileIO(AugmentedFileError),
    DataFileIO(AugmentedFileError),
    OptionsFileIO(AugmentedFileError),
    InvalidAddrArgument,
}
use self::ManagerError as ME;

impl ManagerError {
    pub fn description(&self) -> String {
        match *self {
            ME::IndexFileIO(ref e) => format!("{} index file", e.description()),
            ME::DataFileIO(ref e) => format!("{} data file", e.description()),
            ME::OptionsFileIO(ref e) => format!("{} options file", e.description()),
            ME::InvalidAddrArgument => "invalid addr argument".to_owned(),
        }
    }
}

impl Display for ManagerError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(&self.description())
    }
}


