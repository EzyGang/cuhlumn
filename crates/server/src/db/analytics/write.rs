//! The write of a batch of page views and link clicks.

use std::collections::HashMap;

use cuhlumn_api::{
    analytics::ClickKind,
    site::{host::Host, kind::SiteKind},
};
use rusqlite::{Connection, OptionalExtension, Transaction, params};

use crate::{
    analytics::{
        pending::{DayPending, MAX_CLICK_TARGETS, MAX_EXTERNAL_PAGES, Pending},
        visitor::{Salt, day_of},
    },
    db::{DbError, parse},
};

/// Writes `batch` in one transaction, deletes the visitor hashes of past days, and stores `salt`.
///
/// A site that no longer exists is skipped.
/// The visitors of a day grow by the hashes that the day did not have yet.
/// An external site keeps at most [`MAX_EXTERNAL_PAGES`] page paths for one day,
/// and every site keeps at most [`MAX_CLICK_TARGETS`] link targets for one day.
///
/// # Errors
///
/// Returns [`DbError`] when a query fails.
pub fn flush(
    connection: &mut Connection,
    batch: &Pending,
    salt: &Salt,
    now: i64,
) -> Result<(), DbError> {
    let tx = connection.transaction()?;
    for (host, day, counts) in batch.days() {
        if let Some(kind) = site_kind(&tx, host)? {
            write_day(&tx, host.as_str(), day, counts, kind)?;
        }
    }
    tx.execute(
        "DELETE FROM analytics_visitors WHERE day < ?1",
        [day_of(now)],
    )?;
    tx.execute(
        "INSERT INTO analytics_salt (id, day, salt) VALUES (1, ?1, ?2)
         ON CONFLICT (id) DO UPDATE SET day = excluded.day, salt = excluded.salt",
        params![salt.day, salt.bytes.as_slice()],
    )?;
    tx.commit()?;
    Ok(())
}

/// Returns the kind of the site at `host`, or `None` when the site does not exist.
fn site_kind(tx: &Transaction<'_>, host: &Host) -> Result<Option<SiteKind>, DbError> {
    let kind = tx
        .prepare_cached("SELECT kind FROM sites WHERE host = ?1")?
        .query_row([host.as_str()], |row| parse(row, "kind"))
        .optional()?;
    Ok(kind)
}

/// Adds the counts of one site and day.
fn write_day(
    tx: &Transaction<'_>,
    host: &str,
    day: i64,
    counts: &DayPending,
    kind: SiteKind,
) -> Result<(), DbError> {
    if counts.views > 0 {
        write_totals(tx, host, day, counts)?;
    }
    match kind {
        SiteKind::Blog => write_pages(tx, host, day, &counts.pages)?,
        SiteKind::External => write_external_pages(tx, host, day, &counts.pages)?,
    }
    let mut referrer = tx.prepare_cached(
        "INSERT INTO analytics_referrers (host, day, referrer, views) VALUES (?1, ?2, ?3, ?4)
         ON CONFLICT (host, day, referrer) DO UPDATE SET views = views + excluded.views",
    )?;
    for (name, views) in &counts.referrers {
        let name = name.as_deref().unwrap_or_default();
        referrer.execute(params![host, day, name, count(*views)])?;
    }
    let mut client = tx.prepare_cached(
        "INSERT INTO analytics_clients (host, day, device, browser, views) VALUES (?1, ?2, ?3, ?4, ?5)
         ON CONFLICT (host, day, device, browser) DO UPDATE SET views = views + excluded.views",
    )?;
    for ((device, browser), views) in &counts.clients {
        client.execute(params![
            host,
            day,
            device.as_str(),
            browser.as_str(),
            count(*views)
        ])?;
    }
    write_clicks(tx, host, day, &counts.clicks)
}

