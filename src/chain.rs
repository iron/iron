//! Exposes the `chain` trait and `StackChain` type.

use super::response::Response;
use super::request::Request;
use super::alloy::Alloy;
use super::middleware::{Middleware, Status, Continue};
use super::mixin::Serve;

/// `chains` are the backbone of `Iron`. They coordinate `Middleware`
/// to ensure they are resolved and called in the right order,
/// create and distribute `Alloys`, and handle incoming requests.
///
/// `chains` are internal tools. Unless you want additional
/// or unusual behavior such as enhanced debug logging you
/// probably don't need to mess with `chain` internals.
///
/// That being said, custom `chains` are extremely powerful as they
/// allow you to completely control the resolution of `Middleware`.
pub trait Chain: Send + Clone {
    /// `dispatch` will be called once per `Request`, and may be
    /// called either with or without an existing `Alloy`. `dispatch` is responsible
    /// for delegating the request to the correct `Middleware` and in the correct
    /// order. Effectively, 99% of the work done by a `chain` is done here.
    fn dispatch(&mut self,
                request: &mut Request,
                response: &mut Response,
                opt_alloy: Option<&mut Alloy>) -> Status {
        let mut alloy = &mut Alloy::new();
        match opt_alloy {
            Some(a) => alloy = a,
            None => ()
        };

        let status = self.chain_enter(request, response, alloy);
        let _ = self.chain_exit(request, response, alloy);

        status
    }

    #[doc(hidden)]
    fn chain_enter(&mut self,
             request: &mut Request,
             response: &mut Response,
             alloy: &mut Alloy) -> Status;

    #[doc(hidden)]
    fn chain_exit(&mut self,
                  _request: &mut Request,
                  _response: &mut Response,
                  _alloy: &mut Alloy) -> Status {
        Continue
    }

    /// `link` is responsible for adding new `Middleware` to the `chain's` internal
    /// storage of `Middleware`. Different `chains` may implement different behavior
    /// for `link`, but - ideally - `Middleware` added here will be delegated to during
    /// `Requests`.
    fn link<M: Middleware>(&mut self, _middleware: M);

    /// Create a new instance of `chain`.
    fn new() -> Self;

    #[doc(hidden)]
    fn clone_box(&self) -> Box<Chain + Send> { box self.clone() as Box<Chain + Send> }
}

impl Clone for Box<Chain + Send> {
    fn clone(&self) -> Box<Chain + Send> { self.clone_box() }
}

/// The default `chain` used by `Iron`.
pub mod stackchain {
    use super::super::request::Request;
    use super::super::response::Response;
    use super::super::middleware::{Middleware, Continue, Unwind, Status};
    use super::super::alloy::Alloy;

    use super::Chain;

    /// The default `Chain` used by `Iron`.
    /// `StackChain` runs each `Request` through all `Middleware` in its stack.
    ///
    /// When it hits `Middleware` which returns `Unwind`, it passes
    /// the `Request` back up through all `Middleware` it has hit so far.
    ///
    /// If no `Middleware` return `Unwind` to indicate that they handled
    /// the request, then a 404 is automatically returned.
    pub struct StackChain {
        /// The storage used by `StackChain` to hold all `Middleware`
        /// that have been `linked` to it.
        stack: Vec<Box<Middleware + Send>>,
        unwind: Option<uint>
    }

    impl Clone for StackChain {
        fn clone(&self) -> StackChain {
            StackChain {
                stack: self.stack.clone(),
                unwind: self.unwind.clone()
            }
        }
    }

    /// `StackChain` is a `Chain`
    impl Chain for StackChain {
        fn dispatch(&mut self,
                    request: &mut Request,
                    response: &mut Response,
                    opt_alloy: Option<&mut Alloy>) -> Status {
            let mut alloy = &mut Alloy::new();
            match opt_alloy {
                Some(a) => alloy = a,
                None => ()
            };

            let status = self.chain_enter(request, response, alloy);

            match status {
                Unwind => (),
                Continue => {
                    // If no middleware returned unwind, then we send a 404.
                    // At least one middleware should return unwind when a
                    // terminal endpoint, such as a router, has been reached.
                    //
                    // We don't write to the body as other middleware may want
                    // to change headers.
                    response.status = ::http::status::NotFound;
                }
            }

            let _ = self.chain_exit(request, response, alloy);

            status
        }

