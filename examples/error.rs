#![feature(globs)]

extern crate iron;
extern crate time;

use iron::prelude::*;
use iron::{Handler, BeforeMiddleware, ChainBuilder};
use iron::response::modifiers::{Status, Body};
use iron::status;

struct ErrorHandler;
struct ErrorProducer;

impl Handler for ErrorHandler {
    fn call(&self, _: &mut Request) -> IronResult<Response> {
        Ok(Response::new())
    }

    fn catch(&self, _: &mut Request, err: IronError) -> (Response, IronResult<()>) {
        (Response::new()
            .set(Status(status::InternalServerError))
            .set(Body("Internal Server Error.")),
         Err(err))
    }

}

impl BeforeMiddleware for ErrorProducer {
    fn before(&self, _: &mut Request) -> IronResult<()> {
        Err(box "Error".to_string() as IronError)
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

