//! The analytics report of a site.

use axum::extract::{Query, State, rejection::QueryRejection};
use cuhlumn_api::analytics::{AnalyticsQuery, Period, SiteAnalytics};

use crate::{
    analytics::visitor::day_of,
    api::{auth::Admin, error::ApiError, json::Json, path::SitePath, sites::existing},
    db::{analytics::report::report, unix_now},
    state::AppState,
};

/// Returns the views and visitors of a site for a period that ends today (UTC).
///
/// The waiting page views are written first, so the report is current.
///
/// # Errors
///
/// Returns [`ApiError::Validation`] for an invalid period, and [`ApiError::NotFound`] when the site does not exist.
pub async fn get(
    _admin: Admin,
    State(state): State<AppState>,
    SitePath(host): SitePath,
    query: Result<Query<AnalyticsQuery>, QueryRejection>,
) -> Result<Json<SiteAnalytics>, ApiError> {
    let Query(query) = query.map_err(|_| ApiError::validation("period", period_hint()))?;
    let now = unix_now();
    state.analytics().flush(state.db(), now).await?;
    let analytics = state
        .db()
        .call(move |db| {
            let site = existing(db, &host)?;
            Ok::<_, ApiError>(report(db, &host, &site.title, query.period, day_of(now))?)
        })
        .await?;
    Ok(Json(analytics))
}

/// Returns the message for a query that is not a valid period.
fn period_hint() -> String {
    let names: Vec<&str> = Period::ALL.iter().map(|period| period.as_str()).collect();
    format!("use one of {}", names.join(", "))
}
