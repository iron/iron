//! Example of a simple logger
extern crate iron;
extern crate logger;

use std::io::net::ip::Ipv4Addr;

use iron::{Chain, ChainBuilder, Iron, IronResult, Request, Response};

use logger::Logger;

// Logger has a default formatting of the strings printed
// to console.
fn main() {
    let mut chain = ChainBuilder::new(no_op);
    let (logger_before, logger_after) = Logger::middlewares(None);
    chain.link_before(logger_before);
    // You could chain.link these both at once, but you probably want to make
    // these the first and last middlewares in your chain.
    chain.link_after(logger_after);
    Iron::new(chain).listen(Ipv4Addr(127, 0, 0, 1), 3000);
    fn no_op(_: &mut Request) -> IronResult<Response> {
        Ok(Response::with(iron::status::Ok, ""))
    }
}
