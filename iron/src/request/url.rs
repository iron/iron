//! HTTP/HTTPS URL type for Iron.

use std::fmt;
use std::str::FromStr;
use url::{self, Host};

/// HTTP/HTTPS URL type for Iron.
#[derive(PartialEq, Eq, Clone, Debug)]
pub struct Url {
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
            Err(e) => Err(format!("{}", e)),
        }
    }

    /// Create a `Url` from a `rust-url` `Url`.
    pub fn from_generic_url(raw_url: url::Url) -> Result<Url, String> {
        // Create an Iron URL by verifying the `rust-url` `Url` is a special
        // scheme that Iron supports.
        if raw_url.cannot_be_a_base() {
            Err(format!("Not a special scheme: `{}`", raw_url.scheme()))
        } else if raw_url.port_or_known_default().is_none() {
            Err(format!("Invalid special scheme: `{}`", raw_url.scheme()))
        } else {
            Ok(Url {
                generic_url: raw_url,
            })
        }
    }

    /// Create a `rust-url` `Url` from a `Url`.
    #[deprecated(
        since = "0.4.1",
        note = "use `into` from the `Into` trait instead"
    )]
    pub fn into_generic_url(self) -> url::Url {
        self.generic_url
    }

    /// The lower-cased scheme of the URL, typically "http" or "https".
    pub fn scheme(&self) -> &str {
        self.generic_url.scheme()
    }

    /// The host field of the URL, probably a domain.
    pub fn host(&self) -> Host<&str> {
        // `unwrap` is safe here because urls that cannot be a base don't have a host
        self.generic_url.host().unwrap()
    }

    /// The connection port.
    pub fn port(&self) -> u16 {
        // `unwrap` is safe here because we checked `port_or_known_default`
        // in `from_generic_url`.
        self.generic_url.port_or_known_default().unwrap()
    }

    /// The URL path, the resource to be accessed.
    ///
    /// A *non-empty* vector encoding the parts of the URL path.
    /// Empty entries of `""` correspond to trailing slashes.
    pub fn path(&self) -> Vec<&str> {
        // `unwrap` is safe here because urls that can be a base will have `Some`.
        self.generic_url.path_segments().unwrap().collect()
    }

    /// The URL username field, from the userinfo section of the URL.
    ///
    /// `None` if the `@` character was not part of the input OR
    /// if a blank username was provided.
    /// Otherwise, a non-empty string.
    pub fn username(&self) -> Option<&str> {
        // Map empty usernames to None.
        match self.generic_url.username() {
            "" => None,
            username => Some(username),
        }
    }

    /// The URL password field, from the userinfo section of the URL.
    ///
    /// `None` if the `@` character was not part of the input OR
    /// if a blank password was provided.
    /// Otherwise, a non-empty string.
    pub fn password(&self) -> Option<&str> {
        // Map empty passwords to None.
        match self.generic_url.password() {
            None => None,
            Some(x) if x.is_empty() => None,
            Some(password) => Some(password),
        }
    }

    /// The URL query string.
    ///
    /// `None` if the `?` character was not part of the input.
    /// Otherwise, a possibly empty, percent encoded string.
    pub fn query(&self) -> Option<&str> {
        self.generic_url.query()
    }

    /// The URL fragment.
    ///
    /// `None` if the `#` character was not part of the input.
    /// Otherwise, a possibly empty, percent encoded string.
    pub fn fragment(&self) -> Option<&str> {
        self.generic_url.fragment()
    }
}

impl fmt::Display for Url {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        self.generic_url.fmt(formatter)?;
        Ok(())
    }
}

impl Into<url::Url> for Url {
    fn into(self) -> url::Url {
        self.generic_url
    }
}

impl AsRef<url::Url> for Url {
    fn as_ref(&self) -> &url::Url {
        &self.generic_url
    }
}

impl AsMut<url::Url> for Url {
    fn as_mut(&mut self) -> &mut url::Url {
        &mut self.generic_url
    }
}

impl FromStr for Url {
    type Err = String;
    #[inline]
    fn from_str(input: &str) -> Result<Url, Self::Err> {
        Url::parse(input)
    }
}

#[cfg(test)]
mod test {
    use super::Url;

    #[test]
    fn test_default_port() {
        assert_eq!(Url::parse("http://example.com/wow").unwrap().port(), 80u16);
        assert_eq!(
            Url::parse("https://example.com/wow").unwrap().port(),
            443u16
        );
    }

    #[test]
    fn test_explicit_port() {
        assert_eq!(Url::parse("http://localhost:3097").unwrap().port(), 3097u16);
    }

    #[test]
    fn test_empty_username() {
        assert!(
            Url::parse("http://@example.com")
                .unwrap()
                .username()
                .is_none()
        );
        assert!(
            Url::parse("http://:password@example.com")
                .unwrap()
                .username()
                .is_none()
        );
    }

    #[test]
    fn test_not_empty_username() {
        let url = Url::parse("http://john:pass@example.com").unwrap();
        assert_eq!(url.username().unwrap(), "john");

        let url = Url::parse("http://john:@example.com").unwrap();
        assert_eq!(url.username().unwrap(), "john");
    }

    #[test]
    fn test_empty_password() {
        assert!(
            Url::parse("http://michael@example.com")
                .unwrap()
                .password()
                .is_none()
        );
        assert!(
            Url::parse("http://:@example.com")
                .unwrap()
                .password()
                .is_none()
        );
    }

    #[test]
    fn test_not_empty_password() {
        let url = Url::parse("http://michael:pass@example.com").unwrap();
        assert_eq!(url.password().unwrap(), "pass");

        let url = Url::parse("http://:pass@example.com").unwrap();
        assert_eq!(url.password().unwrap(), "pass");
    }

    #[test]
    fn test_formatting() {
        assert_eq!(
            Url::parse("http://michael@example.com/path/?q=wow")
                .unwrap()
                .to_string(),
            "http://michael@example.com/path/?q=wow".to_string()
        );
    }

    #[test]
    fn test_conversion() {
        let url_str = "https://user:password@iron.com:8080/path?q=wow#fragment";
        let url = Url::parse(url_str).unwrap();

        // Convert to a generic URL and check fidelity.
        let raw_url: ::url::Url = url.clone().into();
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

    #[test]
    fn test_from_str_positive() {
        let u = "http://example.com".parse::<Url>();
        assert!(u.is_ok());
        assert_eq!(u.unwrap(), Url::parse("http://example.com").unwrap());
    }

    #[test]
    fn test_from_str_negative() {
        let u = "not a url".parse::<Url>();
        assert!(u.is_err());
    }
}
