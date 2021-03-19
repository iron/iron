extern crate iron;

use iron::prelude::*;
use iron::StatusCode;
use iron::{BeforeMiddleware, Handler};

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

impl Error for StringError {}

impl Handler for ErrorHandler {
    fn handle(&self, _: &mut Request) -> IronResult<Response> {
        // This is never called!
        //
        // If a BeforeMiddleware returns an error through Err(...),
        // and it is not handled by a subsequent BeforeMiddleware in
        // the chain, the main handler is not invoked.
        Ok(Response::new())
    }
}

impl BeforeMiddleware for ErrorProducer {
    fn before(&self, _: &mut Request) -> IronResult<()> {
        Err(IronError::new(
            StringError("Error".to_string()),
            StatusCode::BAD_REQUEST,
        ))
    }
}

fn main() {
    // Handler is attached here.
    let mut chain = Chain::new(ErrorHandler);

    // Link our error maker.
    chain.link_before(ErrorProducer);

    Iron::new(chain).http("localhost:3000");
}
