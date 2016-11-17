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

use chrono::{Local, TimeZone};

use wsserver::Broadcaster;

use options::{SPOptions, MainConfiguration};

fn main() {
    let configuration = Arc::new(RwLock::new(MainConfiguration::default()));
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
        let mut orig_data: Vec<u32> = r.data;

        let new_raw_data: Vec<u8> = {
            let raw_data_ptr = orig_data.as_mut_ptr();
            let new_len = orig_data.len() * mem::size_of::<u32>();
            let new_cap = orig_data.capacity() * mem::size_of::<u32>();

            unsafe {
                // take full control over memory originally controlled by orig_data
                mem::forget(orig_data);
                Vec::from_raw_parts(raw_data_ptr as *mut u8, new_len, new_cap)
            }
        };

        let _ = broadcaster.send(new_raw_data);
    }
}
