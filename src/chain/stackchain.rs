use super::super::request::Request;
use super::super::response::Response;
use super::super::middleware::{Middleware, Continue, Unwind, Status};
use super::super::alloy::Alloy;

use super::Chain;

/// The default `Furnace` used by `Iron`.
/// `StackChain` runs each `Request` through all `Middleware` in its stack.
/// When it hits `Middleware` which returns `Unwind`, it passes
/// the `Request` back up through all `Middleware` it has hit so far.
pub struct StackChain {
    /// The storage used by `StackChain` to hold all `Middleware`
    /// that have been `linked` to it.
    stack: Vec<Box<Middleware + Send>>,
    exit_stack: Vec<Box<Middleware + Send>>
}

impl Clone for StackChain {
    fn clone(&self) -> StackChain {
        StackChain {
            stack: self.stack.clone(),
            exit_stack: self.exit_stack.clone()
        }
    }
}

/// `StackChain` is a `Furnace`
impl Chain for StackChain {
    fn chain_enter(&mut self,
             request: &mut Request,
             response: &mut Response,
             alloy: &mut Alloy) -> Status {
        // The `exit_stack` will hold all `Middleware` that are passed through
        // in the enter loop. This is so we know to take exactly the same
        // path through `Middleware` in reverse order than we did on the way in.
        self.exit_stack = vec![];

        'enter: for middleware in self.stack.mut_iter() {
            match middleware.enter(request, response, alloy) {
                Unwind   => return Unwind,
                // Mark the middleware for traversal on exit.
                Continue => self.exit_stack.push(middleware.clone_box())
            }
        }

        Continue
    }

    fn chain_exit(&mut self,
             request: &mut Request,
             response: &mut Response,
             alloy: &mut Alloy) -> Status {
        // Reverse the stack so we go through in the reverse order.
        // i.e. LIFO.
        self.exit_stack.reverse();
        // Call each middleware's exit handler.
        'exit: for middleware in self.exit_stack.mut_iter() {
            let _ = middleware.exit(request, response, alloy);
        }

        Continue
    }

    /// Add `Middleware` to the `Furnace`.
    fn link<M: Middleware>(&mut self, middleware: M) {
        self.stack.push(box middleware);
    }

    /// Create a new instance of `StackChain`.
    fn new() -> StackChain {
        StackChain {
            stack: vec![],
            exit_stack: vec![]
        }
    }
}

