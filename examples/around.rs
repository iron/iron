extern crate iron;
extern crate time;

use iron::prelude::*;
use iron::{Handler, AroundMiddleware};
use iron::status;

enum LoggerMode {
    Silent,
    Tiny,
    Large
}

struct Logger {
    mode: LoggerMode
}

struct LoggerHandler<H: Handler> { logger: Logger, handler: H }

impl Logger {
    fn new(mode: LoggerMode) -> Logger {
        Logger { mode: mode }
    }

    fn log(&self, req: &Request, res: Result<&Response, &IronError>, time: u64) {
        match self.mode {
            LoggerMode::Silent => {},
            LoggerMode::Tiny => println!("Req: {:?}\nRes: {:?}\nTook: {}", req, res, time),
            LoggerMode::Large => println!("Request: {:?}\nResponse: {:?}\nResponse-Time: {}", req, res, time)
        }
    }
}

impl<H: Handler> Handler for LoggerHandler<H> {
    fn handle(&self, req: &mut Request) -> IronResult<Response> {
        let entry = ::time::precise_time_ns();
        let res = self.handler.handle(req);
        self.logger.log(req, res.as_ref(), ::time::precise_time_ns() - entry);
        res
    }
}

impl AroundMiddleware for Logger {
    fn around(self, handler: Box<Handler>) -> Box<Handler> {
        Box::new(LoggerHandler {
            logger: self,
            handler: handler
        }) as Box<Handler>
    }
}

fn hello_world(_: &mut Request) -> IronResult<Response> {
    Ok(Response::with((status::Ok, "Hello World!")))
}

fn main() {
    let tiny = Iron::new(Logger::new(LoggerMode::Tiny).around(Box::new(hello_world)));
    let silent = Iron::new(Logger::new(LoggerMode::Silent).around(Box::new(hello_world)));
    let large = Iron::new(Logger::new(LoggerMode::Large).around(Box::new(hello_world)));

    let _tiny_listening = tiny.http("localhost:2000").unwrap();
    let _silent_listening = silent.http("localhost:3000").unwrap();
    let _large_listening = large.http("localhost:4000").unwrap();

    println!("Servers listening on 2000, 3000, and 4000");
}

