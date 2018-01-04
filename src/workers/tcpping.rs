/*
 * Copyright 2016-2017 icasdri
 *
 * This file is part of stabping. The original source code for stabping can be
 * found at <https://github.com/icasdri/stabping>. See COPYING for licensing
 * details.
 */

use std::thread;
use std::sync::mpsc::{channel, Sender};
use std::sync::Arc;
use std::net::TcpStream;
use std::f32::NAN;

use std::time::Duration;
use time::precise_time_ns;
use chrono::Local;

use data::{DataElement, TimePackage};
use manager::{Manager, Feed};
use super::AddrId;

/**
 * TCP Ping worker logic
 */
pub fn run_worker(manager: Arc<Manager>, results_out: Sender<TimePackage>) -> thread::JoinHandle<()> {
    // start a new thread for the worker
    thread::spawn(move || {
        let mut handles = Vec::new();

        // continue to collect data forever
        loop {
            // retrieve the target's current options
            let ref opt = manager.options_read();
            let dur_interval = Duration::from_millis(opt.interval as u64);

            // get the current time (to timestamp this round of data with)
            let timestamp: u32 = Local::now().timestamp() as u32;

            for addr_i in opt.addrs.iter() {
                /*
                 * create channels so the per-addr threads can send back
                 * their data to the worker thread
                 */
                let (tx, rx) = channel();
                handles.push((*addr_i, rx));

                // obtain the address string from the address index
                let addr = manager.index_read().get_addr(*addr_i).unwrap().clone();

                /*
                 * spawn a thread to actually collect the data for each
                 * separate address
                 */
                thread::spawn(move || {
                    let start = precise_time_ns();

                    let dur = if TcpStream::connect(addr.as_str()).is_ok() {
                        (((precise_time_ns() - start) / 100) as f32) / 10_000.
                    } else {
                        NAN
                    };

                    /*
                     * send back milli-second duration
                     *
                     * we don't care if send fails as that likely means
                     * we took too long and the control thread is no longer
                     * waiting for us
                     */
                    let _ = tx.send(dur);
                });
            }

            // release mutex on options before waiting
            drop(opt);

            /*
             * wait out the designated data-collectiong interval, while giving
             * the per-addr subthreads the entire interval of time to come back
             */
            thread::sleep(dur_interval);

            let mut package = TimePackage::new(manager.kind, Feed::Raw);

            // read back the data from the per-addr subthreads
            for (addr_i, h) in handles.drain(..) {
                package.insert(DataElement {
                    time: timestamp,
                    index: addr_i as AddrId,
                    val: h.recv().unwrap_or(NAN),
                    sd: NAN,
                }).expect("tcpping: unexpected error during result packaging");
            }

            // send off our results to the main thread
            if results_out.send(package).is_err() {
                println!("tcpping: failed to send final results back");
            }
        }
    })
}

