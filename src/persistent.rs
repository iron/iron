use iron::{Request, Response, Middleware, Alloy};
use iron::middleware::{Status, Continue};
use std::sync::{Arc, Mutex};

pub struct Persistent<Data, Phantom> {
    data: Arc<Mutex<Data>>
}

impl<D: Send, P> Clone for Persistent<D, P> {
    fn clone(&self) -> Persistent<D, P> {
        Persistent {
            data: self.data.clone()
        }
    }
}

impl<D: Send, P> Middleware for Persistent<D, P> {
    fn enter(&mut self, _: &mut Request,
             _: &mut Response, alloy: &mut Alloy) -> Status {
        alloy.insert::<Persistent<D, P>>(self.clone());
        Continue
    }
}

