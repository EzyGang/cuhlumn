//! The analytics report of one site.

use std::collections::HashMap;

use chrono::DateTime;
use cuhlumn_api::{
    analytics::{
        Browser, BrowserCount, DayCount, Device, DeviceCount, PageCount, Period, ReferrerCount,
        SiteAnalytics,
    },
    site::host::Host,
};
use rusqlite::{Connection, Row, params};

use crate::db::{DbError, parse};

/// The most pages and referrers that a report lists.
const TOP: i64 = 10;

/// Returns the report of `host` for `period`, which ends on the day `today`.
///
/// `site_title` is the title of the home page `/`.
///
/// # Errors
///
/// Returns [`DbError`] when a query fails or a stored value is invalid.
pub fn report(
    connection: &Connection,
    host: &Host,
    site_title: &str,
    period: Period,
    today: i64,
) -> Result<SiteAnalytics, DbError> {
    let from = today - i64::from(period.days()) + 1;
    let range = Range {
        host: host.as_str(),
        from,
        to: today,
    };
    let days = days(connection, &range)?;
    Ok(SiteAnalytics {
        period,
        from: date(from),
        to: date(today),
        views: days.iter().map(|day| day.views).sum(),
        visitors: days.iter().map(|day| day.visitors).sum(),
        days,
        pages: pages(connection, &range, site_title)?,
        referrers: referrers(connection, &range)?,
        devices: grouped(connection, &range, "device", |device: Device, views| {
            DeviceCount { device, views }
        })?,
        browsers: grouped(connection, &range, "browser", |browser: Browser, views| {
            BrowserCount { browser, views }
        })?,
    })
}

/// The site and the days of a report, both days included.
struct Range<'a> {
    host: &'a str,
    from: i64,
    to: i64,
}

/// Returns one entry for each day of the range, with zero for days without views.
fn days(connection: &Connection, range: &Range<'_>) -> Result<Vec<DayCount>, DbError> {
    let stored: HashMap<i64, (u64, u64)> = connection
        .prepare_cached(
            "SELECT day, views, visitors FROM analytics_days
             WHERE host = ?1 AND day BETWEEN ?2 AND ?3",
        )?
        .query_map(params![range.host, range.from, range.to], |row| {
            Ok((row.get(0)?, (count(row, 1)?, count(row, 2)?)))
        })?
        .collect::<Result<_, _>>()?;
    let days = (range.from..=range.to)
        .map(|day| {
            let (views, visitors) = stored.get(&day).copied().unwrap_or_default();
            DayCount {
                date: date(day),
                views,
                visitors,
            }
        })
        .collect();
    Ok(days)
}

/// Returns the most viewed pages with the title of the post, or the site title for `/`.
fn pages(
    connection: &Connection,
    range: &Range<'_>,
    site_title: &str,
) -> Result<Vec<PageCount>, DbError> {
    let pages = connection
        .prepare_cached(
            "SELECT a.path, SUM(a.views) AS views, MAX(p.title) AS title FROM analytics_pages a
             LEFT JOIN posts p ON p.host = a.host AND a.path = '/' || p.slug || '/'
             WHERE a.host = ?1 AND a.day BETWEEN ?2 AND ?3
             GROUP BY a.path ORDER BY views DESC, a.path LIMIT ?4",
        )?
        .query_map(params![range.host, range.from, range.to, TOP], |row| {
            let path: String = row.get(0)?;
            let title = match path.as_str() {
                "/" => Some(site_title.to_owned()),
                _ => row.get(2)?,
            };
            Ok(PageCount {
                views: count(row, 1)?,
                path,
                title,
            })
        })?
        .collect::<Result<_, _>>()?;
    Ok(pages)
}

/// Returns the referrers with the most views. The empty referrer counts direct visits.
fn referrers(connection: &Connection, range: &Range<'_>) -> Result<Vec<ReferrerCount>, DbError> {
    let referrers = connection
        .prepare_cached(
            "SELECT referrer, SUM(views) AS views FROM analytics_referrers
             WHERE host = ?1 AND day BETWEEN ?2 AND ?3
             GROUP BY referrer ORDER BY views DESC, referrer LIMIT ?4",
        )?
        .query_map(params![range.host, range.from, range.to, TOP], |row| {
            let host: String = row.get(0)?;
            Ok(ReferrerCount {
                host: (!host.is_empty()).then_some(host),
                views: count(row, 1)?,
            })
        })?
        .collect::<Result<_, _>>()?;
    Ok(referrers)
}

/// Returns the views of each value of the client `column`, most views first.
fn grouped<T, C, F>(
    connection: &Connection,
    range: &Range<'_>,
    column: &str,
    make: F,
) -> Result<Vec<C>, DbError>
where
    T: TryFrom<String>,
    T::Error: std::error::Error + Send + Sync + 'static,
    F: Fn(T, u64) -> C,
{
    let rows = connection
        .prepare_cached(&format!(
            "SELECT {column} AS name, SUM(views) AS views FROM analytics_clients
             WHERE host = ?1 AND day BETWEEN ?2 AND ?3
             GROUP BY {column} ORDER BY views DESC, {column}"
        ))?
        .query_map(params![range.host, range.from, range.to], |row| {
            Ok(make(parse::<T>(row, "name")?, count(row, 1)?))
        })?
        .collect::<Result<_, _>>()?;
    Ok(rows)
}

/// Returns a day number as `YYYY-MM-DD`.
fn date(day: i64) -> String {
    DateTime::from_timestamp(day.saturating_mul(86_400), 0)
        .map(|time| time.format("%Y-%m-%d").to_string())
        .unwrap_or_default()
}

/// Reads a count column. A negative value reads as zero.
fn count(row: &Row<'_>, index: usize) -> rusqlite::Result<u64> {
    let value: i64 = row.get(index)?;
    Ok(u64::try_from(value).unwrap_or_default())
}
