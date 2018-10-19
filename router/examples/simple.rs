extern crate iron;
extern crate router;

// To run, $ cargo run --example simple
// To use, go to http://localhost:3000/test and see output "test"
// Or, go to http://localhost:3000 to see a default "OK"

use iron::{Iron, Request, Response, IronResult, StatusCode};
use router::{Router};

fn main() {
    let mut router = Router::new();
    router.get("/", handler, "handler");
    router.get("/:query", query_handler, "query_handler");

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
