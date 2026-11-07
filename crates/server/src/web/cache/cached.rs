//! One rendered response with its gzip copy, ETag, and expiry.

use std::io::Write;

use axum::{
    body::Body,
    http::{HeaderMap, HeaderValue, StatusCode, header},
    response::{IntoResponse, Response},
};
use bytes::Bytes;
use flate2::{Compression, write::GzEncoder};
use sha2::{Digest, Sha256};

/// One cached response.
#[derive(Debug)]
pub struct Cached {
    content_type: &'static str,
    body: Bytes,
    gzip: Option<Bytes>,
    etag: String,
    max_age: u32,
    valid_until: Option<i64>,
}

const GZIP_MIN_LEN: usize = 1024;

impl Cached {
    /// Prepares a response body with its ETag and, for text larger than 1024 bytes, a gzip copy.
    ///
    /// PNG bodies are never compressed again. The response stays valid until the next write to its site.
    #[must_use]
    pub fn new<B>(content_type: &'static str, body: B, max_age: u32) -> Self
    where
        B: Into<Bytes>,
    {
        let body = body.into();
        let digest = hex::encode(Sha256::digest(&body));
        let etag = format!("\"{}\"", digest.get(..16).unwrap_or(&digest));
        let gzip = (!content_type.starts_with("image/png") && body.len() > GZIP_MIN_LEN)
            .then(|| gzip(&body))
            .flatten();
        Self {
            content_type,
            body,
            gzip,
            etag,
            max_age,
            valid_until: None,
        }
    }

    /// Sets the time, in unix seconds, from which the cache must not return the response.
    ///
    /// `None` keeps the response until the next write to its site.
    #[must_use]
    pub fn valid_until(mut self, valid_until: Option<i64>) -> Self {
        self.valid_until = valid_until;
        self
    }

    /// Returns `true` when the response is still valid at `now`, in unix seconds.
    #[must_use]
    pub fn is_valid_at(&self, now: i64) -> bool {
        self.valid_until.is_none_or(|until| now < until)
    }

    /// Returns the ETag value with its quotes.
    #[must_use]
    pub fn etag(&self) -> &str {
        &self.etag
    }

    /// Builds the response for a request with these headers.
    ///
    /// A matching `If-None-Match` gets 304. A client that accepts gzip gets the gzip copy.
    #[must_use]
    pub fn respond(&self, request: &HeaderMap) -> Response {
        let mut headers = HeaderMap::new();
        headers.insert(
            header::CONTENT_TYPE,
            HeaderValue::from_static(self.content_type),
        );
        headers.insert(header::VARY, HeaderValue::from_static("accept-encoding"));
        if let Ok(etag) = HeaderValue::from_str(&self.etag) {
            headers.insert(header::ETAG, etag);
        }
        if let Ok(cache) = HeaderValue::from_str(&format!("public, max-age={}", self.max_age)) {
            headers.insert(header::CACHE_CONTROL, cache);
        }
        if self.is_fresh(request) {
            return (StatusCode::NOT_MODIFIED, headers).into_response();
        }
        let body = match &self.gzip {
            Some(gzip) if accepts_gzip(request) => {
                headers.insert(header::CONTENT_ENCODING, HeaderValue::from_static("gzip"));
                gzip.clone()
            }
            _ => self.body.clone(),
        };
        (headers, Body::from(body)).into_response()
    }

    fn is_fresh(&self, request: &HeaderMap) -> bool {
        request
            .get_all(header::IF_NONE_MATCH)
            .iter()
            .filter_map(|value| value.to_str().ok())
            .flat_map(|value| value.split(','))
            .map(|tag| tag.trim())
            .any(|tag| tag == "*" || tag.strip_prefix("W/").unwrap_or(tag) == self.etag)
    }
}

/// Returns `true` when `Accept-Encoding` permits gzip with a quality above zero.
fn accepts_gzip(request: &HeaderMap) -> bool {
    let mut gzip = None;
    let mut any = None;
    let codings = request
        .get_all(header::ACCEPT_ENCODING)
        .iter()
        .filter_map(|value| value.to_str().ok())
        .flat_map(|value| value.split(','));
    for coding in codings {
        let mut parts = coding.split(';');
        let name = parts.next().unwrap_or_default().trim();
        let quality = parts
            .filter_map(|param| param.trim().strip_prefix("q="))
            .find_map(|q| q.trim().parse::<f32>().ok())
            .unwrap_or(1.0);
        if name.eq_ignore_ascii_case("gzip") {
            gzip = Some(quality);
        } else if name == "*" {
            any = Some(quality);
        }
    }
    gzip.or(any).is_some_and(|quality| quality > 0.0)
}

fn gzip(body: &[u8]) -> Option<Bytes> {
    let mut encoder = GzEncoder::new(Vec::with_capacity(body.len() / 2), Compression::best());
    let result = encoder.write_all(body).and_then(|()| encoder.finish());
    match result {
        Ok(compressed) => Some(Bytes::from(compressed)),
        Err(err) => {
            tracing::warn!(error = %err, "gzip compression failed, the response stays uncompressed");
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use axum::http::{HeaderMap, HeaderValue, StatusCode, header};

    use crate::web::cache::cached::Cached;

    fn headers(name: header::HeaderName, value: &'static str) -> HeaderMap {
        let mut headers = HeaderMap::new();
        headers.insert(name, HeaderValue::from_static(value));
        headers
    }

    fn page() -> Cached {
        Cached::new("text/html; charset=utf-8", "x".repeat(2000), 300)
    }

    #[test]
    fn matching_etag_gets_304() {
        let page = page();
        assert_eq!(page.etag().len(), 18);
        let mut request = HeaderMap::new();
        request.insert(
            header::IF_NONE_MATCH,
            HeaderValue::from_str(page.etag()).unwrap(),
        );
        let response = page.respond(&request);
        assert_eq!(response.status(), StatusCode::NOT_MODIFIED);
        assert_eq!(
            response.headers()[header::CACHE_CONTROL],
            "public, max-age=300"
        );
        let other = page.respond(&headers(header::IF_NONE_MATCH, "\"0000000000000000\""));
        assert_eq!(other.status(), StatusCode::OK);
    }

    #[test]
    fn gzip_follows_accept_encoding() {
        let page = page();
        let encoding = |value: &'static str| {
            let response = page.respond(&headers(header::ACCEPT_ENCODING, value));
            response.headers().get(header::CONTENT_ENCODING).cloned()
        };
        assert_eq!(encoding("gzip, br").unwrap(), "gzip");
        assert_eq!(encoding("br;q=1, GZIP;q=0.5").unwrap(), "gzip");
        assert_eq!(encoding("*").unwrap(), "gzip");
        assert!(encoding("gzip;q=0").is_none());
        assert!(encoding("gzip;q=0, *").is_none());
        assert!(encoding("br").is_none());
        assert!(
            page.respond(&HeaderMap::new())
                .headers()
                .get(header::CONTENT_ENCODING)
                .is_none()
        );
        assert_eq!(
            page.respond(&HeaderMap::new()).headers()[header::VARY],
            "accept-encoding"
        );
    }

    #[test]
    fn small_and_png_bodies_stay_uncompressed() {
        let small = Cached::new("text/plain", "short", 60);
        let png = Cached::new("image/png", vec![0_u8; 4000], 60);
        for cached in [small, png] {
            let response = cached.respond(&headers(header::ACCEPT_ENCODING, "gzip"));
            assert!(response.headers().get(header::CONTENT_ENCODING).is_none());
        }
    }

    #[test]
    fn validity_ends_at_valid_until() {
        assert!(page().is_valid_at(i64::MAX));
        let page = page().valid_until(Some(100));
        assert!(page.is_valid_at(99));
        assert!(!page.is_valid_at(100));
    }
}
