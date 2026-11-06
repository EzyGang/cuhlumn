//! Domain names of sites.

use serde::{Deserialize, Serialize};

/// A domain name such as `blog.example.com`, stored without a trailing dot and in lowercase.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[serde(try_from = "String", into = "String")]
pub struct Host(String);

/// A text that is not a valid domain name.
#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
pub enum HostError {
    /// The text is empty.
    #[error("the host name is empty")]
    Empty,
    /// The text has more than 253 bytes.
    #[error("the host name is longer than 253 bytes")]
    TooLong,
    /// One label between the dots breaks the label rules.
    #[error(
        "the host name label {0:?} is not valid: use 1 to 63 letters, digits, or hyphens, without a hyphen at the start or end"
    )]
    Label(String),
}

const MAX_LEN: usize = 253;
const MAX_LABEL_LEN: usize = 63;

impl Host {
    /// Validates a domain name.
    ///
    /// The result has no trailing dot and only lowercase letters.
    /// Single-label names such as `localhost` are valid.
    ///
    /// # Errors
    ///
    /// Returns [`HostError`] when the text is not a valid domain name.
    pub fn new(value: impl Into<String>) -> Result<Self, HostError> {
        let mut value = value.into();
        let len = value.trim_end_matches('.').len();
        value.truncate(len);
        value.make_ascii_lowercase();
        if value.is_empty() {
            return Err(HostError::Empty);
        }
        if value.len() > MAX_LEN {
            return Err(HostError::TooLong);
        }
        if let Some(label) = value.split('.').find(|label| !is_label(label)) {
            return Err(HostError::Label(label.to_owned()));
        }
        Ok(Self(value))
    }
}

string_type!(Host, HostError);

fn is_label(label: &str) -> bool {
    !label.is_empty()
        && label.len() <= MAX_LABEL_LEN
        && !label.starts_with('-')
        && !label.ends_with('-')
        && label
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
}

#[cfg(test)]
mod tests {
    use super::{Host, HostError};

    #[test]
    fn normalizes_case_and_trailing_dot() {
        assert_eq!(
            Host::new("Blog.Example.COM.").unwrap().as_str(),
            "blog.example.com"
        );
        assert_eq!(Host::new("localhost").unwrap().as_str(), "localhost");
        assert_eq!(
            Host::new("xn--d1acufc.xn--p1ai").unwrap().as_str(),
            "xn--d1acufc.xn--p1ai"
        );
    }

    #[test]
    fn rejects_invalid_names() {
        assert_eq!(Host::new(""), Err(HostError::Empty));
        assert_eq!(Host::new("."), Err(HostError::Empty));
        assert_eq!(
            Host::new("not_a_host"),
            Err(HostError::Label("not_a_host".to_owned()))
        );
        assert_eq!(Host::new("-a.com"), Err(HostError::Label("-a".to_owned())));
        assert_eq!(Host::new("a-.com"), Err(HostError::Label("a-".to_owned())));
        assert_eq!(Host::new("a..com"), Err(HostError::Label(String::new())));
        assert_eq!(
            Host::new("a.com:8080"),
            Err(HostError::Label("com:8080".to_owned()))
        );
        assert_eq!(Host::new("../etc"), Err(HostError::Label(String::new())));
    }

    #[test]
    fn enforces_length_limits() {
        let label = "a".repeat(63);
        assert!(Host::new(label.as_str()).is_ok());
        assert!(matches!(
            Host::new("a".repeat(64)),
            Err(HostError::Label(_))
        ));
        let long = [label.as_str(); 4].join(".");
        assert_eq!(long.len(), 255);
        assert_eq!(Host::new(long), Err(HostError::TooLong));
        let max = format!("{}.{}", [label.as_str(); 3].join("."), "a".repeat(61));
        assert_eq!(max.len(), 253);
        assert!(Host::new(max).is_ok());
    }

    #[test]
    fn deserializes_with_validation() {
        let host: Host = serde_json::from_str("\"Example.org\"").unwrap();
        assert_eq!(host.to_string(), "example.org");
        assert!(serde_json::from_str::<Host>("\"exa mple.org\"").is_err());
    }
}
