#![feature(globs)]
extern crate iron;
extern crate time;

use iron::prelude::*;
use iron::{BeforeMiddleware, AfterMiddleware, ChainBuilder, typemap};
use time::precise_time_ns;

struct ResponseTime;

impl typemap::Assoc<u64> for ResponseTime {}

impl BeforeMiddleware for ResponseTime {
    fn before(&self, req: &mut Request) -> IronResult<()> {
        req.extensions.insert::<ResponseTime, u64>(precise_time_ns());
        Ok(())
    }
}

impl AfterMiddleware for ResponseTime {
    fn after(&self, req: &mut Request, _: &mut Response) -> IronResult<()> {
        let delta = precise_time_ns() - *req.extensions.get::<ResponseTime, u64>().unwrap();
        println!("Request took: {} ms", (delta as f64) / 1000000.0);
        Ok(())
    }
}

fn hello_world(_: &mut Request) -> IronResult<Response> {
    Ok(Response::new().set(iron::status::Ok).set("Hello World"))
}

fn main() {
    let mut chain = ChainBuilder::new(hello_world);
    chain.link_before(ResponseTime);
    chain.link_after(ResponseTime);
    Iron::new(chain).listen("localhost:3000").unwrap();
}
