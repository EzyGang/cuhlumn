//! The effective server settings: environment overrides first, then the database values.

use cuhlumn_api::{
    settings::{Locked, Scheme, ServerSettings},
    site::host::Host,
};

use crate::{
    config::{Config, Token},
    db::settings::StoredSettings,
};

/// The server settings that requests use.
///
/// An override from [`Config`] wins over the stored value and locks the field.
#[derive(Clone, Debug)]
pub struct Settings {
    token: Option<Token>,
    console_host: Option<Host>,
    scheme: Scheme,
    locked: Locked,
}

impl Settings {
    /// Combines the overrides of `config` with the `stored` values.
    #[must_use]
    pub fn resolve(config: &Config, stored: &StoredSettings) -> Self {
        let locked = Locked {
            token: config.token.is_some(),
            console_host: config.console_host.is_some(),
            scheme: config.scheme.is_some(),
        };
        let token = config.token.clone().or_else(|| {
            let digest = stored.token_digest.as_deref()?;
            let token = Token::from_digest_hex(digest);
            if token.is_none() {
                tracing::error!("the stored token digest is not valid, so the admin API stays off");
            }
            token
        });
        let console_host = match &config.console_host {
            Some(host) => Some(host.clone()),
            None => stored.console_host.clone(),
        };
        Self {
            token,
            console_host,
            scheme: config.scheme.or(stored.scheme).unwrap_or_default(),
            locked,
        }
    }

    /// Returns the admin token, or `None` before the first setup.
    #[must_use]
    pub fn token(&self) -> Option<&Token> {
        self.token.as_ref()
    }

    /// Returns `true` when the server has no admin token yet.
    #[must_use]
    pub fn needs_setup(&self) -> bool {
        self.token.is_none()
    }

    /// Returns the only host that serves the console, or `None` when every host that is not a site serves it.
    #[must_use]
    pub fn console_host(&self) -> Option<&Host> {
        self.console_host.as_ref()
    }

    /// Returns the scheme of absolute addresses and of the cookie `Secure` flag.
    #[must_use]
    pub fn scheme(&self) -> Scheme {
        self.scheme
    }

    /// Returns the fields that environment variables set.
    #[must_use]
    pub fn locked(&self) -> Locked {
        self.locked
    }

    /// Returns the address of the site root without a trailing slash, such as `https://blog.example.com`.
    #[must_use]
    pub fn origin(&self, host: &Host) -> String {
        format!("{}://{host}", self.scheme)
    }

    /// Returns the response body of `GET /api/settings`.
    #[must_use]
    pub fn to_api(&self) -> ServerSettings {
        ServerSettings {
            console_host: self.console_host.clone(),
            scheme: self.scheme,
            locked: self.locked,
        }
    }
}

#[cfg(test)]
mod tests {
    use cuhlumn_api::{settings::Scheme, site::host::Host};

    use crate::{
        config::{Config, Token},
        db::settings::StoredSettings,
        state::settings::Settings,
    };

    const TOKEN: &str = "harbor-token-0123456789-abcdefghij";

    fn config() -> Config {
        Config {
            port: 0,
            data: "./data".into(),
            token: None,
            console_host: None,
            scheme: None,
            command: None,
        }
    }

    #[test]
    fn database_values_apply_without_overrides() {
        let stored = StoredSettings {
            token_digest: Some(Token::new(TOKEN).unwrap().digest_hex()),
            console_host: Some(Host::new("console.harbor.example.com").unwrap()),
            scheme: Some(Scheme::Http),
            updated_at: 1,
        };
        let settings = Settings::resolve(&config(), &stored);
        assert!(settings.token().unwrap().matches(TOKEN));
        assert_eq!(settings.scheme(), Scheme::Http);
        assert_eq!(settings.locked(), Default::default());
        let empty = Settings::resolve(&config(), &StoredSettings::default());
        assert!(empty.needs_setup());
        assert_eq!(empty.scheme(), Scheme::Https);
        assert_eq!(empty.console_host(), None);
    }

    #[test]
    fn overrides_win_and_lock() {
        let mut config = config();
        config.token = Some(Token::new(&"e".repeat(32)).unwrap());
        config.scheme = Some(Scheme::Https);
        config.console_host = Some(Host::new("admin.harbor.example.com").unwrap());
        let stored = StoredSettings {
            token_digest: Some(Token::new(TOKEN).unwrap().digest_hex()),
            console_host: Some(Host::new("console.harbor.example.com").unwrap()),
            scheme: Some(Scheme::Http),
            updated_at: 1,
        };
        let settings = Settings::resolve(&config, &stored);
        assert!(!settings.token().unwrap().matches(TOKEN));
        assert_eq!(settings.scheme(), Scheme::Https);
        assert_eq!(
            settings.console_host().unwrap().as_str(),
            "admin.harbor.example.com"
        );
        let locked = settings.locked();
        assert!(locked.token && locked.console_host && locked.scheme);
        assert_eq!(
            settings.origin(&Host::new("harbor.example.com").unwrap()),
            "https://harbor.example.com"
        );
    }
}
