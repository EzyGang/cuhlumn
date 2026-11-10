//! The referring host of a page view.

use cuhlumn_api::site::host::Host;

/// The longest referring host that analytics stores, in characters.
const MAX_HOST: usize = 253;

/// Returns the host of an `http` or `https` referrer address, or `None` for a direct visit.
///
/// The host is lowercase, without user information, a port, or a leading `www.`.
/// An empty referrer, another scheme, an invalid host, and the request host itself count as direct visits.
#[must_use]
pub fn referrer_host(referrer: &str, request_host: &Host) -> Option<String> {
    let text = referrer.trim();
    let rest = strip_scheme(text, "https://").or_else(|| strip_scheme(text, "http://"))?;
    let authority = rest.split(['/', '?', '#']).next().unwrap_or_default();
    let host_port = authority
        .rsplit_once('@')
        .map_or(authority, |(_, host)| host);
    let host = without_port(host_port).to_ascii_lowercase();
    let host = host.strip_prefix("www.").unwrap_or(&host);
    if host.is_empty() || host.len() > MAX_HOST || !host.bytes().all(is_host_byte) {
        return None;
    }
    let own = request_host.as_str();
    if host == own.strip_prefix("www.").unwrap_or(own) {
        return None;
    }
    Some(host.to_owned())
}

/// Returns the text after `scheme`, compared without case.
fn strip_scheme<'a>(text: &'a str, scheme: &str) -> Option<&'a str> {
    let head = text.get(..scheme.len())?;
    head.eq_ignore_ascii_case(scheme)
        .then(|| &text[scheme.len()..])
}

/// Removes the port from an authority host. An IPv6 address keeps its brackets.
fn without_port(host_port: &str) -> &str {
    if host_port.starts_with('[') {
        return host_port
            .find(']')
            .map_or(host_port, |end| &host_port[..=end]);
    }
    host_port
        .split_once(':')
        .map_or(host_port, |(host, _)| host)
}

/// Returns `true` for a byte that a host name or an IP address can contain.
const fn is_host_byte(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'.' | b'_' | b'[' | b']' | b':')
}

#[cfg(test)]
mod tests {
    use cuhlumn_api::site::host::Host;

    use crate::analytics::referrer::referrer_host;

    #[test]
    fn keeps_the_host_only() {
        let own = Host::new("harbor.example.com").unwrap();
        let cases = [
            (
                "https://news.example.org/item?id=1",
                Some("news.example.org"),
            ),
            ("http://News.Example.org:8080/", Some("news.example.org")),
            (
                "https://www.search.example/results#top",
                Some("search.example"),
            ),
            (
                "https://mara:secret@forum.example.net/t/1",
                Some("forum.example.net"),
            ),
            ("HTTPS://feeds.example.com", Some("feeds.example.com")),
            ("https://[2001:db8::1]:8443/path", Some("[2001:db8::1]")),
            ("https://127.0.0.1/", Some("127.0.0.1")),
        ];
        for (referrer, host) in cases {
            assert_eq!(referrer_host(referrer, &own).as_deref(), host, "{referrer}");
        }
    }

    #[test]
    fn counts_direct_visits() {
        let own = Host::new("harbor.example.com").unwrap();
        let long = format!("https://{}.example/", "a".repeat(260));
        let cases = [
            "",
            "   ",
            "android-app://com.example.reader/",
            "ftp://files.example.org/",
            "https://harbor.example.com/sqlite-backups/",
            "https://www.harbor.example.com/",
            "https:///path",
            "https://bad host.example/",
            long.as_str(),
        ];
        for referrer in cases {
            assert_eq!(referrer_host(referrer, &own), None, "{referrer}");
        }
    }

    #[test]
    fn treats_the_www_form_of_the_site_as_the_site() {
        let own = Host::new("www.harbor.example.com").unwrap();
        assert_eq!(referrer_host("https://harbor.example.com/", &own), None);
        assert_eq!(
            referrer_host("https://docs.harbor.example.com/", &own).as_deref(),
            Some("docs.harbor.example.com")
        );
    }
}
