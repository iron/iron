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

impl<Rq: Request, Rs: Response> Furnace<Rq, Rs> for IronFurnace<Rq, Rs> {
    fn forge(&mut self, request: &mut Rq, response: &mut Rs, malloy: Option<&mut Alloy>) {
        let mut alloy = &mut AnyMap::new();

        match malloy {
            Some(a) => alloy = a,
            None => ()
        };

        let mut exit_stack = vec![];

        'enter: for ingot in self.stack.mut_iter() {
            match ingot.enter(request, response, alloy) {
                Unwind   => break 'enter,
                Continue => exit_stack.push(ingot)
            }
        }

        'exit: for ingot in exit_stack.mut_iter() {
            let _ = ingot.exit(request, response, alloy);
        }
    }
    fn smelt<I: Ingot<Rq, Rs>>(&mut self, ingot: I) {
        self.stack.push(box ingot);
    }
}

