use std::rc::Rc;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering::Relaxed;
use std::sync::Arc;

use futures::{future, Future};

use self::Kind::{Fine, Prob};

use {BoxIronFuture};

use prelude::*;
use {AfterMiddleware, BeforeMiddleware, AsyncHandler};

#[test] fn test_chain_normal() {
    test_chain(
        (vec![Fine, Fine, Fine], Fine, vec![Fine, Fine, Fine]),
        (vec![Fine, Fine, Fine], Fine, vec![Fine, Fine, Fine])
    );
}

#[test] fn test_chain_before_error() {
    test_chain(
        // Error in before
        (vec![Prob, Prob, Prob], Fine, vec![Prob, Prob, Prob]),
        (vec![Fine, Prob, Prob], Prob, vec![Prob, Prob, Prob])
    );
}

#[test] fn test_chain_handler_error() {
    test_chain(
        // Error in handler
        (vec![Fine, Fine, Fine], Prob, vec![Prob, Prob, Prob]),
        (vec![Fine, Fine, Fine], Fine, vec![Prob, Prob, Prob])
    );
}

#[test] fn test_chain_after_error() {
    test_chain(
        // Error in after
        (vec![Fine, Fine, Fine], Fine, vec![Prob, Prob, Prob]),
        (vec![Fine, Fine, Fine], Fine, vec![Fine, Prob, Prob])
    );
}

#[test] fn test_chain_before_error_then_handle() {
    test_chain(
        // Error and handle in before middleware
        (vec![Prob, Prob, Fine, Fine], Fine, vec![Fine]),
        (vec![Fine, Prob, Prob, Fine], Fine, vec![Fine])
    );
}

#[test] fn test_chain_after_error_then_handle() {
    test_chain(
        // Error and handle in after middleware
        (vec![], Fine, vec![Prob, Prob, Fine, Fine]),
        (vec![], Fine, vec![Fine, Prob, Prob, Fine])
    );
}

#[test] fn test_chain_handler_error_then_handle() {
    test_chain(
        // Error in handler.
        (vec![], Prob, vec![Prob, Fine, Fine]),
        (vec![], Fine, vec![Prob, Prob, Fine])
    );
}

// Used to indicate the action taken by a middleware or handler.
#[derive(Clone, Debug, PartialEq)]
enum Kind {
    Fine,
    Prob
}

struct Middleware {
    normal: Arc<AtomicBool>,
    error: Arc<AtomicBool>,
    mode: Kind
}

impl BeforeMiddleware for Middleware {
    fn before(&self, req: Request) -> BoxIronFuture<Request> {
        assert!(!self.normal.load(Relaxed));
        self.normal.store(true, Relaxed);

        match self.mode {
            Fine => { future::ok(req).boxed() },
            Prob => { future::err(error(req)).boxed() }
        }
    }

    fn catch(&self, err: IronError) -> BoxIronFuture<Request> {
        assert!(!self.error.load(Relaxed));
        self.error.store(true, Relaxed);

        match self.mode {
            Fine => { future::ok(err.request).boxed() },
            Prob => { future::err(error(err.request)).boxed() },
        }
    }
}

impl AsyncHandler for Middleware {
    fn async_handle(&self, req: Request) -> BoxIronFuture<(Request, Response)> {
        assert!(!self.normal.load(Relaxed));
        self.normal.store(true, Relaxed);

        match self.mode {
            Fine => { future::ok((req, response())).boxed() },
            Prob => { future::err(error(req)).boxed() },
        }
    }
}

impl AfterMiddleware for Middleware {
    fn after(&self, req: Request, _: Response) -> BoxIronFuture<(Request, Response)> {
        assert!(!self.normal.load(Relaxed));
        self.normal.store(true, Relaxed);

        match self.mode {
            Fine => { future::ok((req, response())).boxed() },
            Prob => { future::err(error(req)).boxed() }
        }
    }

