extern crate futures;
extern crate iron;
extern crate time;

use iron::prelude::*;
use iron::{AsyncHandler, AroundMiddleware};
use iron::status;

use futures::{future, Future};

use std::rc::Rc;

enum LoggerMode {
    Silent,
    Tiny,
    Large
}

struct Logger {
    mode: LoggerMode
}

struct LoggerHandler<H: AsyncHandler> { logger: Rc<Logger>, handler: H }

impl Logger {
    fn new(mode: LoggerMode) -> Logger {
        Logger { mode: mode }
    }

    fn log(&self, res: Result<&(Request, Response), &IronError>, time: u64) {
        match self.mode {
            LoggerMode::Silent => {},
            LoggerMode::Tiny => println!("Res: {:?}\nTook: {}", res, time),
            LoggerMode::Large => println!("Response: {:?}\nResponse-Time: {}", res, time)
        }
    }
}

impl<H: AsyncHandler> AsyncHandler for LoggerHandler<H> {
    fn async_handle(&self, req: Request) -> BoxIronFuture<(Request, Response)> {
        let entry = ::time::precise_time_ns();
        let logger = self.logger.clone();
        Box::new(self.handler.async_handle(req).then(move |res| {
            logger.log(res.as_ref(), ::time::precise_time_ns() - entry);
            return res;
        }))
    }
}

impl AroundMiddleware for Logger {
    fn around(self, handler: Box<AsyncHandler>) -> Box<AsyncHandler> {
        Box::new(LoggerHandler {
            logger: Rc::new(self),
            handler: handler,
        })
    }
}

fn hello_world(req: Request) -> BoxIronFuture<(Request, Response)> {
    Box::new(future::ok((req, Response::with((status::Ok, "Hello World!")))))
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

