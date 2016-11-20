extern crate chrono;
extern crate time;
extern crate rustc_serialize;

extern crate ws;

extern crate iron;
extern crate router;
extern crate mount;

mod options;
mod webserver;
mod wsserver;
mod tcpping;

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

use options::{SPOptions, MainConfiguration};

enum SPIOError {
    FileOpen(PathBuf),
    FileRead(PathBuf),
    JsonDecode(PathBuf),
}

fn read_json_file<T: Decodable>(path: &Path) -> Result<T, SPIOError> {
    let mut config_buffer = String::new();

    let mut config_file = try!(
        File::open(path)
        .map_err(|e| SPIOError::FileOpen(path.to_owned()))
    );
    try!(
        config_file.read_to_string(&mut config_buffer)
        .map_err(|e| SPIOError::FileRead(path.to_owned()))
    );

    json::decode::<T>(&config_buffer)
        .map_err(|e| SPIOError::JsonDecode(path.to_owned()))
}

fn get_configuration() -> (Arc<RwLock<MainConfiguration>>, PathBuf) {
    let mut path = match env::current_exe() {
        Ok(p) => p,
        Err(_) => match env::current_dir() {
            Ok(mut p) => {
                p.pop();
                p
            },
            Err(_) => {
                println!("Error retrieving both current working and running executable directories. Unable to start.");
                process::exit(5);
            }
        }
    };
    path.push("stabping_config.json");

    match read_json_file(&path) {
        Err(e) => {
            let (msg, p) = match e {
                SPIOError::FileOpen(p) => ("Unable to open", p),
                SPIOError::FileRead(p) => ("Unable to read", p),
                SPIOError::JsonDecode(p) => ("Unable to decode as JSON", p),
            };
            println!(
                "{} '{}'. Please ensure that stabping_config.json is accessible (exists, has correct permissions, etc.) and formatted like:\n\n{}\n",
                msg,
                p.to_str().unwrap_or("???/stabping_config.json"),
                json::as_pretty_json(&MainConfiguration::default())
            );
            drop(path);
            process::exit(2);
        },
        Ok(mc) => {
            path.pop();
            path.push("stabping_data");
            fs::create_dir_all(&path);
            (Arc::new(RwLock::new(mc)), path)
        }
    }
}

fn main() {
    let (configuration, data_path) = get_configuration();
    let options = Arc::new(RwLock::new(SPOptions::default()));

    let broadcaster = Arc::new(Broadcaster::new());

    let web_thread = webserver::web_server(configuration.clone(),
                                           options.clone());
    let ws_thread = wsserver::ws_server(configuration.clone(),
                                        options.clone(),
                                        broadcaster.clone());

    let (tcpping_sender, tcpping_results) = channel();
    let tcpping_thread = tcpping::run_tcpping_workers(options.clone(), tcpping_sender.clone());

    for r in tcpping_results {
        let mut orig_data: Vec<i32> = r.data;

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
