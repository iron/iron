#![cfg_attr(test, deny(warnings))]
#![deny(missing_docs)]

//! A set of middleware for sharing data between requests in the Iron
//! framework.

extern crate iron;
extern crate plugin;

use iron::{Request, Response, BeforeMiddleware, AfterMiddleware, IronResult};
use iron::typemap::Key;
use std::sync::{Arc, RwLock, Mutex};
use std::fmt;
use std::error::Error;
use plugin::Plugin;

/// The type that can be returned by `eval` to indicate error.
#[derive(Clone, Debug)]
pub enum PersistentError {
    /// The value was not found.
    NotFound
}

impl Error for PersistentError {
    fn description(&self) -> &str {
        match *self {
            PersistentError::NotFound => "Value not found in extensions."
        }
    }
}

impl fmt::Display for PersistentError {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        self.description().fmt(f)
    }
}

/// Middleware for data that persists between requests with read and write capabilities.
///
/// The data is stored behind a `RwLock`, so multiple read locks
/// can be taken out concurrently.
///
/// If most threads need to take out a write lock, you may want to
/// consider `Write`, which stores the data behind a `Mutex`, which
/// has a faster locking speed.
///
/// `State` can be linked as `BeforeMiddleware` to add data to the `Request`
/// extensions and it can be linked as an `AfterMiddleware` to add data to
/// the `Response` extensions.
///
/// `State` also implements `Plugin`, so the data stored within can be
/// accessed through `request.get::<State<P>>()` as an `Arc<RwLock<P::Value>>`.
pub struct State<P: Key> {
    data: Arc<RwLock<P::Value>>
}

/// Middleware for data that persists between Requests with read-only capabilities.
///
/// The data is stored behind an Arc, so multiple threads can have
/// concurrent, non-blocking access.
///
/// `Read` can be linked as `BeforeMiddleware` to add data to the `Request`
/// extensions and it can be linked as an `AfterMiddleware` to add data to
/// the `Response` extensions.
///
/// `Read` also implements `Plugin`, so the data stored within can be
/// accessed through `request.get::<Read<P>>()` as an `Arc<P::Value>`.
pub struct Read<P: Key> {
    data: Arc<P::Value>
}

/// Middleware for data that persists between Requests for data which mostly
/// needs to be written instead of read.
///
/// The data is stored behind a `Mutex`, so only one request at a time can
/// access the data. This is more performant than `State` in the case where
/// most uses of the data require a write lock.
///
/// `Write` can be linked as `BeforeMiddleware` to add data to the `Request`
/// extensions and it can be linked as an `AfterMiddleware` to add data to
/// the `Response` extensions.
///
/// `Write` also implements `Plugin`, so the data stored within can be
/// accessed through `request.get::<Write<P>>()` as an `Arc<Mutex<P::Value>>`.
pub struct Write<P: Key> {
    data: Arc<Mutex<P::Value>>
}

macro_rules! impl_persistent {
    ($name:ty, $out:ty, $param:ident) => {
    }
}

impl<P: Key> Clone for Read<P> where P::Value: Send + Sync {
    fn clone(&self) -> Read<P> {
        Read { data: self.data.clone() }
    }
}

impl<P: Key> Clone for State<P> where P::Value: Send + Sync {
    fn clone(&self) -> State<P> {
        State { data: self.data.clone() }
    }
}

impl<P: Key> Clone for Write<P> where P::Value: Send {
    fn clone(&self) -> Write<P> {
        Write { data: self.data.clone() }
    }
}

impl<P: Key> Key for State<P> where P::Value: 'static {
    type Value = Arc<RwLock<P::Value>>;
}

impl<P: Key> Key for Read<P> where P::Value: 'static {
    type Value = Arc<P::Value>;
}

impl<P: Key> Key for Write<P> where P::Value: 'static {
    type Value = Arc<Mutex<P::Value>>;
}

impl<'a, 'b, P: Key> Plugin<Request<'a, 'b>> for State<P> where P::Value: Send + Sync {
    type Error = PersistentError;
    fn eval(req: &mut Request<'a, 'b>) -> Result<Arc<RwLock<P::Value>>, PersistentError> {
        req.extensions.get::<State<P>>().cloned().ok_or(PersistentError::NotFound)
    }
}

