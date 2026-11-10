//! The page views that wait for the next database write.

use std::collections::{HashMap, HashSet};

use cuhlumn_api::{
    analytics::{Browser, Device},
    site::host::Host,
};

use crate::analytics::visitor::VisitorHash;

/// The waiting counts of every site and day.
#[derive(Debug, Default)]
pub struct Pending {
    days: HashMap<(Host, i64), DayPending>,
}

/// The waiting counts of one site on one day.
#[derive(Debug, Default)]
pub struct DayPending {
    pub views: u64,
    pub visitors: HashSet<VisitorHash>,
    pub pages: HashMap<String, u64>,
    /// The views of each referring host. `None` counts direct visits.
    pub referrers: HashMap<Option<String>, u64>,
    pub clients: HashMap<(Device, Browser), u64>,
}

/// One counted page view.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Counted {
    pub path: String,
    pub referrer: Option<String>,
    pub device: Device,
    pub browser: Browser,
    pub visitor: VisitorHash,
}

/// The most distinct referrers of one site and day in one batch. Further referrers count as direct visits.
const MAX_REFERRERS: usize = 1_000;

impl Pending {
    /// Adds one page view of `host` on `day`.
    pub fn add(&mut self, host: &Host, day: i64, view: Counted) {
        let counts = self.days.entry((host.clone(), day)).or_default();
        counts.views += 1;
        counts.visitors.insert(view.visitor);
        *counts.pages.entry(view.path).or_default() += 1;
        let known = view
            .referrer
            .as_ref()
            .is_none_or(|_| counts.referrers.contains_key(&view.referrer));
        let hosts = counts.referrers.len() - usize::from(counts.referrers.contains_key(&None));
        let referrer = match known || hosts < MAX_REFERRERS {
            true => view.referrer,
            false => None,
        };
        *counts.referrers.entry(referrer).or_default() += 1;
        *counts
            .clients
            .entry((view.device, view.browser))
            .or_default() += 1;
    }

    /// Returns the number of waiting page views.
    #[must_use]
    pub fn views(&self) -> u64 {
        self.days.values().map(|counts| counts.views).sum()
    }

    /// Returns the waiting counts of each site and day.
    pub fn days(&self) -> impl Iterator<Item = (&Host, i64, &DayPending)> {
        self.days
            .iter()
            .map(|((host, day), counts)| (host, *day, counts))
    }
}

#[cfg(test)]
mod tests {
    use cuhlumn_api::{
        analytics::{Browser, Device},
        site::host::Host,
    };

    use crate::analytics::pending::{Counted, MAX_REFERRERS, Pending};

    fn view(path: &str, referrer: Option<String>, visitor: u8) -> Counted {
        Counted {
            path: path.to_owned(),
            referrer,
            device: Device::Mobile,
            browser: Browser::Firefox,
            visitor: [visitor; 16],
        }
    }

    #[test]
    fn adds_views_per_site_and_day() {
        let host = Host::new("harbor.example.com").unwrap();
        let mut pending = Pending::default();
        pending.add(&host, 10, view("/", None, 1));
        pending.add(&host, 10, view("/", Some("news.example.org".to_owned()), 1));
        pending.add(&host, 11, view("/sqlite-backups/", None, 2));
        assert_eq!(pending.views(), 3);
        let (_, _, today) = pending.days().find(|(_, day, _)| *day == 10).unwrap();
        assert_eq!(today.views, 2);
        assert_eq!(today.visitors.len(), 1);
        assert_eq!(today.pages["/"], 2);
        assert_eq!(today.referrers[&None], 1);
        assert_eq!(today.clients[&(Device::Mobile, Browser::Firefox)], 2);
    }

    #[test]
    fn caps_distinct_referrers() {
        let host = Host::new("harbor.example.com").unwrap();
        let mut pending = Pending::default();
        for index in 0..=MAX_REFERRERS {
            pending.add(&host, 10, view("/", Some(format!("r{index}.example")), 1));
        }
        pending.add(&host, 10, view("/", Some("r0.example".to_owned()), 1));
        let (_, _, today) = pending.days().next().unwrap();
        assert_eq!(today.referrers.len(), MAX_REFERRERS + 1);
        assert_eq!(today.referrers[&None], 1);
        assert_eq!(today.referrers[&Some("r0.example".to_owned())], 2);
    }
}
