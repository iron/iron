#![feature(globs)]

extern crate iron;
extern crate time;

use iron::prelude::*;
use iron::{Handler, BeforeMiddleware, ChainBuilder};
use iron::status;

use std::error::Error;

struct ErrorHandler;
struct ErrorProducer;

#[deriving(Show)]
struct StringError(String);

impl Error for StringError {
    fn description(&self) -> &str { self.0.as_slice() }
}

impl Handler for ErrorHandler {
    fn call(&self, _: &mut Request) -> IronResult<Response> {
        Ok(Response::new())
    }

    fn catch(&self, _: &mut Request, err: IronError) -> (Response, IronResult<()>) {
        (Response::with((status::InternalServerError, "Internal Server Error.")),
         Err(err))
    }

}

impl BeforeMiddleware for ErrorProducer {
    fn before(&self, _: &mut Request) -> IronResult<()> {
        Err(box StringError("Error".to_string()) as IronError)
    }
}

fn main() {
    // Handler is attached here.
    let mut chain = ChainBuilder::new(ErrorHandler);

    // Link our error maker.
    chain.link_before(ErrorProducer);

    Iron::new(chain).listen("localhost:3000").unwrap();
    println!("On 3000");
}

