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

pub struct Worker {
    manager: Arc<Manager>,
}

impl Worker {
    fn new(manager: Arc<Manager>) -> Self {
        Worker {
            manager: manager,
        }
    }

    fn run(&self, results_out: Sender<TimePackage>) -> thread::JoinHandle<()> {
        match self.manager.kind {
            Kind::TcpPing => tcpping::run_worker(self, results_out),
        }
    }
}
