#![crate_id = "logger"]
#![license = "MIT"]

//! Request logging middleware for Iron

extern crate iron;
extern crate time;

use iron::{Ingot, Alloy, Request, Response};
use iron::ingot::{Status, Continue};

use time::precise_time_ns;

#[deriving(Clone)]
pub struct Logger {
    entry_time: u64
}

impl Logger {
    pub fn new() -> Logger {
        Logger { entry_time: 0u64 }
    }
}

impl<Rq: Request, Rs: Response> Ingot<Rq, Rs> for Logger {
    fn enter(&mut self, _req: &mut Rq, _res: &mut Rs, _alloy: &mut Alloy) -> Status {
        self.entry_time = precise_time_ns();
        Continue
    }
    fn exit(&mut self, req: &mut Rq, res: &mut Rs, _al: &mut Alloy) -> Status {
        let response_time_ms = (precise_time_ns() - self.entry_time) as f64 / 1000000.0;
        println!("Request body: {}\nMethod: {}\nURI: {}\nStatus: {}\nResponse time: {} ms\n",
                 req.body(), req.method(), req.uri(), res.status(), response_time_ms);
        Continue
    }
}
