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
    Regex::new("^".to_string().append(debound.as_slice()).append("$").as_slice()).unwrap()
}

