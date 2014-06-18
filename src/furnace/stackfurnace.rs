use super::super::request::Request;
use super::super::response::Response;
use super::super::middleware::{Middleware, Continue, Unwind};
use super::super::alloy::Alloy;

use super::Furnace;

/// The default `Furnace` used by `Iron`.
/// `StackFurnace` runs each `Request` through all `Middleware` in its stack.
/// When it hits `Middleware` which returns `Unwind`, it passes
/// the `Request` back up through all `Middleware` it has hit so far.
pub struct StackFurnace {
    /// The storage used by `StackFurnace` to hold all `Middleware`
    /// that have been `smelted` on to it.
    stack: Vec<Box<Middleware + Send>>
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

        // The `exit_stack` will hold all `Middleware` that are passed through
        // in the enter loop. This is so we know to take exactly the same
        // path through `Middleware` in reverse order than we did on the way in.
        let mut exit_stack = vec![];

        'enter: for middleware in self.stack.mut_iter() {
            match middleware.enter(request, response, alloy) {
                Unwind   => break 'enter,
                // Mark the middleware for traversal on exit.
                Continue => exit_stack.push(middleware)
            }
        }

        // Reverse the stack so we go through in the reverse order.
        // i.e. LIFO.
        exit_stack.reverse();
        // Call each middleware's exit handler.
        'exit: for middleware in exit_stack.mut_iter() {
            let _ = middleware.exit(request, response, alloy);
        }
    }
    /// Add `Middleware` to the `Furnace`.
    fn smelt<M: Middleware>(&mut self, middleware: M) {
        self.stack.push(box middleware);
    }

    /// Create a new instance of `StackFurnace`.
    fn new() -> StackFurnace {
        StackFurnace {
            stack: vec![]
        }
    }
}
