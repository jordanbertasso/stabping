use std::thread;
use std::error::Error;
use std::fmt;
use std::io::Read;
use std::sync::Arc;
use std::sync::RwLock;

use iron::prelude::{Request, Response, Iron, IronResult, IronError};
use iron::middleware::Handler;
use iron::method::Method;
use iron::headers::ContentType;
use iron::modifiers::Header;
use iron::request::Body;
use iron::status;
use router::Router;
use mount::Mount;

use rustc_serialize::{json, Decodable};

use reader::{SPDataReader, DataRequest};
use persist::{TargetManager, ManagerError};
use options::{MainConfiguration, TargetOptions};

#[derive(Debug)]
enum SPWebError {
    NotFound,
    InvalidMethod,
    NotImplemented,
    BadRequest,
    ServerError,
    NonceConflict,
}

impl Error for SPWebError {
    fn description(&self) -> &str {
        match *self {
            SPWebError::NotFound => "Resource not found.",
            SPWebError::InvalidMethod => "Invalid method.",
            SPWebError::NotImplemented => "Handler not yet implemented.",
            SPWebError::BadRequest => "Bad request (malformed or missing fields).",
            SPWebError::ServerError => "Server encountered an error.",
            SPWebError::NonceConflict => "The nonce given does not match the current nonce, refusing update.",
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

trait JsonBody {
    fn read_json<T: Decodable>(&mut self) -> Result<T, IronError>;
}

impl<'a, 'b> JsonBody for Body<'a, 'b> {
    fn read_json<T: Decodable>(&mut self) -> Result<T, IronError> {
        let mut buf = String::new();

        try!(
            self.read_to_string(&mut buf)
            .map_err(|_| {
                println!("Failed to read request body.");
                IronError::new(SPWebError::ServerError, status::InternalServerError)
            })
        );

        Ok(try!(
            json::decode::<T>(&buf)
            .map_err(|_| {
                println!("Failed to parse request body.");
                IronError::new(SPWebError::BadRequest, status::BadRequest)
            })
        ))
    }
}

struct TargetHandler {
    manager: Arc<TargetManager>,
}

impl TargetHandler {
    fn new(manager: Arc<TargetManager>) -> Self {
        TargetHandler {
            manager: manager,
        }
    }
}

impl Handler for TargetHandler {
    fn handle(&self, req: &mut Request) -> IronResult<Response> {
        match req.method {
            Method::Get => {
                // retrieve options
                println!("Request for {} options.", self.manager.kind.compact_name());
                let options_ser = {
                    let options_guard = self.manager.options_read();
                    json::encode(&*options_guard).unwrap()
                };
                Ok(Response::with((status::Ok, options_ser)))
            },
            Method::Post => {
                // retrieve data
                let dr: DataRequest = try!(req.body.read_json());
                println!("Request for {} data: {:?}", self.manager.kind.compact_name(), dr);

                let body_writer = try!(
                    SPDataReader::new(self.manager.clone(), dr)
                    .ok_or_else(|| {
                        println!("Failed to create SPDataReader.");
                        IronError::new(SPWebError::BadRequest, status::BadRequest)
                    })
                );

                let r = Response::with((status::Ok));
                Ok(Response {
                    status: r.status,
                    headers: r.headers,
                    extensions: r.extensions,
                    body: Some(Box::new(body_writer)),
                })
            },
            Method::Put => {
                // update options
                let mut new_options: TargetOptions = try!(req.body.read_json());
                if new_options.nonce != self.manager.options_read().nonce {
                    return Err(IronError::new(SPWebError::NonceConflict, status::Conflict));
                }

                {
                    let (n, over) = new_options.nonce.overflowing_add(1);
                    if over {
                        new_options.nonce = 0;
                    } else {
                        new_options.nonce = n;
                    }
                }

                try!(
                    self.manager.options_update(new_options)
                    .map_err(|_| IronError::new(SPWebError::ServerError, status::InternalServerError))
                );
                Ok(Response::with((status::Ok)))
            },
            _ => Err(IronError::new(SPWebError::InvalidMethod, status::MethodNotAllowed))
        }
    }
}


pub fn web_server<'a, T>(configuration: Arc<RwLock<MainConfiguration>>,
                         targets: T) -> thread::JoinHandle<()>
                         where T: Iterator<Item=&'a Arc<TargetManager>> {
    let ws_port_str = format!("{}", configuration.read().unwrap().ws_port);
    let ws_port_handler = move |_: &mut Request| -> IronResult<Response> {
        Ok(Response::with((status::Ok, ws_port_str.as_str())))
    };

    let mut router = Router::new();
    router.get("/", webassets_handler, "index");
    router.get("/api/config/ws_port", ws_port_handler, "api_config_ws_port");

    for tm in targets {
        router.any(format!("/api/target/{}", tm.kind.compact_name()),
                   TargetHandler::new(tm.clone()),
                   format!("target_{}", tm.kind.compact_name()));
    }

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
