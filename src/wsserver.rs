use std::thread;
use std::net::ToSocketAddrs;
use std::fmt::Debug;
use std::sync::{Arc, Mutex, RwLock};

use ws;
use ws::{Builder, Factory, Handler, Settings, WebSocket};

use options::{TargetOptions, SPOptions, MainConfiguration};

struct ServerHandler {
    out: ws::Sender,
}

impl ws::Handler for ServerHandler {
    fn on_open(&mut self, shake: ws::Handshake) -> ws::Result<()> {
        println!("Websocket connection opened.");
        self.out.send("Hello World!");
        Ok(())
    }
}

struct ServerFactory;

impl ServerFactory {
    fn new() -> ServerFactory {
        ServerFactory {}
    }
}

impl ws::Factory for ServerFactory {
    type Handler = ServerHandler;

    fn connection_made(&mut self, sender: ws::Sender) -> ServerHandler {
        ServerHandler {
            out: sender
        }
    }
}


enum BroadcastError {
    SocketNotAvail,
    WebSocketError(ws::Error)
}

pub struct Broadcaster {
    sender: Mutex<Option<ws::Sender>>,
}

impl Broadcaster {
    pub fn new() -> Broadcaster {
        Broadcaster {
            sender: Mutex::new(None),
        }
    }

    fn update(&self, new_sender: ws::Sender) {
        let mut guard = self.sender.lock().unwrap();
        *guard = Some(new_sender);
    }

    pub fn send<M>(&self, msg: M) -> Result<(), BroadcastError> where M: Into<ws::Message> {
        let guard = self.sender.lock().unwrap();
        if let Some(ref b) = *guard {
            b.send(msg).map_err(|e| BroadcastError::WebSocketError(e))
        } else {
            Err(BroadcastError::SocketNotAvail)
        }
    }
}

fn get_socket() -> ws::WebSocket<ServerFactory> {
    let mut builder = Builder::new();
    builder.with_settings(Settings::default());
    builder.build(ServerFactory::new()).unwrap()
}

pub fn ws_server(configuration: Arc<RwLock<MainConfiguration>>,
                 _: Arc<RwLock<SPOptions>>,
                 broadcaster: Arc<Broadcaster>) -> thread::JoinHandle<()> {
    thread::spawn(move || {
        loop {
            let socket = get_socket();
            broadcaster.update(socket.broadcaster());
            println!("New WebSocket created to accept connections");
            socket.listen(configuration.read().unwrap().ws_listen.as_str());
        }
    })
}
