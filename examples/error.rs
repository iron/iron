#![feature(phase)]

extern crate iron;
extern crate http;
extern crate time;
#[phase(plugin, link)] extern crate log;

use std::io::net::ip::Ipv4Addr;
use std::fmt::Show;
use iron::{Iron, Chain, Request, Response,
           Middleware, Server, Status,
           Error, FromFn};
use http::status;

#[deriving(Clone)]
struct ErrorHandler;

impl ErrorHandler { fn new() -> ErrorHandler { ErrorHandler } }

impl Middleware for ErrorHandler {
    fn on_error(&mut self,
                _: &mut Request,
                res: &mut Response,
                _: &mut Show) {
        error!("Error when handling request.");
        res.serve(status::InternalServerError, "Internal Server Error.");
    }

}

fn error(_: &mut Request, _: &mut Response) -> Status {
    Error(box "Error!".to_string() as Box<Show>)
}

fn main() {
    let mut server: Server = Iron::new();

    server.chain.link(ErrorHandler::new());
    server.chain.link(FromFn::new(error));

    // Start the server on localhost:3000
    server.listen(Ipv4Addr(127, 0, 0, 1), 3000);
}

