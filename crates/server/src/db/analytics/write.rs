//! The write of a batch of page views.

use cuhlumn_api::site::host::Host;
use rusqlite::{Connection, Transaction, params};

use crate::{
    analytics::{
        pending::{DayPending, Pending},
        visitor::{Salt, day_of},
    },
    db::DbError,
};

/// Writes `batch` in one transaction, deletes the visitor hashes of past days, and stores `salt`.
///
/// A site that no longer exists is skipped.
/// The visitors of a day grow by the hashes that the day did not have yet.
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
        if site_exists(&tx, host)? {
            write_day(&tx, host, day, counts)?;
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

fn site_exists(tx: &Transaction<'_>, host: &Host) -> Result<bool, DbError> {
    let found = tx
        .prepare_cached("SELECT EXISTS (SELECT 1 FROM sites WHERE host = ?1)")?
        .query_row([host.as_str()], |row| row.get(0))?;
    Ok(found)
}

/// Adds the counts of one site and day.
fn write_day(
    tx: &Transaction<'_>,
    host: &Host,
    day: i64,
    counts: &DayPending,
) -> Result<(), DbError> {
    let host = host.as_str();
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
    let mut page = tx.prepare_cached(
        "INSERT INTO analytics_pages (host, day, path, views) VALUES (?1, ?2, ?3, ?4)
         ON CONFLICT (host, day, path) DO UPDATE SET views = views + excluded.views",
    )?;
    for (path, views) in &counts.pages {
        page.execute(params![host, day, path, count(*views)])?;
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
    Ok(())
}

/// Converts a count for SQLite, which stores signed 64-bit integers.
fn count(value: u64) -> i64 {
    i64::try_from(value).unwrap_or(i64::MAX)
}
