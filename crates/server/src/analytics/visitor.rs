//! Daily visitor hashes: a random salt for each UTC day, and the client address of a request.

use std::net::{IpAddr, SocketAddr};

use axum::http::HeaderMap;
use sha2::{Digest, Sha256};

use crate::db::analytics::StoredSalt;

/// The random salt of one UTC day.
#[derive(Clone, PartialEq, Eq)]
pub struct Salt {
    /// The day, in days since 1970-01-01 (UTC).
    pub day: i64,
    pub bytes: [u8; 32],
}

/// A visitor hash: the first 16 bytes of a SHA-256 digest.
pub type VisitorHash = [u8; 16];

/// The seconds in one day.
const DAY_SECONDS: i64 = 86_400;

/// The client address headers of proxies, in order of trust.
const IP_HEADERS: &[&str] = &["cf-connecting-ip", "x-real-ip"];

/// Returns the UTC day of a unix time, in days since 1970-01-01.
#[must_use]
pub const fn day_of(time: i64) -> i64 {
    time.div_euclid(DAY_SECONDS)
}

/// Returns the client address of a request.
///
/// `CF-Connecting-IP` comes first, then `X-Real-IP`, then the last `X-Forwarded-For` entry, then the socket peer.
/// A header that is not a valid address is skipped.
#[must_use]
pub fn client_ip(headers: &HeaderMap, peer: Option<SocketAddr>) -> Option<IpAddr> {
    let header = |name: &str| headers.get(name).and_then(|value| value.to_str().ok());
    IP_HEADERS
        .iter()
        .filter_map(|name| header(name))
        .chain(header("x-forwarded-for").and_then(|list| list.rsplit(',').next()))
        .find_map(|value| value.trim().parse().ok())
        .or_else(|| peer.map(|peer| peer.ip()))
}

/// Returns the visitor hash of a reader on `host` for the day of `salt`.
///
/// Without a client address, the hash depends on the User-Agent only.
#[must_use]
pub fn visitor_hash(salt: &Salt, host: &str, ip: Option<IpAddr>, user_agent: &str) -> VisitorHash {
    let address = ip.map(|ip| ip.to_string()).unwrap_or_default();
    let digest = Sha256::new()
        .chain_update(salt.bytes)
        .chain_update(host.as_bytes())
        .chain_update([0])
        .chain_update(address.as_bytes())
        .chain_update([0])
        .chain_update(user_agent.as_bytes())
        .finalize();
    let (head, _) = digest.as_chunks::<16>();
    head.first().copied().unwrap_or_default()
}

impl Salt {
    /// Returns a new random salt for `day`.
    ///
    /// # Errors
    ///
    /// Returns [`getrandom::Error`] when the operating system random source fails.
    pub fn new(day: i64) -> Result<Self, getrandom::Error> {
        let mut bytes = [0; 32];
        getrandom::fill(&mut bytes)?;
        Ok(Self { day, bytes })
    }

    /// Returns a stored salt when it belongs to `day` and has 32 bytes, else a new random salt.
    ///
    /// # Errors
    ///
    /// Returns [`getrandom::Error`] when the operating system random source fails.
    pub fn restore(stored: Option<StoredSalt>, day: i64) -> Result<Self, getrandom::Error> {
        let Some(stored) = stored.filter(|stored| stored.day == day) else {
            return Self::new(day);
        };
        match <[u8; 32]>::try_from(stored.bytes) {
            Ok(bytes) => Ok(Self { day, bytes }),
            Err(_) => Self::new(day),
        }
    }

