//! HTTP/HTTPS URL type for Iron.

use url::{self, Host};
use std::fmt;

/// HTTP/HTTPS URL type for Iron.
#[derive(PartialEq, Eq, Clone, Debug)]
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
    pub fragment: Option<String>,

    /// The generic rust-url that corresponds to this Url
    generic_url: url::Url,
}

impl Url {
    /// Create a URL from a string.
    ///
    /// The input must be a valid URL with a special scheme for this to succeed.
    ///
    /// HTTP and HTTPS are special schemes.
    ///
    /// See: http://url.spec.whatwg.org/#special-scheme
    pub fn parse(input: &str) -> Result<Url, String> {
        // Parse the string using rust-url, then convert.
        match url::Url::parse(input) {
            Ok(raw_url) => Url::from_generic_url(raw_url),
            Err(e) => Err(format!("{}", e))
        }
    }

    /// Create a `Url` from a `rust-url` `Url`.
    pub fn from_generic_url(raw_url: url::Url) -> Result<Url, String> {
        // Create an Iron URL by extracting the special scheme data.
        if raw_url.cannot_be_a_base() {
            Err(format!("Not a special scheme: `{}`", raw_url.scheme()))
        } else {
            // Extract the port as a 16-bit unsigned integer.
            let port: u16 = match raw_url.port_or_known_default() {
                // If explicitly defined or has a known default, unwrap it.
                Some(port) => port,

                // Otherwise, use the scheme's default port.
                None => return Err(format!("Invalid special scheme: `{}`",
                                                raw_url.scheme())),
            };
            // Map empty usernames to None.
            let username = match raw_url.username() {
                "" => None,
                _ => Some(raw_url.username().to_string())
            };
            // Map empty passwords to None.
            let password = match raw_url.password() {
                None => None,
                Some(ref x) if x.is_empty() => None,
                Some(password) => Some(password.to_string())
            };
            Ok(Url {
                scheme: raw_url.scheme().to_string(),
                // `unwrap` is safe here because urls that cannot be a base don't have a host
                host: raw_url.host().unwrap().to_owned(),
                port: port,
                // `unwrap` is safe here because urls that can be a base will have `Some`.
                path: raw_url.path_segments().unwrap().map(str::to_string).collect(),
                username: username,
                password: password,
                query: raw_url.query().map(str::to_string),
                fragment: raw_url.fragment().map(str::to_string),
                generic_url: raw_url,
            })
        }
    }

    /// Create a `rust-url` `Url` from a `Url`.
    pub fn into_generic_url(self) -> url::Url {
        self.generic_url
    }
}

impl fmt::Display for Url {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        try!(self.generic_url.fmt(formatter));
        Ok(())
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
    fn test_not_empty_username() {
        let user = Url::parse("http://john:pass@example.com").unwrap().username;
        assert_eq!(user.unwrap(), "john");

        let user = Url::parse("http://john:@example.com").unwrap().username;
        assert_eq!(user.unwrap(), "john");
    }

    #[test]
    fn test_empty_password() {
        assert!(Url::parse("http://michael@example.com").unwrap().password.is_none());
        assert!(Url::parse("http://:@example.com").unwrap().password.is_none());
    }

    #[test]
    fn test_not_empty_password() {
        let pass = Url::parse("http://michael:pass@example.com").unwrap().password;
        assert_eq!(pass.unwrap(), "pass");

        let pass = Url::parse("http://:pass@example.com").unwrap().password;
        assert_eq!(pass.unwrap(), "pass");
    }

    #[test]
    fn test_formatting() {
        assert_eq!(Url::parse("http://michael@example.com/path/?q=wow").unwrap().to_string(),
                    "http://michael@example.com/path/?q=wow".to_string());
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

    #[test]
    fn test_https_non_default_port() {
        let parsed = Url::parse("https://example.com:8080").unwrap().to_string();
        assert_eq!(parsed, "https://example.com:8080/");
    }

    #[test]
    fn test_https_default_port() {
        let parsed = Url::parse("https://example.com:443").unwrap().to_string();
        assert_eq!(parsed, "https://example.com/");
    }
}
