#![allow(unused_mut)]
// Bug in the compiler with Arc/Mutex references.

extern crate iron;
extern crate http;
extern crate persistent;

use std::io::net::ip::Ipv4Addr;
use persistent::Persistent;
use http::status;
use iron::{Request, Response, Alloy, Iron, ServerT, Chain};
use iron::middleware::{Status, Continue, FromFn};
use iron::mixin::Serve;

pub struct HitCounter;

fn hit_counter(_: &mut Request, _: &mut Response, alloy: &mut Alloy) -> Status {
    // Bug in the compiler
    let mut count = alloy.find::<Persistent<uint, HitCounter>>().unwrap().data.write();
    *count += 1;
    println!("{} hits!", *count);
    Continue
}

fn serve_hits(_: &mut Request, res: &mut Response, alloy: &mut Alloy) -> Status {
    let mut count = alloy.find::<Persistent<uint, HitCounter>>().unwrap().data.read();
    let _ = res.serve(status::Ok, format!("{} hits!", *count).as_slice());
    Continue
}

fn main() {
    let mut server: ServerT = Iron::new();
    let counter: Persistent<uint, HitCounter> = Persistent::new(0u);
    server.chain.link(counter);
    server.chain.link(FromFn::new(hit_counter));
    server.chain.link(FromFn::new(serve_hits));
    server.listen(Ipv4Addr(127, 0, 0, 1), 3001);
}

