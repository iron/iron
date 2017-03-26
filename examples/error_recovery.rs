// This example illustrates the error flow of a Request in the middleware Chain.
// Here is the chain used and the path of the request through the middleware pieces:
//
// Normal Flow : __[ErrorProducer::before]__   [ErrorRecover::before]  __[handle::HelloWorldHandler]__[ErrorProducer::after]__   [ErrorRecover::after]  __ ...
// Error Flow  :   [ErrorProducer::catch ]  |__[ErrorRecover::catch ]__|                              [ErrorProducer::catch]  |__[ErrorRecover::catch]__|
//
//               --------------- BEFORE MIDDLEWARE ----------------- || --------- HANDLER --------  ||  ---------------- AFTER MIDDLEWARE --------------

extern crate futures;
extern crate iron;

use iron::prelude::*;
use iron::status;
use iron::{AsyncHandler, BeforeMiddleware, AfterMiddleware};

use futures::future;

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

impl AsyncHandler for HelloWorldHandler {
    fn async_handle(&self, req: Request) -> BoxIronFuture<(Request, Response)> {
        // This will be called since we are in the normal flow before reaching the Handler.
        // However, the AfterMiddleware chain will override the Response.
        println!("The HelloWorldHandler has been called !");
        Box::new(future::ok((req, Response::with((status::Ok, "Hello world !")))))
    }
}

impl BeforeMiddleware for ErrorProducer {
    fn before(&self, req: Request) -> BoxIronFuture<Request> {
        // The error produced here switches to the error flow.
        // The catch method of following middleware pieces will be called.
        // The Handler will be skipped unless the error is handled by another middleware piece.
        // IronError::error tells the next middleware what went wrong.
        // IronError::response is the Response that will be sent back to the client if this error is not handled.
        // Here status::BadRequest acts as modifier, thus we can put more there than just a status.
        Box::new(future::err(IronError::new(StringError("Error in ErrorProducer BeforeMiddleware".to_string()), req, status::BadRequest)))
    }
}

impl AfterMiddleware for ErrorProducer {
    fn after(&self, req: Request, _: Response) -> BoxIronFuture<(Request, Response)> {
        // The behavior here is the same as in ErrorProducer::before.
        // The previous response (from the Handler) is discarded and replaced with a new response (created from the modifier).
        Box::new(future::err(IronError::new(StringError("Error in ErrorProducer AfterMiddleware".to_string()), req, (status::BadRequest, "Response created in ErrorProducer"))))
    }
}

impl BeforeMiddleware for ErrorRecover {
    fn catch(&self, err: IronError) -> BoxIronFuture<Request> {
        // We can use the IronError from previous middleware to decide what to do.
        // Returning Ok() from a catch method resumes the normal flow and
        // passes the Request forward to the next middleware piece in the chain (here the HelloWorldHandler).
        println!("{} caught in ErrorRecover BeforeMiddleware.", err.error);
        match err.response.status {
            Some(status::BadRequest) => Box::new(future::ok(err.request)),
            _ => Box::new(future::err(err))
        }
    }
}

impl AfterMiddleware for ErrorRecover {
    fn catch(&self, err: IronError) -> BoxIronFuture<(Request, Response)> {
        // Just like in the BeforeMiddleware, we can return Ok(Response) here to return to the normal flow.
        // In this case, ErrorRecover is the last middleware in the chain
        // and the Response created in the ErrorProducer is modified and sent back to the client.
        println!("{} caught in ErrorRecover AfterMiddleware.", err.error);
        match err.response.status {
            Some(status::BadRequest) => Box::new(future::ok((err.request, err.response.set(status::Ok)))),
            _ => Box::new(future::err(err))
        }
    }
}

fn main() {
    let mut chain = Chain::new(HelloWorldHandler);
    chain.link_before(ErrorProducer);
    chain.link_before(ErrorRecover);

    chain.link_after(ErrorProducer);
    chain.link_after(ErrorRecover);

    Iron::new(chain).http("localhost:3000").unwrap();
}