        fn chain_enter(&mut self,
                 request: &mut Request,
                 response: &mut Response,
                 alloy: &mut Alloy) -> Status {
            // The `exit_stack` will hold all `Middleware` that are passed through
            // in the enter loop. This is so we know to take exactly the same
            // path through `Middleware` in reverse order than we did on the way in.
            self.unwind = None;

            'enter: for (i, middleware) in self.stack.mut_iter().enumerate() {
                match middleware.enter(request, response, alloy) {
                    Unwind   => {
                        self.unwind = Some(i);
                        return Unwind;
                    }
                    // Mark the middleware for traversal on exit.
                    Continue => ()
                }
            }

            Continue
        }

        fn chain_exit(&mut self,
                 request: &mut Request,
                 response: &mut Response,
                 alloy: &mut Alloy) -> Status {
            match self.unwind {
                Some(i) => {
                    'exit: for middleware in self.stack.mut_slice_to(i).mut_iter().rev() {
                        let _ = middleware.exit(request, response, alloy);
                    }
                },
                None => {
                    'exit: for middleware in self.stack.mut_iter().rev() {
                        let _ = middleware.exit(request, response, alloy);
                    }
                }
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
                unwind: None
            }
        }
    }

    impl FromIterator<Box<Middleware + Send>> for StackChain {
        fn from_iter<T: Iterator<Box<Middleware + Send>>>(mut iterator: T) -> StackChain {
            StackChain {
                stack: iterator.collect(),
                unwind: None
            }
        }
    }

    #[cfg(test)]
    mod test {
        pub use super::*;
        pub use super::super::super::request::Request;
        pub use super::super::super::response::Response;
        pub use super::super::super::alloy::Alloy;
        pub use super::super::super::middleware::{Middleware, Status, Continue, Unwind};
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

        #[deriving(Clone)]
        pub struct Stopper;

        impl Middleware for Stopper {
            fn enter(&mut self, _req: &mut Request,
                     _res: &mut Response, _alloy: &mut Alloy) -> Status {
                Unwind // Stop .status from being accessed, which fails.
            }
        }

        mod dispatch {
            use super::{CallCount, Arc, Mutex, Stopper};
            use super::super::StackChain;
            use super::super::super::Chain;
            use std::mem::{uninitialized};

            #[test]
            fn calls_middleware_enter() {
                let mut testchain: StackChain = Chain::new();
                let enter = Arc::new(Mutex::new(0));
                let exit = Arc::new(Mutex::new(0));
                testchain.link(CallCount { enter: enter.clone(), exit: exit.clone() });
                testchain.link(Stopper);
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
                testchain.link(Stopper);
                unsafe {
                    let _ = testchain.dispatch(
                        uninitialized(),
                        uninitialized(),
                        uninitialized()
                    );
                }
                assert_eq!(*exit.lock(), 1);
            }

            #[test]
            fn calls_all_middleware_enter_exit() {
                let mut testchain: StackChain = Chain::new();
                let enter_exits: Vec<(Arc<Mutex<u64>>, Arc<Mutex<u64>>)> = vec![];

                for _ in range(0u8, 10) {
                    let (enter, exit) = (Arc::new(Mutex::new(0)), Arc::new(Mutex::new(0)));
                    testchain.link(CallCount { enter: enter.clone(), exit: exit.clone() });
                }

                testchain.link(Stopper);
                unsafe {
                    let _ = testchain.dispatch(
                        uninitialized(),
                        uninitialized(),
                        uninitialized()
                    );
                }

                for (enter, exit) in enter_exits.move_iter() {
                    assert_eq!(*enter.lock(), 1);
                    assert_eq!(*exit.lock(), 1);
                }
            }
        }

        mod chain_enter {
            use super::{CallCount, Arc, Mutex, Stopper};
            use super::super::StackChain;
            use super::super::super::Chain;
            use std::mem::{uninitialized};

            #[test]
            fn calls_middleware_enter() {
                let mut testchain: StackChain = Chain::new();
                let enter = Arc::new(Mutex::new(0));
                let exit = Arc::new(Mutex::new(0));
                testchain.link(CallCount { enter: enter.clone(), exit: exit.clone() });
                testchain.link(Stopper);
                unsafe {
                    let _ = testchain.chain_enter(
                        uninitialized(),
                        uninitialized(),
                        uninitialized()
                    );
                }
                assert_eq!(*enter.lock(), 1);
            }

            #[test]
            fn doesnt_call_middleware_exit() {
                let mut testchain: StackChain = Chain::new();
                let enter = Arc::new(Mutex::new(0));
                let exit = Arc::new(Mutex::new(0));
                testchain.link(CallCount { enter: enter.clone(), exit: exit.clone() });
                testchain.link(Stopper);
                unsafe {
                    let _ = testchain.chain_enter(
                        uninitialized(),
                        uninitialized(),
                        uninitialized()
                    );
                }
                assert_eq!(*exit.lock(), 0);
            }
        }

        mod chain_exit {
            use super::{CallCount, Arc, Mutex, Stopper};
            use super::super::StackChain;
            use super::super::super::Chain;
            use std::mem::uninitialized;

            #[test]
            fn calls_middleware_exit() {
                let mut testchain: StackChain = Chain::new();
                let enter = Arc::new(Mutex::new(0));
                let exit = Arc::new(Mutex::new(0));
                testchain.link(CallCount {
                    enter: enter.clone(), exit: exit.clone()
                });
                testchain.link(Stopper);
                unsafe {
                    let _  = testchain.chain_enter(
                        uninitialized(),
                        uninitialized(),
                        uninitialized()
                    );

                    let _  = testchain.chain_exit(
                        uninitialized(),
                        uninitialized(),
                        uninitialized()
                    );
                }
                assert_eq!(*exit.lock(), 1);
            }

            #[test]
            fn doesnt_call_middleware_enter() {
                let mut testchain: StackChain = Chain::new();
                let enter = Arc::new(Mutex::new(0));
                let exit = Arc::new(Mutex::new(0));
                testchain.link(CallCount {
                    enter: enter.clone(), exit: exit.clone()
                });
                testchain.unwind = Some(1);
                unsafe {
                    let _  = testchain.chain_exit(
                        uninitialized(),
                        uninitialized(),
                        uninitialized()
                    );
                }
                assert_eq!(*enter.lock(), 0);
            }
        }

        mod bench {
            use super::super::super::super::middleware::Middleware;
            pub use super::Stopper;

            #[deriving(Clone)]
            struct Noop;

            impl Middleware for Noop {}

            macro_rules! bench_noop_x (
                ($name:ident, $num:expr, $method:ident) => {
                    #[bench]
                    fn $name(b: &mut Bencher) {
                        let mut testchain: StackChain = Chain::new();
                        for _ in range(0, $num) {
                            testchain.link(Noop);
                        }
                        testchain.link(Stopper);
                        b.iter(|| {
                            black_box(unsafe {
                                let _ = testchain.$method(
                                    uninitialized(),
                                    uninitialized(),
                                    uninitialized()
                                );
                            })
                        });
                    }
                }
            )

            macro_rules! bench_method (
                ($method:ident) => {
                    mod $method {
                        use std::mem::uninitialized;
                        use test::{Bencher, black_box};
                        use super::{Noop, Stopper};
                        use super::super::super::StackChain;
                        use super::super::super::super::Chain;

                        bench_noop_x!(bench_empty, 0u8, $method)
                        bench_noop_x!(bench_1, 1u8, $method)
                        bench_noop_x!(bench_2, 2u8, $method)
                        bench_noop_x!(bench_3, 3u8, $method)
                        bench_noop_x!(bench_4, 4u8, $method)
                        bench_noop_x!(bench_10, 10u8, $method)
                        bench_noop_x!(bench_100, 100u8, $method)
                    }
                }
            )

            bench_method!(dispatch)
            bench_method!(chain_enter)
        }
    }
}
