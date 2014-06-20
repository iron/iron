//! Exposes the `Middleware` trait which must be implemented by
//! all middleware.

use super::response::Response;
use super::request::Request;
use super::alloy::Alloy;

/// The Status returned by `Middleware's` `enter` and `exit` methods. This indicates
/// to the `Furnace` whether this is a terminal `Middleware` or whether to continue
/// passing the `Request` and `Response` down the `Furnace's` stack.
///
/// Most `Furnaces` will ignore the returned `Status` from the `exit` method of
/// `Middleware`.
#[deriving(Clone, Show)]
pub enum Status {
    /// `Continue` indicates that this is an intermediate `Middleware` in the stack
    /// and the `Furnace` should continue passing requests down the `Furnace's`
    /// stack.
    ///
    /// Most `Middleware` will return `Continue` from both `enter` and `exit`.
    Continue,

    /// `Unwind` indicates that this is a terminal `Middleware` or that something
    /// went wrong. It can be used to immediately stop passing requests down the
    /// `Furnace's` stack and start calling `exit` of all previous `Middleware`.
    ///
    /// For instance, an authorization `Middleware` might return `Unwind` if the
    /// `Request` fails an authentication check, and `Continue` otherwise.
    Unwind
}

/// All middleware should implement `Middleware`, which allows it to be `smelted`
/// to a `Furnace` so that it will be called for each incoming request.
///
/// There are two sorts of data associated with `Middleware`, data internal
/// to the `Middleware` and data that the `Middleware` would like to expose to
/// other `Middleware` further down the stack or to terminal controllers.
///
/// Internal data should be stored on the `struct` that implements `Middleware`
/// itself. All `Middleware` are cloned for each client request, so the object
/// initially smelted on to the `Iron` instance will be provided as `&mut self` to
/// enter for every request. Data stored on a `Middleware` instance does _not_ persist
/// between requests and is _not_ shared between different, concurrent, requests.
/// The same is true for data stored on an `Alloy`.
///
/// External data should be stored in the `Alloy` passed to both `enter` and
/// `exit`. `Alloy` is a thin wrapper around `AnyMap` and is effectively a
/// a key value store from a type to an instance of that type. This means
/// that each `Middleware` can have a unique type that it stores in the `Alloy`.
/// This can either be an instance of that `Middleware` or some other type. Since
/// the same `Alloy` is passed to all further `Middleware` in the `Furnace`, this
/// scheme allows you to expose data or functionality to future `Middleware`.
pub trait Middleware: Send + Clone {
    /// `enter` is called for each `Middleware` in a `Furnace` as a client request
    /// comes down the stack. `Middleware` have their `enter` methods called in the order
    /// in which they were added to the stack, that is, FIFO. `Middleware` should expose
    /// data through `Alloy` and store any data that will persist through the request here.
    ///
    /// Returning `Unwind` from this handler will cause the `Furnace` to stop
    /// going down its stack and start bubbling back up through `Middleware`
    /// and calling `exit` on them.
    fn enter(&mut self,
             _request: &mut Request,
             _response: &mut Response,
             _alloy: &mut Alloy) -> Status {
        Continue
    }

    /// `exit` is called for each `Middleware` in a `Furnace` that has had its `enter`
    /// method called for this client request. A `Middleware's` `exit` method will be called
    /// as the stack is unwound in FILO order - i.e, `Middleware` have their `exit`
    /// methods called in opposite order from which `enter` was called.
    ///
    /// While this method must return a `Status`, most `Furnaces` will ignore
    /// this method's return value.
    fn exit(&mut self,
            _request: &mut Request,
            _response: &mut Response,
            _alloy: &mut Alloy) -> Status {
        Continue
    }

    // Helper function to clone the Middleware.
    #[allow(missing_doc)]
    fn clone_box(&self) -> Box<Middleware + Send> { box self.clone() as Box<Middleware + Send> }
}

impl Clone for Box<Middleware + Send> {
    fn clone(&self) -> Box<Middleware + Send> { self.clone_box() }
}
