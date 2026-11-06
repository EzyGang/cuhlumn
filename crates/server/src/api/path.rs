//! Validated path parameters of the API routes.

use axum::{
    extract::{FromRequestParts, Path},
    http::request::Parts,
};
use cuhlumn_api::{post::slug::Slug, site::host::Host};

use crate::{api::error::ApiError, media::name::MediaName};

/// The `{host}` parameter.
#[derive(Clone, Debug)]
pub struct SitePath(pub Host);

/// The `{host}` and `{slug}` parameters.
#[derive(Clone, Debug)]
pub struct PostPath(pub Host, pub Slug);

/// The `{host}` and `{name}` parameters.
#[derive(Clone, Debug)]
pub struct MediaPath(pub Host, pub MediaName);

struct Params(Vec<(String, String)>);

impl<S> FromRequestParts<S> for SitePath
where
    S: Send + Sync,
{
    type Rejection = ApiError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let params = Params::from_parts(parts, state).await?;
        Ok(Self(params.host()?))
    }
}

impl<S> FromRequestParts<S> for PostPath
where
    S: Send + Sync,
{
    type Rejection = ApiError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let params = Params::from_parts(parts, state).await?;
        let slug =
            Slug::new(params.get("slug")?).map_err(|err| ApiError::validation("slug", err))?;
        Ok(Self(params.host()?, slug))
    }
}

impl<S> FromRequestParts<S> for MediaPath
where
    S: Send + Sync,
{
    type Rejection = ApiError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let params = Params::from_parts(parts, state).await?;
        let name =
            MediaName::new(params.get("name")?).map_err(|err| ApiError::validation("name", err))?;
        Ok(Self(params.host()?, name))
    }
}

impl Params {
    async fn from_parts<S>(parts: &mut Parts, state: &S) -> Result<Self, ApiError>
    where
        S: Send + Sync,
    {
        let Path(params) = Path::<Vec<(String, String)>>::from_request_parts(parts, state)
            .await
            .map_err(|rejection| ApiError::BadRequest(rejection.body_text()))?;
        Ok(Self(params))
    }

    fn get(&self, name: &str) -> Result<&str, ApiError> {
        self.0
            .iter()
            .find(|(key, _)| key == name)
            .map(|(_, value)| value.as_str())
            .ok_or_else(|| ApiError::BadRequest(format!("the path has no {name}")))
    }

    fn host(&self) -> Result<Host, ApiError> {
        Host::new(self.get("host")?).map_err(|err| ApiError::validation("host", err))
    }
}
