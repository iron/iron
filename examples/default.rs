//! Example of a simple logger
extern crate iron;
extern crate logger;

use iron::{Chain, ChainBuilder, Iron, IronResult, Request, Response};

use logger::Logger;


// Logger has a default formatting of the strings printed
// to console.
fn main() {
    let (logger_before, logger_after) = Logger::new(None);

    let mut chain = ChainBuilder::new(no_op_handler);

    // Link logger_before as your first before middleware.
    chain.link_before(logger_before);

    // Link logger_after as your *last* after middleware.
    chain.link_after(logger_after);

    Iron::new(chain).listen("127.0.0.1:3000");
}

fn no_op_handler(_: &mut Request) -> IronResult<Response> {
    Ok(Response::with(iron::status::Ok))
}
