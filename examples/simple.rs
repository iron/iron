extern crate iron;
extern crate http;
extern crate router;

// To build, $ cargo test
// To use, go to http://127.0.0.1:3000/test

use std::io::net::ip::Ipv4Addr;
use iron::{Iron, Request, Response, IronResult, Set};
use iron::status;
use iron::response::modifiers::{Status, Body};
use router::{Router, Params};

fn main() {
    let mut router = Router::new();
    router.get("/", handler);
    router.get("/:query", handler);

    Iron::new(router).listen(Ipv4Addr(127, 0, 0, 1), 3000);

    fn handler(req: &mut Request) -> IronResult<Response> {
        let ref query = req.extensions.find::<Router, Params>().unwrap().find("query").unwrap_or("/");
        Ok(Response::new().set(Status(status::Ok)).set(Body(*query)))
    }
}
