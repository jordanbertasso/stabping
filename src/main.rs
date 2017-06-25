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

mod data;
mod workers;
mod augmented_file;
mod config;
mod persist;
mod reader;
mod webserver;
mod wsserver;

use std::env;
use std::path::PathBuf;
use std::fs;
use std::fs::{OpenOptions, File};
use std::sync::Arc;
use std::sync::RwLock;
use std::sync::mpsc::channel;

use rustc_serialize::json;

use wsserver::Broadcaster;

use data::{AsBytes, ToWire};
use helpers::{SPIOError, SPFile, VecIntoRawBytes};
use options::{TargetKind, MainConfiguration};
use persist::ManagerError;


fn main() {
    // try and obtain our configuration and data directory path
    let (configuration, data_path) = match config::get_configuration() {
        Some(c) => c,
        None => {
            panic!("Failed to get configuration");
        }
    };

    // create managers for all the targets
    let targets = match TargetKind::new_managers_for_all(&data_path) {
        Ok(targets) => targets,
        Err(e) => handle_fatal_error(e),
    };

    // create a broadcaster to be initialized with the websockets server
    let broadcaster = Arc::new(Broadcaster::new());

    // start the web and websockets servers
    webserver::web_server(configuration.clone(), targets.iter());
    wsserver::ws_server(configuration.clone(), broadcaster.clone());

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

        // broadcast the live data over websockets
        let mut bytes = Vec::with_capacity(data.space_necessary());
        bytes.extend_from_slice((kind as u32).as_bytes());
        data.to_wire(&mut bytes);
        let _ = broadcaster.send(bytes);
    }
}

fn handle_fatal_error(e: ManagerError) -> ! {
    panic!("{}", e);
}
