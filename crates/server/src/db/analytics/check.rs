//! The host and page check of an analytics beacon.

use cuhlumn_api::site::host::Host;
use rusqlite::Connection;

use crate::{db::DbError, render::markdown::slugify, web::route::Route};

/// Returns `true` when `host` is a canonical site host and `route` is a page of it that answers 200.
///
/// The pages are the home page, a published post or page other than the home page, and a tag page.
/// A tag page needs a published post with the tag, by the rule of the tag page itself.
///
/// # Errors
///
/// Returns [`DbError`] when a query fails or a stored tag list is invalid.
pub fn counts(connection: &Connection, host: &Host, route: &Route) -> Result<bool, DbError> {
    match route {
        Route::Home => exists(
            connection,
            "SELECT EXISTS (SELECT 1 FROM sites WHERE host = ?1)",
            &[host.as_str()],
        ),
        Route::Entry(slug) => exists(
            connection,
            "SELECT EXISTS (SELECT 1 FROM posts p JOIN sites s ON s.host = p.host
               WHERE p.host = ?1 AND p.slug = ?2 AND p.status = 'published' AND s.home IS NOT p.slug)",
            &[host.as_str(), slug.as_str()],
        ),
        Route::Tag(tag) => has_tag(connection, host, tag),
        _ => Ok(false),
    }
}

/// Runs a `SELECT EXISTS` query.
fn exists(connection: &Connection, sql: &str, params: &[&str]) -> Result<bool, DbError> {
    let found = connection
        .prepare_cached(sql)?
        .query_row(rusqlite::params_from_iter(params), |row| row.get(0))?;
    Ok(found)
}

/// Returns `true` when a published post of `host` has a tag whose address slug is `slug`.
fn has_tag(connection: &Connection, host: &Host, slug: &str) -> Result<bool, DbError> {
    let mut statement = connection.prepare_cached(
        "SELECT tags FROM posts WHERE host = ?1 AND status = 'published' AND kind = 'post'",
    )?;
    let mut rows = statement.query([host.as_str()])?;
    while let Some(row) = rows.next()? {
        let tags: String = row.get(0)?;
        let tags: Vec<String> = serde_json::from_str(&tags)?;
        if tags.iter().any(|tag| slugify(tag) == slug) {
            return Ok(true);
        }
    }
    Ok(false)
}
