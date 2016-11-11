use std::thread;
use std::path::Path;

use iron::prelude::{Request, Response, Chain, Iron, IronResult};
use iron::status;
use router::Router;
use mount::Mount;
use staticfile::Static;

pub fn web_server(addr: String) -> thread::JoinHandle<()> {
    fn hello_handler(req: &mut Request) -> IronResult<Response> {
        Ok(Response::with((status::Ok, "Hello World!")))
    }

    let mut router = Router::new();
    router.get("/", Static::new(Path::new("client/index.html")), "index");
    router.get("/hello", hello_handler, "hello");

    let mut mount = Mount::new();
    mount.mount("/", router);
    mount.mount("/assets/", Static::new(Path::new("client/")));

    let iron = Iron::new(mount);

    thread::spawn(move || {
        iron.http(addr.as_str()).unwrap();
    })
}
