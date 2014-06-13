use super::super::request::Request;
use super::super::response::Response;
use super::super::ingot::{Ingot, Continue, Unwind};
use super::super::anymap::AnyMap;
use super::super::alloy::Alloy;

use super::Furnace;

struct IronFurnace<Rq, Rs> {
    stack: Vec<Box<Ingot<Rq, Rs>: Send>>
}

impl<Rq: Request, Rs: Response> Clone for IronFurnace<Rq, Rs> {
    fn clone(&self) -> IronFurnace<Rq, Rs> { IronFurnace { stack: self.stack.clone() } }
}

impl<Rq: Request, Rs: Response> Clone for Box<Ingot<Rq, Rs>> {
    fn clone(&self) -> Box<Ingot<Rq, Rs>> { self.clone_box() }
}

