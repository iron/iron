// This example shows how to create a basic router that maps url to different handlers.
// If you're looking for real routing middleware, check https://github.com/iron/router

extern crate hyper;
extern crate iron;

use std::collections::HashMap;

use iron::prelude::*;
use iron::Handler;
use iron::StatusCode;

struct Router {
    // Routes here are simply matched with the url path.
    routes: HashMap<String, Box<dyn Handler>>,
}

impl Router {
    fn new() -> Self {
        Router {
            routes: HashMap::new(),
        }
    }

    fn add_route<H>(&mut self, path: String, handler: H)
    where
        H: Handler,
    {
        self.routes.insert(path, Box::new(handler));
    }
}

impl Handler for Router {
    fn handle(&self, req: &mut Request) -> IronResult<Response> {
        match self.routes.get(&req.url.path().join("/")) {
            Some(handler) => handler.handle(req),
            None => Ok(Response::with(StatusCode::NOT_FOUND)),
        }
    }
}

fn main() {
    let mut router = Router::new();

    router.add_route("hello".to_string(), |_: &mut Request| {
        Ok(Response::with((StatusCode::OK, "Hello world !")))
    });

    router.add_route("hello/again".to_string(), |_: &mut Request| {
        Ok(Response::with((StatusCode::OK, "Hello again !")))
    });

    router.add_route("error".to_string(), |_: &mut Request| {
        Ok(Response::with(StatusCode::BAD_REQUEST))
    });

    Iron::new(router).http("localhost:3000");
}
