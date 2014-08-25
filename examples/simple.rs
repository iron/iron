extern crate iron;
extern crate http;
extern crate router;

// To build, $ cargo test
// To use, go to http://127.0.0.1:3000/test

use std::io::net::ip::Ipv4Addr;
use iron::{Server, Iron, Request, Response, Chain, Status, Unwind, FromFn};
use http::status;
use router::{Router, Params};

fn main() {
    let mut server: Server = Iron::new();
    let mut router = Router::new();

    fn handler(req: &mut Request, res: &mut Response) -> Status {
        let ref query = req.extensions.find::<Router, Params>().unwrap().find("query").unwrap();
        let _ = res.serve(status::Ok, query.as_slice());
        Unwind
    }

    // Setup our route.
    router.get("/:query", FromFn::new(handler));

    server.chain.link(router);
    server.listen(Ipv4Addr(127, 0, 0, 1), 3000);
}
