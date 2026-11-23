//! Page paths of external sites and link targets of clicks: values that the server cannot check against its own data.

use cuhlumn_api::site::host::Host;

use crate::web::route::decode_path;

/// The longest stored page path or link target, in bytes.
pub const MAX_LEN: usize = 256;

/// The link target of a click: a host and a path, without a scheme, a port, a query, or a fragment.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Target {
    host: Host,
    path: String,
}

/// Returns the path that analytics stores for a page of an external site.
///
/// The path loses its query and its fragment, and its percent-encoding is decoded.
/// Returns `None` when the path does not start with `/`, has a control character, or is longer than [`MAX_LEN`].
#[must_use]
pub fn page_path(raw: &str) -> Option<String> {
    clean_path(raw).filter(|path| path.len() <= MAX_LEN)
}

impl Target {
    /// Parses a host name with an optional path, such as `github.com/EzyGang/cuhlumn`.
    ///
    /// The host becomes lowercase, and the path changes as [`page_path`] describes. A target without a path gets `/`.
    /// Returns `None` when the host or the path is not valid, or the target is longer than [`MAX_LEN`].
    #[must_use]
    pub fn parse(raw: &str) -> Option<Self> {
        let raw = raw.trim();
        let (host, rest) = raw.split_at(raw.find(['/', '?', '#']).unwrap_or(raw.len()));
        let host = Host::new(host).ok()?;
        let path = match rest.starts_with('/') {
            true => clean_path(rest)?,
            false => "/".to_owned(),
        };
        (host.as_str().len() + path.len() <= MAX_LEN).then_some(Self { host, path })
    }

    /// Returns the host of the link.
    #[must_use]
    pub const fn host(&self) -> &Host {
        &self.host
    }

    /// Returns the text that analytics stores, such as `github.com/EzyGang/cuhlumn`.
    #[must_use]
    pub fn into_text(self) -> String {
        format!("{}{}", self.host, self.path)
    }
}

/// Removes the query and the fragment, and decodes the percent-encoding.
///
/// Returns `None` when the result does not start with `/` or has a control character.
fn clean_path(raw: &str) -> Option<String> {
    let path = raw.split(['?', '#']).next().unwrap_or_default();
    let path = decode_path(path);
    (path.starts_with('/') && !path.chars().any(char::is_control)).then(|| path.into_owned())
}

#[cfg(test)]
mod tests {
    use crate::analytics::unverified::{Target, page_path};

    #[test]
    fn page_paths_lose_the_query_and_the_fragment() {
        let cases = [
            ("/", Some("/")),
            ("/guide/install/", Some("/guide/install/")),
            ("/guide/?utm_source=news#setup", Some("/guide/")),
            ("/guide/#setup?x=1", Some("/guide/")),
            ("/caf%C3%A9/", Some("/café/")),
            ("/a%3Fb/", Some("/a?b/")),
            ("", None),
            ("guide/", None),
            ("https://docs.example.org/guide/", None),
            ("/a%0Ab/", None),
            ("/a\u{7f}b/", None),
        ];
        for (raw, path) in cases {
            assert_eq!(page_path(raw).as_deref(), path, "{raw}");
        }
    }

    #[test]
    fn targets_keep_the_host_and_the_path() {
        let text = |raw: &str| Target::parse(raw).map(Target::into_text);
        let cases = [
            (
                "github.com/EzyGang/cuhlumn",
                Some("github.com/EzyGang/cuhlumn"),
            ),
            ("GitHub.com", Some("github.com/")),
            (
                "pypi.org/project/py-cachify/?tab=files#history",
                Some("pypi.org/project/py-cachify/"),
            ),
            ("example.org?x=1", Some("example.org/")),
            (
                "files.example.org/My%20App.zip",
                Some("files.example.org/My App.zip"),
            ),
            ("", None),
            ("/only/a/path", None),
            ("https://github.com/EzyGang", None),
            ("github.com:8443/EzyGang", None),
            ("bad host.example/x", None),
            ("example.org/a%00b", None),
        ];
        for (raw, target) in cases {
            assert_eq!(text(raw).as_deref(), target, "{raw}");
        }
        let host = Target::parse("Docs.Example.org/guide/").unwrap();
        assert_eq!(host.host().as_str(), "docs.example.org");
    }
}
