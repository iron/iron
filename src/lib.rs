#![crate_name = "persistent"]
#![license = "MIT"]
#![deny(missing_doc)]
#![deny(warnings)]
#![feature(default_type_params)]

//! A set of middleware for sharing data between requests in the Iron
//! framework.

extern crate iron;
extern crate typemap;
extern crate plugin;

use iron::{Request, Response, BeforeMiddleware, AfterMiddleware, IronResult};
use std::sync::{Arc, RWLock, Mutex};
use typemap::Assoc;
use plugin::{PluginFor, Phantom};

/// Middleware for data that persists between requests with read and write capabilities.
///
/// The data is stored behind a `RWLock`, so multiple read locks
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
/// `State` also implements `PluginFor`, so the data stored within can be
/// accessed through `request.get::<State<P, D>>()` as an `Arc<RWLock<D>>`.
pub struct State<P, D> {
    data: Arc<RWLock<D>>
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
/// `Read` also implements `PluginFor`, so the data stored within can be
/// accessed through `request.get::<Read<P, D>>()` as an `Arc<D>`.
pub struct Read<P, D> {
    data: Arc<D>
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
/// `Write` also implements `PluginFor`, so the data stored within can be
/// accessed through `request.get::<Write<P, D>>()` as an `Arc<Mutex<D>>`.
pub struct Write<P, D> {
    data: Arc<Mutex<D>>
}

impl<P, D: Send + Sync> Clone for Read<P, D> {
    fn clone(&self) -> Read<P, D> {
        Read { data: self.data.clone() }
    }
}

impl<P, D: Send + Sync> Clone for State<P, D> {
    fn clone(&self) -> State<P, D> {
        State { data: self.data.clone() }
    }
}

impl<P, D: Send> Clone for Write<P, D> {
    fn clone(&self) -> Write<P, D> {
        Write { data: self.data.clone() }
    }
}

impl<P, D:'static> Assoc<Arc<RWLock<D>>> for State<P, D> where P: Assoc<D> {}
impl<P, D:'static> Assoc<Arc<D>> for Read<P, D> where P: Assoc<D> {}
impl<P, D:'static> Assoc<Arc<Mutex<D>>> for Write<P, D> where P: Assoc<D> {}

impl<P, D> PluginFor<Request, Arc<RWLock<D>>> for State<P, D>
    where D: Send + Sync,
          P: Assoc<D> {
    fn eval(req: &mut Request, _: Phantom<State<P, D>>) -> Option<Arc<RWLock<D>>> {
        req.extensions.find::<State<P, D>, Arc<RWLock<D>>>()
            .map(|x| x.clone())
    }
}

impl<P, D> PluginFor<Request, Arc<D>> for Read<P, D>
    where D: Send + Sync,
          P: Assoc<D> {
    fn eval(req: &mut Request, _: Phantom<Read<P, D>>) -> Option<Arc<D>> {
        req.extensions.find::<Read<P, D>, Arc<D>>()
            .map(|x| x.clone())
    }
}

impl<P, D> PluginFor<Request, Arc<Mutex<D>>> for Write<P, D>
    where D: Send,
          P: Assoc<D> {
    fn eval(req: &mut Request, _: Phantom<Write<P, D>>) -> Option<Arc<Mutex<D>>> {
        req.extensions.find::<Write<P, D>, Arc<Mutex<D>>>()
            .map(|x| x.clone())
    }
}

impl<D: Send + Sync, P: Assoc<D>> BeforeMiddleware for State<P, D> {
    fn before(&self, req: &mut Request) -> IronResult<()> {
        req.extensions.insert::<State<P, D>, Arc<RWLock<D>>>(self.data.clone());
        Ok(())
    }
}

impl<D: Send + Sync, P: Assoc<D>> AfterMiddleware for State<P, D> {
    fn after(&self, _: &mut Request, res: &mut Response) -> IronResult<()> {
        res.extensions.insert::<State<P, D>, Arc<RWLock<D>>>(self.data.clone());
        Ok(())
    }
}

impl<D: Send + Sync, P: Assoc<D>> BeforeMiddleware for Read<P, D> {
    fn before(&self, req: &mut Request) -> IronResult<()> {
        req.extensions.insert::<Read<P, D>, Arc<D>>(self.data.clone());
        Ok(())
    }
}

impl<D: Send + Sync, P: Assoc<D>> AfterMiddleware for Read<P, D> {
    fn after(&self, _: &mut Request, res: &mut Response) -> IronResult<()> {
        res.extensions.insert::<Read<P, D>, Arc<D>>(self.data.clone());
        Ok(())
    }
}

impl<D: Send, P: Assoc<D>> BeforeMiddleware for Write<P, D> {
    fn before(&self, req: &mut Request) -> IronResult<()> {
        req.extensions.insert::<Write<P, D>, Arc<Mutex<D>>>(self.data.clone());
        Ok(())
    }
}

impl<D: Send, P: Assoc<D>> AfterMiddleware for Write<P, D> {
    fn after(&self, _: &mut Request, res: &mut Response) -> IronResult<()> {
        res.extensions.insert::<Write<P, D>, Arc<Mutex<D>>>(self.data.clone());
        Ok(())
    }
}

impl<P, D> State<P, D> where D: Send + Sync, P: Assoc<D> {
    /// Construct a new pair of `State` that can be passed directly to `Chain::link`.
    ///
    /// The data is initialized with the passed-in value.
    pub fn both(start: D) -> (State<P, D>, State<P, D>) {
        let x = State { data: Arc::new(RWLock::new(start)) };
        (x.clone(), x)
    }

    /// Construct a new `State` that can be passed directly to
    /// `Chain::link_before` or `Chain::link_after`.
    ///
    /// The data is initialized with the passed-in value.
    pub fn one(start: D) -> State<P, D> {
        State { data: Arc::new(RWLock::new(start)) }
    }
}

impl<P, D> Read<P, D> where D: Send + Sync, P: Assoc<D> {
    /// Construct a new pair of `Read` that can be passed directly to `Chain::link`.
    ///
    /// The data is initialized with the passed-in value.
    pub fn both(start: D) -> (Read<P, D>, Read<P, D>) {
        let x = Read { data: Arc::new(start) };
        (x.clone(), x)
    }

    /// Construct a new `Read` that can be passed directly to
    /// `Chain::link_before` or `Chain::link_after`.
    ///
    /// The data is initialized with the passed-in value.
    pub fn one(start: D) -> Read<P, D> {
        Read { data: Arc::new(start) }
    }
}

impl<P, D> Write<P, D> where D: Send, P: Assoc<D> {
    /// Construct a new pair of `Write` that can be passed directly to `Chain::link`.
    ///
    /// The data is initialized with the passed-in value.
    pub fn both(start: D) -> (Write<P, D>, Write<P, D>) {
        let x = Write { data: Arc::new(Mutex::new(start)) };
        (x.clone(), x)
    }

    /// Construct a new `Write` that can be passed directly to
    /// `Chain::link_before` or `Chain::link_after`.
    ///
    /// The data is initialized with the passed-in value.
    pub fn one(start: D) -> Write<P, D> {
        Write { data: Arc::new(Mutex::new(start)) }
    }
}
