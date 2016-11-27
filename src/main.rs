extern crate chrono;
extern crate time;
extern crate rustc_serialize;
extern crate memmap;

extern crate ws;

extern crate iron;
extern crate router;
extern crate mount;

mod options;
mod persist;
mod webserver;
mod wsserver;
mod tcpping;

use std::error::Error;
use std::mem;
use std::sync::Arc;
use std::sync::RwLock;
use std::sync::mpsc::channel;
use std::process;
use std::env;
use std::path::{Path, PathBuf};
use std::fs;
use std::fs::File;
use std::io::Read;

use rustc_serialize::Decodable;
use rustc_serialize::json;

use wsserver::Broadcaster;

use options::{TargetKind, TargetOptions, MainConfiguration};
use persist::{TargetManager, ManagerError, SPIOError};

fn read_json_file<T: Decodable>(path: &Path) -> Result<T, SPIOError> {
    let mut config_buffer = String::new();

    let mut config_file = try!(
        File::open(path)
        .map_err(|e| SPIOError::Open(Some(path.to_owned())))
    );
    try!(
        config_file.read_to_string(&mut config_buffer)
        .map_err(|e| SPIOError::Read(Some(path.to_owned())))
    );

    json::decode::<T>(&config_buffer)
        .map_err(|e| SPIOError::Parse(Some(path.to_owned())))
}

fn get_configuration() -> (Arc<RwLock<MainConfiguration>>, PathBuf) {
    let mut dirs_to_try = vec![
        env::current_dir().unwrap(),
        env::current_exe().unwrap()
    ];

    let mut mc_found = None;
    let mut config_path_found = None;

    for mut p in dirs_to_try.drain(..) {
        p.push("stabping_config.json");
        match read_json_file(&p) {
            Err(io_err) => {
                match io_err {
                    SPIOError::Parse(_) => {
                        println!(concat!(
                            "{} configuration file. Invalid or missing JSON fields.\n",
                            "Please ensure that this file is formatted like:\n{}\n"),
                            io_err.description(),
                            json::as_pretty_json(&MainConfiguration::default())
                        );
                        process::exit(3);
                    }
                    _ => {}
                }
            },
            Ok(mc) => {
                config_path_found = Some(p);
                mc_found = Some(mc);
                break;
            }
        }
    }

    if let Some(mut path) = config_path_found {
        println!("Using configuration file at '{}'.", path.to_str().unwrap());
        path.pop();
        path.push("stabping_data");
        fs::create_dir_all(&path);
        (Arc::new(RwLock::new(mc_found.unwrap())), path)
    } else {
        println!(concat!(
            "Please ensure that 'stabping_config.json' is accessible in either\n",
            "- the current working directory\n     {}\n",
            "- the directory where stabping is located\n     {}\n",
            "\nThis file should be formatted like:\n{}\n"),
            env::current_dir().unwrap().to_str().unwrap(),
            env::current_exe().unwrap().to_str().unwrap(),
            json::as_pretty_json(&MainConfiguration::default())
        );
        process::exit(2);
    }
}

fn main() {
    let (configuration, data_path) = get_configuration();

    let targets = match TargetKind::new_managers_for_all(&data_path) {
        Ok(targets) => targets,
        Err(e) => handle_fatal_error(e),
    };

    let broadcaster = Arc::new(Broadcaster::new());

    let web_thread = webserver::web_server(configuration.clone(),
                                           targets.iter());
    let ws_thread = wsserver::ws_server(configuration.clone(),
                                        broadcaster.clone());

    let (sender, results) = channel();
    for tm in targets.iter() {
        tm.kind.run_worker(tm.clone(), sender.clone());
    }

    for r in results {
        let mut orig_data: Vec<i32> = r.0;

        let new_raw_data: Vec<u8> = {
            let raw_data_ptr = orig_data.as_mut_ptr();
            let new_len = orig_data.len() * mem::size_of::<i32>();
            let new_cap = orig_data.capacity() * mem::size_of::<i32>();

            unsafe {
                // take full control over memory originally controlled by orig_data
                mem::forget(orig_data);
                Vec::from_raw_parts(raw_data_ptr as *mut u8, new_len, new_cap)
            }
        };

        let _ = broadcaster.send(new_raw_data);
    }
}

fn handle_fatal_error(e: ManagerError) -> ! {
    panic!("{}", e);
}
