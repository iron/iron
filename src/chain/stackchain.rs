use super::super::request::Request;
use super::super::response::Response;
use super::super::middleware::{Middleware, Continue, Unwind, Status};
use super::super::alloy::Alloy;

use super::Chain;

/// The default `Chain` used by `Iron`.
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

/// `StackChain` is a `Chain`
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

    /// Add `Middleware` to the `Chain`.
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

#[cfg(test)]
mod test {
    pub use super::*;
    pub use super::super::super::request::Request;
    pub use super::super::super::response::Response;
    pub use super::super::super::alloy::Alloy;
    pub use super::super::super::middleware::{Middleware, Status, Continue};
    pub use std::sync::{Arc, Mutex};

    #[deriving(Clone)]
    pub struct CallCount {
        enter: Arc<Mutex<u64>>,
        exit: Arc<Mutex<u64>>
    }

    impl Middleware for CallCount {
        fn enter(&mut self, _req: &mut Request,
                 _res: &mut Response, _alloy: &mut Alloy) -> Status {
            let mut enter = self.enter.lock();
            *enter += 1;
            Continue
        }

        fn exit(&mut self, _req: &mut Request,
                _res: &mut Response, _alloy: &mut Alloy) -> Status {
            let mut exit = self.exit.lock();
            *exit += 1;
            Continue
        }
    }

    mod dispatch {
        use super::{CallCount, Arc, Mutex};
        use super::super::StackChain;
        use super::super::super::Chain;
        use std::mem::{uninitialized};

        #[test]
        fn calls_middleware_enter() {
            let mut testchain: StackChain = Chain::new();
            let enter = Arc::new(Mutex::new(0));
            let exit = Arc::new(Mutex::new(0));
            testchain.link(CallCount { enter: enter.clone(), exit: exit.clone() });
            unsafe {
                let _ = testchain.dispatch(
                    uninitialized(),
                    uninitialized(),
                    uninitialized()
                );
            }
            assert_eq!(*enter.lock(), 1);
        }

        #[test]
        fn calls_middleware_exit() {
            let mut testchain: StackChain = Chain::new();
            let enter = Arc::new(Mutex::new(0));
            let exit = Arc::new(Mutex::new(0));
            testchain.link(CallCount { enter: enter.clone(), exit: exit.clone() });
            unsafe {
                let _ = testchain.dispatch(
                    uninitialized(),
                    uninitialized(),
                    uninitialized()
                );
            }
            assert_eq!(*exit.lock(), 1);
        }
    }
}

