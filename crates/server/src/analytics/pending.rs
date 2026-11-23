//! The page views and link clicks that wait for the next database write.

use std::collections::{HashMap, HashSet};

use cuhlumn_api::{
    analytics::{Browser, ClickKind, Device},
    site::{host::Host, kind::SiteKind},
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
    /// The clicks on each link target.
    pub clicks: HashMap<(ClickKind, String), u64>,
}

/// One counted page view.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Counted {
    /// The kind of the site. The server cannot check the paths of an external site, so they have a limit.
    pub site: SiteKind,
    pub path: String,
    pub referrer: Option<String>,
    pub device: Device,
    pub browser: Browser,
    pub visitor: VisitorHash,
}

/// The most distinct page paths of one external site and day.
///
/// A view of a further path counts in the site totals, but the path gets no count of its own.
pub const MAX_EXTERNAL_PAGES: usize = 1_000;

/// The most distinct link targets of one site and day. A click on a further target does not count.
pub const MAX_CLICK_TARGETS: usize = 1_000;

/// The most distinct referrers of one site and day in one batch. Further referrers count as direct visits.
const MAX_REFERRERS: usize = 1_000;

impl Pending {
    /// Adds one page view of `host` on `day`.
    pub fn add(&mut self, host: &Host, day: i64, view: Counted) {
        let counts = self.days.entry((host.clone(), day)).or_default();
        counts.views += 1;
        counts.visitors.insert(view.visitor);
        let listed = view.site == SiteKind::Blog
            || counts.pages.len() < MAX_EXTERNAL_PAGES
            || counts.pages.contains_key(&view.path);
        if listed {
            *counts.pages.entry(view.path).or_default() += 1;
        }
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

    /// Adds one click on the link `target` of `host` on `day`.
    pub fn add_click(&mut self, host: &Host, day: i64, kind: ClickKind, target: String) {
        let clicks = &mut self.days.entry((host.clone(), day)).or_default().clicks;
        let key = (kind, target);
        if clicks.len() < MAX_CLICK_TARGETS || clicks.contains_key(&key) {
            *clicks.entry(key).or_default() += 1;
        }
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
        analytics::{Browser, ClickKind, Device},
        site::{host::Host, kind::SiteKind},
    };

    use crate::analytics::pending::{
        Counted, MAX_CLICK_TARGETS, MAX_EXTERNAL_PAGES, MAX_REFERRERS, Pending,
    };

    fn view(path: &str, referrer: Option<String>, visitor: u8) -> Counted {
        Counted {
            site: SiteKind::Blog,
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

    #[test]
    fn caps_distinct_pages_of_external_sites_only() {
        let host = Host::new("docs.example.org").unwrap();
        let external = |index: usize| Counted {
            site: SiteKind::External,
            ..view(&format!("/p{index}/"), None, 1)
        };
        let mut pending = Pending::default();
        for index in 0..MAX_EXTERNAL_PAGES + 5 {
            pending.add(&host, 10, external(index));
        }
        pending.add(&host, 10, external(0));
        let (_, _, today) = pending.days().next().unwrap();
        assert_eq!(usize::try_from(today.views), Ok(MAX_EXTERNAL_PAGES + 6));
        assert_eq!(today.pages.len(), MAX_EXTERNAL_PAGES);
        assert_eq!(today.pages["/p0/"], 2);
        let mut blog = Pending::default();
        for index in 0..MAX_EXTERNAL_PAGES + 5 {
            blog.add(&host, 10, view(&format!("/p{index}/"), None, 1));
        }
        let (_, _, today) = blog.days().next().unwrap();
        assert_eq!(today.pages.len(), MAX_EXTERNAL_PAGES + 5);
    }

    #[test]
    fn caps_distinct_click_targets() {
        let host = Host::new("docs.example.org").unwrap();
        let mut pending = Pending::default();
        for index in 0..MAX_CLICK_TARGETS + 5 {
            pending.add_click(&host, 10, ClickKind::Outbound, format!("t{index}.example/"));
        }
        pending.add_click(&host, 10, ClickKind::Outbound, "t0.example/".to_owned());
        pending.add_click(&host, 10, ClickKind::Download, "t0.example/".to_owned());
        let (_, _, today) = pending.days().next().unwrap();
        assert_eq!(today.clicks.len(), MAX_CLICK_TARGETS);
        assert_eq!(
            today.clicks[&(ClickKind::Outbound, "t0.example/".to_owned())],
            2
        );
        assert_eq!(today.views, 0);
        assert_eq!(pending.views(), 0);
    }
}
