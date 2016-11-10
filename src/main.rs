extern crate chrono;
extern crate time;

extern crate iron;
#[macro_use(router)] extern crate router;
extern crate staticfile;

mod tcpping;

use std::path::Path;

use iron::prelude::{Request, Response, Chain, Iron, IronResult};
use iron::status;
use router::Router;
use staticfile::Static;

fn main() {
    fn hello_handler(req: &mut Request) -> IronResult<Response> {
        Ok(Response::with((status::Ok, "Hello World!")))
    }

    let router = router!(index: get "/" => Static::new(Path::new("client/index.html")),
                         hello: get "/hello" => hello_handler);
    Iron::new(router).http("localhost:5001").unwrap();
}
