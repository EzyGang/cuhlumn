//! IndexNow pings that tell search engines about new and changed pages.
//!
//! See the [IndexNow protocol](https://www.indexnow.org/documentation).

use std::{sync::Arc, time::Duration};

use cuhlumn_api::{settings::Scheme, site::Site};
use serde::Serialize;
use tokio::task::JoinHandle;

/// The IndexNow client. Callers ping only for sites that turn IndexNow on.
#[derive(Clone, Debug)]
pub struct IndexNow {
    client: reqwest::Client,
    endpoint: Arc<str>,
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
    /// Builds the client for the public IndexNow endpoint.
    ///
    /// # Errors
    ///
    /// Returns [`reqwest::Error`] when the HTTP client cannot be built.
    pub fn new() -> Result<Self, reqwest::Error> {
        Self::with_endpoint(ENDPOINT.to_owned())
    }

    /// Builds the client for another endpoint, such as a local test server.
    ///
    /// # Errors
    ///
    /// Returns [`reqwest::Error`] when the HTTP client cannot be built.
    pub fn with_endpoint(endpoint: String) -> Result<Self, reqwest::Error> {
        Ok(Self {
            client: reqwest::Client::builder().timeout(TIMEOUT).build()?,
            endpoint: endpoint.into(),
        })
    }

    /// Sends the addresses of `site` in a background task and logs the result.
    ///
    /// Returns `None` without a task when the site turns IndexNow off.
    pub fn ping(&self, scheme: Scheme, site: &Site, urls: Vec<String>) -> Option<JoinHandle<()>> {
        if !site.indexnow {
            return None;
        }
        let host = site.host.as_str();
        let key = site.indexnow_key.as_str();
        let body = Ping {
            host,
            key,
            key_location: format!("{scheme}://{host}/{key}.txt"),
            url_list: urls,
        };
        let request = self.client.post(&*self.endpoint).json(&body);
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
    use cuhlumn_api::{
        settings::Scheme,
        site::{Site, brand::Brand, host::Host, indexnow_key::IndexNowKey, lang::Lang},
    };
    use tokio::{io::AsyncReadExt, net::TcpListener};

    use crate::indexnow::IndexNow;

    fn site(indexnow: bool) -> Site {
        Site {
            host: Host::new("harbor.example.com").unwrap(),
            title: "Harbor Notes".to_owned(),
            description: String::new(),
            aliases: Vec::new(),
            author: String::new(),
            author_url: None,
            project_name: String::new(),
            project_url: None,
            cta: String::new(),
            show_cta: true,
            lang: Lang::default(),
            home: None,
            indexnow_key: IndexNowKey::new("0123456789abcdef").unwrap(),
            brand: Brand::default(),
            indexnow,
            announcement: None,
            updated_at: 0,
        }
    }

    #[tokio::test]
    async fn pings_only_sites_that_turn_it_on() {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let endpoint = format!("http://{}/indexnow", listener.local_addr().unwrap());
        let indexnow = IndexNow::with_endpoint(endpoint).unwrap();
        let urls = vec!["https://harbor.example.com/sqlite-backups/".to_owned()];
        assert!(
            indexnow
                .ping(Scheme::Https, &site(false), urls.clone())
                .is_none()
        );
        let task = indexnow.ping(Scheme::Https, &site(true), urls).unwrap();
        let (mut socket, _) = listener.accept().await.unwrap();
        let mut request = vec![0_u8; 4096];
        let read = socket.read(&mut request).await.unwrap();
        let request = String::from_utf8_lossy(&request[..read]).into_owned();
        assert!(request.starts_with("POST /indexnow "), "{request}");
        drop(socket);
        task.await.unwrap();
    }
}
