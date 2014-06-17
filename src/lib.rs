#![crate_id = "logger"]
#![license = "MIT"]

//! Request logging middleware for Iron

extern crate debug;
extern crate iron;

use iron::{Ingot, Alloy, Request, Response};
use iron::ingot::{Status, Continue};

#[deriving(Clone)]
pub struct Logger;

impl Logger {
    pub fn new() -> Logger {
        Logger
    }
}

impl<Rq: Request, Rs: Response> Ingot<Rq, Rs> for Logger {
    fn enter(&mut self, _req: &mut Rq, _res: &mut Rs, _alloy: &mut Alloy) -> Status {
        Continue
    }
    fn exit(&mut self, req: &mut Rq, _res: &mut Rs, _al: &mut Alloy) -> Status {
        println!("{:?}", req);
        Continue
    }
}
