use super::super::request::Request;
use super::super::response::Response;
use super::super::ingot::{Ingot, Continue, Unwind};
use super::super::alloy::Alloy;

use super::Furnace;

/// The default `Furnace` used by `Iron`.
/// `IronFurnace` just runs each request through all `Ingots` in its stack,
/// then, when it hits an `Ingot` which returns `Unwind`, it will
/// pass the request back up through all `Ingots` it has hit
/// so far.
pub struct IronFurnace<'a, Rq, Rs> {
    /// The storage used by `IronFurnace` to hold all Ingots
    /// that have been smelted on to it.
    stack: Vec<Box<Ingot<'a, Rq, Rs>: Send>>
}

impl<'a, Rq: Request, Rs: Response<'a>> Clone for IronFurnace<'a, Rq, Rs> {
    fn clone(&self) -> IronFurnace<'a, Rq, Rs> { IronFurnace { stack: self.stack.clone() } }
}

/// `IronFurnace` is a `Furnace`
impl<'a, Rq: Request, Rs: Response<'a>> Furnace<'a, Rq, Rs> for IronFurnace<'a, Rq, Rs> {
    fn forge(&mut self, request: &mut Rq, response: &mut Rs, opt_alloy: Option<&mut Alloy>) {
        // Create a placeholder alloy.
        let mut alloy = &mut Alloy::new();

        // See if we were passed one.
        match opt_alloy {
            // If so, forget about our placeholder.
            Some(a) => alloy = a,
            // Else just use our new Alloy.
            None => ()
        };

        // The exit_stack will hold all Ingots that are passed through
        // in the enter loop. This is so we know to take exactly the same
        // path through ingots in reverse order that we did on the way in.
        let mut exit_stack = vec![];

        'enter: for ingot in self.stack.mut_iter() {
            match ingot.enter(request, response, alloy) {
                Unwind   => break 'enter,
                // Mark the ingot for traversal on exit.
                Continue => exit_stack.push(ingot)
            }
        }

        // Reverse the stack so we go through in the reverse order.
        // i.e. LIFO.
        exit_stack.reverse();
        // Call each ingots exit handler.
        'exit: for ingot in exit_stack.mut_iter() {
            let _ = ingot.exit(request, response, alloy);
        }
    }
    /// Add an `Ingot` to the `Furnace`.
    fn smelt<I: Ingot<'a, Rq, Rs>>(&mut self, ingot: I) {
        self.stack.push(box ingot);
    }

    /// Create a new instance of `IronFurnace`.
    fn new() -> IronFurnace<'a, Rq, Rs> {
        IronFurnace {
            stack: vec![]
        }
    }
}
