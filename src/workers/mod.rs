mod kinds;
mod tcpping;

use std::thread;
use std::sync::{Arc, RwLock};
use std::sync::mpsc::{channel, Sender};

use data::TimePackage;
use manager::Manager;

pub use self::kinds::Kind;

pub type AddrId = u32;

#[derive(RustcEncodable, RustcDecodable, Debug)]
pub struct Options {
    pub addrs: Vec<AddrId>,  // Vec of addrs indices
    pub interval: u32,  // interval between collection attempts, in millis
}


fn run_worker(manager: Arc<Manager>, results_out: Sender<TimePackage>) -> thread::JoinHandle<()> {
    match manager.kind {
        Kind::TcpPing => tcpping::run_worker(manager, results_out),
    }
}
