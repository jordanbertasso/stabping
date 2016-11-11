extern crate chrono;
extern crate time;

extern crate ws;

extern crate iron;
extern crate router;
extern crate mount;
extern crate staticfile;

mod webserver;
mod wsserver;
mod tcpping;

use std::thread;
use std::time::Duration;

fn main() {
    let web_thread = webserver::web_server("localhost:5001".to_owned());
    let ws_broadcast = wsserver::ws_server("localhost:5002".to_owned());

    for i in 0..10 {
        thread::sleep(Duration::from_secs(1));
        ws_broadcast.send("Hey!");
    }

    web_thread.join();
}
