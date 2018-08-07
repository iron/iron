extern crate iron;

use iron::{AfterMiddleware, Chain, Iron, IronResult, Request, Response};

struct DefaultContentType;
impl AfterMiddleware for DefaultContentType {
    // This is run for every requests, AFTER all handlers have been executed
    fn after(&self, _req: &mut Request, mut resp: Response) -> IronResult<Response> {
        if resp.headers.get(iron::headers::CONTENT_TYPE) == None {
            // Set a standard header
            resp.headers.insert(
                iron::headers::CONTENT_TYPE,
                iron::mime::TEXT_PLAIN.as_ref().parse().unwrap(),
            );
        }
        Ok(resp)
    }
}

fn info(req: &mut Request) -> IronResult<Response> {
    // Get a header using a standard iron::headers
    let ua = match req.headers.get(iron::headers::USER_AGENT) {
        Some(ua_header) => format!("User Agent: {}\n", ua_header.to_str().unwrap()),
        None => "No User Agent.\n".to_string(),
    };
    // Get a non-standard header
    let x_forwarded_for = match req.headers.get("X-Forwarded-For") {
        Some(proxies) => format!("Proxies: {}\n", proxies.to_str().unwrap()),
        None => "No proxy.\n".to_string(),
    };
    let body = format!("{}{}\n", ua, x_forwarded_for);

    Ok(Response::with((iron::StatusCode::OK, body)))
}

fn main() {
    let mut chain = Chain::new(info);
    chain.link_after(DefaultContentType);
    Iron::new(chain).http(format!("localhost:{}", 3000));
}
