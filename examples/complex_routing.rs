extern crate iron;
extern crate router;

use iron::Handler;
use iron::status;
use iron::IronResult;
use iron::Response;
use iron::Request;
use iron::Iron;
use router::Router;

struct EchoHandler {
    message: String
}

impl Handler for EchoHandler {
    fn handle(&self, _: &mut Request) -> IronResult<Response> {
        Ok(Response::with((status::Ok, self.message.clone())))
    }
}

fn main() {
    let echo = EchoHandler {
        message: "You've found the index page!".to_string()
    };

    let mut router = Router::new();
    router.get("/", echo, "index");

    Iron::new(router).http("localhost:3000").unwrap();
}
