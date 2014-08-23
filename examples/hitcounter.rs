#![allow(unused_mut)]
// Bug in the compiler with Arc/Mutex references.

extern crate iron;
extern crate http;
extern crate typemap;
extern crate persistent;

use std::io::net::ip::Ipv4Addr;
use std::sync::{Arc, RWLock};
use persistent::Persistent;
use typemap::Assoc;
use http::status;
use iron::{Request, Response, Iron, Server, Chain, Status, Continue, FromFn};

pub struct HitCounter;
impl Assoc<Arc<RWLock<uint>>> for HitCounter {}

fn hit_counter(req: &mut Request, _: &mut Response) -> Status {
    let mut count = req.extensions.find::<HitCounter, Arc<RWLock<uint>>>().unwrap().write();
    *count += 1;
    println!("{} hits!", *count);
    Continue
}

fn serve_hits(req: &mut Request, res: &mut Response) -> Status {
    let mut count = req.extensions.find::<HitCounter, Arc<RWLock<uint>>>().unwrap().read();
    let _ = res.serve(status::Ok, format!("{} hits!", *count));
    Continue
}

fn main() {
    let mut server: Server = Iron::new();
    let counter: Persistent<uint, HitCounter> = Persistent::new(0u);
    server.chain.link(counter);
    server.chain.link(FromFn::new(hit_counter));
    server.chain.link(FromFn::new(serve_hits));
    server.listen(Ipv4Addr(127, 0, 0, 1), 3001);
}
