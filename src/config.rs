/*
 * Copyright 2016-2017 icasdri
 *
 * This file is part of stabping. The original source code for stabping can be
 * found at <https://github.com/icasdri/stabping>. See COPYING for licensing
 * details.
 */

use std::env;
use std::path::PathBuf;
use std::fs;
use std::fs::{OpenOptions, File};
use std::sync::Arc;
use std::sync::RwLock;

use helpers::{SPIOError, SPFile};

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

static CONFIG_FILENAME: &'static str = "stabping_config.json";

/**
 * Attempts to discover the configuration file and associated data directory.
 *
 * Returns a tuple of an `Arc` to the `MainConfiguration` and the path to the
 * data directory if found.
 */
fn get_configuration() -> Option<(Arc<RwLock<Config>>, PathBuf)> {
    /*
     * the list of (description, path) tuples of directories to try/places we
     * want to check for the existence of the configuration file
     */
    let dirs_to_try = &[
        ("current working directory",
         env::current_dir().ok().map(|mut cur| { cur.push(CONFIG_FILENAME); cur })),
        ("user configuration directory",
         env::home_dir().map(|mut home| { home.push(".config"); home.push(CONFIG_FILENAME); home })),
        ("global configuration directory",
         Some({ let mut p = PathBuf::from("/etc"); p.push(CONFIG_FILENAME); p })),
        ("directory where stabping is located",
         env::current_exe().ok().map(|mut exe| { exe.pop(); exe.push(CONFIG_FILENAME); exe })),
    ];

    println!("Searching for configuration file '{}'.", CONFIG_FILENAME);

    // loop through all the directories we want to try
    for &(desc, ref maybe_p) in dirs_to_try {
        if let &Some(ref p) = maybe_p {
            /*
             * if we could obtain a path to this location, try and open the
             * configuration file that might be there
             */
            println!("- checking {}:\n    {}", desc, p.to_str().unwrap());
            if let Ok(mut file) = File::open_from(OpenOptions::new().read(true), &p) {
                match file.read_json_p(&p) {
                    Err(err @ SPIOError::Parse(_)) => {
                        /*
                         * if we found the file, could open it, but it was not
                         * filled with JSON, then tell the user
                         */
                        println!(
                            "\n{} configuration file. Invalid or missing JSON fields. Please ensure that this file is formatted like:\n{}\n",
                            err.description(),
                            json::as_pretty_json(&MainConfiguration::default())
                        );
                        return None
                    },
                    Ok(mc) => {
                        /*
                         * we found a valid configuration file
                         */
                        println!("\nUsing configuration file in {}:\n  {}",
                                 desc, p.to_str().unwrap());
                        let mut data_path = p.clone();
                        data_path.pop();
                        data_path.push("stabping_data");
                        if fs::create_dir_all(&data_path).is_err() {
                            println!("Failed to create data directory '{}'. Please ensure this directory is writable by stabping.", data_path.to_str().unwrap());
                            return None;
                        }
                        return Some((Arc::new(RwLock::new(mc)), data_path));
                    },
                    _ => {
                        /*
                         * we ran into some other issue with what looked like
                         * the configuration file, continue to try other
                         * locations
                         */
                    }
                };
            }
        } else {
            /*
             * we couldn't obtain the path to this location, continue to try
             * other locations
             */
            println!("- could not obtain {}", desc);
        }
    }

    /*
     * we looked everywhere and couldn't find the configuration file, tell the
     * user
     */
    println!(
        "\nFailed to find configuration file. Please ensure that 'stabping_config.json' is accessible in one of the above checked locations, and is formatted like:\n{}\n",
        json::as_pretty_json(&MainConfiguration::default())
    );
    None
}
