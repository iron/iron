//! HTTP/HTTPS URL type for Iron.

use url::{Host, RelativeSchemeData};
use url::{whatwg_scheme_type_mapper};
use url::{mod, SchemeData, SchemeType};
use url::format::{PathFormatter, UserInfoFormatter};
use std::fmt::{mod, Show};

use {serialize};

/// HTTP/HTTPS URL type for Iron.
#[deriving(PartialEq, Eq, Clone)]
pub struct Url {
    /// The lower-cased scheme of the URL, typically "http" or "https".
    pub scheme: String,

    /// The host field of the URL, probably a domain.
    pub host: Host,

    /// The connection port.
    pub port: u16,

    /// The URL path, the resource to be accessed.
    ///
    /// A *non-empty* vector encoding the parts of the URL path.
    /// Empty entries of `""` correspond to trailing slashes.
    pub path: Vec<String>,

    /// The URL username field, from the userinfo section of the URL.
    ///
    /// `None` if the `@` character was not part of the input OR
    /// if a blank username was provided.
    /// Otherwise, a non-empty string.
    pub username: Option<String>,

    /// The URL password field, from the userinfo section of the URL.
    ///
    /// `None` if the `@` character was not part of the input OR
    /// if a blank password was provided.
    /// Otherwise, a non-empty string.
    pub password: Option<String>,

    /// The URL query string.
    ///
    /// `None` if the `?` character was not part of the input.
    /// Otherwise, a possibly empty, percent encoded string.
    pub query: Option<String>,

    /// The URL fragment.
    ///
    /// `None` if the `#` character was not part of the input.
    /// Otherwise, a possibly empty, percent encoded string.
    pub fragment: Option<String>
}

impl Url {
    /// Create a URL from a string.
    ///
    /// The input must be a valid URL in a relative scheme for this to succeed.
    ///
    /// HTTP and HTTPS are relative schemes.
    ///
    /// See: http://url.spec.whatwg.org/#relative-scheme
    pub fn parse(input: &str) -> Result<Url, String> {
        // Parse the string using rust-url, then convert.
        match url::Url::parse(input) {
            Ok(raw_url) => Url::from_generic_url(raw_url),
            Err(e) => Err(format!("{}", e))
        }
    }

    /// Create a `Url` from a `rust-url` `Url`.
    pub fn from_generic_url(raw_url: url::Url) -> Result<Url, String> {
        // Create an Iron URL by extracting the relative scheme data.
        match raw_url.scheme_data {
            SchemeData::Relative(data) => {
                // Extract the port as a 16-bit unsigned integer.
                let port: u16 = match data.port {
                    // If explicitly defined, unwrap it.
                    Some(port) => port,

                    // Otherwise, use the scheme's default port.
                    None => {
                        match whatwg_scheme_type_mapper(raw_url.scheme.as_slice()) {
                            SchemeType::Relative(port) => port,
                            _ => return Err(format!("Invalid relative scheme: `{}`", raw_url.scheme))
                        }
                    }
                };

                // Map empty usernames to None.
                let username = match data.username.as_slice() {
                    "" => None,
                    _ => Some(data.username)
                };

                // Map empty passwords to None.
                let password = match data.password {
                    None => None,
                    Some(ref x) if x.as_slice().is_empty() => None,
                    Some(password) => Some(password)
                };

                Ok(Url {
                    scheme: raw_url.scheme,
                    host: data.host,
                    port: port,
                    path: data.path,
                    username: username,
                    password: password,
                    query: raw_url.query,
                    fragment: raw_url.fragment
                })
            },
            _ => Err(format!("Not a relative scheme: `{}`", raw_url.scheme))
        }
    }

    /// Create a `rust-url` `Url` from a `Url`.
    pub fn into_generic_url(self) -> url::Url {
        let default_port = whatwg_scheme_type_mapper(self.scheme[]).default_port();

        url::Url {
            scheme: self.scheme,
            scheme_data: SchemeData::Relative(
                RelativeSchemeData {
                    username: self.username.unwrap_or("".to_string()),
                    password: self.password,
                    host: self.host,
                    port: Some(self.port),
                    default_port: default_port,
                    path: self.path
                }
            ),
            query: self.query,
            fragment: self.fragment
        }
    }
}

impl Show for Url {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        // Write the scheme.
        try!(self.scheme.fmt(formatter));
        try!("://".fmt(formatter));

        // Write the user info.
        try!(UserInfoFormatter {
            username: self.username.as_ref().map(|s| s.as_slice()).unwrap_or(""),
            password: self.password.as_ref().map(|s| s.as_slice())
        }.fmt(formatter));

        // Write the host.
        try!(self.host.fmt(formatter));

        // Write the port.
        try!(":".fmt(formatter));
        try!(self.port.fmt(formatter));

        // Write the path.
        try!(PathFormatter { path: self.path.as_slice() }.fmt(formatter));

        // Write the query.
        match self.query {
            Some(ref query) => {
                try!("?".fmt(formatter));
                try!(query.fmt(formatter));
            },
            None => ()
        }

        // Write the fragment.
        match self.fragment {
            Some(ref fragment) => {
                try!("#".fmt(formatter));
                try!(fragment.fmt(formatter));
            },
            None => ()
        }

        Ok(())
    }
}

impl<E, S: serialize::Encoder<E>> serialize::Encodable<S, E> for Url {
    fn encode(&self, encoder: &mut S) -> Result<(), E> {
        encoder.emit_str(self.to_string().as_slice())
    }
}

impl<E, D: serialize::Decoder<E>> serialize::Decodable<D, E> for Url {
    fn decode(decoder: &mut D) -> Result<Url, E> {
        Url::parse(try!(decoder.read_str()).as_slice()).map_err(|error| {
            decoder.error(format!("URL parsing error: {}", error).as_slice())
        })
    }
}

#[cfg(test)]
mod test {
    use super::Url;

    #[test]
    fn test_default_port() {
        assert_eq!(Url::parse("http://example.com/wow").unwrap().port, 80u16);
        assert_eq!(Url::parse("https://example.com/wow").unwrap().port, 443u16);
    }

    #[test]
    fn test_explicit_port() {
        assert_eq!(Url::parse("http://localhost:3097").unwrap().port, 3097u16);
    }

    #[test]
    fn test_empty_username() {
        assert!(Url::parse("http://@example.com").unwrap().username.is_none());
        assert!(Url::parse("http://:password@example.com").unwrap().username.is_none());
    }

    #[test]
    fn test_empty_password() {
        assert!(Url::parse("http://michael@example.com").unwrap().password.is_none());
        assert!(Url::parse("http://:@example.com").unwrap().password.is_none());
    }

    #[test]
    fn test_formatting() {
        assert_eq!(Url::parse("http://michael@example.com/path/?q=wow").unwrap().to_string(),
                    "http://michael@example.com:80/path/?q=wow".to_string());
    }

    #[test]
    fn test_conversion() {
        let url_str = "https://user:password@iron.com:8080/path?q=wow#fragment";
        let url = Url::parse(url_str).unwrap();

        // Convert to a generic URL and check fidelity.
        let raw_url = url.clone().into_generic_url();
        assert_eq!(::url::Url::parse(url_str).unwrap(), raw_url);

        // Convert back to an Iron URL and check fidelity.
        let new_url = Url::from_generic_url(raw_url).unwrap();
        assert_eq!(url, new_url);
    }
}
