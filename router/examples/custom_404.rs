extern crate iron;
extern crate router;

// To run, $ cargo run --example custom_404
// To use, go to http://localhost:3000/foobar to see the custom 404
// Or, go to http://localhost:3000 for a standard 200 OK

use iron::{Iron, Request, Response, IronResult, AfterMiddleware, Chain, StatusCode};
use iron::error::{IronError};
use router::{Router, NoRoute};

struct Custom404;

impl AfterMiddleware for Custom404 {
    fn catch(&self, _: &mut Request, err: IronError) -> IronResult<Response> {
        println!("Hitting custom 404 middleware");

        if err.error.is::<NoRoute>() {
            Ok(Response::with((StatusCode::NOT_FOUND, "Custom 404 response")))
        } else {
            Err(err)
        }
    }
}

fn main() {
    let mut router = Router::new();
    router.get("/", handler, "example");

    let mut chain = Chain::new(router);
    chain.link_after(Custom404);

    Iron::new(chain).http("localhost:3000");
}

fn handler(_: &mut Request) -> IronResult<Response> {
    Ok(Response::with((StatusCode::OK, "Handling response")))
}
