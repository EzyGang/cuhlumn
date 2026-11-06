//! IndexNow pings that tell search engines about new and changed pages.
//!
//! See the [IndexNow protocol](https://www.indexnow.org/documentation).

use std::time::Duration;

use cuhlumn_api::site::Site;
use serde::Serialize;
use tokio::task::JoinHandle;

use crate::config::{Scheme, Switch};

/// The IndexNow client. It sends nothing when the setting is off.
#[derive(Clone, Debug)]
pub struct IndexNow {
    client: Option<reqwest::Client>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct Ping<'a> {
    host: &'a str,
    key: &'a str,
    key_location: String,
    url_list: Vec<String>,
}

const ENDPOINT: &str = "https://api.indexnow.org/indexnow";
const TIMEOUT: Duration = Duration::from_secs(15);

impl IndexNow {
    /// Builds the client when `switch` is on.
    ///
    /// # Errors
    ///
    /// Returns [`reqwest::Error`] when the HTTP client cannot be built.
    pub fn new(switch: Switch) -> Result<Self, reqwest::Error> {
        let client = match switch {
            Switch::On => Some(reqwest::Client::builder().timeout(TIMEOUT).build()?),
            Switch::Off => None,
        };
        Ok(Self { client })
    }

    /// Returns `true` when pings go out.
    #[must_use]
    pub fn is_enabled(&self) -> bool {
        self.client.is_some()
    }

    /// Sends the addresses of `site` in a background task and logs the result.
    ///
    /// Returns `None` without a task when IndexNow is off.
    pub fn ping(&self, scheme: Scheme, site: &Site, urls: Vec<String>) -> Option<JoinHandle<()>> {
        let client = self.client.clone()?;
        let host = site.host.as_str();
        let key = site.indexnow_key.as_str();
        let body = Ping {
            host,
            key,
            key_location: format!("{scheme}://{host}/{key}.txt"),
            url_list: urls,
        };
        let request = client.post(ENDPOINT).json(&body);
        let host = host.to_owned();
        Some(tokio::spawn(async move {
            match request.send().await {
                Ok(response) => {
                    tracing::info!(%host, status = %response.status(), "sent IndexNow ping");
                }
                Err(err) => tracing::warn!(%host, error = %err, "IndexNow ping failed"),
            }
        }))
    }
}

#[cfg(test)]
mod tests {
    use cuhlumn_api::site::{Brand, Site, host::Host, indexnow_key::IndexNowKey, lang::Lang};

    use crate::{
        config::{Scheme, Switch},
        indexnow::IndexNow,
    };

    #[tokio::test]
    async fn off_sends_nothing() {
        let indexnow = IndexNow::new(Switch::Off).unwrap();
        assert!(!indexnow.is_enabled());
        let site = Site {
            host: Host::new("blog.example.com").unwrap(),
            title: "Notes".to_owned(),
            description: String::new(),
            aliases: Vec::new(),
            author: String::new(),
            author_url: None,
            project_name: String::new(),
            project_url: None,
            cta: String::new(),
            lang: Lang::default(),
            home: None,
            indexnow_key: IndexNowKey::new("0123456789abcdef").unwrap(),
            brand: Brand::default(),
            updated_at: 0,
        };
        let urls = vec!["https://blog.example.com/a/".to_owned()];
        assert!(indexnow.ping(Scheme::Https, &site, urls).is_none());
    }
}