impl<'a, 'b, P: Key> Plugin<Request<'a, 'b>> for Read<P> where P::Value: Send + Sync {
    type Error = PersistentError;
    fn eval(req: &mut Request<'a, 'b>) -> Result<Arc<P::Value>, PersistentError> {
        req.extensions.get::<Read<P>>().cloned().ok_or(PersistentError::NotFound)
    }
}

impl<'a, 'b, P: Key> Plugin<Request<'a, 'b>> for Write<P> where P::Value: Send {
    type Error = PersistentError;
    fn eval(req: &mut Request<'a, 'b>) -> Result<Arc<Mutex<P::Value>>, PersistentError> {
        req.extensions.get::<Write<P>>().cloned().ok_or(PersistentError::NotFound)
    }
}

impl<P: Key> BeforeMiddleware for State<P> where P::Value: Send + Sync {
    fn before(&self, req: &mut Request) -> IronResult<()> {
        req.extensions.insert::<State<P>>(self.data.clone());
        Ok(())
    }
}

impl<P: Key> BeforeMiddleware for Read<P> where P::Value: Send + Sync {
    fn before(&self, req: &mut Request) -> IronResult<()> {
        req.extensions.insert::<Read<P>>(self.data.clone());
        Ok(())
    }
}

impl<P: Key> BeforeMiddleware for Write<P> where P::Value: Send {
    fn before(&self, req: &mut Request) -> IronResult<()> {
        req.extensions.insert::<Write<P>>(self.data.clone());
        Ok(())
    }
}

impl<P: Key> AfterMiddleware for State<P> where P::Value: Send + Sync {
    fn after(&self, _: &mut Request, mut res: Response) -> IronResult<Response> {
        res.extensions.insert::<State<P>>(self.data.clone());
        Ok(res)
    }
}

impl<P: Key> AfterMiddleware for Read<P> where P::Value: Send + Sync {
    fn after(&self, _: &mut Request, mut res: Response) -> IronResult<Response> {
        res.extensions.insert::<Read<P>>(self.data.clone());
        Ok(res)
    }
}

impl<P: Key> AfterMiddleware for Write<P> where P::Value: Send {
    fn after(&self, _: &mut Request, mut res: Response) -> IronResult<Response> {
        res.extensions.insert::<Write<P>>(self.data.clone());
        Ok(res)
    }
}

impl<P: Key> State<P> where P::Value: Send + Sync {
    /// Construct a new pair of `State` that can be passed directly to `Chain::link`.
    ///
    /// The data is initialized with the passed-in value.
    pub fn both(start: P::Value) -> (State<P>, State<P>) {
        let x = State { data: Arc::new(RwLock::new(start)) };
        (x.clone(), x)
    }

    /// Construct a new `State` that can be passed directly to
    /// `Chain::link_before` or `Chain::link_after`.
    ///
    /// The data is initialized with the passed-in value.
    pub fn one(start: P::Value) -> State<P> {
        State { data: Arc::new(RwLock::new(start)) }
    }
}

impl<P: Key> Read<P> where P::Value: Send + Sync {
    /// Construct a new pair of `Read` that can be passed directly to `Chain::link`.
    ///
    /// The data is initialized with the passed-in value.
    pub fn both(start: P::Value) -> (Read<P>, Read<P>) {
        let x = Read { data: Arc::new(start) };
        (x.clone(), x)
    }

    /// Construct a new `Read` that can be passed directly to
    /// `Chain::link_before` or `Chain::link_after`.
    ///
    /// The data is initialized with the passed-in value.
    pub fn one(start: P::Value) -> Read<P> {
        Read { data: Arc::new(start) }
    }
}

impl<P: Key> Write<P> where P::Value: Send {
    /// Construct a new pair of `Write` that can be passed directly to `Chain::link`.
    ///
    /// The data is initialized with the passed-in value.
    pub fn both(start: P::Value) -> (Write<P>, Write<P>) {
        let x = Write { data: Arc::new(Mutex::new(start)) };
        (x.clone(), x)
    }

    /// Construct a new `Write` that can be passed directly to
    /// `Chain::link_before` or `Chain::link_after`.
    ///
    /// The data is initialized with the passed-in value.
    pub fn one(start: P::Value) -> Write<P> {
        Write { data: Arc::new(Mutex::new(start)) }
    }
}
