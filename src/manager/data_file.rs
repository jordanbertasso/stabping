
        let data_file = try!(
            File::open_from(OpenOptions::new().read(true).append(true).create(true), &path)
            .map_err(|e| ManagerError::DataFileIO(e))
        );
