extern crate chrono;
extern crate time;
extern crate rustc_serialize;
extern crate memmap;
extern crate ws;
extern crate iron;
extern crate router;
extern crate mount;

mod helpers;
mod options;
mod persist;
mod reader;
mod webserver;
mod wsserver;
mod tcpping;

use std::env;
use std::path::PathBuf;
use std::fs;
use std::fs::{OpenOptions, File};
use std::sync::Arc;
use std::sync::RwLock;
use std::sync::mpsc::channel;

use rustc_serialize::json;

use wsserver::Broadcaster;

use helpers::{SPIOError, SPFile, VecIntoRawBytes};
use options::{TargetKind, MainConfiguration};
use persist::ManagerError;

static CONFIG_FILENAME: &'static str = "stabping_config.json";

fn get_configuration() -> Option<(Arc<RwLock<MainConfiguration>>, PathBuf)> {
    let dirs_to_try = &[
        ("current working directory",
         env::current_dir().ok().map(|mut cur| { cur.push(CONFIG_FILENAME); cur })),
        ("user configuration directory",
         env::home_dir().map(|mut home| { home.push(".config"); home.push(CONFIG_FILENAME); home })),
        ("global configuration directory",
         Some({ let mut p = PathBuf::from("/etc"); p.push(CONFIG_FILENAME); p })),
        ("directory where stabping is located",
         env::current_exe().ok().map(|mut exe| { exe.pop(); exe.push(CONFIG_FILENAME); exe })),
    ];

    println!("Searching for configuration file '{}'.", CONFIG_FILENAME);
    for &(desc, ref maybe_p) in dirs_to_try {
        if let &Some(ref p) = maybe_p {
            println!("- checking {}:\n    {}", desc, p.to_str().unwrap());
            if let Ok(mut file) = File::open_from(OpenOptions::new().read(true), &p) {
                match file.read_json_p(&p) {
                    Err(err @ SPIOError::Parse(_)) => {
                        println!(
                            "\n{} configuration file. Invalid or missing JSON fields. Please ensure that this file is formatted like:\n{}\n",
                            err.description(),
                            json::as_pretty_json(&MainConfiguration::default())
                        );
                        return None
                    },
                    Ok(mc) => {
                        println!("\nUsing configuration file in {}:\n  {}",
                                 desc, p.to_str().unwrap());
                        let mut data_path = p.clone();
                        data_path.pop();
                        data_path.push("stabping_data");
                        if fs::create_dir_all(&data_path).is_err() {
                            println!("Failed to create data directory '{}'. Please ensure this directory is writable by stabping.", data_path.to_str().unwrap());
                            return None;
                        }
                        return Some((Arc::new(RwLock::new(mc)), data_path));
                    },
                    _ => {}
                };
            }
        } else {
            println!("- could not obtain {}", desc);
        }
    }

    println!(
        "\nFailed to find configuration file. Please ensure that 'stabping_config.json' is accessible in one of the above checked locations, and is formatted like:\n{}\n",
        json::as_pretty_json(&MainConfiguration::default())
    );
    None
}

fn main() {
    let (configuration, data_path) = match get_configuration() {
        Some(c) => c,
        None => {
            panic!("Failed to get configuration");
        }
    };

    let targets = match TargetKind::new_managers_for_all(&data_path) {
        Ok(targets) => targets,
        Err(e) => handle_fatal_error(e),
    };

    let broadcaster = Arc::new(Broadcaster::new());

    webserver::web_server(configuration.clone(), targets.iter());
    wsserver::ws_server(configuration.clone(), broadcaster.clone());

    let (sender, results) = channel();
    for tm in targets.iter() {
        tm.kind.run_worker(tm.clone(), sender.clone());
    }

    for r in results {
        let kind_id = r.0[0];
        if let Err(e) = targets[kind_id as usize].append_data(&r) {
            handle_fatal_error(e);
        }

        let raw_data_bytes = r.0.into_raw_bytes();
        let _ = broadcaster.send(raw_data_bytes);
    }
}

fn handle_fatal_error(e: ManagerError) -> ! {
    panic!("{}", e);
}
