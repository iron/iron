#![feature(globs)]
extern crate iron;
extern crate persistent;

use iron::prelude::*;

use persistent::Write;
use iron::typemap::Assoc;
use iron::ChainBuilder;
use iron::status;
use iron::response::modifiers::{Status, Body};

pub struct HitCounter;
impl Assoc<uint> for HitCounter {}

fn serve_hits(req: &mut Request) -> IronResult<Response> {
    let mutex = req.get::<Write<HitCounter, uint>>().unwrap();
    let mut count = mutex.lock();

    *count += 1;
    Ok(Response::new().set(Status(status::Ok)).set(Body(format!("Hits: {}", *count))))
}

fn main() {
    let mut chain = ChainBuilder::new(serve_hits);
    chain.link(Write::<HitCounter, uint>::both(0u));
    Iron::new(chain).listen("localhost:3000").unwrap();
}

