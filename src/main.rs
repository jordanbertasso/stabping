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
            println!("- checking {}:\n    '{}'", desc, p.to_str().unwrap());
            match read_json_file(&p) {
                Err(io_err) => match io_err {
                    SPIOError::Parse(_) => {
                        println!(concat!(
                            "\n{} configuration file. Invalid or missing JSON fields.\n",
                            "Please ensure that this file is formatted like:\n{}\n"),
                            io_err.description(),
                            json::as_pretty_json(&MainConfiguration::default())
                        );
                        return None
                    }
                    _ => (),
                },
                Ok(mc) => {
                    println!("\nUsing configuration file in {}:\n  '{}'",
                             desc, p.to_str().unwrap());
                    let mut data_path = p.clone();
                    data_path.pop();
                    data_path.push("stabping_data");
                    fs::create_dir_all(&data_path);
                    return Some((Arc::new(RwLock::new(mc)), data_path));
                }
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
