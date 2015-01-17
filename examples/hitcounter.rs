extern crate iron;
extern crate persistent;

use iron::prelude::*;

use persistent::Write;
use iron::typemap::Key;
use iron::{ChainBuilder, status};

#[derive(Copy)]
pub struct HitCounter;

impl Key for HitCounter { type Value = usize; }

fn serve_hits(req: &mut Request) -> IronResult<Response> {
    let mutex = req.get::<Write<HitCounter>>().unwrap();
    let mut count = mutex.lock().unwrap();

    *count += 1;
    Ok(Response::with((status::Ok, format!("Hits: {}", *count))))
}

fn main() {
    let mut chain = ChainBuilder::new(serve_hits);
    chain.link(Write::<HitCounter>::both(0));
    Iron::new(chain).listen("localhost:3000").unwrap();
}

