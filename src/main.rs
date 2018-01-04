/*
 * Copyright 2016-2017 icasdri
 *
 * This file is part of stabping. The original source code for stabping can be
 * found at <https://github.com/icasdri/stabping>. See COPYING for licensing
 * details.
 */

extern crate chrono;
extern crate time;
extern crate rustc_serialize;
extern crate memmap;
extern crate ws;
extern crate iron;
extern crate router;
extern crate mount;

mod augmented_file;
mod data;
mod config;
mod workers;
mod manager;

use std::sync::Arc;
use std::sync::mpsc::channel;
use std::fmt::Display;
use std::ops::Deref;
use std::process::exit as process_exit;

use workers::{Kind, ALL_KINDS, run_worker};
use manager::{Manager, ManagerError, ManagerError as ME};
use augmented_file::AugmentedFileError as AFE;


enum TopLevelError {
    InConfiguration,
    InManager(ME),
}
use TopLevelError as TLE;

impl From<ManagerError> for TopLevelError {
    fn from(m: ManagerError) -> Self {
        TLE::InManager(m)
    }
}

fn main() {
    let result = wrapped_main();

    fn exit<M>(msg: M, code: i32) -> ! where M: Display {
        println!("{}", msg);
        process_exit(code);
    }

    match result {
        Ok(_) => (),
        Err(e) => match e {
            TLE::InConfiguration => exit("Failed to get configuration", 2),
            TLE::InManager(m) => exit(m, 3),
        }
    }
}

fn wrapped_main() -> Result<(), TopLevelError> {
    // try and obtain our configuration and data directory path
    let (configuration, data_path) = try!(
        config::get_configuration().ok_or(TLE::InConfiguration)
    );

    // create managers for all targets
    let managers: Vec<Arc<Manager>> = try!(
        ALL_KINDS.iter().map(|&kind| Manager::new(kind, &data_path))
                        .map(|manager_res| manager_res.map(|manager| Arc::new(manager)))
                        .collect()
    );

    // start workers for all targets (passing one end of channel so we can receive data)
    let (sender, results) = channel();
    for manager in managers.iter() {
        run_worker(manager.clone(), sender.clone());
    }

    /*
     * receive the live data coming from the workers and process it
     */
    for package in results {
        let manager = &managers[package.kind.id() as usize];
        if let Err(e) = manager.append_package(&package) {
            println!("main: Error appending data: {}", e);
        }

        // TODO: append data to respective manager data file
        // TODO: broadcast the live data over websockets
    }

    Ok(())
}
