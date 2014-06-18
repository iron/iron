//! Exposes the `Ingot` trait which must be implemented by
//! all middleware.

use super::response::Response;
use super::request::Request;
use super::alloy::Alloy;

/// The Status returned by `Ingot's` `enter` and `exit` methods. This indicates
/// to the `Furnace` whether this is a terminal `Ingot` or whether to continue
/// passing the `Request` and `Response` down the `Furnace's` stack.
///
/// Most `Furnaces` will ignore the returned `Status` from the `exit` method of
/// `Ingots`.
pub enum Status {
    /// `Continue` indicates that this is an intermediate `Ingot` in the stack
    /// and the `Furnace` should continue passing requests down the `Furnace's`
    /// stack.
    ///
    /// Most `Ingots` will return `Continue` from both `enter` and `exit`.
    Continue,

    /// `Unwind` indicates that this is a terminal `Ingot` or something went
    /// wrong. It can be used to immediately stop passing requests down the
    /// `Furnace's` stack and start calling `exit` of all previous `Ingots`.
    ///
    /// For instance, an authorization `Ingot` might return `Unwind` if the
    /// request fails an authentication check, and `Continue` otherwise.
    Unwind
}

/// All middleware should implement `Ingot`, which allows it to be `smelted`
/// to a `Furnace` so that it will be called for each incoming request.
///
/// There are two sorts of data associated with `Ingots`, data internal
/// to the `Ingot` and data that the `Ingot` would like to expose to `Ingots`
/// further down the stack or terminal controllers.
///
/// Internal data should be stored on the `struct` that implements `Ingot`
/// itself. All `Ingots` are cloned for each client request, so the object
/// initially smelted on to the `Iron` instance will be provided as `&self` to
/// enter for every request. Data stored on `Ingot` instances does _not_ persist
/// between requests and is _not_ shared between different, concurrent, requests.
/// The same is true for data stored on `Alloys`.
///
/// External data should be stored in the `Alloy` passed to both `enter` and
/// `exit`. `Alloy` is a thin wrapper around `AnyMap` and is effectively a
/// a key value store from a type to an instance of that type. This means
/// that each `Ingot` can have a unique type that it stores in the `Alloy`.
/// This can either be an instance of that `Ingot` or some other type. Since
/// the same `Alloy` is passed to all further `Ingots` in the `Furnace`, this
/// scheme allows you to expose data or functionality to future `Ingots`.
pub trait Middleware: Send + Clone {
    /// `enter` is called for each `Ingot` in a `Furnace` as a client request
    /// comes down the stack. `Ingots` should expose data through `Alloy` and
    /// store any data that will persist through the request here.
    ///
    /// returning `Unwind` from this handler will cause the `Furnace` to stop
    /// going down the `Furnace's` stack and start bubbling back up and calling
    /// `exit`.
    fn enter(&mut self,
             _request: &mut Request,
             _response: &mut Response,
             _alloy: &mut Alloy) -> Status {
        Continue
    }

    /// `exit` is called for each `Ingot` in `Furnace` that has had it's `enter`
    /// method called for this request. `Ingot's``exit` method will be called
    /// as the stack is unwound in FILO order. `Ingots` have their `exit`
    /// methods called in opposite order from which `enter` was called, which
    /// is FIFO.
    ///
    /// While this method must return a `Status`, most `Furnaces` will ignore
    /// this method's return value.
    fn exit(&mut self,
            _request: &mut Request,
            _response: &mut Response,
            _alloy: &mut Alloy) -> Status {
        Continue
    }

    fn clone_box(&self) -> Box<Middleware> { box self.clone() as Box<Middleware> }
}

impl Clone for Box<Middleware> {
    fn clone(&self) -> Box<Middleware> { self.clone_box() }
}