    /// Replaces the salt with a new random salt when `day` is another day.
    ///
    /// # Errors
    ///
    /// Returns [`getrandom::Error`] when the operating system random source fails.
    /// The salt then stays unchanged.
    pub fn rotate(&mut self, day: i64) -> Result<(), getrandom::Error> {
        if self.day != day {
            *self = Self::new(day)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::net::{IpAddr, Ipv4Addr, SocketAddr};

    use axum::http::{HeaderMap, HeaderValue};

    use crate::{
        analytics::visitor::{Salt, client_ip, day_of, visitor_hash},
        db::analytics::StoredSalt,
    };

    fn headers(pairs: &[(&'static str, &'static str)]) -> HeaderMap {
        let mut map = HeaderMap::new();
        for (name, value) in pairs {
            map.insert(*name, HeaderValue::from_static(value));
        }
        map
    }

    #[test]
    fn client_ip_follows_the_header_order() {
        let peer = Some(SocketAddr::from((Ipv4Addr::new(10, 0, 0, 9), 4000)));
        let ip = |pairs: &[(&'static str, &'static str)]| client_ip(&headers(pairs), peer);
        let all = [
            ("x-forwarded-for", "198.51.100.1, 198.51.100.2"),
            ("x-real-ip", "203.0.113.5"),
            ("cf-connecting-ip", "2001:db8::7"),
        ];
        assert_eq!(ip(&all), "2001:db8::7".parse().ok());
        assert_eq!(ip(&all[..2]), "203.0.113.5".parse().ok());
        assert_eq!(ip(&all[..1]), "198.51.100.2".parse().ok());
        assert_eq!(ip(&[]), Some(IpAddr::V4(Ipv4Addr::new(10, 0, 0, 9))));
        assert_eq!(
            ip(&[
                ("x-real-ip", "not an address"),
                ("x-forwarded-for", " 198.51.100.3 ")
            ]),
            "198.51.100.3".parse().ok()
        );
        assert_eq!(client_ip(&HeaderMap::new(), None), None);
    }

    #[test]
    fn salt_rotates_when_the_day_changes() {
        let mut salt = Salt::new(20_000).unwrap();
        let first = salt.clone();
        salt.rotate(20_000).unwrap();
        assert!(salt == first);
        salt.rotate(20_001).unwrap();
        assert_eq!(salt.day, 20_001);
        assert_ne!(salt.bytes, first.bytes);
    }

    #[test]
    fn restore_keeps_only_a_salt_of_today() {
        let stored = |day: i64, bytes: Vec<u8>| Some(StoredSalt { day, bytes });
        let today = Salt::restore(stored(20_000, vec![7; 32]), 20_000).unwrap();
        assert_eq!(today.bytes, [7; 32]);
        let old = Salt::restore(stored(19_999, vec![7; 32]), 20_000).unwrap();
        assert_eq!(old.day, 20_000);
        assert_ne!(old.bytes, [7; 32]);
        let short = Salt::restore(stored(20_000, vec![7; 3]), 20_000).unwrap();
        assert_eq!(short.day, 20_000);
    }

    #[test]
    fn hashes_differ_by_salt_ip_and_agent() {
        let salt = Salt {
            day: 1,
            bytes: [1; 32],
        };
        let other = Salt {
            day: 2,
            bytes: [2; 32],
        };
        let ip = "203.0.113.5".parse().ok();
        let base = visitor_hash(&salt, "harbor.example.com", ip, "Firefox/132.0");
        assert_eq!(
            base,
            visitor_hash(&salt, "harbor.example.com", ip, "Firefox/132.0")
        );
        assert_ne!(
            base,
            visitor_hash(&other, "harbor.example.com", ip, "Firefox/132.0")
        );
        assert_ne!(
            base,
            visitor_hash(&salt, "harbor.example.com", None, "Firefox/132.0")
        );
        assert_ne!(
            base,
            visitor_hash(&salt, "harbor.example.com", ip, "Chrome/130.0")
        );
        assert_ne!(
            base,
            visitor_hash(&salt, "notes.example.com", ip, "Firefox/132.0")
        );
    }

    #[test]
    fn days_are_utc() {
        assert_eq!(day_of(0), 0);
        assert_eq!(day_of(86_399), 0);
        assert_eq!(day_of(86_400), 1);
        assert_eq!(day_of(-1), -1);
    }
}
