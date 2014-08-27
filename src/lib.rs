#![doc(html_logo_url = "https://avatars0.githubusercontent.com/u/7853871?s=128", html_favicon_url = "https://avatars0.githubusercontent.com/u/7853871?s=256", html_root_url = "http://ironframework.io/core/persistent")]
#![license = "MIT"]
//#![deny(missing_doc)]
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

pub struct State<P, D> {
    data: Arc<RWLock<D>>
}

pub struct Read<P, D> {
    data: Arc<D>
}

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

impl<P, D: Send + Sync> Clone for Write<P, D> {
    fn clone(&self) -> Write<P, D> {
        Write { data: self.data.clone() }
    }
}

impl<P, D> Assoc<Arc<RWLock<D>>> for State<P, D> where P: Assoc<D> {}
impl<P, D> Assoc<Arc<D>> for Read<P, D> where P: Assoc<D> {}
impl<P, D> Assoc<Arc<Mutex<D>>> for Write<P, D> where P: Assoc<D> {}

impl<P, D> PluginFor<Request, Arc<RWLock<D>>> for State<P, D>
    where D: Send + Sync,
          P: Assoc<D> {
    fn eval(req: &Request, _: Phantom<State<P, D>>) -> Option<Arc<RWLock<D>>> {
        req.extensions.find::<State<P, D>, Arc<RWLock<D>>>()
            .map(|x| x.clone())
    }
}

impl<P, D> PluginFor<Request, Arc<D>> for Read<P, D>
    where D: Send + Sync,
          P: Assoc<D> {
    fn eval(req: &Request, _: Phantom<Read<P, D>>) -> Option<Arc<D>> {
        req.extensions.find::<Read<P, D>, Arc<D>>()
            .map(|x| x.clone())
    }
}

impl<P, D> PluginFor<Request, Arc<Mutex<D>>> for Write<P, D>
    where D: Send + Sync,
          P: Assoc<D> {
    fn eval(req: &Request, _: Phantom<Write<P, D>>) -> Option<Arc<Mutex<D>>> {
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

impl<D: Send + Sync, P: Assoc<D>> BeforeMiddleware for Write<P, D> {
    fn before(&self, req: &mut Request) -> IronResult<()> {
        req.extensions.insert::<Write<P, D>, Arc<Mutex<D>>>(self.data.clone());
        Ok(())
    }
}

impl<D: Send + Sync, P: Assoc<D>> AfterMiddleware for Write<P, D> {
    fn after(&self, _: &mut Request, res: &mut Response) -> IronResult<()> {
        res.extensions.insert::<Write<P, D>, Arc<Mutex<D>>>(self.data.clone());
        Ok(())
    }
}

impl<D: Send + Sync, P: Assoc<D>> State<P, D> {
    pub fn new(start: D) -> (State<P, D>, State<P, D>) {
        let x = Arc::new(RWLock::new(start));
        (State { data: x.clone() }, State { data: x })
    }
}

impl<D: Send + Sync, P: Assoc<D>> Read<P, D> {
    pub fn new(start: D) -> (Read<P, D>, Read<P, D>) {
        let x = Arc::new(start);
        (Read { data: x.clone() }, Read { data: x })
    }
}

impl<D: Send + Sync, P: Assoc<D>> Write<P, D> {
    pub fn new(start: D) -> (Write<P, D>, Write<P, D>) {
        let x = Arc::new(Mutex::new(start));
        (Write { data: x.clone() }, Write { data: x })
    }
}

