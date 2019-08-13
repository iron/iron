use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering::Relaxed;
use std::sync::Arc;

use self::Kind::{Fine, Prob};

use prelude::*;
use {AfterMiddleware, BeforeMiddleware, Handler};

#[test]
fn test_chain_normal() {
    test_chain(
        (vec![Fine, Fine, Fine], Fine, vec![Fine, Fine, Fine]),
        (vec![Fine, Fine, Fine], Fine, vec![Fine, Fine, Fine]),
    );
}

#[test]
fn test_chain_before_error() {
    test_chain(
        // Error in before
        (vec![Prob, Prob, Prob], Fine, vec![Prob, Prob, Prob]),
        (vec![Fine, Prob, Prob], Prob, vec![Prob, Prob, Prob]),
    );
}

#[test]
fn test_chain_handler_error() {
    test_chain(
        // Error in handler
        (vec![Fine, Fine, Fine], Prob, vec![Prob, Prob, Prob]),
        (vec![Fine, Fine, Fine], Fine, vec![Prob, Prob, Prob]),
    );
}

#[test]
fn test_chain_after_error() {
    test_chain(
        // Error in after
        (vec![Fine, Fine, Fine], Fine, vec![Prob, Prob, Prob]),
        (vec![Fine, Fine, Fine], Fine, vec![Fine, Prob, Prob]),
    );
}

#[test]
fn test_chain_before_error_then_handle() {
    test_chain(
        // Error and handle in before middleware
        (vec![Prob, Prob, Fine, Fine], Fine, vec![Fine]),
        (vec![Fine, Prob, Prob, Fine], Fine, vec![Fine]),
    );
}

#[test]
fn test_chain_after_error_then_handle() {
    test_chain(
        // Error and handle in after middleware
        (vec![], Fine, vec![Prob, Prob, Fine, Fine]),
        (vec![], Fine, vec![Fine, Prob, Prob, Fine]),
    );
}

#[test]
fn test_chain_handler_error_then_handle() {
    test_chain(
        // Error in handler.
        (vec![], Prob, vec![Prob, Fine, Fine]),
        (vec![], Fine, vec![Prob, Prob, Fine]),
    );
}

// Used to indicate the action taken by a middleware or handler.
#[derive(Debug, PartialEq)]
enum Kind {
    Fine,
    Prob,
}

struct Middleware {
    normal: Arc<AtomicBool>,
    error: Arc<AtomicBool>,
    mode: Kind,
}

impl BeforeMiddleware for Middleware {
    fn before(&self, _: &mut Request) -> IronResult<()> {
        assert!(!self.normal.load(Relaxed));
        self.normal.store(true, Relaxed);

        match self.mode {
            Fine => Ok(()),
            Prob => Err(error()),
        }
    }

    fn catch(&self, _: &mut Request, _: IronError) -> IronResult<()> {
        assert!(!self.error.load(Relaxed));
        self.error.store(true, Relaxed);

        match self.mode {
            Fine => Ok(()),
            Prob => Err(error()),
        }
    }
}

impl Handler for Middleware {
    fn handle(&self, _: &mut Request) -> IronResult<Response> {
        assert!(!self.normal.load(Relaxed));
        self.normal.store(true, Relaxed);

        match self.mode {
            Fine => Ok(response()),
            Prob => Err(error()),
        }
    }
}

impl AfterMiddleware for Middleware {
    fn after(&self, _: &mut Request, _: Response) -> IronResult<Response> {
        assert!(!self.normal.load(Relaxed));
        self.normal.store(true, Relaxed);

        match self.mode {
            Fine => Ok(response()),
            Prob => Err(error()),
        }
    }

    fn catch(&self, _: &mut Request, _: IronError) -> IronResult<Response> {
        assert!(!self.error.load(Relaxed));
        self.error.store(true, Relaxed);

        match self.mode {
            Fine => Ok(response()),
            Prob => Err(error()),
        }
    }
}

// Stub request
fn request() -> Request {
    Request::stub()
}

// Stub response
fn response() -> Response {
    Response::new()
}

// Stub error
fn error() -> IronError {
    use std::error::Error as StdError;
    use std::fmt::{self, Debug, Display};

    #[derive(Debug)]
    struct SomeError;

    impl Display for SomeError {
        fn fmt(&self, fmt: &mut fmt::Formatter) -> Result<(), fmt::Error> {
            Debug::fmt(self, fmt)
        }
    }

    impl StdError for SomeError {
        fn description(&self) -> &str {
            "Some Error"
        }
    }

    IronError {
        error: Box::new(SomeError),
        response: response(),
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
        befores
            .iter()
            .map(|_| (sharedbool(false), sharedbool(false)))
            .collect::<Vec<_>>(),
        (sharedbool(false), sharedbool(false)),
        afters
            .iter()
            .map(|_| (sharedbool(false), sharedbool(false)))
            .collect::<Vec<_>>(),
    )
}

fn to_chain(counters: &ChainLike<Twice<Arc<AtomicBool>>>, chain: ChainLike<Kind>) -> Chain {
    let (befores, handler, afters) = chain;
    let (ref beforec, ref handlerc, ref afterc) = *counters;

    let befores = befores
        .into_iter()
        .zip(beforec.iter())
        .map(into_middleware)
        .map(|m| Box::new(m) as Box<dyn BeforeMiddleware>)
        .collect::<Vec<_>>();

    let handler = into_middleware((handler, handlerc));

    let afters = afters
        .into_iter()
        .zip(afterc.iter())
        .map(into_middleware)
        .map(|m| Box::new(m) as Box<dyn AfterMiddleware>)
        .collect::<Vec<_>>();

    Chain {
        befores: befores,
        handler: Some(Box::new(handler) as Box<dyn Handler>),
        afters: afters,
    }
}

fn into_middleware(input: (Kind, &Twice<Arc<AtomicBool>>)) -> Middleware {
    let mode = input.0;
    let (ref normal, ref error) = *input.1;

    Middleware {
        normal: normal.clone(),
        error: error.clone(),
        mode: mode,
    }
}

fn to_kind(val: bool) -> Kind {
    if val {
        Fine
    } else {
        Prob
    }
}

fn test_chain(chain: ChainLike<Kind>, expected: ChainLike<Kind>) {
    let actual = counters(&chain);
    let chain = to_chain(&actual, chain);

    // Run the chain
    let _ = chain.handle(&mut request());

    // Get all the results
    let outbefores = actual
        .0
        .into_iter()
        .map(|(normal, _)| to_kind(normal.load(Relaxed)))
        .collect::<Vec<_>>();

    let outhandler = to_kind((actual.1).0.load(Relaxed));

    let outafters = actual
        .2
        .into_iter()
        .map(|(normal, _)| to_kind(normal.load(Relaxed)))
        .collect::<Vec<_>>();

    let outchain = (outbefores, outhandler, outafters);

    // Yay! Actually do the test!
    assert_eq!(outchain, expected);
}
