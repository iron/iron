extern crate iron;
extern crate logger;
extern crate term;

use std::io::net::ip::Ipv4Addr;

use iron::{Iron, ServerT, Request, Response, Chain};

use logger::Logger;
use logger::format::{Format, FunctionAttrs};

use term::attr;

fn main() {
    let format_str =
        "@[red A]URI: {uri}@@, @[blue blink underline]Method: {method}@@, @[yellow standout]Status: {status}@@, @[brightgreen]Time: {response_time}@@";
    fn attrs(req: &Request, _res: &Response) -> Vec<attr::Attr> {
        match format!("{}", req.request_uri).as_slice() {
            "/" => vec![attr::Blink],
            _ => vec![]
        }
    }
    let logger = Logger::new(Format::from_format_string(format_str, &mut vec![], &mut vec![FunctionAttrs(attrs)]));
    let mut server: ServerT = Iron::new();
    server.chain.link(logger);
    server.listen(Ipv4Addr(127, 0, 0, 1), 3000);
}
