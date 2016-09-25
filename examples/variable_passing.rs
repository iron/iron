extern crate iron;
extern crate persistent;

use std::string::String;

use iron::prelude::*;

use persistent::Read;
use iron::typemap::Key;
use iron::{status};

#[derive(Copy, Clone)]
pub struct Log;
impl Key for Log { type Value = String; }


fn serve_hits(req: &mut Request) -> IronResult<Response> {
    let arc = req.get::<Read<Log>>().unwrap();
    let log_path = arc.as_ref();

    Ok(Response::with((status::Ok, format!("Hits: {}", log_path))))
}

fn main() {
    // This can be passed from command line arguments for example.
    let log_path = String::from("/path/to/a/log/file.log");
    let mut chain = Chain::new(serve_hits);
    chain.link(Read::<Log>::both(log_path));
    Iron::new(chain).http("localhost:3000").unwrap();
}

