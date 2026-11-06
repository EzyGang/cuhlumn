//! Post handlers.

use axum::{extract::State, http::StatusCode};
use cuhlumn_api::{
    post::{Kind, Post, PostInput, Status, slug::Slug},
    site::host::Host,
    wire::PostSaved,
};

use crate::{
    api::{
        auth::Admin,
        error::ApiError,
        json::Json,
        path::{PostPath, SitePath},
        sites::existing,
    },
    db::{
        posts::{self, Visibility},
        unix_now,
    },
    state::AppState,
};

/// Returns the posts and pages of a site, drafts included, newest first.
///
/// # Errors
///
/// Returns [`ApiError::NotFound`] when the site does not exist.
pub async fn list(
    _admin: Admin,
    State(state): State<AppState>,
    SitePath(host): SitePath,
) -> Result<Json<Vec<Post>>, ApiError> {
    let scheme = state.config().scheme;
    let posts = state
        .db()
        .call(move |db| {
            existing(db, &host)?;
            Ok::<_, ApiError>(posts::list(db, scheme, &host, Visibility::All)?)
        })
        .await?;
    Ok(Json(posts))
}

/// Returns one post or page.
///
/// # Errors
///
/// Returns [`ApiError::NotFound`] when the site or the post does not exist.
pub async fn get(
    _admin: Admin,
    State(state): State<AppState>,
    PostPath(host, slug): PostPath,
) -> Result<Json<Post>, ApiError> {
    let scheme = state.config().scheme;
    let post = state
        .db()
        .call(move |db| {
            existing(db, &host)?;
            posts::get(db, scheme, &host, &slug)?.ok_or_else(|| not_found(&host, &slug))
        })
        .await?;
    Ok(Json(post))
}

/// Creates or replaces a post. A published post goes to IndexNow.
///
/// # Errors
///
/// Returns [`ApiError::Conflict`] when the post is the home page and the input changes its kind.
pub async fn put(
    _admin: Admin,
    State(state): State<AppState>,
    PostPath(host, slug): PostPath,
    Json(input): Json<PostInput>,
) -> Result<Json<PostSaved>, ApiError> {
    let scheme = state.config().scheme;
    let (site, post) = state
        .db()
        .call(move |db| {
            let site = existing(db, &host)?;
            if site.home.as_ref() == Some(&slug) && input.kind != Kind::Page {
                return Err(ApiError::Conflict(format!(
                    "{slug} is the home page of {host}, so it must stay a page"
                )));
            }
            let post = posts::put(db, scheme, &host, &slug, &input, unix_now())?;
            Ok((site, post))
        })
        .await?;
    state.cache().invalidate(&site.host);
    if post.status == Status::Published {
        let urls = vec![
            post.url.clone(),
            format!("{}/", state.config().origin(&site.host)),
        ];
        let _ping = state.indexnow().ping(scheme, &site, urls);
    }
    Ok(Json(PostSaved {
        url: post.url.clone(),
        post,
    }))
}

/// Deletes one post.
///
/// # Errors
///
/// Returns [`ApiError::Conflict`] for the home page and [`ApiError::NotFound`] when the post does not exist.
pub async fn delete(
    _admin: Admin,
    State(state): State<AppState>,
    PostPath(host, slug): PostPath,
) -> Result<StatusCode, ApiError> {
    let target = host.clone();
    state
        .db()
        .call(move |db| {
            let site = existing(db, &target)?;
            if site.home.as_ref() == Some(&slug) {
                return Err(ApiError::Conflict(format!(
                    "{slug} is the home page of {target}; choose another home page first"
                )));
            }
            if !posts::delete(db, &target, &slug)? {
                return Err(not_found(&target, &slug));
            }
            Ok(())
        })
        .await?;
    state.cache().invalidate(&host);
    Ok(StatusCode::NO_CONTENT)
}

fn not_found(host: &Host, slug: &Slug) -> ApiError {
    ApiError::NotFound(format!("the site {host} has no post {slug}"))
}
