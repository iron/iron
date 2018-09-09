extern crate iron;
#[macro_use]
extern crate router;

// To run, $ cargo run --example simple_with_macro
// To use, go to http://localhost:3000/test and see output "test"
// Or, go to http://localhost:3000 to see a default "OK"

use iron::{Iron, Request, Response, IronResult, StatusCode};
use router::{Router};

fn main() {
    let router = router!(root: get "/" => handler, query: get "/:query" => query_handler);

    Iron::new(router).http("localhost:3000");

    fn handler(_: &mut Request) -> IronResult<Response> {
        Ok(Response::with((StatusCode::OK, "OK")))
    }

    fn query_handler(req: &mut Request) -> IronResult<Response> {
        let ref query = req.extensions.get::<Router>()
            .unwrap().find("query").unwrap_or("/");
        Ok(Response::with((StatusCode::OK, *query)))
    }
}
