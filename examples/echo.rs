// An example that echoes the body of the request back as the response.
//
// Shows how to read the request body with error handling and how to return a
// response. See `helper_macros` example for a different way to handle errors.

extern crate iron;

use std::io::Read;

use iron::prelude::*;
use iron::status;

fn echo(request: &mut Request) -> IronResult<Response> {
    let mut body = Vec::new();
    request
        .body
        .read_to_end(&mut body)
        .map_err(|e| IronError::new(e, (status::InternalServerError, "Error reading request")))?;
    Ok(Response::with((status::Ok, body)))
}

fn main() {
    Iron::new(echo).http("localhost:3000").unwrap();
}
