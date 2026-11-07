//! First launch setup, server settings, and token rotation.

use axum::{
    extract::State,
    http::{HeaderMap, header},
    response::Response,
};
use cuhlumn_api::{
    settings::{ServerSettings, ServerSettingsInput, TokenInput},
    site::host::Host,
};
use rusqlite::Connection;

use crate::{
    api::{
        auth::{Admin, ConsoleRequest, check_origin},
        error::ApiError,
        json::Json,
        session,
    },
    config::Token,
    db::{settings, sites, unix_now},
    state::{AppState, settings::Settings},
};

/// Stores the first admin token and starts a session.
///
/// Only the first request wins. Console requests get the session cookie.
///
/// # Errors
///
/// Returns [`ApiError::Conflict`] when the server already has a token.
pub async fn setup(
    State(state): State<AppState>,
    ConsoleRequest(console): ConsoleRequest,
    headers: HeaderMap,
    Json(input): Json<TokenInput>,
) -> Result<Response, ApiError> {
    if let Some(host) = &console {
        check_origin_if_sent(&headers, host)?;
    }
    let token = token(&input)?;
    let digest = token.digest_hex();
    let (stored, _) = state
        .change_settings(move |db, current| {
            if current.token().is_some() {
                return Ok::<_, ApiError>(false);
            }
            Ok(settings::set_token_if_empty(db, &digest, unix_now())?)
        })
        .await?;
    if !stored {
        return Err(ApiError::Conflict(
            "the server already has an admin token".to_owned(),
        ));
    }
    session::start(&state, &token, console.is_some()).await
}

/// Returns the effective server settings.
pub async fn get(_admin: Admin, State(state): State<AppState>) -> Json<ServerSettings> {
    Json(state.settings().to_api())
}

/// Stores the console host and the scheme.
///
/// A scheme change drops every cached page, because pages contain absolute addresses.
///
/// # Errors
///
/// Returns [`ApiError::Locked`] when an environment variable sets a field to another value,
/// and [`ApiError::FieldConflict`] when the console host is a site domain or alias.
pub async fn put(
    _admin: Admin,
    State(state): State<AppState>,
    Json(input): Json<ServerSettingsInput>,
) -> Result<Json<ServerSettings>, ApiError> {
    let (before, after) = state
        .change_settings(move |db, current| {
            store_server(db, current, &input)?;
            Ok::<_, ApiError>(current.scheme())
        })
        .await?;
    if before != after.scheme() {
        state.cache().clear();
    }
    Ok(Json(after.to_api()))
}

/// Replaces the admin token, deletes every session, and starts a new session.
///
/// Console requests get the new session cookie.
///
/// # Errors
///
/// Returns [`ApiError::Locked`] when `CUHLUMN_TOKEN` sets the token.
pub async fn put_token(
    _admin: Admin,
    State(state): State<AppState>,
    ConsoleRequest(console): ConsoleRequest,
    Json(input): Json<TokenInput>,
) -> Result<Response, ApiError> {
    let token = token(&input)?;
    let digest = token.digest_hex();
    state
        .change_settings(move |db, current| {
            if current.locked().token {
                return Err(locked("token"));
            }
            Ok(settings::replace_token(db, &digest, unix_now())?)
        })
        .await?;
    session::start(&state, &token, console.is_some()).await
}

fn store_server(
    db: &Connection,
    current: &Settings,
    input: &ServerSettingsInput,
) -> Result<(), ApiError> {
    let fixed = current.locked();
    if fixed.console_host && input.console_host.as_ref() != current.console_host() {
        return Err(locked("console_host"));
    }
    if fixed.scheme && input.scheme != current.scheme() {
        return Err(locked("scheme"));
    }
    if let Some(host) = input.console_host.as_ref().filter(|_| !fixed.console_host)
        && let Some(owner) = sites::owner(db, host)?
    {
        return Err(ApiError::FieldConflict {
            field: "console_host".to_owned(),
            message: format!("{host} belongs to the site {owner}"),
        });
    }
    let stored = settings::get(db)?;
    let console_host = match fixed.console_host {
        true => stored.console_host,
        false => input.console_host.clone(),
    };
    let scheme = match fixed.scheme {
        true => stored.scheme.unwrap_or_default(),
        false => input.scheme,
    };
    settings::set_server(db, console_host.as_ref(), scheme, unix_now())?;
    Ok(())
}

fn token(input: &TokenInput) -> Result<Token, ApiError> {
    Token::new(&input.token).map_err(|err| ApiError::validation("token", err))
}

fn locked(field: &str) -> ApiError {
    ApiError::Locked {
        field: field.to_owned(),
    }
}

fn check_origin_if_sent(headers: &HeaderMap, host: &Host) -> Result<(), ApiError> {
    match headers.contains_key(header::ORIGIN) {
        true => check_origin(headers, host),
        false => Ok(()),
    }
}
