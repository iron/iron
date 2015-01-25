#![allow(unstable)]
extern crate iron;
extern crate time;

use iron::prelude::*;
use iron::{Handler, BeforeMiddleware, ChainBuilder};
use iron::status;

use std::error::Error;
use std::fmt::{self, Debug};

struct ErrorHandler;
struct ErrorProducer;

#[derive(Debug)]
struct StringError(String);

impl fmt::Display for StringError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        Debug::fmt(self, f)
    }
}

impl Error for StringError {
    fn description(&self) -> &str { &*self.0 }
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
        Err(Box::new(StringError("Error".to_string())) as IronError)
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

