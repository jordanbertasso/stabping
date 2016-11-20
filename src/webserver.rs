use std::thread;
use std::error::Error;
use std::fmt;
use std::sync::Arc;
use std::sync::RwLock;

use iron::prelude::{Request, Response, Iron, IronResult, IronError};
use iron::middleware::Handler;
use iron::method::Method;
use iron::headers::ContentType;
use iron::modifiers::Header;
use iron::response::WriteBody;
use iron::status;
use router::Router;
use mount::Mount;

use rustc_serialize::json;

use options::{SPOptions, MainConfiguration};

#[derive(Debug)]
enum SPWebError {
    NotFound,
    InvalidMethod,
    NotImplemented,
}

impl Error for SPWebError {
    fn description(&self) -> &str {
        match *self {
            SPWebError::NotFound => "Resource not found.",
            SPWebError::InvalidMethod => "Invalid method.",
            SPWebError::NotImplemented => "Handler not yet implemented.",
        }
    }
}

impl fmt::Display for SPWebError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(self.description())
    }
}


enum WebAssetContainer {
    Binary(&'static [u8]),
    Text(&'static str)
}

include!(concat!(env!("OUT_DIR"), "/webassets_handler_body.rs"));

fn webassets_handler(req: &mut Request) -> IronResult<Response> {
    let path = {
        let p = req.url.path()[0];
        if p.len() > 0 {
            p
        } else {
            "index.html"
        }
    };

    match _webassets_handler_body(path) {
        Some((asset, ct)) => {
            let s = status::Ok;
            let h = Header(ContentType(ct.parse().unwrap()));
            Ok(match asset {
                WebAssetContainer::Binary(b) => Response::with((s, h, b)),
                WebAssetContainer::Text(t) => Response::with((s, h, t)),
            })
        },
        None => Err(IronError::new(SPWebError::NotFound, status::NotFound))
    }
}

struct OptionsHandler {
    options: Arc<RwLock<SPOptions>>,
}

impl OptionsHandler {
    fn new(options: Arc<RwLock<SPOptions>>) -> Self {
        OptionsHandler {
            options: options,
        }
    }
}

impl Handler for OptionsHandler {
    fn handle(&self, req: &mut Request) -> IronResult<Response> {
        match req.method {
            Method::Get => {
                let options_ser = {
                    let options = self.options.read().unwrap();
                    json::encode(&*options).unwrap()
                };
                Ok(Response::with((status::Ok, options_ser)))
            },
            Method::Post => {
                // TODO
                Err(IronError::new(SPWebError::NotImplemented, status::NotImplemented))
            },
            _ => Err(IronError::new(SPWebError::InvalidMethod, status::MethodNotAllowed))
        }
    }
}


pub fn web_server(configuration: Arc<RwLock<MainConfiguration>>,
                  options: Arc<RwLock<SPOptions>>) -> thread::JoinHandle<()> {
    let ws_port_str = format!("{}", configuration.read().unwrap().ws_port);
    let ws_port_handler = move |_: &mut Request| -> IronResult<Response> {
        Ok(Response::with((status::Ok, ws_port_str.as_str())))
    };

    let mut router = Router::new();
    router.get("/", webassets_handler, "index");
    router.get("/api/config/ws_port", ws_port_handler, "api_config_ws_port");
    router.any("/api/options", OptionsHandler::new(options.clone()), "api_options");

    let mut mount = Mount::new();
    mount.mount("/", router);
    mount.mount("/assets/", webassets_handler);

    let iron = Iron::new(mount);

    let web_port = configuration.read().unwrap().web_port;
    thread::spawn(move || {
        println!("Web server listening on port {}.", web_port);
        iron.http(("0.0.0.0", web_port)).unwrap();
    })
}
