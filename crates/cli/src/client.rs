//! Blocking client for the cuhlumn HTTP API.

use std::time::Duration;

use cuhlumn_api::{
    fonts::{FontItem, GoogleFontInput},
    post::{Post, PostInput, slug::Slug},
    site::{Site, SiteInput, host::Host, link::AbsoluteUrl},
    wire::{ErrorBody, ErrorCode, Health, MediaItem, MediaSaved, PostSaved, Preview, PreviewInput},
};
use reqwest::{
    Method, StatusCode,
    blocking::{RequestBuilder, Response},
    header::{AUTHORIZATION, CONTENT_TYPE},
};
use serde::{Serialize, de::DeserializeOwned};

/// A connection to one cuhlumn server.
#[derive(Debug)]
pub struct Client {
    http: reqwest::blocking::Client,
    base: String,
    token: Option<String>,
}

/// A failed API call.
#[derive(Debug, thiserror::Error)]
pub enum ClientError {
    /// The server rejected the request with an error body.
    #[error("{}: {}", .0.error.code, .0.error.message)]
    Api(ErrorBody),
    /// The server answered with an unexpected status and no error body.
    #[error("the server answered {0} without an error body; check the server address")]
    Status(StatusCode),
    /// The request did not reach the server, or the answer did not arrive.
    #[error("cannot reach the server")]
    Transport(#[from] reqwest::Error),
    /// The answer is not the JSON that the CLI expects.
    #[error("the server answer is not the expected JSON; check the server address and version")]
    Decode(#[source] serde_json::Error),
    /// The request body cannot become JSON.
    #[error("cannot write the request as JSON")]
    Encode(#[source] serde_json::Error),
}

const TIMEOUT: Duration = Duration::from_secs(120);

impl Client {
    /// Builds a client for the server at `base`, such as `https://blog.example.com`.
    pub fn new(base: &AbsoluteUrl, token: Option<String>) -> Result<Self, ClientError> {
        let http = reqwest::blocking::Client::builder()
            .user_agent(concat!("cuhlumn-cli/", env!("CARGO_PKG_VERSION")))
            .timeout(TIMEOUT)
            .build()?;
        Ok(Self {
            http,
            base: base.as_str().trim_end_matches('/').to_owned(),
            token,
        })
    }

    pub fn health(&self) -> Result<Health, ClientError> {
        receive(self.request(Method::GET, "/health"))
    }

    pub fn sites(&self) -> Result<Vec<Site>, ClientError> {
        receive(self.request(Method::GET, "/sites"))
    }

    pub fn site(&self, host: &Host) -> Result<Site, ClientError> {
        receive(self.request(Method::GET, &format!("/sites/{host}")))
    }

    pub fn put_site(&self, host: &Host, input: &SiteInput) -> Result<Site, ClientError> {
        receive(with_json(
            self.request(Method::PUT, &format!("/sites/{host}")),
            input,
        )?)
    }

    pub fn delete_site(&self, host: &Host) -> Result<(), ClientError> {
        receive_empty(self.request(Method::DELETE, &format!("/sites/{host}")))
    }

    pub fn posts(&self, host: &Host) -> Result<Vec<Post>, ClientError> {
        receive(self.request(Method::GET, &format!("/sites/{host}/posts")))
    }

    pub fn post(&self, host: &Host, slug: &Slug) -> Result<Post, ClientError> {
        receive(self.request(Method::GET, &format!("/sites/{host}/posts/{slug}")))
    }

    /// Returns the post, or `None` when the site has no post with this slug.
    pub fn find_post(&self, host: &Host, slug: &Slug) -> Result<Option<Post>, ClientError> {
        match self.post(host, slug) {
            Ok(post) => Ok(Some(post)),
            Err(error) if error.is_not_found() => Ok(None),
            Err(error) => Err(error),
        }
    }

    pub fn put_post(
        &self,
        host: &Host,
        slug: &Slug,
        input: &PostInput,
    ) -> Result<PostSaved, ClientError> {
        let path = format!("/sites/{host}/posts/{slug}");
        receive(with_json(self.request(Method::PUT, &path), input)?)
    }

    pub fn delete_post(&self, host: &Host, slug: &Slug) -> Result<(), ClientError> {
        receive_empty(self.request(Method::DELETE, &format!("/sites/{host}/posts/{slug}")))
    }

    pub fn upload(
        &self,
        host: &Host,
        content_type: &str,
        bytes: Vec<u8>,
    ) -> Result<MediaSaved, ClientError> {
        let request = self
            .request(Method::POST, &format!("/sites/{host}/media"))
            .header(CONTENT_TYPE, content_type)
            .body(bytes);
        receive(request)
    }

    pub fn media(&self, host: &Host) -> Result<Vec<MediaItem>, ClientError> {
        receive(self.request(Method::GET, &format!("/sites/{host}/media")))
    }

    pub fn delete_media(&self, host: &Host, name: &str) -> Result<(), ClientError> {
        receive_empty(self.request(Method::DELETE, &format!("/sites/{host}/media/{name}")))
    }

    pub fn preview(&self, host: &Host, input: &PreviewInput) -> Result<Preview, ClientError> {
        let path = format!("/sites/{host}/preview");
        receive(with_json(self.request(Method::POST, &path), input)?)
    }

    pub fn fonts(&self) -> Result<Vec<FontItem>, ClientError> {
        receive(self.request(Method::GET, "/fonts"))
    }

    pub fn import_google_font(&self, url: AbsoluteUrl) -> Result<Vec<FontItem>, ClientError> {
        let input = GoogleFontInput { url };
        receive(with_json(
            self.request(Method::POST, "/fonts/google"),
            &input,
        )?)
    }

    pub fn upload_font(&self, bytes: Vec<u8>) -> Result<FontItem, ClientError> {
        let request = self
            .request(Method::POST, "/fonts")
            .header(CONTENT_TYPE, "application/octet-stream")
            .body(bytes);
        receive(request)
    }

    pub fn delete_font(&self, name: &str) -> Result<(), ClientError> {
        receive_empty(self.request(Method::DELETE, &format!("/fonts/{name}")))
    }

    fn request(&self, method: Method, path: &str) -> RequestBuilder {
        let request = self
            .http
            .request(method, format!("{}/api{path}", self.base));
        match &self.token {
            Some(token) => request.header(AUTHORIZATION, format!("Bearer {token}")),
            None => request,
        }
    }
}

impl ClientError {
    /// Returns `true` when the server answered that the resource does not exist.
    #[must_use]
    pub fn is_not_found(&self) -> bool {
        matches!(self, Self::Api(body) if body.error.code == ErrorCode::NotFound)
    }
}

fn receive<T>(request: RequestBuilder) -> Result<T, ClientError>
where
    T: DeserializeOwned,
{
    let bytes = checked(request.send()?)?.bytes()?;
    serde_json::from_slice(&bytes).map_err(ClientError::Decode)
}

fn receive_empty(request: RequestBuilder) -> Result<(), ClientError> {
    checked(request.send()?).map(drop)
}

fn with_json<T>(request: RequestBuilder, body: &T) -> Result<RequestBuilder, ClientError>
where
    T: Serialize,
{
    let bytes = serde_json::to_vec(body).map_err(ClientError::Encode)?;
    Ok(request.header(CONTENT_TYPE, "application/json").body(bytes))
}

/// Returns the response when its status is a success, or the error body of the server.
fn checked(response: Response) -> Result<Response, ClientError> {
    let status = response.status();
    if status.is_success() {
        return Ok(response);
    }
    let bytes = response.bytes()?;
    match serde_json::from_slice::<ErrorBody>(&bytes) {
        Ok(body) => Err(ClientError::Api(body)),
        Err(_) => Err(ClientError::Status(status)),
    }
}
