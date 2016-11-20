#[derive(RustcEncodable, RustcDecodable, Debug)]
pub struct TargetOptions {
    pub addrs: Vec<String>,  // Vec of addresses (IPs to hit with TCP, files to download, etc.)
    pub interval: u32,  // interval between collection attempts, in millis
    pub avg_across: u32,  // number of sub-attempts average across for each interval
    pub pause: u32,  // pause between sub-attempts, in millis
}

pub static SENTINEL_ERROR: i32 = -2_100_000_000;
pub static SENTINEL_NODATA: i32 = -2_000_000_000;

pub struct TargetResults {
    /*
     * Data for each address. Structured as:
     * [timestamp, datapoint1, datapoint2, ...]
     * where timestamp is in seconds from epoch,
     * and each datapoint is for each address in TargetOptions.addrs
     * (encoding of data inside the i32 is target-defined, or one of the
     * sentinel values for error or nodata)
     */
    pub data: Vec<i32>,
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
