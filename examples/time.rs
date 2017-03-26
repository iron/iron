extern crate futures;
extern crate iron;
extern crate time;

use iron::prelude::*;
use iron::{BeforeMiddleware, AfterMiddleware, typemap};
use time::precise_time_ns;

use futures::future;

use std::sync::Arc;

struct ResponseTime;

impl typemap::Key for ResponseTime { type Value = u64; }

impl BeforeMiddleware for ResponseTime {
    fn before(&self, mut req: Request) -> BoxIronFuture<Request> {
        req.extensions.insert::<ResponseTime>(precise_time_ns());
        Box::new(future::ok(req))
    }
}

impl AfterMiddleware for ResponseTime {
    fn after(&self, req: Request, res: Response) -> BoxIronFuture<(Request, Response)> {
        let delta = precise_time_ns() - *req.extensions.get::<ResponseTime>().unwrap();
        println!("Request took: {} ms", (delta as f64) / 1000000.0);
        Box::new(future::ok((req, res)))
    }
}

fn hello_world(_: &mut Request) -> IronResult<Response> {
    Ok(Response::with((iron::status::Ok, "Hello World")))
}

fn main() {
    let mut chain = Chain::new(Arc::new(hello_world));
    chain.link_before(ResponseTime);
    chain.link_after(ResponseTime);
    Iron::new(chain).http("localhost:3000").unwrap();
}
