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

use std::thread;
use std::time::Duration;
use std::sync::Arc;
use std::sync::RwLock;
use std::sync::mpsc::{channel, Receiver, Sender};

use rustc_serialize::json;

use chrono::{Local, TimeZone};

use wsserver::Broadcaster;

use options::{TargetOptions, SPOptions, MainConfiguration};

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
        let mut result_string = format!("{}<br>", Local.timestamp(r.time as i64, 0));
        for (addr, val) in options.read().unwrap().tcpping_options.addrs.iter().zip(r.data) {
            let whole_ms = val / 1000;
            let part_ms = (val % 1000) / 10;
            result_string.push_str(format!("TCP connection to '{}' took {}.{}ms.<br>",
                                           addr, whole_ms, part_ms).as_str());
        }
        result_string.push_str("<br>");
        broadcaster.send(result_string);
    }
}
