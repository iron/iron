// An example that echoes the body of the request back as the response.
//
// Shows how to read the request body with error handling and how to return a
// response. See `helper_macros` example for a different way to handle errors.

extern crate iron;

use iron::prelude::*;
use iron::StatusCode;

fn echo(request: &mut Request) -> IronResult<Response> {
    let body = request.get_body_contents().map_err(|e| {
        IronError::new(
            e,
            (StatusCode::INTERNAL_SERVER_ERROR, "Error reading request"),
        )
    })?;
    Ok(Response::with((StatusCode::OK, body.clone())))
}

fn main() {
    Iron::new(echo).http("localhost:3000");
}
