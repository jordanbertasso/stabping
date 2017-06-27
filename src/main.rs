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

use workers::run_worker;
use manager::{Manager, ManagerError as ME};


fn main() {
    // try and obtain our configuration and data directory path
    let (configuration, data_path) = match config::get_configuration() {
        Some(c) => c,
        None => {
            panic!("Failed to get configuration");
        }
    };
/*
    // create managers for all the targets
    let targets = match TargetKind::new_managers_for_all(&data_path) {
        Ok(targets) => targets,
        Err(e) => handle_fatal_error(e),
    };

    /*
     * start the workers for all the targets, passing them one end of an MPSC
     * communications channel so that we can receive all the data
     */
    let (sender, results) = channel();
    for tm in targets.iter() {
        tm.kind.run_worker(tm.clone(), sender.clone());
    }

    /*
     * receive the live data coming from the workers and process it
     */
    for (kind, data) in results {
        // append the data to the data file via the appropriate manager
        if let Err(e) = targets[kind].append_data(&r) {
            handle_fatal_error(e);
        }

        // TODO: broadcast the live data over websockets
    }
*/
}
