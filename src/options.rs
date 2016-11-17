#[derive(RustcEncodable, RustcDecodable, Debug)]
pub struct TargetOptions {
    pub addrs: Vec<String>,  // Vec of addresses (IPs to hit with TCP, files to download, etc.)
    pub interval: u32,  // interval between collection attempts, in millis
    pub avg_across: u32,  // number of sub-attempts average across for each interval
    pub pause: u32,  // pause between sub-attempts, in millis
}

pub struct TargetResults {
    /*
     * Data for each address. Structured as:
     * [timestamp, datapoint1, datapoint2, ...]
     * where timestamp is in seconds from epoch,
     * and each datapoint is for each address in TargetOptions.addrs
     * (encoding of data inside the u32 is target-defined)
     */
    pub data: Vec<u32>,
}

#[derive(RustcEncodable, RustcDecodable, Debug)]
pub struct SPOptions {
    pub tcpping_options: TargetOptions,
}

impl Default for SPOptions {
    fn default() -> Self {
        SPOptions {
            tcpping_options: TargetOptions {
                addrs: vec!["google.com:80".to_owned(), "8.8.8.8:53".to_owned()],
                interval: 10_000,
                avg_across: 3,
                pause: 100,
            }
        }
    }
}

#[derive(RustcEncodable, RustcDecodable, Debug)]
pub struct MainConfiguration {
    pub web_listen: String,
    pub ws_listen: String,
}

impl Default for MainConfiguration {
    fn default() -> Self {
        MainConfiguration {
            web_listen: "localhost:5001".to_owned(),
            ws_listen: "localhost:5002".to_owned(),
        }
    }
}
