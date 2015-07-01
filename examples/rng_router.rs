extern crate iron;
extern crate router;
extern crate rand;

use iron::prelude::*;
use iron::status::Status;

use router::Router;

use rand::Rng;
use rand::OsRng;

/// These functions return a single unsigned integer of size 64 bits or a
/// float between 0 and 1 of size 64 bits.
///
/// They are matched by the routes /u and /f respectively.

fn u_rand(_: &mut Request) -> IronResult<Response> {
    let mut rng = OsRng::new().unwrap();
    let rand_num: u64 = rng.next_u64();

    Ok(Response::with((Status::Ok, format!("{}", rand_num))))
}

fn f_rand(_: &mut Request) -> IronResult<Response> {
    let mut rng = OsRng::new().unwrap();
    let rand_num: f64 = rng.next_f64();

    Ok(Response::with((Status::Ok, format!("{}", rand_num))))
}

/// These functions return an unsigned integer of size 64 bits or a float of
/// size 64 bits defined by a user given range.
///
/// They are matched by the routes /range/u/:min/:max and /range/f/:min/:max
/// respectively.

fn u_range(req: &mut Request) -> IronResult<Response> {
    let mut rng = OsRng::new().unwrap();

    let ref min_str = req.extensions.get::<Router>().unwrap().find("min").unwrap();
    let ref max_str = req.extensions.get::<Router>().unwrap().find("max").unwrap();
    let min: u64 = min_str.parse().unwrap();
    let max: u64 = max_str.parse().unwrap();
    if min >= max {
        return Ok(Response::with((Status::Ok, "The minimum should be less than max!")));
    }

    let rand_num: u64 = rng.gen_range(min, max);

    Ok(Response::with((Status::Ok, format!("{}", rand_num))))
}

fn f_range(req: &mut Request) -> IronResult<Response> {
    let mut rng = OsRng::new().unwrap();

    let ref min_str = req.extensions.get::<Router>().unwrap().find("min").unwrap();
    let ref max_str = req.extensions.get::<Router>().unwrap().find("max").unwrap();
    let min: f64 = min_str.parse().unwrap();
    let max: f64 = max_str.parse().unwrap();
    if min >= max {
        return Ok(Response::with((Status::Ok, "The minimum should be less than max!")));
    }

    let rand_num: f64 = rng.gen_range(min, max);

    Ok(Response::with((Status::Ok, format!("{}", rand_num))))
}

fn main() {
    let mut router = Router::new();

    router.get("/u", u_rand);
    router.get("/f", f_rand);
    router.get("/range/u/:min/:max", u_range);
    router.get("/range/f/:min/:max", f_range);

    let _ = Iron::new(router).http("localhost:3000");
}
