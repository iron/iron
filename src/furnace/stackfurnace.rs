use super::super::request::Request;
use super::super::response::Response;
use super::super::ingot::{Ingot, Continue, Unwind};
use super::super::alloy::Alloy;

use super::Furnace;

/// The default `Furnace` used by `Iron`.
/// `StackFurnace` just runs each request through all `Ingots` in its stack,
/// then, when it hits an `Ingot` which returns `Unwind`, it will
/// pass the request back up through all `Ingots` it has hit so far.
pub struct StackFurnace {
    /// The storage used by `StackFurnace` to hold all Ingots
    /// that have been smelted on to it.
    stack: Vec<Box<Ingot + Send>>
}

impl Clone for StackFurnace {
    fn clone(&self) -> StackFurnace { StackFurnace { stack: self.stack.clone() } }
}

/// `StackFurnace` is a `Furnace`
impl Furnace for StackFurnace {
    fn forge(&mut self,
             request: &mut Request,
             response: &mut Response,
             opt_alloy: Option<&mut Alloy>) {
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
    fn smelt<I: Ingot>(&mut self, ingot: I) {
        self.stack.push(box ingot);
    }

    /// Create a new instance of `StackFurnace`.
    fn new() -> StackFurnace {
        StackFurnace {
            stack: vec![]
        }
    }
}
