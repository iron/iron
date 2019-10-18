//! Example of a simple logger
extern crate iron;
extern crate logger;
extern crate env_logger;

use iron::prelude::*;
use logger::Logger;

// Logger has a default formatting of the strings printed
// to console.
fn main() {
    env_logger::init();

    let (logger_before, logger_after) = Logger::new(None);

    let mut chain = Chain::new(no_op_handler);

    // Link logger_before as your first before middleware.
    chain.link_before(logger_before);

    // Link logger_after as your *last* after middleware.
    chain.link_after(logger_after);

    println!("Run `RUST_LOG=logger=info cargo run --example default` to see logs.");
    match Iron::new(chain).http("127.0.0.1:3000") {
        Result::Ok(listening) => println!("{:?}", listening),
        Result::Err(err) => panic!("{:?}", err),
    }
}

fn no_op_handler(_: &mut Request) -> IronResult<Response> {
    Ok(Response::with(iron::status::Ok))
}
