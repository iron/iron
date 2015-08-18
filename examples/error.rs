extern crate iron;
extern crate time;

use iron::prelude::*;
use iron::{Handler, BeforeMiddleware, AroundMiddleware};
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
        Err(IronError::new(StringError("Error you cannot access the resource!".to_string()), status::BadRequest))
    }
}


// to handle the Error message by using an AroundMiddleware
struct LogEnabler;

struct HandlerWithLog<H: Handler> {
    handler: H
}

impl AroundMiddleware for LogEnabler {

    fn around(self, handler: Box<Handler>) -> Box<Handler> {
        Box::new(HandlerWithLog { handler: handler } ) as Box<Handler>
    }
}
 
impl <H: Handler> Handler for HandlerWithLog<H> {

    fn handle(&self, req: &mut Request) -> IronResult<Response> {
    
        let res = self.handler.handle(req);
        
        {
          match res {
            Ok(_) => panic!("is not possible!"),
            Err(IronError {error: ref what_went_wrong, ref response }) => println!("what went wrong: {:?} - response: {:?}", what_went_wrong, response)
          };          
        }
        
        res
    }
}

pub fn get_log_enabled_handler(handler : Box<Handler>) -> Box<Handler> {
  
  let use_log = LogEnabler;
    
  use_log.around(handler)
}


fn main() {
    // Handler is attached here.
    
    let mut chain = Chain::new(ErrorHandler);

    // Link our error maker.
    chain.link_before(ErrorProducer);
    
    // enable the tracing of the error by using an AroundMiddleware
    let logged_handler = get_log_enabled_handler(Box::new(chain));
    
    println!("server running at http://localhost:3000");
    println!("try it by opening the url in a browser or by: 'curl -v http://localhost:3000', 'curl -v http://localhost:3000/any_other_destination/same_bad_request'");
    Iron::new(logged_handler).http("localhost:3000").unwrap();
}

