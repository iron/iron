extern crate iron;
extern crate time;

use std::io::net::ip::Ipv4Addr;
use iron::{Iron, Chain, Request, Response, Middleware, Alloy, ServerT, Status, Continue};

use time::precise_time_ns;

#[deriving(Clone)]
struct ResponseTime {
    entry: u64
}

impl ResponseTime { fn new() -> ResponseTime { ResponseTime { entry: 0u64 } } }

impl Middleware for ResponseTime {
    fn enter(&mut self, _req: &mut Request, _res: &mut Response, _al: &mut Alloy) -> Status {
        self.entry = precise_time_ns();
        Continue
    }

    fn exit(&mut self, _req: &mut Request, _res: &mut Response, _al: &mut Alloy) -> Status {
        let delta = precise_time_ns() - self.entry;
        println!("Request took: {} ms", (delta as f64) / 100000.0);
        Continue
    }
}

fn main() {
    let mut server: ServerT = Iron::new();

    // This adds the ResponseTime middleware so that
    // all requests and responses are passed through it.
    server.chain.link(ResponseTime::new());

    // Start the server on localhost:3000
    server.listen(Ipv4Addr(127, 0, 0, 1), 3000);
}

