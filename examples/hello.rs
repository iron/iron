extern crate iron;

use std::io::net::ip::Ipv4Addr;

use iron::iron::Iron;
use iron::ingot::{Ingot, Status, Continue};
use iron::furnace::Furnace;
use iron::furnace::ironfurnace::IronFurnace;
use iron::alloy::Alloy;
use iron::request::Request;
use iron::request::ironrequest::IronRequest;
use iron::response::Response;
use iron::response::ironresponse::IronResponse;

#[deriving(Clone)]
struct HelloWorld;

impl<Rq: Request, Rs: Response> Ingot<Rq, Rs> for HelloWorld {
    fn enter(&mut self, _request: &mut Rq, response: &mut Rs, _alloy: &mut Alloy) -> Status {
        response.write(bytes!("Hello World!"));
        Continue
    }
}

fn main() {
    let mut server: Iron<IronRequest, IronResponse<'static, 'static>, IronFurnace<IronRequest, IronResponse<'static, 'static>>> =
        Iron::new();
    server.smelt(HelloWorld);
    server.listen(Ipv4Addr(127, 0, 0, 1), 3000);
}

