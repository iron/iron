use regex::Regex;
use std::collections::hashmap::HashMap;

/// `Params` contains a `HashMap` of all the glob parameters stored in alloy.
///
/// For instance, for a glob pattern like `/users/:userid` `Params` will
/// contain the key `userid` matched with the value from the request url.
/// This works for any number of parameters in a glob pattern.
///
/// These values can be accessed using `Params::get`.
pub struct Params {
    captures: HashMap<String, String>
}

impl Params {
    #[doc(hidden)]
    pub fn new<I: Iterator<String>>(uri: &str, matcher: Regex, params: I) -> Params {
        let captures = matcher.captures(uri).unwrap();
        Params {
            captures: params.map(
                          // Map captures into (String, String) pairs so they
                          // can be collected into a HashMap<String, String>
                          |p| (p.clone(), captures.name(p.as_slice()).to_string())
                      ).collect()
        }
    }

    /// `get` allows you to query the HashMap contained inside Params.
    ///
    /// You can use `get` to access the parameters in the glob pattern
    /// for the matched route which are from the request url.
    ///
    /// If your route contains `:groupid`, for instance, then you can get its
    /// value from a `Params` by doing `params.get("groupid")`.
    pub fn get<'a>(&'a self, param: &str) -> Option<String> {
        self.captures.find(&param.to_string()).and_then(|p| Some(p.clone()))
    }
}

#[cfg(test)]
mod test {
    use test::{Bencher, black_box};
    use super::*;
    use super::super::glob::deglob;

    #[test]
    fn test_new() {
        let params = Params::new(
            "/users/7324/235",
            deglob("/users/:userid/:groupid".to_string()),
            vec!["userid".to_string(), "groupid".to_string()].move_iter());
        assert_eq!(params.get("userid").unwrap(), "7324".to_string());
        assert_eq!(params.get("groupid").unwrap(), "235".to_string());
    }

    #[bench]
    fn bench_get(b: &mut Bencher) {
        let params = Params::new("/users/7324", deglob("/users/:userid".to_string()), vec!["userid".to_string()].move_iter());
        b.iter(|| {
            black_box(params.get("userid").unwrap())
        })
    }
}

