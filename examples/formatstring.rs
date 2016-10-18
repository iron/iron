//! Example of logger with custom formatting
extern crate iron;
extern crate logger;

use iron::prelude::*;

use logger::Logger;
use logger::format::Format;

static FORMAT: &'static str =
    "Uri: {uri}, Method: {method}, Status: {status}, Duration: {response-time}, Time: {request-time}";

// This is an example of using a format string that can specify colors and attributes
// to specific words that are printed out to the console.
fn main() {
    let mut chain = Chain::new(no_op_handler);
    let format = Format::new(FORMAT);
    chain.link(Logger::new(Some(format.unwrap())));
    Iron::new(chain).http("localhost:3000").unwrap();
}

fn no_op_handler(_: &mut Request) -> IronResult<Response> {
    Ok(Response::with(iron::status::Ok))
}
