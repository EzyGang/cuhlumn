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
/// An admin address turns the console host off and locks it.
#[derive(Clone, Debug)]
pub struct Settings {
    token: Option<Token>,
    console_host: Option<Host>,
    scheme: Scheme,
    admin_listener: bool,
    locked: Locked,
}

impl Settings {
    /// Combines the overrides of `config` with the `stored` values.
    #[must_use]
    pub fn resolve(config: &Config, stored: &StoredSettings) -> Self {
        let admin_listener = config.admin_addr.is_some();
        let locked = Locked {
            token: config.token.is_some(),
            console_host: config.console_host.is_some() || admin_listener,
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
        let console_host = match (admin_listener, &config.console_host) {
            (true, _) => None,
            (false, Some(host)) => Some(host.clone()),
            (false, None) => stored.console_host.clone(),
        };
        Self {
            token,
            console_host,
            scheme: config.scheme.or(stored.scheme).unwrap_or_default(),
            admin_listener,
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
    ///
    /// With an admin listener, it is always `None`, because the console has no host there.
    #[must_use]
    pub fn console_host(&self) -> Option<&Host> {
        self.console_host.as_ref()
    }

    /// Returns `true` when the console and the admin API are only on the admin listener.
    #[must_use]
    pub fn admin_listener(&self) -> bool {
        self.admin_listener
    }

    /// Returns the scheme of absolute addresses.
    ///
    /// Without an admin listener, it also sets the cookie `Secure` flag.
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
            admin_listener: self.admin_listener,
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
            admin_addr: None,
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

    #[test]
    fn admin_listener_turns_the_console_host_off() {
        let mut config = config();
        config.admin_addr = Some("100.92.3.4:8081".parse().unwrap());
        config.console_host = Some(Host::new("admin.harbor.example.com").unwrap());
        let stored = StoredSettings {
            console_host: Some(Host::new("console.harbor.example.com").unwrap()),
            ..StoredSettings::default()
        };
        let settings = Settings::resolve(&config, &stored);
        assert!(settings.admin_listener());
        assert_eq!(settings.console_host(), None);
        assert!(settings.locked().console_host);
        let api = settings.to_api();
        assert!(api.admin_listener && api.locked.console_host);
        assert_eq!(api.console_host, None);
    }
}