/// Adds the views of the day and the visitors that the day did not have yet.
fn write_totals(
    tx: &Transaction<'_>,
    host: &str,
    day: i64,
    counts: &DayPending,
) -> Result<(), DbError> {
    let mut new_visitors = 0_i64;
    let mut insert = tx.prepare_cached(
        "INSERT OR IGNORE INTO analytics_visitors (host, day, hash) VALUES (?1, ?2, ?3)",
    )?;
    for hash in &counts.visitors {
        new_visitors +=
            i64::try_from(insert.execute(params![host, day, hash.as_slice()])?).unwrap_or_default();
    }
    tx.prepare_cached(
        "INSERT INTO analytics_days (host, day, views, visitors) VALUES (?1, ?2, ?3, ?4)
         ON CONFLICT (host, day) DO UPDATE SET
           views = views + excluded.views, visitors = visitors + excluded.visitors",
    )?
    .execute(params![host, day, count(counts.views), new_visitors])?;
    Ok(())
}

/// Adds the page views of a blog, whose paths the beacon check already limited to existing pages.
fn write_pages(
    tx: &Transaction<'_>,
    host: &str,
    day: i64,
    pages: &HashMap<String, u64>,
) -> Result<(), DbError> {
    let mut page = tx.prepare_cached(
        "INSERT INTO analytics_pages (host, day, path, views) VALUES (?1, ?2, ?3, ?4)
         ON CONFLICT (host, day, path) DO UPDATE SET views = views + excluded.views",
    )?;
    for (path, views) in pages {
        page.execute(params![host, day, path, count(*views)])?;
    }
    Ok(())
}

/// Adds the page views of an external site. A new path gets a row only below the limit of the day.
fn write_external_pages(
    tx: &Transaction<'_>,
    host: &str,
    day: i64,
    pages: &HashMap<String, u64>,
) -> Result<(), DbError> {
    if pages.is_empty() {
        return Ok(());
    }
    let mut rows = row_count(tx, "analytics_pages", host, day)?;
    let mut update = tx.prepare_cached(
        "UPDATE analytics_pages SET views = views + ?4 WHERE host = ?1 AND day = ?2 AND path = ?3",
    )?;
    let mut insert = tx.prepare_cached(
        "INSERT INTO analytics_pages (host, day, path, views) VALUES (?1, ?2, ?3, ?4)",
    )?;
    for (path, views) in pages {
        let views = count(*views);
        if update.execute(params![host, day, path, views])? == 0 && rows < MAX_EXTERNAL_PAGES {
            insert.execute(params![host, day, path, views])?;
            rows += 1;
        }
    }
    Ok(())
}

/// Adds the link clicks. A new target gets a row only below the limit of the day.
fn write_clicks(
    tx: &Transaction<'_>,
    host: &str,
    day: i64,
    clicks: &HashMap<(ClickKind, String), u64>,
) -> Result<(), DbError> {
    if clicks.is_empty() {
        return Ok(());
    }
    let mut rows = row_count(tx, "analytics_clicks", host, day)?;
    let mut update = tx.prepare_cached(
        "UPDATE analytics_clicks SET clicks = clicks + ?5
         WHERE host = ?1 AND day = ?2 AND kind = ?3 AND target = ?4",
    )?;
    let mut insert = tx.prepare_cached(
        "INSERT INTO analytics_clicks (host, day, kind, target, clicks) VALUES (?1, ?2, ?3, ?4, ?5)",
    )?;
    for ((kind, target), clicks) in clicks {
        let (kind, clicks) = (kind.as_str(), count(*clicks));
        let stored = update.execute(params![host, day, kind, target, clicks])? > 0;
        if !stored && rows < MAX_CLICK_TARGETS {
            insert.execute(params![host, day, kind, target, clicks])?;
            rows += 1;
        }
    }
    Ok(())
}

/// Returns the number of rows that `table` has for one site and day.
fn row_count(tx: &Transaction<'_>, table: &str, host: &str, day: i64) -> Result<usize, DbError> {
    let rows: i64 = tx
        .prepare_cached(&format!(
            "SELECT COUNT(*) FROM {table} WHERE host = ?1 AND day = ?2"
        ))?
        .query_row(params![host, day], |row| row.get(0))?;
    Ok(usize::try_from(rows).unwrap_or(usize::MAX))
}

/// Converts a count for SQLite, which stores signed 64-bit integers.
fn count(value: u64) -> i64 {
    i64::try_from(value).unwrap_or(i64::MAX)
}
