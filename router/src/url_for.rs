use std::collections::HashMap;

use url::Url;

use iron::prelude::*;
use router::RouterInner;

/// Generate a URL based off of the currently requested URL.
///
/// The `route_id` used during route registration will be used here again.
///
/// `params` will be inserted as route parameters if fitting, the rest will be appended as query
/// parameters.
pub fn url_for(request: &Request, route_id: &str, params: HashMap<String, String>) -> ::iron::Url {
    let inner = request.extensions.get::<RouterInner>().expect("Couldn\'t find router set up properly.");
    let glob = inner.route_ids.get(route_id).expect("No route with that ID");

    let mut url = request.url.clone();
    url_for_impl(url.as_mut(), glob, params);
    url
}

fn url_for_impl(url: &mut Url, glob: &str, mut params: HashMap<String, String>) {
    {
        let mut url_path_segments = url.path_segments_mut().unwrap();
        url_path_segments.clear();
        for path_segment in glob.split('/') {
            if path_segment.len() > 1 && (path_segment.starts_with(':') || path_segment.starts_with('*')) {
                let key = &path_segment[1..];
                match params.remove(key) {
                    Some(x) => url_path_segments.push(&x),
                    None => panic!("No value for key {}", key)
                };
            } else {
                url_path_segments.push(path_segment);
            }
        }
    }

    // Now add on the remaining parameters that had no path match.
    url.set_query(None);
    if !params.is_empty() {
        url.query_pairs_mut()
            .extend_pairs(params.into_iter());
    }

    url.set_fragment(None);
}

#[cfg(test)]
mod test {
    use super::url_for_impl;
    use std::collections::HashMap;

    #[test]
    fn test_no_trailing_slash() {
        let mut url = "http://localhost/foo/bar/baz".parse().unwrap();
        url_for_impl(&mut url, "/foo/:user", {
            let mut rv = HashMap::new();
            rv.insert("user".into(), "bam".into());
            rv
        });
        assert_eq!(url.to_string(), "http://localhost/foo/bam");
    }

    #[test]
    fn test_trailing_slash() {
        let mut url = "http://localhost/foo/bar/baz".parse().unwrap();
        url_for_impl(&mut url, "/foo/:user/", {
            let mut rv = HashMap::new();
            rv.insert("user".into(), "bam".into());
            rv
        });
        assert_eq!(url.to_string(), "http://localhost/foo/bam/");
    }
}
