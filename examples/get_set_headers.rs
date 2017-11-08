extern crate iron;

use iron::{Iron, Request, Response, IronResult, AfterMiddleware, Chain};

struct DefaultContentType;
impl AfterMiddleware for DefaultContentType {
    // This is run for every requests, AFTER all handlers have been executed
    fn after(&self, _req: &mut Request, mut resp: Response) -> IronResult<Response> {
        if resp.headers.get::<iron::headers::ContentType>() == None {
            // Set a standard header
            resp.headers.set(iron::headers::ContentType::plaintext());
        }
        Ok(resp)
    }
}



fn info(req: &mut Request) -> IronResult<Response> {
    // Get a header using a standard iron::headers
    let ua = match req.headers.get::<iron::headers::UserAgent>() {
        Some(ua_header) => format!("User Agent: {}\n", ua_header),
        None => "No User Agent.\n".to_string(),
    };
    // Get a non-standard header using the raw header
    let x_forwarded_for = match req.headers.get_raw("X-Forwarded-For") {
        Some(proxies) => format!("Proxies: {}\n", std::str::from_utf8(&proxies[0]).unwrap()),
        None => "No proxy.\n".to_string(),
    };
    let body = format!("{}{}\n", ua, x_forwarded_for);

    Ok(Response::with((iron::status::Ok, body)))
}

fn main() {
    let mut chain = Chain::new(info);
    chain.link_after(DefaultContentType);
    Iron::new(chain)
        .http(format!("localhost:{}", 3000))
        .unwrap();
}
