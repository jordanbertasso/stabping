use std::thread;
use std::sync::{Arc, Mutex, RwLock};

use ws;
use ws::{Factory, Handler, Settings, Builder, WebSocket, Handshake};

use options::{TargetOptions, MainConfiguration};

struct ServerHandler {
    out: ws::Sender,
}

impl Handler for ServerHandler {
    fn on_open(&mut self, _: Handshake) -> ws::Result<()> {
        println!("Websocket connection opened.");
        Ok(())
    }
}

struct ServerFactory;

impl ServerFactory {
    fn new() -> ServerFactory {
        ServerFactory {}
    }
}

impl Factory for ServerFactory {
    type Handler = ServerHandler;

    fn connection_made(&mut self, sender: ws::Sender) -> ServerHandler {
        ServerHandler {
            out: sender
        }
    }
}


pub enum BroadcastError {
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

fn get_socket() -> WebSocket<ServerFactory> {
    let mut builder = Builder::new();
    builder.with_settings(Settings::default());
    builder.build(ServerFactory::new()).unwrap()
}

pub fn ws_server(configuration: Arc<RwLock<MainConfiguration>>,
                 broadcaster: Arc<Broadcaster>) -> thread::JoinHandle<()> {
    let ws_port = configuration.read().unwrap().ws_port;
    thread::spawn(move || {
        loop {
            let socket = get_socket();
            broadcaster.update(socket.broadcaster());
            println!("WebSocket server (re)listening on port {}.", ws_port);
            socket.listen(("0.0.0.0", ws_port))
                  .expect("Unable to listen on websocket.");
        }
    })
}
