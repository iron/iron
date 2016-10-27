extern crate iron;

use iron::Handler;
use iron::status;
use iron::IronResult;
use iron::Response;
use iron::Request;
use iron::Iron;

struct MessageHandler {
    message: String
}

impl Handler for MessageHandler {
    fn handle(&self, _: &mut Request) -> IronResult<Response> {
        Ok(Response::with((status::Ok, self.message.clone())))
    }
}

fn main() {
    let echo = MessageHandler {
        message: "You've found the index page!".to_string()
    };

    let mut router = Router::new();
    router.get("/", echo, "index");

    Iron::new(router).http("localhost:3000").unwrap();
}
