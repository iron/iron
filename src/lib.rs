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
use std::sync::{Arc, RWLock};
use typemap::Assoc;
use plugin::{PluginFor, Phantom};

pub struct Persistent<P, D> {
    data: Arc<RWLock<D>>
}

pub struct Config<P, C> {
    data: Arc<C>
}

impl<P, D> Assoc<Arc<RWLock<D>>> for Persistent<P, D> where P: Assoc<D> {}
impl<P, C> Assoc<Arc<C>> for Config<P, C> where P: Assoc<C> {}

impl<P, D> PluginFor<Request, Arc<RWLock<D>>> for Persistent<P, D>
    where D: Send + Sync,
          P: Assoc<D> {
    fn eval(req: &Request, _: Phantom<Persistent<P, D>>) -> Option<Arc<RWLock<D>>> {
        req.extensions.find::<Persistent<P, D>, Arc<RWLock<D>>>()
            .map(|x| x.clone())
    }
}

impl<P, C> PluginFor<Request, Arc<C>> for Config<P, C>
    where C: Send + Sync,
          P: Assoc<C> {
    fn eval(req: &Request, _: Phantom<Config<P, C>>) -> Option<Arc<C>> {
        req.extensions.find::<Config<P, C>, Arc<C>>()
            .map(|x| x.clone())
    }
}

impl<D: Send + Sync, P: Assoc<D>> BeforeMiddleware for Persistent<P, D> {
    fn before(&self, req: &mut Request) -> IronResult<()> {
        req.extensions.insert::<Persistent<P, D>, Arc<RWLock<D>>>(self.data.clone());
        Ok(())
    }
}

impl<D: Send + Sync, P: Assoc<D>> AfterMiddleware for Persistent<P, D> {
    fn after(&self, _: &mut Request, res: &mut Response) -> IronResult<()> {
        res.extensions.insert::<Persistent<P, D>, Arc<RWLock<D>>>(self.data.clone());
        Ok(())
    }
}

impl<D: Send + Sync, P: Assoc<D>> BeforeMiddleware for Config<P, D> {
    fn before(&self, req: &mut Request) -> IronResult<()> {
        req.extensions.insert::<Config<P, D>, Arc<D>>(self.data.clone());
        Ok(())
    }
}

impl<D: Send + Sync, P: Assoc<D>> AfterMiddleware for Config<P, D> {
    fn after(&self, _: &mut Request, res: &mut Response) -> IronResult<()> {
        res.extensions.insert::<Config<P, D>, Arc<D>>(self.data.clone());
        Ok(())
    }
}

