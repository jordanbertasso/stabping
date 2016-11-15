use std::thread;
use std::path::Path;
use std::error::Error;
use std::fmt;

use iron::prelude::{Request, Response, Chain, Iron, IronResult, IronError};
use iron::headers::ContentType;
use iron::mime::Mime;
use iron::modifiers::Header;
use iron::status;
use router::Router;
use mount::Mount;
use staticfile::Static;

#[derive(Debug)]
struct NotFoundError;

impl Error for NotFoundError {
    fn description(&self) -> &str {
        "Not found."
    }
}
impl fmt::Display for NotFoundError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(self.description())
    }
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
        Some((s, ct)) => Ok(Response::with(
                (status::Ok, Header(ContentType(ct.parse().unwrap())), s))),
        None => Err(IronError::new(NotFoundError, status::NotFound))
    }
}

pub fn web_server(addr: String) -> thread::JoinHandle<()> {
    fn hello_handler(req: &mut Request) -> IronResult<Response> {
        Ok(Response::with((status::Ok, "Hello World!")))
    }

    let mut router = Router::new();
    router.get("/", webassets_handler, "index");
    router.get("/hello", hello_handler, "hello");

    let mut mount = Mount::new();
    mount.mount("/", router);
    mount.mount("/assets/", webassets_handler);

    let iron = Iron::new(mount);

    thread::spawn(move || {
        iron.http(addr.as_str()).unwrap();
    })
}
