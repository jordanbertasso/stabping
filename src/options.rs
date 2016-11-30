use std::mem;
use std::path::Path;
use std::thread;
use std::sync::Arc;
use std::sync::mpsc::Sender;

use persist::{TargetManager, ManagerError};
use tcpping::run_tcpping_worker;

pub trait VecIntoRawBytes {
    fn into_raw_bytes(self) -> Vec<u8>;
}

impl VecIntoRawBytes for Vec<i32> {
    fn into_raw_bytes(mut self) -> Vec<u8> {
        let raw_ptr = self.as_mut_ptr();
        let new_len = self.len() * mem::size_of::<i32>();
        let new_cap = self.capacity() * mem::size_of::<i32>();

        unsafe {
            // take full control over memory originally controlled by orig_data
            mem::forget(self);
            Vec::from_raw_parts(raw_ptr as *mut u8, new_len, new_cap)
        }
    }
}

#[derive(RustcEncodable, RustcDecodable, Debug)]
pub struct TargetOptions {
    pub nonce: i32,
    pub addrs: Vec<String>,  // Vec of addresses (IPs to hit with TCP, files to download, etc.)
    pub interval: u32,  // interval between collection attempts, in millis
    pub avg_across: u32,  // number of sub-attempts average across for each interval
    pub pause: u32,  // pause between sub-attempts, in millis
}

pub static SENTINEL_ERROR: i32 = -2_100_000_000;
pub static SENTINEL_NODATA: i32 = -2_000_000_000;

/*
 * Data for each address. Structured as:
 * [timestamp, datapoint1, datapoint2, ..., nonce]
 *
 * where timestamp is in seconds from epoch,
 *
 * and each datapoint is for each address in TargetOptions.addrs
 * (encoding of data inside the i32 is target-defined, or one of the
 * sentinel values for error or nodata),
 *
 * and the nonce represents the state of TargetOptions when these data were
 * collected.
 */
pub struct TargetResults(pub Vec<i32>);

pub enum TargetKind {
    TcpPing,
    HttpDownload,
}

static ALL_KINDS: [TargetKind; 1] = [TargetKind::TcpPing];

impl TargetKind {
    pub fn compact_name(&self) -> &'static str {
        match *self {
            TargetKind::TcpPing => "tcpping",
            TargetKind::HttpDownload => "httpdownload",
        }
    }

    pub fn default_options(&self) -> TargetOptions {
        match *self {
            TargetKind::TcpPing => TargetOptions {
                nonce: 0,
                addrs: vec!["google.com:80".to_owned(), "8.8.8.8:53".to_owned()],
                interval: 10_000,
                avg_across: 3,
                pause: 100,
            },
            _ => unimplemented!()
        }
    }

    pub fn run_worker(&self, manager: Arc<TargetManager>,
                             results_out: Sender<TargetResults>) -> thread::JoinHandle<()> {
        match *self {
            TargetKind::TcpPing => run_tcpping_worker(manager, results_out),
            _ => unimplemented!()
        }
    }

    pub fn new_managers_for_all<'a>(data_path: &'a Path) -> Result<Vec<Arc<TargetManager>>, ManagerError> {
        let mut targets = Vec::with_capacity(ALL_KINDS.len());
        for k in ALL_KINDS.iter() {
            targets.push(
                Arc::new(try!(TargetManager::new(k, data_path)))
            );
        }
        Ok(targets)
    }
}

#[derive(RustcEncodable, RustcDecodable, Debug)]
pub struct MainConfiguration {
    pub web_port: u16,
    pub ws_port: u16,
}

impl Default for MainConfiguration {
    fn default() -> Self {
        MainConfiguration {
            web_port: 5001,
            ws_port: 5002,
        }
    }
}
