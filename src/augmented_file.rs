/*
 * Copyright 2016 icasdri
 *
 * This file is part of stabping. The original source code for stabping can be
 * found at <https://github.com/icasdri/stabping>. See COPYING for licensing
 * details.
 */

/*!
 * Helper traits and functions for reducing verbosity, wraping errors, and
 * containing unsafe code for many commonly used I/O and parsing operations.
 */
use std::fmt::{self, Display};
use std::path::{Path, PathBuf};
use std::fs::{OpenOptions, File};
use std::io::{Read, Write};

use rustc_serialize::{json, Encodable, Decodable};

/**
 * Stabping-specific I/O error container, representing the possible failrue
 * cases when working with files, and wrapping an optional path (if one is
 * known).
 */
#[derive(Debug)]
pub enum AugmentedFileError {
    Open(Option<PathBuf>),
    Read(Option<PathBuf>),
    Metadata(Option<PathBuf>),
    Write(Option<PathBuf>),
    Parse(Option<PathBuf>),
}
use self::AugmentedFileError as AFE;

impl AugmentedFileError {
    pub fn description(&self) -> String {
        let (verb, maybe_path) = match *self {
            AFE::Open(ref p) => ("open", p),
            AFE::Read(ref p) => ("read", p),
            AFE::Metadata(ref p) => ("get metadata", p),
            AFE::Write(ref p) => ("write", p),
            AFE::Parse(ref p) => ("parse", p),
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

impl Display for AugmentedFileError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(&self.description())
    }
}


/**
 * Expands the functionality of `File` to include JSON encoding, a
 * generalized `open()` and streamlined access to `metadata.length`. All
 * methods return a `Result<_, AFE>`, the `Result` wrapping the error
 * container from this module.
 *
 * Methods come in "basic" and `_p`/"with optional path" form. They accomplish
 * the same thing, except one takes an optional path that will be wrapped in
 * the error container if an error is encountered.
 */
pub trait AugmentedFile {
    /**
     * Opens a file from the given path with the given `OpenOptions`.
     */
    fn open_from<'a, 'b>(oo: &'b mut OpenOptions, path: &'a Path) -> Result<File, AFE>;

    /**
     * Attempts to read from this file and decode all its contents as a JSON
     * object (`rustc::Decodable`).
     */
    fn read_json<T: Decodable>(&mut self) -> Result<T, AFE> {
        self._read_json(None)
    }
    fn read_json_p<'a, T: Decodable>(&mut self, path: &'a Path) -> Result<T, AFE> {
        self._read_json(Some(path))
    }
    fn _read_json<'a, T: Decodable>(&mut self, path: Option<&'a Path>) -> Result<T, AFE>;


    /**
     * Attempts to write a JSON object (`rustc::Encodable`) to this file.
     */
    fn write_json<'b, T: Encodable>(&mut self, obj: &'b T) -> Result<(), AFE> {
        self._write_json(obj, None)
    }
    fn write_json_p<'a, 'b, T: Encodable>(&mut self, obj: &'b T, path: &'a Path) -> Result<(), AFE> {
        self._write_json(obj, Some(path))
    }
    fn _write_json<'a, 'b, T: Encodable>(&mut self, obj: &'b T, path: Option<&'a Path>) -> Result<(), AFE>;


    /**
     * Attempts to obtain the length of this file from filesystem metadata.
     */
    fn length(&mut self) -> Result<u64, AFE> {
        self._length(None)
    }
    fn length_p<'a>(&mut self, path: &'a Path) -> Result<u64, AFE> {
        self._length(Some(path))
    }
    fn _length<'a>(&mut self, path: Option<&'a Path>) -> Result<u64, AFE>;
}

impl AugmentedFile for File {
    fn open_from<'a, 'b>(oo: &'b mut OpenOptions, path: &'a Path) -> Result<File, AFE> {
        Ok(try!(
            oo.open(path)
            .map_err(|_| AFE::Open(Some(path.to_owned())))
        ))
    }

    fn _read_json<'a, T: Decodable>(&mut self, path: Option<&'a Path>) -> Result<T, AFE> {
        let mut buffer = String::new();
        try!(
            self.read_to_string(&mut buffer)
            .map_err(|_| AFE::Read(path.map(|p| p.to_owned())))
        );
        json::decode::<T>(&buffer)
            .map_err(|_| AFE::Parse(path.map(|p| p.to_owned())))
    }

    fn _write_json<'a, 'b, T: Encodable>(&mut self, obj: &'b T, path: Option<&'a Path>) -> Result<(), AFE> {
        let buffer = json::encode(obj).unwrap();
        try!(
            self.write_all(buffer.as_bytes())
            .map_err(|_| AFE::Write(path.map(|p| p.to_owned())))
        );
        try!(
            self.flush()
            .map_err(|_| AFE::Write(path.map(|p| p.to_owned())))
        );
        Ok(())
    }

    fn _length<'a>(&mut self, path: Option<&'a Path>) -> Result<u64, AFE> {
        let meta = try!(
            self.metadata()
            .map_err(|_| AFE::Metadata(path.map(|p| p.to_owned())))
        );
        Ok(meta.len())
    }
}

/**
 * Overwrite (create if necessary, truncate if already exists) the file
 * residing at the given path with the given JSON object (`rustc::Encodable`).
 */
pub fn overwrite_json<'a, 'b, T: Encodable>(obj: &'a T, path: &'b Path) -> Result<(), AFE> {
    let mut file = try!(
        OpenOptions::new().write(true).truncate(true).create(true).open(path)
        .map_err(|_| AFE::Open(Some(path.to_owned())))
    );

    try!(file.write_json_p(obj, path));
    Ok(())
}
