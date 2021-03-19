extern crate iron;
extern crate persistent;

use std::string::String;

use iron::prelude::*;

use iron::typemap::Key;
use iron::StatusCode;
use persistent::Read;

#[derive(Copy, Clone)]
pub struct Log;
impl Key for Log {
    type Value = String;
}

fn serve_hits(req: &mut Request) -> IronResult<Response> {
    let arc = req.get::<Read<Log>>().unwrap();
    let log_path = arc.as_ref();

    Ok(Response::with((
        StatusCode::OK,
        format!("Hits: {}", log_path),
    )))
}

fn main() {
    // This can be passed from command line arguments for example.
    let log_path = String::from("/path/to/a/log/file.log");
    let mut chain = Chain::new(serve_hits);
    chain.link(Read::<Log>::both(log_path));
    Iron::new(chain).http("localhost:3000");
}
