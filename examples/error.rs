extern crate iron;

use iron::prelude::*;
use iron::{Handler, BeforeMiddleware, AfterMiddleware};
use iron::status;

use std::error::Error;
use std::fmt::{self, Debug};

struct NeverCalledHandler;
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

impl Handler for NeverCalledHandler {
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
        Err(IronError::new(StringError("You cannot access the resource!".to_string()), status::BadRequest))
    }
}

// the error is simply printed on stdout and returned as is
struct ErrorHandler;
impl AfterMiddleware for ErrorHandler {
    fn catch(&self, _: &mut Request, err: IronError) -> IronResult<Response> {
      println!("handled the error: {:?}",err.error);
      Err(err)
    }
}

fn main() {
    // Handler is attached here.

    let mut chain = Chain::new(NeverCalledHandler);

    // Link our error maker.
    chain.link_before(ErrorProducer);

    // enable the tracing of the error by catching it with an AfterMiddleware
    chain.link_after(ErrorHandler);

    println!("server running at http://localhost:3000");
    println!("try it by opening the url in a browser or by: 'curl -v http://localhost:3000', 'curl -v http://localhost:3000/any_other_destination/same_bad_request'");
    Iron::new(chain).http("localhost:3000").unwrap();
}

