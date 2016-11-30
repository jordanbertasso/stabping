use std::mem;
use std::fmt::{self, Display};
use std::path::{Path, PathBuf};
use std::fs::{OpenOptions, File};
use std::io::{Read, Write};

use rustc_serialize::{json, Encodable, Decodable};

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


pub trait VecIntoRawBytes {
    fn into_raw_bytes(self) -> Vec<u8>;
}

impl VecIntoRawBytes for Vec<i32> {
    fn into_raw_bytes(mut self) -> Vec<u8> {
        let raw_ptr = self.as_mut_ptr();
        let new_len = self.len() * mem::size_of::<i32>();
        let new_cap = self.capacity() * mem::size_of::<i32>();

        unsafe {
            // take full control over memory originally controlled by orig_data
            mem::forget(self);
            Vec::from_raw_parts(raw_ptr as *mut u8, new_len, new_cap)
        }
    }
}

pub trait SPFile {
    fn open_from<'a, 'b>(oo: &'b mut OpenOptions, path: &'a Path) -> Result<File, SPIOError>;

    fn _read_json<'a, T: Decodable>(&mut self, path: Option<&'a Path>) -> Result<T, SPIOError>;

    fn read_json<T: Decodable>(&mut self) -> Result<T, SPIOError> {
        self._read_json(None)
    }

    fn read_json_p<'a, T: Decodable>(&mut self, path: &'a Path) -> Result<T, SPIOError> {
        self._read_json(Some(path))
    }


    fn _overwrite_json<'a, 'b, T: Encodable>(&mut self, obj: &'b T, path: Option<&'a Path>) -> Result<(), SPIOError>;

    fn overwrite_json<'b, T: Encodable>(&mut self, obj: &'b T) -> Result<(), SPIOError> {
        self._overwrite_json(obj, None)
    }

    fn overwrite_json_p<'a, 'b, T: Encodable>(&mut self, obj: &'b T, path: &'a Path) -> Result<(), SPIOError> {
        self._overwrite_json(obj, Some(path))
    }

    fn _length<'a>(&mut self, path: Option<&'a Path>) -> Result<u64, SPIOError>;

    fn length(&mut self) -> Result<u64, SPIOError> {
        self._length(None)
    }

    fn length_p<'a>(&mut self, path: &'a Path) -> Result<u64, SPIOError> {
        self._length(Some(path))
    }
}

impl SPFile for File {
    fn open_from<'a, 'b>(oo: &'b mut OpenOptions, path: &'a Path) -> Result<File, SPIOError> {
        Ok(try!(
            oo.open(path)
            .map_err(|_| SPIOError::Open(Some(path.to_owned())))
        ))
    }

    fn _read_json<'a, T: Decodable>(&mut self, path: Option<&'a Path>) -> Result<T, SPIOError> {
        let mut buffer = String::new();
        try!(
            self.read_to_string(&mut buffer)
            .map_err(|_| SPIOError::Read(path.map(|p| p.to_owned())))
        );
        json::decode::<T>(&buffer)
            .map_err(|_| SPIOError::Parse(path.map(|p| p.to_owned())))
    }

    fn _overwrite_json<'a, 'b, T: Encodable>(&mut self, obj: &'b T, path: Option<&'a Path>) -> Result<(), SPIOError> {
        let buffer = json::encode(obj).unwrap();
        try!(
            self.set_len(0)
            .map_err(|_| SPIOError::Write(path.map(|p| p.to_owned())))
        );
        try!(
            self.write_all(buffer.as_bytes())
            .map_err(|_| SPIOError::Write(path.map(|p| p.to_owned())))
        );
        Ok(())
    }

    fn _length<'a>(&mut self, path: Option<&'a Path>) -> Result<u64, SPIOError> {
        let meta = try!(
            self.metadata()
            .map_err(|_| SPIOError::Metadata(path.map(|p| p.to_owned())))
        );
        Ok(meta.len())
    }
}
