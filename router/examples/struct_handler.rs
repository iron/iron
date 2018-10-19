extern crate iron;
extern crate router;

use iron::{Handler, StatusCode, IronResult, Response, Request, Iron};
use router::Router;

struct MessageHandler {
    message: String
}

impl Handler for MessageHandler {
    fn handle(&self, _: &mut Request) -> IronResult<Response> {
        Ok(Response::with((StatusCode::OK, self.message.clone())))
    }
}

fn main() {
    let handler = MessageHandler {
        message: "You've found the index page!".to_string()
    };

    let mut router = Router::new();
    router.get("/", handler, "index");

    Iron::new(router).http("localhost:3000");
}
