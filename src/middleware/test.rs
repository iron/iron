use std::io::net::ip::ToSocketAddr;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering::Relaxed;
use std::sync::Arc;

use self::Kind::{Fine, Bad};

use prelude::*;
use {method, headers};
use {AfterMiddleware, BeforeMiddleware, Handler, TypeMap, Url};

#[test] fn test_chain_normal() {
    test_chain(
        (
            vec![(Fine, Fine), (Fine, Fine), (Fine, Fine)],
            (Fine, Fine),
            vec![(Fine, Fine), (Fine, Fine), (Fine, Fine)]
        ),

        (vec![Fine, Fine, Fine], Fine, vec![Fine, Fine, Fine])
    );
}

#[test] fn test_chain_before_error() {
    test_chain(
        (
            vec![
                (Bad, Fine),
                // Error in before
                (Fine, Bad),
                (Fine, Bad)
            ],
            (Fine, Fine),
            vec![
                (Fine, Bad),
                (Fine, Bad),
                (Fine, Bad)
            ]
        ),

        (vec![Fine, Bad, Bad], Bad, vec![Bad, Bad, Bad])
    );
}

#[test] fn test_chain_handler_error() {
    test_chain(
        (
            vec![
                (Fine, Fine),
                (Fine, Fine),
                (Fine, Fine)
            ],
            // Error in handler
            (Bad, Bad),
            vec![
                (Fine, Bad),
                (Fine, Bad),
                (Fine, Bad)
            ]
        ),

        (vec![Fine, Fine, Fine], Fine, vec![Bad, Bad, Bad])
    );
}

#[test] fn test_chain_after_error() {
    test_chain(
        (
            vec![
                (Fine, Fine),
                (Fine, Fine),
                (Fine, Fine)
            ],
            (Fine, Fine),
            vec![
                // Error in after
                (Bad, Fine),
                (Fine, Bad),
                (Fine, Bad)
            ]
        ),

        (vec![Fine, Fine, Fine], Fine, vec![Fine, Bad, Bad])
    );
}

#[test] fn test_chain_before_error_then_handle() {
    test_chain(
        (
            vec![
                (Bad, Fine),
                (Fine, Bad),
                // Handle in before middleware
                (Fine, Fine),
                (Fine, Fine)
            ],
            (Fine, Fine),
            vec![(Fine, Fine)]
        ),

        (vec![Fine, Bad, Bad, Fine], Fine, vec![Fine])
    );
}

#[test] fn test_chain_after_error_then_handle() {
    test_chain(
        (
            vec![],
            (Fine, Fine),
            vec![
                // Error and handle in after middleware
                (Bad, Fine),
                (Fine, Bad),
                (Fine, Fine),
                (Fine, Fine)
            ]
        ),

        (vec![], Fine, vec![Fine, Bad, Bad, Fine])
    );
}

#[test] fn test_chain_handler_error_then_handle() {
    test_chain(
        (
            vec![],
            // Error in handler.
            (Bad, Fine),
            vec![
                (Fine, Bad),
                (Fine, Fine),
                (Fine, Fine),
            ]
        ),

        (vec![], Fine, vec![Bad, Bad, Fine])
    );
}

// Used to indicate the action taken by a middleware or handler.
#[derive(Debug, PartialEq)]
enum Kind {
    Fine,
    Bad
}

struct Middleware {
    normal: Arc<AtomicBool>,
    error: Arc<AtomicBool>,
    normal_action: Kind,
    error_action: Kind
}

impl BeforeMiddleware for Middleware {
    fn before(&self, _: &mut Request) -> IronResult<()> {
        assert!(!self.normal.load(Relaxed));
        self.normal.store(true, Relaxed);

        match self.normal_action {
            Fine => { Ok(()) },
            Bad => { Err(error()) }
        }
    }

