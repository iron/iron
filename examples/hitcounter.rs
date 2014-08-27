extern crate iron;
extern crate typemap;
extern crate persistent;

use std::io::net::ip::Ipv4Addr;
use persistent::Write;
use typemap::Assoc;
use iron::{Request, Response, Iron, Chain, Plugin, IronResult, ChainBuilder};
use iron::status;

pub struct HitCounter;
impl Assoc<uint> for HitCounter {}

fn serve_hits(req: &mut Request) -> IronResult<Response> {
    let mutex = req.get::<Write<HitCounter, uint>>().unwrap();
    let mut count = mutex.lock();

    *count += 1;
    Ok(Response::with(status::Ok, format!("Hits: {}", *count)))
}

fn main() {
    let mut chain = ChainBuilder::new(serve_hits);
    chain.link(Write::<HitCounter, uint>::both(0u));
    Iron::new(chain).listen(Ipv4Addr(127, 0, 0, 1), 3000);
}

