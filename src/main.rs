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

use rustc_serialize::json;

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

    for i in 0..10 {
        thread::sleep(Duration::from_secs(1));
        broadcaster.send("Hey!");
    }

    ws_thread.join();
}
