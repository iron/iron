extern crate iron;

use std::env;

use iron::status;
use iron::mime::*;
use iron::headers::*;
use iron::prelude::*;

// All these variants do the same thing, with more or less options for customization.

fn variant1(_: &mut Request) -> IronResult<Response> {
    let mut resp = Response::with((status::Ok, "{}"));
    let content_type = ContentType(Mime(TopLevel::Application, SubLevel::Json, vec![]));

    resp.headers.set(content_type);

    Ok(resp)
}

fn variant2(_: &mut Request) -> IronResult<Response> {
    let mut resp = Response::with((status::Ok, "{}"));
    let content_type = ContentType(Mime(TopLevel::Application, SubLevel::Json, vec![]));
    resp.headers.set(content_type);

    Ok(resp)
}

fn variant3(_: &mut Request) -> IronResult<Response> {
    let mut resp = Response::with((status::Ok, "{}"));
    let content_type = ContentType(Mime(TopLevel::Application, SubLevel::Json, vec![]));
    resp.headers.set(content_type);

    Ok(resp)
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let variant_index = if args.len() > 1 {
        args[1].parse().unwrap()
    } else {
        1
    };
    let handler = match variant_index {
        1 => variant1,
        2 => variant2,
        3 => variant3,
        _ => panic!("No such variant"),
    };
    println!("Using variant{}", variant_index);
    Iron::new(handler).http("localhost:3000");
}
