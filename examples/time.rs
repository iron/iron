extern crate iron;
extern crate typemap;
extern crate time;

use std::io::net::ip::Ipv4Addr;

use iron::{Request, Response, BeforeMiddleware, AfterMiddleware,
           IronResult, Iron, ChainBuilder, Chain, Set};
use iron::response::modifiers::{Status, Body};
use typemap::Assoc;
use time::precise_time_ns;

struct ResponseTime;

impl Assoc<u64> for ResponseTime {}

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
    Ok(Response::new()
           .set(Status(iron::status::Ok))
           .set(Body("Hello World")))
}

fn main() {
    let mut chain = ChainBuilder::new(hello_world);
    chain.link_before(ResponseTime);
    chain.link_after(ResponseTime);
    Iron::new(chain).listen(Ipv4Addr(127, 0, 0, 1), 3000);
}
