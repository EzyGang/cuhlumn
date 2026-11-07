//! Server settings that the console can change.

use std::fmt;

use serde::{Deserialize, Deserializer, Serialize, de::Error as _};

use crate::site::host::Host;

text_enum! {
    /// The scheme of public addresses.
    pub enum Scheme / SchemeError ("scheme") {
        Http = "http",
        #[default]
        Https = "https",
    }
}

/// The response of `GET /api/settings`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
pub struct ServerSettings {
    /// The only host that serves the console. `None` lets every host that is not a site serve it.
    pub console_host: Option<Host>,
    pub scheme: Scheme,
    /// `CUHLUMN_ADMIN_ADDR` is set, so the console and the admin API are only on the admin port.
    pub admin_listener: bool,
    /// The fields that environment variables set.
    pub locked: Locked,
}

/// The settings that environment variables set, so the console cannot change them.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
pub struct Locked {
    /// `CUHLUMN_TOKEN` is set.
    pub token: bool,
    /// `CUHLUMN_CONSOLE_HOST` or `CUHLUMN_ADMIN_ADDR` is set.
    pub console_host: bool,
    /// `CUHLUMN_SCHEME` is set.
    pub scheme: bool,
}

/// The body of `PUT /api/settings`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[serde(deny_unknown_fields)]
pub struct ServerSettingsInput {
    #[serde(default)]
    pub console_host: Option<Host>,
    #[serde(default)]
    pub scheme: Scheme,
}

/// The body of `POST /api/setup` and `PUT /api/settings/token`.
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[serde(deny_unknown_fields)]
pub struct TokenInput {
    /// The new admin token, 32 to 512 bytes.
    #[serde(deserialize_with = "de_token")]
    pub token: String,
}

/// A text that is not a permitted admin token.
#[derive(Clone, Copy, Debug, PartialEq, Eq, thiserror::Error)]
pub enum TokenError {
    /// The token is shorter than 32 bytes.
    #[error("the token must have at least 32 bytes")]
    TooShort,
    /// The token is longer than 512 bytes.
    #[error("the token must have at most 512 bytes")]
    TooLong,
}

const TOKEN_MIN: usize = 32;
const TOKEN_MAX: usize = 512;

impl TokenInput {
    /// Validates a new admin token.
    ///
    /// # Errors
    ///
    /// Returns [`TokenError`] when the token has fewer than 32 or more than 512 bytes.
    pub fn new(token: impl Into<String>) -> Result<Self, TokenError> {
        let token = token.into();
        if token.len() < TOKEN_MIN {
            return Err(TokenError::TooShort);
        }
        if token.len() > TOKEN_MAX {
            return Err(TokenError::TooLong);
        }
        Ok(Self { token })
    }
}

impl fmt::Debug for TokenInput {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("TokenInput")
            .field("token", &"<redacted>")
            .finish()
    }
}

fn de_token<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    let value = String::deserialize(deserializer)?;
    TokenInput::new(value)
        .map(|input| input.token)
        .map_err(D::Error::custom)
}

#[cfg(test)]
mod tests {
    use super::{Scheme, ServerSettingsInput, TokenError, TokenInput};

    #[test]
    fn token_length_is_limited() {
        assert_eq!(TokenInput::new("a".repeat(31)), Err(TokenError::TooShort));
        assert!(TokenInput::new("a".repeat(32)).is_ok());
        assert!(TokenInput::new("a".repeat(512)).is_ok());
        assert_eq!(TokenInput::new("a".repeat(513)), Err(TokenError::TooLong));
        let short = r#"{"token":"short"}"#;
        assert!(serde_json::from_str::<TokenInput>(short).is_err());
    }

    #[test]
    fn debug_redacts_the_token() {
        let secret = "s".repeat(40);
        let input = TokenInput::new(secret.clone()).unwrap();
        let debug = format!("{input:?}");
        assert!(!debug.contains(&secret));
        assert!(debug.contains("<redacted>"));
    }

    #[test]
    fn settings_input_defaults_and_rejects_unknown_fields() {
        let input: ServerSettingsInput = serde_json::from_str("{}").unwrap();
        assert_eq!(input.scheme, Scheme::Https);
        assert_eq!(input.console_host, None);
        assert!(serde_json::from_str::<ServerSettingsInput>(r#"{"token":"x"}"#).is_err());
        assert_eq!(
            "ftp".parse::<Scheme>().unwrap_err().to_string(),
            r#"the scheme "ftp" is not valid: use http or https"#
        );
    }
}
