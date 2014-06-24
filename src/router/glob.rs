use regex::{Regex, Captures};

static PARAMS: Regex = regex!(r":([a-zA-Z0-9_-]*)");

pub fn deglob(glob: String) -> Regex {
    // Replace glob patterns with corresponding regexs.
    let deglobbed = glob
        // Have to do this because the ** regex contains *
        .replace("**", "___DOUBLE_WILDCARD___")
        // Now only __DOUBLE_WILDCARD___ remains.
        .replace("*", "[a-zA-Z0-9_-]*")
        // Replace ** with its associated regex.
        .replace("___DOUBLE_WILDCARD___", "[/a-zA-Z0-9_-]*");
    // Replace :param patterns with corresponding regexs.
    let debound = PARAMS
        .replace_all(deglobbed.as_slice(), |cap: &Captures| {
            "(?P<".to_string().append(cap.at(1)).append(">[a-zA-Z0-9_-]*)")
        });
    Regex::new("^".to_string().append(debound.as_slice()).append(r"(\?[a-zA-Z0-9_=&-]*)?$").as_slice()).unwrap()
}

#[cfg(test)]
mod test {
    use super::*;
    use test::{Bencher, black_box};

    #[test]
    fn test_segment_match() {
        // Do literal matches work?
        let glob_regex = deglob("/users/nested/groups".to_string());
        assert!(glob_regex.is_match("/users/nested/groups"));
        assert!(!glob_regex.is_match("/notusers/hello"))
    }

    #[test]
    fn test_wildcard_match() {
        // Do wilcard matches work?
        let glob_regex = deglob("/users/*/groups".to_string());
        assert!(glob_regex.is_match("/users/nested/groups"));
        assert!(glob_regex.is_match("/users//groups"));
        assert!(!glob_regex.is_match("/users/deeply/nested/groups"));
    }

    #[test]
    fn test_double_wildcard_match() {
        // Do double wildcard matches work?
        let glob_regex = deglob("/users/**/groups".to_string());
        assert!(glob_regex.is_match("/users/deeply/nested/groups"));
        assert!(glob_regex.is_match("/users//groups"));
        assert!(!glob_regex.is_match("/notusers/groups/"))
    }

    #[test]
    fn test_params_match() {
        // Does param matching work?
        let glob_regex = deglob("/users/:groupid/:userid".to_string());
        assert!(glob_regex.is_match("/users/73564/87684"));
        assert!(!glob_regex.is_match("/users/234/groups/2343"));
    }

    #[test]
    fn test_params_value() {
        // Do params have the correct value?
        let glob_regex = deglob("/users/:groupid/:userid".to_string());
        assert_eq!(glob_regex.captures("/users/7374/234").unwrap().name("userid"), "234");
        assert_eq!(glob_regex.captures("/users/7374/234").unwrap().name("groupid"), "7374");
    }

    #[test]
    fn test_querystring_match() {
        // Does this work with url parameters?
        let glob_regex = deglob("/users".to_string());
        assert!(glob_regex.is_match("/users?foo=bar"));
    }

    #[bench]
    fn bench_explicit_match(b: &mut Bencher) {
        let glob_regex = deglob("/users/get".to_string());
        b.iter(|| {
                glob_regex.is_match("/users/get");
        })
    }

    #[bench]
    fn bench_explicit_match_native(b: &mut Bencher) {
        let glob_regex = regex!(r"/users/get");
        b.iter(|| {
                glob_regex.is_match("/users/get");
        })
    }

    #[bench]
    fn bench_easy_match(b: &mut Bencher) {
        let glob_regex = deglob("/users/*".to_string());
        b.iter(|| {
            black_box({
                glob_regex.is_match("/users/jonathan");
            })
        })
    }

    #[bench]
    fn bench_hard_match(b: &mut Bencher) {
        let glob_regex = deglob("/users/**/:userid/*/groups".to_string());
        b.iter(|| {
            black_box({
                glob_regex.is_match("/users/jonathan/reem/768/random/groups");
            })
        })
    }

    #[bench]
    fn bench_hard_match_native(b: &mut Bencher) {
        let glob_regex = regex!(r"/users/[/a-zA-Z0-9_-]*/(?P<userid>[a-zA-Z0-9_-]*)/[a-zA-Z0-9_-]*/groups");
        b.iter(|| {
            black_box({
                glob_regex.is_match("/users/jonathan/reem/768/random/groups");
            })
        })
    }
}

