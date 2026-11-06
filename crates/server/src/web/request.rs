//! Host names of incoming requests.

use axum::http::{HeaderMap, Uri, header};
use cuhlumn_api::site::host::Host;

/// Returns the normalized host that a request addresses.
///
/// The `Host` header comes first; HTTP/2 requests without it use the URI authority.
#[must_use]
pub fn request_host(headers: &HeaderMap, uri: &Uri) -> Option<Host> {
    let raw = headers
        .get(header::HOST)
        .and_then(|value| value.to_str().ok())
        .or_else(|| uri.authority().map(|authority| authority.as_str()))?;
    normalize_host(raw)
}

/// Removes a port, a trailing dot, and uppercase letters from a host header value.
///
/// Returns `None` when the remaining text is not a valid domain name.
#[must_use]
pub fn normalize_host(raw: &str) -> Option<Host> {
    let name = match raw.rsplit_once(':') {
        Some((name, port)) if port.bytes().all(|byte| byte.is_ascii_digit()) => name,
        _ => raw,
    };
    Host::new(name).ok()
}

/// Returns the normalized host of an `Origin` header value such as `https://console.example.com:8443`.
#[must_use]
pub fn origin_host(origin: &str) -> Option<Host> {
    let (_, authority) = origin.split_once("://")?;
    normalize_host(authority)
}

#[cfg(test)]
mod tests {
    use axum::http::{HeaderMap, HeaderValue, Uri, header};

    use crate::web::request::{normalize_host, origin_host, request_host};

    #[test]
    fn strips_port_dot_and_case() {
        let host = |raw: &str| normalize_host(raw).map(|host| host.to_string());
        assert_eq!(
            host("Blog.Example.com:8080").as_deref(),
            Some("blog.example.com")
        );
        assert_eq!(
            host("blog.example.com.").as_deref(),
            Some("blog.example.com")
        );
        assert_eq!(host("localhost").as_deref(), Some("localhost"));
        assert_eq!(host("[::1]:8080"), None);
        assert_eq!(host(""), None);
    }

    #[test]
    fn reads_the_host_header_or_the_authority() {
        let mut headers = HeaderMap::new();
        let uri: Uri = "https://other.example.org/path".parse().unwrap();
        assert_eq!(
            request_host(&headers, &uri).unwrap().as_str(),
            "other.example.org"
        );
        headers.insert(header::HOST, HeaderValue::from_static("blog.example.com"));
        assert_eq!(
            request_host(&headers, &uri).unwrap().as_str(),
            "blog.example.com"
        );
    }

    #[test]
    fn reads_origin_hosts() {
        assert_eq!(
            origin_host("http://localhost:5173").unwrap().as_str(),
            "localhost"
        );
        assert_eq!(
            origin_host("https://Console.Example.com").unwrap().as_str(),
            "console.example.com"
        );
        assert_eq!(origin_host("null"), None);
    }
}
