extern crate iron;
#[macro_use] extern crate router;

// To run, $ cargo run --example url_for
// Go to http://localhost:3000 to see "Please go to: /test?extraparam=foo", dynamically generated
// from the original route.
// Go to http://localhost:3000/test to see "test".
// Go to http://localhost:3000/foo to see "foo".

use iron::prelude::*;
use iron::status;
use router::Router;

fn main() {
    let router = router!{
        id_1: get "/" => handler,
        id_2: get "/:query" => query_handler
    };

    Iron::new(router).http("localhost:3000").unwrap();

    fn handler(r: &mut Request) -> IronResult<Response> {
        Ok(Response::with((
            status::Ok,
            format!("Please go to: {}",
                    url_for!(r, "id_2",
                             "query" => "test",
                             "extraparam" => "foo"))
        )))
    }

    fn query_handler(req: &mut Request) -> IronResult<Response> {
        let ref query = req.extensions.get::<Router>()
            .unwrap().find("query").unwrap_or("/");
        Ok(Response::with((status::Ok, *query)))
    }


}
