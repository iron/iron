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

    pub fn get<'a>(&'a self, param: String) -> Option<&'a String> {
        self.captures.find(&param)
    }
}

