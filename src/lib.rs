#![doc(html_logo_url = "https://avatars0.githubusercontent.com/u/7853871?s=128", html_favicon_url = "https://avatars0.githubusercontent.com/u/7853871?s=256", html_root_url = "http://ironframework.io/core/persistent")]
#![license = "MIT"]
//#![deny(missing_doc)]
#![deny(warnings)]

//! A set of middleware for sharing data between requests in the Iron
//! framework.

extern crate iron;
extern crate typemap;

use iron::{Request, Response, BeforeMiddleware, AfterMiddleware, IronResult};
use std::sync::{Arc, RWLock};
use typemap::Assoc;

pub struct Persistent<P, D> {
    data: Arc<RWLock<D>>
}

pub struct Config<P, C> {
    data: Arc<C>
}

impl<P, D> Assoc<Arc<RWLock<D>>> for Persistent<P, D> where P: Assoc<D> {}
impl<P, C> Assoc<Arc<C>> for Config<P, C> where P: Assoc<C> {}

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

