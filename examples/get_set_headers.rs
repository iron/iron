extern crate iron;

use iron::{Iron, Request, Response, IronResult, AfterMiddleware, Chain};

struct DefaultContentType;
impl AfterMiddleware for DefaultContentType {
    fn after(&self, _req: &mut Request, mut resp: Response) -> IronResult<Response> {
        if resp.headers.get::<iron::headers::ContentType>() == None {
            resp.headers.set(iron::headers::ContentType::plaintext());
        }
        Ok(resp)
    }
}



fn user_agent(req: &mut Request) -> IronResult<Response> {
    let ua = match req.headers.get::<iron::headers::UserAgent>() {
        Some(ua_header) => ua_header,
        None => "No User-Agent",
    };
    Ok(Response::with((iron::status::Ok, format!("{}\n", ua))))
}

fn main() {
    let mut chain = Chain::new(user_agent);
    chain.link_after(DefaultContentType);
    Iron::new(chain)
        .http(format!("localhost:{}", 3000))
        .unwrap();
}
