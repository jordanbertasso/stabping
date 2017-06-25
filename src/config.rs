/*
 * Copyright 2016-2017 icasdri
 *
 * This file is part of stabping. The original source code for stabping can be
 * found at <https://github.com/icasdri/stabping>. See COPYING for licensing
 * details.
 */

#[derive(RustcEncodable, RustcDecodable, Debug)]
pub struct Config {
    pub web_port: u16,
    pub ws_port: u16,
}

impl Default for Config {
    fn default() -> Self {
        MainConfiguration {
            web_port: 5001,
            ws_port: 5002,
        }
    }
}