    fn catch(&self, _: &mut Request, _: IronError) -> IronResult<()> {
        assert!(!self.error.load(Relaxed));
        self.error.store(true, Relaxed);

        match self.error_action {
            Fine => { Ok(()) },
            Bad => { Err(error()) },
        }
    }
}

impl Handler for Middleware {
    fn handle(&self, _: &mut Request) -> IronResult<Response> {
        assert!(!self.normal.load(Relaxed));
        self.normal.store(true, Relaxed);

        match self.normal_action {
            Fine => { Ok(response()) },
            Bad => { Err(error()) }
        }
    }
}

impl AfterMiddleware for Middleware {
    fn after(&self, _: &mut Request, _: Response) -> IronResult<Response> {
        assert!(!self.normal.load(Relaxed));
        self.normal.store(true, Relaxed);

        match self.normal_action {
            Fine => { Ok(response()) },
            Bad => { Err(error()) }
        }
    }

    fn catch(&self, _: &mut Request, _: IronError) -> IronResult<Response> {
        assert!(!self.error.load(Relaxed));
        self.error.store(true, Relaxed);

        match self.error_action {
            Fine => { Ok(response()) },
            Bad => { Err(error()) },
        }
    }
}

// Stub request
fn request() -> Request {
    Request {
        url: Url::parse("http://www.rust-lang.org").unwrap(),
        remote_addr: "localhost:3000".to_socket_addr().unwrap(),
        local_addr: "localhost:3000".to_socket_addr().unwrap(),
        headers: headers::Headers::new(),
        body: vec![],
        method: method::Get,
        extensions: TypeMap::new()
    }
}

// Stub response
fn response() -> Response { Response::new() }

// Stub error
fn error() -> IronError {
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
        response: response()
    }
}

type ChainLike<T> = (Vec<T>, T, Vec<T>);
type Twice<T> = (T, T);

fn sharedbool(val: bool) -> Arc<AtomicBool> {
    Arc::new(AtomicBool::new(val))
}

fn counters(chain: &ChainLike<Twice<Kind>>) -> ChainLike<Twice<Arc<AtomicBool>>> {
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
            chain: ChainLike<Twice<Kind>>) -> Chain {
    let (befores, handler, afters) = chain;
    let (ref beforec, ref handlerc, ref afterc) = *counters;

    let befores = befores.into_iter().zip(beforec.iter())
        .map(into_middleware)
        .map(|&: m| box m as Box<BeforeMiddleware>)
        .collect::<Vec<_>>();

    let handler = into_middleware((handler, handlerc));

    let afters = afters.into_iter().zip(afterc.iter())
        .map(into_middleware)
        .map(|&: m| box m as Box<AfterMiddleware>)
        .collect::<Vec<_>>();

    Chain {
        befores: befores,
        handler: Some(box handler as Box<Handler>),
        afters: afters
    }
}

fn into_middleware(input: (Twice<Kind>, &Twice<Arc<AtomicBool>>)) -> Middleware {
    let (normal, error) = input.0;
    let (ref normalc, ref errorc) = *input.1;

    Middleware {
        normal: normalc.clone(),
        error: errorc.clone(),
        normal_action: normal,
        error_action: error
    }
}

fn to_kind(val: bool) -> Kind {
    if val { Fine } else { Bad }
}

fn test_chain(chain: ChainLike<(Kind, Kind)>, expected: ChainLike<Kind>) {
    let actual = counters(&chain);
    let chain = to_chain(&actual, chain);

    // Run the chain
    let _ = chain.handle(&mut request());

    // Get all the results
    let outbefores = actual.0.into_iter()
        .map(|(normal, _)| to_kind(normal.load(Relaxed))).collect::<Vec<_>>();

    let outhandler = to_kind((actual.1).0.load(Relaxed));

    let outafters = actual.2.into_iter()
        .map(|(normal, _)| to_kind(normal.load(Relaxed))).collect::<Vec<_>>();

    let outchain = (outbefores, outhandler, outafters);

    // Yay! Actually do the test!
    assert_eq!(outchain, expected);
}

