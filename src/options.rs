#[derive(RustcEncodable, RustcDecodable, Debug)]
pub struct TargetOptions {
    pub addrs: Vec<String>,  // Vec of addresses (IPs to hit with TCP, files to download, etc.)
    pub interval: u32,  // interval between collection attempts, in millis
    pub avg_across: u32,  // number of sub-attempts average across for each interval
    pub pause: u32,  // pause between sub-attempts, in millis
}

pub struct TargetResults {
    pub time: u32,  // seconds from epoch timestamp
    pub data: Vec<u32>,  // data for each address (encoding specific to target type)
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
