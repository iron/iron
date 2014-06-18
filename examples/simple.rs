extern crate iron;
extern crate http;
extern crate router;

use std::io::net::ip::Ipv4Addr;
use iron::{ServerT, Iron, Alloy, IronRequest, IronResponse};
use http::method::Get;
use router::{Router, Params};

fn main() {
    let mut server: ServerT = Iron::new();
    let mut router = Router::new();

    fn handler(_req: &mut IronRequest, res: &mut IronResponse, alloy: &mut Alloy) {
        let query = alloy.find::<Params>().unwrap().get("query").unwrap();
        let _ = res.write(query.as_bytes());
    }

    // Setup our route.
    router.route(
        Get,
        "/:query".to_string(),
        vec!["query".to_string()], 
        handler);

    server.smelt(router);
    server.listen(Ipv4Addr(127, 0, 0, 1), 3000);
}

