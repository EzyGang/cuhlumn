//! Post queries.

use cuhlumn_api::{
    post::{Kind, Post, PostInput, Status, slug::Slug},
    site::host::Host,
};
use rusqlite::{Connection, OptionalExtension, Row, named_params, params};

use crate::{
    config::Scheme,
    db::{DbError, parse, parse_json, parse_optional},
};

/// The posts that a list query returns.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Visibility {
    /// Drafts and published posts.
    All,
    /// Published posts only.
    Published,
}

/// Returns one post or page of the site at `host`.
///
/// # Errors
///
/// Returns [`DbError`] when the query fails or a stored value is invalid.
pub fn get(
    connection: &Connection,
    scheme: Scheme,
    host: &Host,
    slug: &Slug,
) -> Result<Option<Post>, DbError> {
    let post = connection
        .query_row(
            &format!("{SELECT} WHERE p.host = ?1 AND p.slug = ?2"),
            [host.as_str(), slug.as_str()],
            |row| post_row(row, scheme, host),
        )
        .optional()?;
    Ok(post)
}

/// Returns the posts and pages of the site at `host`, newest first.
///
/// # Errors
///
/// Returns [`DbError`] when the query fails or a stored value is invalid.
pub fn list(
    connection: &Connection,
    scheme: Scheme,
    host: &Host,
    visibility: Visibility,
) -> Result<Vec<Post>, DbError> {
    let posts = connection
        .prepare_cached(&format!(
            "{SELECT} WHERE p.host = ?1 AND (?2 OR p.status = 'published')
             ORDER BY COALESCE(p.published_at, p.updated_at) DESC, p.slug"
        ))?
        .query_map(
            params![host.as_str(), visibility == Visibility::All],
            |row| post_row(row, scheme, host),
        )?
        .collect::<Result<_, _>>()?;
    Ok(posts)
}

/// Creates or replaces a post and returns the stored post.
///
/// An input `published_at` replaces the stored time.
/// Without it, the first publication time stays, and a first publication stores `now`.
///
/// # Errors
///
/// Returns [`DbError`] when a query fails or a stored value is invalid.
pub fn put(
    connection: &Connection,
    scheme: Scheme,
    host: &Host,
    slug: &Slug,
    input: &PostInput,
    now: i64,
) -> Result<Post, DbError> {
    let stored: Option<i64> = connection
        .query_row(
            "SELECT published_at FROM posts WHERE host = ?1 AND slug = ?2",
            [host.as_str(), slug.as_str()],
            |row| row.get(0),
        )
        .optional()?
        .flatten();
    let first_publication = (input.status == Status::Published).then_some(now);
    let published_at = input.published_at.or(stored).or(first_publication);
    connection.execute(
        "INSERT INTO posts (host, slug, title, description, body, tags, kind, status, cover,
           og_image, published_at, updated_at)
         VALUES (:host, :slug, :title, :description, :body, :tags, :kind, :status, :cover,
           :og_image, :published_at, :now)
         ON CONFLICT (host, slug) DO UPDATE SET title = excluded.title,
           description = excluded.description, body = excluded.body, tags = excluded.tags,
           kind = excluded.kind, status = excluded.status, cover = excluded.cover,
           og_image = excluded.og_image, published_at = excluded.published_at,
           updated_at = excluded.updated_at",
        named_params! {
            ":host": host.as_str(),
            ":slug": slug.as_str(),
            ":title": input.title,
            ":description": input.description,
            ":body": input.body,
            ":tags": serde_json::to_string(&input.tags)?,
            ":kind": input.kind.as_str(),
            ":status": input.status.as_str(),
            ":cover": input.cover.as_ref().map(|url| url.as_str()),
            ":og_image": input.og_image.as_ref().map(|url| url.as_str()),
            ":published_at": published_at,
            ":now": now,
        },
    )?;
    get(connection, scheme, host, slug)?
        .ok_or(DbError::Sqlite(rusqlite::Error::QueryReturnedNoRows))
}

/// Returns the kind of one post, or `None` when the post does not exist.
///
/// # Errors
///
/// Returns [`DbError`] when the query fails or the stored kind is invalid.
pub fn kind(connection: &Connection, host: &Host, slug: &Slug) -> Result<Option<Kind>, DbError> {
    let kind = connection
        .query_row(
            "SELECT kind FROM posts WHERE host = ?1 AND slug = ?2",
            [host.as_str(), slug.as_str()],
            |row| parse::<Kind>(row, "kind"),
        )
        .optional()?;
    Ok(kind)
}

/// Deletes one post. Returns `false` when the post does not exist.
///
/// # Errors
///
/// Returns [`DbError`] when the query fails.
pub fn delete(connection: &Connection, host: &Host, slug: &Slug) -> Result<bool, DbError> {
    let deleted = connection.execute(
        "DELETE FROM posts WHERE host = ?1 AND slug = ?2",
        [host.as_str(), slug.as_str()],
    )?;
    Ok(deleted > 0)
}

const SELECT: &str = "SELECT p.slug, p.title, p.description, p.body, p.tags, p.kind, p.status,
  p.cover, p.og_image, p.published_at, p.updated_at, s.home
  FROM posts p JOIN sites s ON s.host = p.host";

fn post_row(row: &Row<'_>, scheme: Scheme, host: &Host) -> rusqlite::Result<Post> {
    let slug: Slug = parse(row, "slug")?;
    let home: Option<String> = row.get("home")?;
    let url = if home.as_deref() == Some(slug.as_str()) {
        format!("{scheme}://{host}/")
    } else {
        format!("{scheme}://{host}/{slug}/")
    };
    Ok(Post {
        title: row.get("title")?,
        description: row.get("description")?,
        body: row.get("body")?,
        tags: parse_json(row, "tags")?,
        kind: parse(row, "kind")?,
        status: parse(row, "status")?,
        cover: parse_optional(row, "cover")?,
        og_image: parse_optional(row, "og_image")?,
        published_at: row.get("published_at")?,
        updated_at: row.get("updated_at")?,
        slug,
        url,
    })
}
