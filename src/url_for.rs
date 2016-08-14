use std::collections::HashMap;
use std::iter::FromIterator;

use url::Url;

use iron::prelude::*;
use router::RouterInner;

/// Generate a URL based off of the currently requested URL.
///
/// The `route_id` used during route registration will be used here again.
///
/// `params` will be inserted as route parameters if fitting, the rest will be appended as query
/// parameters.
pub fn url_for(request: &Request, route_id: &str, params: HashMap<String, String>) -> Url {
    let inner = request.extensions.get::<RouterInner>().expect("Couldn\'t find router set up properly.");
    let glob = inner.route_ids.get(route_id).expect("No route with that ID");

    let mut url = request.url.clone().into_generic_url();
    url_for_impl(&mut url, glob, params);
    url
}

fn url_for_impl(url: &mut Url, glob: &str, mut params: HashMap<String, String>) {
    let mut glob_iter = glob.chars();
    let mut path = String::new();

    while glob_iter.size_hint().1.unwrap() > 0 {
        path.extend(glob_iter.by_ref().take_while(|&x| x != ':' && x != '*'));

        let key = String::from_iter(glob_iter.by_ref().take_while(|&x| x != '/'));
        if key.is_empty() { continue }

        let value = match params.remove(&key) {
            Some(x) => x,
            None => panic!("No value for key {}", key)
        };
        path.push_str(&value);
    }

    url.set_path(&path);

    // Now add on the remaining parameters that had no path match.
    url.query_pairs_mut()
        .clear()
        .extend_pairs(params.into_iter());

    url.set_fragment(None);
}
