extern crate iron;
extern crate time;

use iron::prelude::*;
use iron::{typemap, AfterMiddleware, BeforeMiddleware};
use time::precise_time_ns;

struct ResponseTime;

impl typemap::Key for ResponseTime {
    type Value = u64;
}

impl BeforeMiddleware for ResponseTime {
    fn before(&self, req: &mut Request) -> IronResult<()> {
        req.extensions.insert::<ResponseTime>(precise_time_ns());
        Ok(())
    }
}

impl AfterMiddleware for ResponseTime {
    fn after(&self, req: &mut Request, res: Response) -> IronResult<Response> {
        let delta = precise_time_ns() - *req.extensions.get::<ResponseTime>().unwrap();
        println!("Request took: {} ms", (delta as f64) / 1000000.0);
        Ok(res)
    }
}

fn hello_world(_: &mut Request) -> IronResult<Response> {
    Ok(Response::with((iron::StatusCode::OK, "Hello World")))
}

fn main() {
    let mut chain = Chain::new(hello_world);
    chain.link_before(ResponseTime);
    chain.link_after(ResponseTime);
    Iron::new(chain).http("localhost:3000");
}
