use std::thread;
use std::net::ToSocketAddrs;
use std::fmt::Debug;

use ws;
use ws::{Builder, Factory, Handler, Settings};

struct ServerHandler {
    out: ws::Sender,
}

impl ws::Handler for ServerHandler {
    fn on_open(&mut self, shake: ws::Handshake) -> ws::Result<()> {
        self.out.send("Hello World!");
        Ok(())
    }
}

pub fn ws_server(addr: String) -> ws::Sender {
    let mut builder = Builder::new();
    builder.with_settings(Settings::default());

    let socket = builder.build(|sender| {
        ServerHandler {
            out: sender,
        }
    }).unwrap();

    let broadcaster = socket.broadcaster();

    thread::spawn(move || {
        socket.listen(addr.as_str());
    });

    broadcaster
}
