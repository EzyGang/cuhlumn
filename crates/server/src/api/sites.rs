//! Site handlers.

use axum::{extract::State, http::StatusCode};
use cuhlumn_api::{
    post::{Kind, slug::Slug},
    site::{Site, SiteInput, host::Host, indexnow_key::IndexNowKey},
};
use rusqlite::Connection;

use crate::{
    api::{auth::Admin, error::ApiError, json::Json, path::SitePath},
    db::{fonts, posts, sites, unix_now},
    state::AppState,
};

/// Returns every site, ordered by domain.
///
/// # Errors
///
/// Returns [`ApiError`] when the database fails.
pub async fn list(
    _admin: Admin,
    State(state): State<AppState>,
) -> Result<Json<Vec<Site>>, ApiError> {
    let sites = state.db().call(|db| sites::list(db)).await?;
    Ok(Json(sites))
}

/// Returns one site.
///
/// # Errors
///
/// Returns [`ApiError::NotFound`] when no site has this canonical domain.
pub async fn get(
    _admin: Admin,
    State(state): State<AppState>,
    SitePath(host): SitePath,
) -> Result<Json<Site>, ApiError> {
    let site = state.db().call(move |db| existing(db, &host)).await?;
    Ok(Json(site))
}

/// Creates or replaces a site.
///
/// # Errors
///
/// Returns [`ApiError::Conflict`] when the domain or an alias belongs to another site,
/// [`ApiError::FieldConflict`] when it is the console host,
/// and [`ApiError::Validation`] when `home` does not name a page of the site
/// or a new `brand.font_family` is not a family of the font library.
pub async fn put(
    _admin: Admin,
    State(state): State<AppState>,
    SitePath(host): SitePath,
    Json(input): Json<SiteInput>,
) -> Result<Json<Site>, ApiError> {
    check_console(state.settings().console_host(), &host, &input.aliases)?;
    let new_key = random_key()?;
    let site = state
        .db()
        .call(move |db| {
            check_names(db, &host, &input.aliases)?;
            check_home(db, &host, input.home.as_ref())?;
            check_font_family(db, &host, input.brand.font_family.as_deref())?;
            sites::put(db, &host, &input, &new_key, unix_now())?;
            existing(db, &host)
        })
        .await?;
    state.cache().invalidate(&site.host);
    Ok(Json(site))
}

/// Deletes a site with its posts, media records, and image files.
///
/// # Errors
///
/// Returns [`ApiError::NotFound`] when no site has this canonical domain.
pub async fn delete(
    _admin: Admin,
    State(state): State<AppState>,
    SitePath(host): SitePath,
) -> Result<StatusCode, ApiError> {
    let target = host.clone();
    let deleted = state
        .db()
        .call(move |db| sites::delete(db, &target))
        .await?;
    if !deleted {
        return Err(not_found(&host));
    }
    state.cache().invalidate(&host);
    state.media().remove_site(&host).await?;
    Ok(StatusCode::NO_CONTENT)
}

/// Returns the site at the canonical domain `host`.
///
/// # Errors
///
/// Returns [`ApiError::NotFound`] when no site has this canonical domain.
pub fn existing(db: &Connection, host: &Host) -> Result<Site, ApiError> {
    sites::get(db, host)?.ok_or_else(|| not_found(host))
}

fn not_found(host: &Host) -> ApiError {
    ApiError::NotFound(format!("no site has the domain {host}"))
}

/// Rejects a site domain or alias that is the console host.
///
/// Without a console host, for example with an admin listener, every name passes.
fn check_console(console: Option<&Host>, host: &Host, aliases: &[Host]) -> Result<(), ApiError> {
    let Some(console) = console else {
        return Ok(());
    };
    let conflict = |field: &str| ApiError::FieldConflict {
        field: field.to_owned(),
        message: format!("{console} is the console host"),
    };
    if host == console {
        return Err(conflict("host"));
    }
    match aliases.iter().position(|alias| alias == console) {
        Some(index) => Err(conflict(&format!("aliases[{index}]"))),
        None => Ok(()),
    }
}

fn check_names(db: &Connection, host: &Host, aliases: &[Host]) -> Result<(), ApiError> {
    if let Some(owner) = sites::owner(db, host)?.filter(|owner| owner != host) {
        return Err(ApiError::Conflict(format!(
            "{host} is an alias of the site {owner}"
        )));
    }
    for alias in aliases.iter().filter(|alias| *alias != host) {
        if let Some(owner) = sites::owner(db, alias)?.filter(|owner| owner != host) {
            return Err(ApiError::Conflict(format!(
                "{alias} already belongs to the site {owner}"
            )));
        }
    }
    Ok(())
}

fn check_home(db: &Connection, host: &Host, home: Option<&Slug>) -> Result<(), ApiError> {
    let Some(slug) = home else {
        return Ok(());
    };
    match posts::kind(db, host, slug)? {
        Some(Kind::Page) => Ok(()),
        Some(Kind::Post) => Err(ApiError::validation(
            "home",
            format!("{slug} is a post; the home page must be a page"),
        )),
        None => Err(ApiError::validation(
            "home",
            format!("the site has no page {slug}"),
        )),
    }
}

/// Rejects a font family that changes to a family that the font library does not have.
///
/// The stored value always passes, so a site whose library font was deleted can still save other fields.
fn check_font_family(db: &Connection, host: &Host, family: Option<&str>) -> Result<(), ApiError> {
    let Some(family) = family else {
        return Ok(());
    };
    if fonts::has_family(db, family)? {
        return Ok(());
    }
    let stored = sites::get(db, host)?.and_then(|site| site.brand.font_family);
    match stored.as_deref() == Some(family) {
        true => Ok(()),
        false => Err(ApiError::validation(
            "brand.font_family",
            format!("the font library has no family {family}"),
        )),
    }
}

fn random_key() -> Result<IndexNowKey, ApiError> {
    let mut bytes = [0_u8; 16];
    getrandom::fill(&mut bytes).map_err(ApiError::Random)?;
    Ok(IndexNowKey::new(hex::encode(bytes))?)
}
