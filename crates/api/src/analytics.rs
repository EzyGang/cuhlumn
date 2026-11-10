//! The page-view analytics of a site.

use serde::{Deserialize, Serialize};

text_enum! {
    /// The time span of an analytics report, ending today (UTC).
    pub enum Period / PeriodError ("period") {
        /// The last 7 days.
        Week = "week",
        /// The last 30 days.
        #[default]
        Month = "month",
        /// The last 90 days.
        Quarter = "quarter",
        /// The last 365 days.
        Year = "year",
    }
}

text_enum! {
    /// The device class of a reader, from the User-Agent.
    pub enum Device / DeviceError ("device") {
        #[default]
        Desktop = "desktop",
        Mobile = "mobile",
        Tablet = "tablet",
    }
}

text_enum! {
    /// The browser family of a reader, from the User-Agent.
    pub enum Browser / BrowserError ("browser") {
        Chrome = "chrome",
        Safari = "safari",
        Firefox = "firefox",
        Edge = "edge",
        Opera = "opera",
        Samsung = "samsung",
        #[default]
        Other = "other",
    }
}

impl Period {
    /// Returns the number of days in the report, today included.
    #[must_use]
    pub const fn days(self) -> u16 {
        match self {
            Self::Week => 7,
            Self::Month => 30,
            Self::Quarter => 90,
            Self::Year => 365,
        }
    }
}

/// The query of `GET /api/sites/{host}/analytics`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[serde(default, deny_unknown_fields)]
pub struct AnalyticsQuery {
    pub period: Period,
}

/// The response of `GET /api/sites/{host}/analytics`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
pub struct SiteAnalytics {
    pub period: Period,
    /// The first day of the report, as `YYYY-MM-DD` (UTC).
    pub from: String,
    /// The last day of the report, as `YYYY-MM-DD` (UTC).
    pub to: String,
    /// The page views in the period.
    #[cfg_attr(feature = "ts", ts(type = "number"))]
    pub views: u64,
    /// The sum of the daily unique visitors in the period.
    #[cfg_attr(feature = "ts", ts(type = "number"))]
    pub visitors: u64,
    /// One entry for each day of the period, oldest first, with zero for days without views.
    pub days: Vec<DayCount>,
    /// The most viewed pages, at most 10.
    pub pages: Vec<PageCount>,
    /// The referrers with the most views, at most 10.
    pub referrers: Vec<ReferrerCount>,
    pub devices: Vec<DeviceCount>,
    pub browsers: Vec<BrowserCount>,
}

/// The views and visitors of one day.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
pub struct DayCount {
    /// The day, as `YYYY-MM-DD` (UTC).
    pub date: String,
    #[cfg_attr(feature = "ts", ts(type = "number"))]
    pub views: u64,
    #[cfg_attr(feature = "ts", ts(type = "number"))]
    pub visitors: u64,
}

/// The views of one page.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
pub struct PageCount {
    /// The page path, for example `/sqlite-backups/`.
    pub path: String,
    /// The post title, or the site title for `/`, when the page still exists.
    pub title: Option<String>,
    #[cfg_attr(feature = "ts", ts(type = "number"))]
    pub views: u64,
}

/// The views that came from one referrer.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
pub struct ReferrerCount {
    /// The referring host, or `None` for direct visits.
    pub host: Option<String>,
    #[cfg_attr(feature = "ts", ts(type = "number"))]
    pub views: u64,
}

/// The views from one device class.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
pub struct DeviceCount {
    pub device: Device,
    #[cfg_attr(feature = "ts", ts(type = "number"))]
    pub views: u64,
}

/// The views from one browser family.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
pub struct BrowserCount {
    pub browser: Browser,
    #[cfg_attr(feature = "ts", ts(type = "number"))]
    pub views: u64,
}