    fn catch(&self, err: IronError) -> BoxIronFuture<(Request, Response)> {
        assert!(!self.error.load(Relaxed));
        self.error.store(true, Relaxed);

        match self.mode {
            Fine => { future::ok((err.request, response())).boxed() },
            Prob => { future::err(error(err.request)).boxed() }
        }
    }
}

// Stub request
fn request() -> Request {
    Request::stub()
}

// Stub response
fn response() -> Response { Response::new() }

// Stub error
fn error(request: Request) -> IronError {
    use std::fmt::{self, Debug, Display};
    use std::error::Error as StdError;

    #[derive(Debug)]
    struct SomeError;

    impl Display for SomeError {
        fn fmt(&self, fmt: &mut fmt::Formatter) -> Result<(), fmt::Error> {
            Debug::fmt(self, fmt)
        }
    }

    impl StdError for SomeError {
        fn description(&self) -> &str { "Some Error" }
    }

    IronError {
        error: Box::new(SomeError),
        request: request,
        response: response()
    }
}

type ChainLike<T> = (Vec<T>, T, Vec<T>);
type Twice<T> = (T, T);

fn sharedbool(val: bool) -> Arc<AtomicBool> {
    Arc::new(AtomicBool::new(val))
}

fn counters(chain: &ChainLike<Kind>) -> ChainLike<Twice<Arc<AtomicBool>>> {
    let (ref befores, _, ref afters) = *chain;

    (
        befores.iter()
            .map(|_| (sharedbool(false), sharedbool(false)))
            .collect::<Vec<_>>(),

        (sharedbool(false), sharedbool(false)),

        afters.iter()
            .map(|_| (sharedbool(false), sharedbool(false)))
            .collect::<Vec<_>>()
    )
}

fn to_chain(counters: &ChainLike<Twice<Arc<AtomicBool>>>,
            chain: ChainLike<Kind>) -> Chain {
    let (befores, handler, afters) = chain;
    let (ref beforec, ref handlerc, ref afterc) = *counters;

    let befores = befores.into_iter().zip(beforec.iter())
        .map(into_middleware)
        .map(|m| Rc::new(m) as Rc<BeforeMiddleware>)
        .collect::<Vec<_>>();

    let handler = into_middleware((handler, handlerc));

    let afters = afters.into_iter().zip(afterc.iter())
        .map(into_middleware)
        .map(|m| Rc::new(m) as Rc<AfterMiddleware>)
        .collect::<Vec<_>>();

    Chain {
        befores: befores,
        handler: Some(Rc::new(Box::new(handler) as Box<AsyncHandler>)),
        afters: afters
    }
}

fn into_middleware(input: (Kind, &Twice<Arc<AtomicBool>>)) -> Middleware {
    let mode = input.0;
    let (ref normal, ref error) = *input.1;

    Middleware {
        normal: normal.clone(),
        error: error.clone(),
        mode: mode
    }
}

fn to_kind(normal: bool, error: bool) -> Option<Kind> {
    if normal { Some(Fine) } else if error { Some(Prob) } else { None }
}

fn test_chain(chain: ChainLike<Kind>, expected: ChainLike<Kind>) {
    let actual = counters(&chain);
    let chain = to_chain(&actual, chain);

    // Run the chain
    let _ = chain.async_handle(request()).wait();

    // Get all the results
    let outbefores = actual.0.into_iter()
        .map(|(normal, error)| to_kind(normal.load(Relaxed), error.load(Relaxed)).unwrap()).collect::<Vec<_>>();

    let outhandler = to_kind((actual.1).0.load(Relaxed), (actual.1).1.load(Relaxed)).unwrap_or_else(|| outbefores.iter().last().cloned().unwrap());

    let outafters = actual.2.into_iter()
        .map(|(normal, error)| to_kind(normal.load(Relaxed), error.load(Relaxed)).unwrap()).collect::<Vec<_>>();

    let outchain = (outbefores, outhandler, outafters);

    // Yay! Actually do the test!
    assert_eq!(outchain, expected);
}

