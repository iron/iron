//! Example of logger with custom formatting
extern crate iron;
extern crate logger;
extern crate term;

use std::io::net::ip::Ipv4Addr;

use iron::{Chain, ChainBuilder, Iron, IronResult, Request, Response};

use logger::Logger;
use logger::format::{Format, FunctionAttrs};

use term::attr;

// This is an example of using a format string that can specify colors and attributes
// to specific words that are printed out to the console.
fn main() {
    let format_str =
        "@[red A]URI: {uri}@@, @[blue blink underline]Method: {method}@@, @[yellow standout]Status: {status}@@, @[brightgreen]Time: {response_time}@@";
    fn attrs(req: &Request, _res: &Response) -> Vec<attr::Attr> {
        match format!("{}", req.url).as_slice() {
            "/" => vec![attr::Blink],
            _ => vec![]
        }
    }
    let mut chain = ChainBuilder::new(no_op_handler);
    chain.link(Logger::new(Format::from_format_string(format_str, &mut vec![], &mut vec![FunctionAttrs(attrs)])));
    Iron::new(chain).listen(Ipv4Addr(127, 0, 0, 1), 3000);
}

fn no_op_handler(_: &mut Request) -> IronResult<Response> {
    Ok(Response::with(iron::status::Ok, ""))
}
