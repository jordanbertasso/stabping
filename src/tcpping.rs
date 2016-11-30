use std::thread;
use std::sync::mpsc::{channel, Sender};
use std::sync::Arc;

use std::time::Duration;
use time::precise_time_ns;
use chrono::Local;

use std::net::TcpStream;

use options::SENTINEL_ERROR;
use options::{TargetOptions, TargetResults};
use persist::TargetManager;

pub fn run_tcpping_worker(manager: Arc<TargetManager>,
                          results_out: Sender<TargetResults>) -> thread::JoinHandle<()> {
    thread::spawn(move || {
        let mut handles = Vec::new();

        loop {
            let (dur_interval, avg_across, dur_pause, num_addrs) = {
                let ref opt = manager.options_read();
                (
                    Duration::from_millis(opt.interval as u64),
                    opt.avg_across,
                    Duration::from_millis(opt.pause as u64),
                    opt.addrs.len(),
                )
            };
            let timestamp: i32 = Local::now().timestamp() as i32;

            let nonce = {
                let ref t_opt = manager.options_read();
                for addr in t_opt.addrs.iter() {
                    let a = addr.clone();

                    let (tx, rx) = channel();
                    handles.push(rx);

                    thread::spawn(move || {
                        let mut sum = 0;
                        let mut denom = 0;
                        for _ in 0..avg_across {
                            let start = precise_time_ns();
                            if TcpStream::connect(a.as_str()).is_ok() {
                                sum += precise_time_ns() - start;
                                denom += 1;
                            }
                            thread::sleep(dur_pause);
                        }

                        if denom != 0 {
                            /*
                             * send back micro-second average.
                             *
                             * we don't care if send fails as that likely means
                             * we took too long and the control thread is no longer
                             * waiting for us
                             */
                            let _ = tx.send((sum / denom / 1000) as i32);
                        }
                    });
                }
                t_opt.nonce
            };

            thread::sleep(dur_interval);

            let mut data: Vec<i32> = Vec::with_capacity(2 + num_addrs);

            data.push(manager.kind.kind_id());
            data.push(nonce);
            data.push(timestamp);

            for h in handles.drain(..) {
                if let Ok(val) = h.try_recv() {
                    data.push(val);
                } else {
                    // on error or timeout, hand back a sentinel value
                    data.push(SENTINEL_ERROR);
                }
            }

            if results_out.send(TargetResults(data)).is_err() {
                println!("Worker Control: failed to send final results back.");
            }
        }
    })
}

