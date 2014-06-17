use regex::Regex;
use std::collections::hashmap::HashMap;

pub struct Params {
    captures: HashMap<String, String>
}

impl Params {
    pub fn new<I: Iterator<String>>(uri: &str, matcher: Regex, params: I) -> Params {
        let captures = matcher.captures(uri).unwrap();
        Params {
            captures: params.map(|p| (p.clone(), captures.name(p.as_slice()).to_string())).collect()
        }
    }

    pub fn get<'a>(&'a self, param: &'static str) -> Option<String> {
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
        let params = Params::new("/users/7324", deglob("/users/:userid".to_string()), vec!["userid".to_string()].move_iter());
        assert_eq!(params.get("userid").unwrap(), "7324".to_string());
    }

    #[bench]
    fn bench_get(b: &mut Bencher) {
        let params = Params::new("/users/7324", deglob("/users/:userid".to_string()), vec!["userid".to_string()].move_iter());
        b.iter(|| {
            black_box(params.get("userid").unwrap())
        })
    }
}

