//! Site and alias queries.

use cuhlumn_api::site::{Brand, Site, SiteInput, host::Host, indexnow_key::IndexNowKey};
use rusqlite::{Connection, OptionalExtension, Row, named_params};

use crate::db::{DbError, parse, parse_json, parse_optional};

/// Returns the site whose canonical domain is `host`.
///
/// # Errors
///
/// Returns [`DbError`] when the query fails or a stored value is invalid.
pub fn get(connection: &Connection, host: &Host) -> Result<Option<Site>, DbError> {
    let site = connection
        .query_row(
            &format!("{SELECT} WHERE host = ?1"),
            [host.as_str()],
            site_row,
        )
        .optional()?;
    site.map(|site| with_aliases(connection, site)).transpose()
}

/// Returns the site that `name` addresses, as its canonical domain or as one of its aliases.
///
/// Compare the returned `host` with `name` to find out if `name` is an alias.
///
/// # Errors
///
/// Returns [`DbError`] when the query fails or a stored value is invalid.
pub fn resolve(connection: &Connection, name: &Host) -> Result<Option<Site>, DbError> {
    match owner(connection, name)? {
        Some(host) => get(connection, &host),
        None => Ok(None),
    }
}

/// Returns every site, ordered by domain.
///
/// # Errors
///
/// Returns [`DbError`] when the query fails or a stored value is invalid.
pub fn list(connection: &Connection) -> Result<Vec<Site>, DbError> {
    let sites = connection
        .prepare(&format!("{SELECT} ORDER BY host"))?
        .query_map([], site_row)?
        .collect::<Result<Vec<_>, _>>()?;
    sites
        .into_iter()
        .map(|site| with_aliases(connection, site))
        .collect()
}

/// Returns the canonical domain of the site that uses `name` as its domain or as an alias.
///
/// # Errors
///
/// Returns [`DbError`] when the query fails or a stored value is invalid.
pub fn owner(connection: &Connection, name: &Host) -> Result<Option<Host>, DbError> {
    let host = connection
        .query_row(
            "SELECT host FROM sites WHERE host = ?1
             UNION ALL SELECT host FROM site_aliases WHERE alias = ?1
             LIMIT 1",
            [name.as_str()],
            |row| parse::<Host>(row, "host"),
        )
        .optional()?;
    Ok(host)
}

/// Creates or replaces the site at `host` and its aliases in one transaction.
///
/// The alias list drops `host` itself.
/// A new site without an input key gets `new_key`; an existing site keeps its key.
///
/// # Errors
///
/// Returns [`DbError`] when a query fails, for example when an alias belongs to another site.
pub fn put(
    connection: &mut Connection,
    host: &Host,
    input: &SiteInput,
    new_key: &IndexNowKey,
    now: i64,
) -> Result<(), DbError> {
    let brand = &input.brand;
    let links = serde_json::to_string(&brand.links)?;
    let tx = connection.transaction()?;
    tx.execute(
        "INSERT INTO sites (host, title, description, author, author_url, project_name, project_url,
           cta, lang, home, indexnow_key, accent, font, logo, favicon, og_image, links, head, footer,
           credit, updated_at)
         VALUES (:host, :title, :description, :author, :author_url, :project_name, :project_url,
           :cta, :lang, :home, COALESCE(:key, :new_key), :accent, :font, :logo, :favicon, :og_image,
           :links, :head, :footer, :credit, :now)
         ON CONFLICT (host) DO UPDATE SET title = excluded.title,
           description = excluded.description, author = excluded.author,
           author_url = excluded.author_url, project_name = excluded.project_name,
           project_url = excluded.project_url, cta = excluded.cta, lang = excluded.lang,
           home = excluded.home, indexnow_key = COALESCE(:key, sites.indexnow_key),
           accent = excluded.accent, font = excluded.font, logo = excluded.logo,
           favicon = excluded.favicon, og_image = excluded.og_image, links = excluded.links,
           head = excluded.head, footer = excluded.footer, credit = excluded.credit,
           updated_at = excluded.updated_at",
        named_params! {
            ":host": host.as_str(),
            ":title": input.title,
            ":description": input.description,
            ":author": input.author,
            ":author_url": input.author_url.as_ref().map(|url| url.as_str()),
            ":project_name": input.project_name,
            ":project_url": input.project_url.as_ref().map(|url| url.as_str()),
            ":cta": input.cta,
            ":lang": input.lang.as_str(),
            ":home": input.home.as_ref().map(|slug| slug.as_str()),
            ":key": input.indexnow_key.as_ref().map(|key| key.as_str()),
            ":new_key": new_key.as_str(),
            ":accent": brand.accent.as_str(),
            ":font": brand.font.as_str(),
            ":logo": brand.logo.as_ref().map(|url| url.as_str()),
            ":favicon": brand.favicon.as_ref().map(|url| url.as_str()),
            ":og_image": brand.og_image.as_ref().map(|url| url.as_str()),
            ":links": links,
            ":head": brand.head,
            ":footer": brand.footer,
            ":credit": brand.credit,
            ":now": now,
        },
    )?;
    tx.execute("DELETE FROM site_aliases WHERE host = ?1", [host.as_str()])?;
    {
        let mut insert = tx.prepare("INSERT INTO site_aliases (alias, host) VALUES (?1, ?2)")?;
        for alias in input.aliases.iter().filter(|alias| *alias != host) {
            insert.execute([alias.as_str(), host.as_str()])?;
        }
    }
    tx.commit()?;
    Ok(())
}

/// Deletes the site at `host` with its aliases, posts, and media records.
///
/// Returns `false` when no site exists at `host`.
///
/// # Errors
///
/// Returns [`DbError`] when the query fails.
pub fn delete(connection: &Connection, host: &Host) -> Result<bool, DbError> {
    Ok(connection.execute("DELETE FROM sites WHERE host = ?1", [host.as_str()])? > 0)
}

const SELECT: &str = "SELECT host, title, description, author, author_url, project_name,
  project_url, cta, lang, home, indexnow_key, accent, font, logo, favicon, og_image, links, head,
  footer, credit, updated_at FROM sites";

fn site_row(row: &Row<'_>) -> rusqlite::Result<Site> {
    Ok(Site {
        host: parse(row, "host")?,
        title: row.get("title")?,
        description: row.get("description")?,
        aliases: Vec::new(),
        author: row.get("author")?,
        author_url: parse_optional(row, "author_url")?,
        project_name: row.get("project_name")?,
        project_url: parse_optional(row, "project_url")?,
        cta: row.get("cta")?,
        lang: parse(row, "lang")?,
        home: parse_optional(row, "home")?,
        indexnow_key: parse(row, "indexnow_key")?,
        brand: Brand {
            accent: parse(row, "accent")?,
            font: parse(row, "font")?,
            logo: parse_optional(row, "logo")?,
            favicon: parse_optional(row, "favicon")?,
            og_image: parse_optional(row, "og_image")?,
            links: parse_json(row, "links")?,
            head: row.get("head")?,
            footer: row.get("footer")?,
            credit: row.get("credit")?,
        },
        updated_at: row.get("updated_at")?,
    })
}

fn with_aliases(connection: &Connection, mut site: Site) -> Result<Site, DbError> {
    site.aliases = connection
        .prepare_cached("SELECT alias FROM site_aliases WHERE host = ?1 ORDER BY alias")?
        .query_map([site.host.as_str()], |row| parse::<Host>(row, "alias"))?
        .collect::<Result<_, _>>()?;
    Ok(site)
}
