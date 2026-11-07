//! Server settings from flags and environment variables.

use std::{
    ffi::{OsStr, OsString},
    fmt,
    path::PathBuf,
    str::FromStr,
};

use clap::{CommandFactory, FromArgMatches, Parser, Subcommand};
use cuhlumn_api::{settings::Scheme, site::host::Host};
use sha2::{Digest, Sha256};
use subtle::ConstantTimeEq;

/// The server settings.
///
/// The token, the console host, and the scheme are overrides.
/// When one is set, it wins over the database value and the console cannot change it.
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
    /// Folder for `cuhlumn.db`, the uploaded images, and the font library.
    #[arg(long, env = "CUHLUMN_DATA", default_value = "./data")]
    pub data: PathBuf,
    /// Admin token of at least 32 bytes. It overrides the token that the console stores.
    #[arg(long, env = "CUHLUMN_TOKEN", hide_env_values = true)]
    pub token: Option<Token>,
    /// Domain that serves the console. It overrides the console setting.
    #[arg(long, env = "CUHLUMN_CONSOLE_HOST")]
    pub console_host: Option<Host>,
    /// Scheme of absolute addresses and of the cookie `Secure` flag, `http` or `https`.
    #[arg(long, env = "CUHLUMN_SCHEME")]
    pub scheme: Option<Scheme>,
    /// An optional task to run instead of the server.
    #[command(subcommand)]
    pub command: Option<Task>,
}

/// A task that runs instead of the server.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Subcommand)]
pub enum Task {
    /// Asks the local server for its health and exits with 0 when it is healthy.
    Healthcheck,
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
    /// Parses the process arguments and environment variables. An empty environment variable counts as unset.
    ///
    /// # Errors
    ///
    /// Returns [`clap::Error`] when a value is not valid, or for `--help` and `--version`.
    pub fn from_env() -> Result<Self, clap::Error> {
        Self::try_from_args(std::env::args_os(), |name| std::env::var_os(name))
    }

    /// Parses `args`. `lookup` reads an environment variable. An empty value counts as unset.
    ///
    /// # Errors
    ///
    /// Returns [`clap::Error`] when a value is not valid, or for `--help` and `--version`.
    pub fn try_from_args<I, T, F>(args: I, lookup: F) -> Result<Self, clap::Error>
    where
        I: IntoIterator<Item = T>,
        T: Into<OsString> + Clone,
        F: Fn(&OsStr) -> Option<OsString>,
    {
        let matches = command(lookup).try_get_matches_from(args)?;
        Self::from_arg_matches(&matches)
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

    /// Restores a token from the SHA-256 hex digest that the database stores.
    ///
    /// Returns `None` when the text is not 64 hex digits.
    #[must_use]
    pub fn from_digest_hex(hex_digest: &str) -> Option<Self> {
        let mut digest = [0_u8; 32];
        hex::decode_to_slice(hex_digest, &mut digest).ok()?;
        Some(Self { digest })
    }

    /// Returns the SHA-256 hex digest that the database stores.
    #[must_use]
    pub fn digest_hex(&self) -> String {
        hex::encode(self.digest)
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

/// Returns the command line definition without the environment variables that `lookup` reports as empty.
fn command<F>(lookup: F) -> clap::Command
where
    F: Fn(&OsStr) -> Option<OsString>,
{
    Config::command().mut_args(|arg| {
        let empty = arg
            .get_env()
            .and_then(&lookup)
            .is_some_and(|value| value.is_empty());
        match empty {
            true => arg.env(None::<&str>),
            false => arg,
        }
    })
}

#[cfg(test)]
mod tests {
    use std::ffi::{OsStr, OsString};

    use clap::CommandFactory;
    use cuhlumn_api::settings::Scheme;

    use crate::config::{Config, Task, Token, command};

    fn no_env(_: &OsStr) -> Option<OsString> {
        None
    }

    #[test]
    fn token_needs_32_bytes() {
        assert!(Token::new("short").is_err());
        let token = Token::new(&"a".repeat(32)).unwrap();
        assert!(token.matches(&"a".repeat(32)));
        assert!(!token.matches(&"a".repeat(31)));
        assert_eq!(format!("{token:?}"), "Token(<redacted>)");
        let restored = Token::from_digest_hex(&token.digest_hex()).unwrap();
        assert!(restored.matches(&"a".repeat(32)));
        assert!(Token::from_digest_hex("zz").is_none());
    }

    #[test]
    fn flags_parse_into_typed_values() {
        let config = Config::try_from_args(
            [
                "cuhlumn-server",
                "--port",
                "9000",
                "--scheme",
                "http",
                "--console-host",
                "Console.Example.com",
                "--token",
                "0123456789abcdef0123456789abcdef",
            ],
            no_env,
        )
        .unwrap();
        assert_eq!(config.port, 9000);
        assert_eq!(config.scheme, Some(Scheme::Http));
        assert_eq!(config.console_host.unwrap().as_str(), "console.example.com");
        assert_eq!(config.command, None);
        let bad = Config::try_from_args(["cuhlumn-server", "--token", "short"], no_env);
        assert!(bad.is_err());
        let check = Config::try_from_args(["cuhlumn-server", "healthcheck"], no_env).unwrap();
        assert_eq!(check.command, Some(Task::Healthcheck));
        assert_eq!(check.scheme, None);
    }

    #[test]
    fn empty_environment_values_count_as_unset() {
        let names = ["CUHLUMN_TOKEN", "CUHLUMN_CONSOLE_HOST", "CUHLUMN_SCHEME"];
        let watched = |command: &clap::Command| {
            command
                .get_arguments()
                .filter(|arg| {
                    arg.get_env()
                        .is_some_and(|env| names.iter().any(|n| env == *n))
                })
                .count()
        };
        assert_eq!(watched(&Config::command()), names.len());
        let empty = |name: &OsStr| names.iter().any(|n| name == *n).then(OsString::new);
        assert_eq!(watched(&command(empty)), 0);
        assert_eq!(watched(&command(no_env)), names.len());
        let config = Config::try_from_args(["cuhlumn-server"], empty).unwrap();
        assert!(config.token.is_none());
        assert!(config.scheme.is_none());
    }
}
