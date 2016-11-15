extern crate chrono;
extern crate time;

extern crate ws;

extern crate iron;
extern crate router;
extern crate mount;

mod webserver;
mod wsserver;
mod tcpping;

use std::thread;
use std::time::Duration;
use std::sync::Arc;

use wsserver::Broadcaster;

fn main() {
    let broadcaster = Arc::new(Broadcaster::new());

    let web_thread = webserver::web_server("localhost:5001".to_owned());
    let ws_thread = wsserver::ws_server("localhost:5002".to_owned(), broadcaster.clone());

    for i in 0..10 {
        thread::sleep(Duration::from_secs(1));
        broadcaster.send("Hey!");
    }

    ws_thread.join();
}
