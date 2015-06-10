//! Example of logger with custom formatting
extern crate iron;
extern crate logger;
extern crate term;

use iron::prelude::*;

use logger::Logger;
use logger::format::Format;
use logger::format::FormatAttr::FunctionAttrs;

use term::Attr;

static FORMAT: &'static str =
    "@[red A]Uri: {uri}@, @[blue blink underline]Method: {method}@, @[yellow standout]Status: {status}@, @[brightgreen]Time: {response-time}@";

// This is an example of using a format string that can specify colors and attributes
// to specific words that are printed out to the console.
fn main() {
    fn attrs(req: &Request, _res: &Response) -> Vec<Attr> {
        match format!("{}", req.url).as_ref() {
            "/" => vec![Attr::Blink],
            _ => vec![]
        }
    }

    let mut chain = Chain::new(no_op_handler);
    let format = Format::new(FORMAT, vec![], vec![FunctionAttrs(attrs)]);
    chain.link(Logger::new(Some(format.unwrap())));
    Iron::new(chain).http("localhost:3000").unwrap();
}

fn no_op_handler(_: &mut Request) -> IronResult<Response> {
    Ok(Response::with(iron::status::Ok))
}
