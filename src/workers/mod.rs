mod kinds
mod tcpping

use std::thread;

pub use kinds::Kind;

pub type AddrId = u32;

#[derive(RustcEncodable, RustcDecodable, Debug)]
pub struct Options {
    pub addrs: Vec<AddrId>,  // Vec of addrs indices
    pub interval: u32,  // interval between collection attempts, in millis
}

pub struct Worker {
    kind: Kind,
    manager: Arc<Manager>,
    results_out: Sender<TimePackage>,
}

impl Worker {
    fn new(kind: Kind, manager: Arc<Manager>, results_out: Sender<TimePackage>) -> Self {
        Worker {
            kind: kind,
            manager: manager,
            results_out: results_out,
        }
    }

    fn run(&self) -> thread::JoinHandle<()> {
        match self.kind {
            Kind::TcpPing => tcpping::run_worker(self),
        }
    }
}
