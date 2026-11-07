//! Rendered responses in memory, with gzip and ETag, until the next write to their site or their expiry.

pub mod cached;

use std::{
    collections::HashMap,
    sync::{Arc, PoisonError, RwLock},
};

use cuhlumn_api::site::host::Host;

use crate::web::cache::cached::Cached;

/// Rendered responses per canonical site host and request path.
///
/// Each write to a site calls [`PageCache::invalidate`], which gives the host a new generation.
/// A response that was built before the write has an old generation, so it is never stored.
#[derive(Debug, Default)]
pub struct PageCache {
    inner: RwLock<Pages>,
}

#[derive(Debug, Default)]
struct Pages {
    /// The last generation that a change gave out. Hosts without an entry have this generation.
    epoch: u64,
    hosts: HashMap<Host, HostPages>,
}

#[derive(Debug)]
struct HostPages {
    generation: u64,
    pages: HashMap<String, Arc<Cached>>,
}

impl PageCache {
    /// Returns the cached response for `path` on `host`, unless it expired before `now`, in unix seconds.
    #[must_use]
    pub fn get(&self, host: &Host, path: &str, now: i64) -> Option<Arc<Cached>> {
        let inner = self.inner.read().unwrap_or_else(PoisonError::into_inner);
        let cached = inner.hosts.get(host)?.pages.get(path)?;
        cached.is_valid_at(now).then(|| Arc::clone(cached))
    }

    /// Returns `true` when `host` has cached responses, so it is the canonical host of a site.
    #[must_use]
    pub fn has_pages(&self, host: &Host) -> bool {
        let inner = self.inner.read().unwrap_or_else(PoisonError::into_inner);
        inner
            .hosts
            .get(host)
            .is_some_and(|entry| !entry.pages.is_empty())
    }

    /// Returns the current generation of `host`. Read it before a build and pass it to [`PageCache::insert`].
    #[must_use]
    pub fn generation(&self, host: &Host) -> u64 {
        let inner = self.inner.read().unwrap_or_else(PoisonError::into_inner);
        inner
            .hosts
            .get(host)
            .map_or(inner.epoch, |entry| entry.generation)
    }

    /// Stores a response that was built at `generation`.
    ///
    /// The response is dropped when a change gave `host` a new generation during the build.
    pub fn insert(&self, host: &Host, path: String, generation: u64, cached: Arc<Cached>) {
        let mut inner = self.inner.write().unwrap_or_else(PoisonError::into_inner);
        let epoch = inner.epoch;
        let entry = inner
            .hosts
            .entry(host.clone())
            .or_insert_with(|| HostPages::new(epoch));
        if entry.generation != generation {
            return;
        }
        entry.pages.insert(path, cached);
    }

    /// Drops every response of `host` and gives it a new generation.
    pub fn invalidate(&self, host: &Host) {
        let mut inner = self.inner.write().unwrap_or_else(PoisonError::into_inner);
        inner.epoch += 1;
        let epoch = inner.epoch;
        inner.hosts.insert(host.clone(), HostPages::new(epoch));
    }

    /// Drops every response of every host, for example after a scheme or font change.
    pub fn clear(&self) {
        let mut inner = self.inner.write().unwrap_or_else(PoisonError::into_inner);
        inner.epoch += 1;
        inner.hosts.clear();
    }
}

impl HostPages {
    fn new(generation: u64) -> Self {
        Self {
            generation,
            pages: HashMap::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use cuhlumn_api::site::host::Host;

    use crate::web::cache::{PageCache, cached::Cached};

    fn page() -> Arc<Cached> {
        Arc::new(Cached::new(
            "text/html; charset=utf-8",
            "x".repeat(2000),
            300,
        ))
    }

    #[test]
    fn stale_generation_is_not_stored() {
        let cache = PageCache::default();
        let host = Host::new("blog.example.com").unwrap();
        let generation = cache.generation(&host);
        cache.invalidate(&host);
        cache.insert(&host, "/".to_owned(), generation, page());
        assert!(cache.get(&host, "/", 0).is_none());
        let generation = cache.generation(&host);
        cache.insert(&host, "/".to_owned(), generation, page());
        assert!(cache.get(&host, "/", 0).is_some());
        let other = Host::new("other.example.com").unwrap();
        cache.invalidate(&other);
        assert!(cache.get(&host, "/", 0).is_some());
        cache.invalidate(&host);
        assert!(cache.get(&host, "/", 0).is_none());
    }

    #[test]
    fn clear_drops_every_host_and_builds_in_flight() {
        let cache = PageCache::default();
        let (a, b) = (
            Host::new("a.example.com").unwrap(),
            Host::new("b.example.com").unwrap(),
        );
        cache.insert(&a, "/".to_owned(), cache.generation(&a), page());
        let in_flight = cache.generation(&b);
        cache.clear();
        assert!(cache.get(&a, "/", 0).is_none());
        cache.insert(&b, "/".to_owned(), in_flight, page());
        assert!(cache.get(&b, "/", 0).is_none());
        cache.insert(&b, "/".to_owned(), cache.generation(&b), page());
        assert!(cache.get(&b, "/", 0).is_some());
    }

    #[test]
    fn expired_pages_are_not_returned() {
        let cache = PageCache::default();
        let host = Host::new("harbor.example.com").unwrap();
        let cached = Cached::new("text/html; charset=utf-8", "<p>Hi</p>", 60).valid_until(Some(50));
        cache.insert(
            &host,
            "/".to_owned(),
            cache.generation(&host),
            Arc::new(cached),
        );
        assert!(cache.get(&host, "/", 49).is_some());
        assert!(cache.get(&host, "/", 50).is_none());
    }
}
