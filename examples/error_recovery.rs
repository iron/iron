// This example illustrates the error flow of a Request in BeforeMiddleware.
// Here is the chain used and the path of the request through the middleware pieces:
//
// Normal Flow : ____[ErrorProducer::before]_____     [ErrorRecover::before]     ____[HelloWorldHandler::handle]___
// Error Flow  :     [ErrorProducer::catch ]     |____[ErrorRecover::catch ]_____|


extern crate iron;

use iron::prelude::*;
use iron::status;
use iron::{Handler, BeforeMiddleware};

use std::error::Error;
use std::fmt::{self, Debug};

struct HelloWorldHandler;
struct ErrorProducer;
struct ErrorRecover;

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

impl Handler for HelloWorldHandler {
    fn handle(&self, _: &mut Request) -> IronResult<Response> {
        Ok(Response::with((status::Ok, "Hello world !")))
    }
}

impl BeforeMiddleware for ErrorProducer {
    fn before(&self, _: &mut Request) -> IronResult<()> {
        // The error produced here switches to the error flow.
        // The catch method of following middleware pieces will be called.
        // The Handler will be skipped unless the error is handled by another middleware piece.
        Err(IronError::new(StringError("Error in ErrorProducer".to_string()), status::BadRequest))
    }
}

impl BeforeMiddleware for ErrorRecover {
    fn catch(&self, _: &mut Request, err: IronError) -> IronResult<()> {
        // We can use the IronError from previous middleware to decide what to do.
        // Returning Ok() from a catch method resumes the normal flow.        
        println!("{} caught in ErrorRecover.", err.error);
        match err.response.status {
            Some(status::BadRequest) => Ok(()),
            _ => Err(err)
        }
    }
}

fn main() {
    let mut chain = Chain::new(HelloWorldHandler);
    chain.link_before(ErrorProducer);
    chain.link_before(ErrorRecover);

    Iron::new(chain).http("localhost:3000").unwrap();
}
