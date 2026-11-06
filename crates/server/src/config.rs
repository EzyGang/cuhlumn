//! Server settings from flags and environment variables.

use std::{fmt, path::PathBuf, str::FromStr};

use clap::{Parser, ValueEnum};
use cuhlumn_api::site::host::Host;
use sha2::{Digest, Sha256};
use subtle::ConstantTimeEq;

/// The server settings.
#[derive(Clone, Debug, Parser)]
#[command(
    name = "cuhlumn-server",
    version,
    about = "Serves many blogs, each on its own domain"
)]
pub struct Config {
    /// Port to listen on, on all addresses.
    #[arg(long, env = "PORT", default_value_t = 8080)]
    pub port: u16,
    /// Folder for `cuhlumn.db` and the uploaded images.
    #[arg(long, env = "CUHLUMN_DATA", default_value = "./data")]
    pub data: PathBuf,
    /// Admin token of at least 32 bytes. Without it, the API and the console return 503.
    #[arg(long, env = "CUHLUMN_TOKEN", hide_env_values = true)]
    pub token: Option<Token>,
    /// Scheme of absolute addresses and of the cookie `Secure` flag.
    #[arg(long, env = "CUHLUMN_SCHEME", value_enum, default_value_t = Scheme::Https)]
    pub scheme: Scheme,
    /// Domain that serves the console. Without it, the console is off.
    #[arg(long, env = "CUHLUMN_CONSOLE_HOST")]
    pub console_host: Option<Host>,
    /// Sends IndexNow pings after publication.
    #[arg(long, env = "CUHLUMN_INDEXNOW", value_enum, default_value_t = Switch::On)]
    pub indexnow: Switch,
    /// Extra font folder for share cards.
    #[arg(long, env = "CUHLUMN_FONTS")]
    pub fonts: Option<PathBuf>,
}

/// The scheme of public addresses.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, ValueEnum)]
pub enum Scheme {
    Http,
    Https,
}

/// An on or off setting.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, ValueEnum)]
pub enum Switch {
    On,
    Off,
}

/// The admin token. Only its SHA-256 digest stays in memory.
#[derive(Clone)]
pub struct Token {
    digest: [u8; 32],
}

/// A token that is too short.
#[derive(Clone, Copy, Debug, PartialEq, Eq, thiserror::Error)]
#[error("the admin token must have at least {MIN_TOKEN_LEN} bytes")]
pub struct TokenError;

const MIN_TOKEN_LEN: usize = 32;

impl Config {
    /// Returns the admin token, or `None` when the API is off.
    #[must_use]
    pub fn token(&self) -> Option<&Token> {
        self.token.as_ref()
    }

    /// Returns `true` when `host` is the console domain.
    #[must_use]
    pub fn is_console_host(&self, host: &Host) -> bool {
        self.console_host.as_ref() == Some(host)
    }

    /// Returns the address of the site root without a trailing slash, such as `https://blog.example.com`.
    #[must_use]
    pub fn origin(&self, host: &Host) -> String {
        format!("{}://{host}", self.scheme)
    }
}

impl Scheme {
    /// Returns `http` or `https`.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Http => "http",
            Self::Https => "https",
        }
    }
}

impl fmt::Display for Scheme {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl Token {
    /// Validates a token of at least 32 bytes.
    ///
    /// # Errors
    ///
    /// Returns [`TokenError`] when the token is shorter than 32 bytes.
    pub fn new(value: &str) -> Result<Self, TokenError> {
        if value.len() < MIN_TOKEN_LEN {
            return Err(TokenError);
        }
        Ok(Self {
            digest: Sha256::digest(value.as_bytes()).into(),
        })
    }

    /// Compares a candidate with the token in constant time.
    #[must_use]
    pub fn matches(&self, candidate: &str) -> bool {
        let candidate: [u8; 32] = Sha256::digest(candidate.as_bytes()).into();
        self.digest.ct_eq(&candidate).into()
    }

    /// Returns the SHA-256 hex digest that the database stores for a console session identifier.
    ///
    /// The digest covers the token digest and the identifier, so a new token makes old sessions invalid.
    #[must_use]
    pub fn session_hash(&self, id: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(self.digest);
        hasher.update(id.as_bytes());
        hex::encode(hasher.finalize())
    }
}

impl FromStr for Token {
    type Err = TokenError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Self::new(value)
    }
}

impl fmt::Debug for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("Token(<redacted>)")
    }
}

#[cfg(test)]
mod tests {
    use clap::Parser;

    use crate::config::{Config, Scheme, Switch, Token};

    #[test]
    fn token_needs_32_bytes() {
        assert!(Token::new("short").is_err());
        let token = Token::new(&"a".repeat(32)).unwrap();
        assert!(token.matches(&"a".repeat(32)));
        assert!(!token.matches(&"a".repeat(31)));
        assert_eq!(format!("{token:?}"), "Token(<redacted>)");
    }

    #[test]
    fn flags_parse_into_typed_values() {
        let config = Config::try_parse_from([
            "cuhlumn-server",
            "--port",
            "9000",
            "--scheme",
            "http",
            "--console-host",
            "Console.Example.com",
            "--indexnow",
            "off",
            "--token",
            "0123456789abcdef0123456789abcdef",
        ])
        .unwrap();
        assert_eq!(config.port, 9000);
        assert_eq!(config.scheme, Scheme::Http);
        assert_eq!(config.indexnow, Switch::Off);
        assert_eq!(config.console_host.unwrap().as_str(), "console.example.com");
        assert!(Config::try_parse_from(["cuhlumn-server", "--token", "short"]).is_err());
    }
}
