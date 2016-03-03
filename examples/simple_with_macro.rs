extern crate iron;
#[macro_use]
extern crate router;

// To run, $ cargo run --example simple_with_macro
// To use, go to http://localhost:3000/test and see output "test"

use iron::{Iron, Request, Response, IronResult};
use iron::status;
use router::{Router};

fn main() {
    let router = router!(get "/" => handler, get "/:query" => handler);

    Iron::new(router).http("localhost:3000").unwrap();

    fn handler(req: &mut Request) -> IronResult<Response> {
        let ref query = req.extensions.get::<Router>()
            .unwrap().find("query").unwrap_or("/");
        Ok(Response::with((status::Ok, *query)))
    }
}
