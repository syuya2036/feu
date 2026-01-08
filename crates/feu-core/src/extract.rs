use crate::error::Error;
use crate::types::{FeuBody, FeuRequest};
use bytes::Bytes;

use serde::de::DeserializeOwned;
use std::mem;

/// Trait for extracting data from the request.
///
/// Types implementing this trait can be used as handler arguments.
#[allow(async_fn_in_trait)]
pub trait FromRequest<E>: Sized {
    /// Extract the value from the request.
    ///
    /// This method allows mutation of the request, e.g., draining the body.
    async fn from_request(req: &mut FeuRequest, env: &E) -> Result<Self, Error>;
}

// --- Basic Extractors ---

/// Extract the full request body as `Bytes`.
impl<E> FromRequest<E> for Bytes {
    async fn from_request(req: &mut FeuRequest, _env: &E) -> Result<Self, Error> {
        let body = mem::replace(req.body_mut(), FeuBody::Empty);
        match body {
            FeuBody::Empty => Ok(Bytes::new()),
            FeuBody::Bytes(b) => Ok(b),
            FeuBody::Text(s) => Ok(Bytes::from(s.into_bytes())),
            #[cfg(feature = "streaming")]
            FeuBody::Stream(_) => Err(Error::msg(
                "Cannot extract Bytes from Stream (await collection needed)",
            )),
        }
    }
}

/// Extract the request body as `String`.
/// Fails if body is not valid UTF-8.
impl<E> FromRequest<E> for String {
    async fn from_request(req: &mut FeuRequest, _env: &E) -> Result<Self, Error> {
        let bytes = Bytes::from_request(req, _env).await?;
        String::from_utf8(bytes.to_vec()).map_err(|e| Error::msg(format!("Invalid UTF-8: {}", e)))
    }
}

// --- Query Extractor ---

pub struct Query<T>(pub T);

impl<E, T: DeserializeOwned> FromRequest<E> for Query<T> {
    async fn from_request(req: &mut FeuRequest, _env: &E) -> Result<Self, Error> {
        let query_str = req.uri().query().unwrap_or("");
        let value = serde_urlencoded::from_str(query_str)
            .map_err(|e| Error::msg(format!("Query parse error: {}", e)))?;
        Ok(Query(value))
    }
}

// --- JSON Extractor ---

#[cfg(feature = "json")]
pub struct Json<T>(pub T);

#[cfg(feature = "json")]
impl<E, T: DeserializeOwned> FromRequest<E> for Json<T> {
    async fn from_request(req: &mut FeuRequest, env: &E) -> Result<Self, Error> {
        if let Some(ctype) = req.headers().get(http::header::CONTENT_TYPE) {
            if ctype != "application/json" {
                // Hono allows flexible json parsing or strict?
                // Strict is safer.
                // But let's be loose for now or Check "starts_with"?
                // return Err(Error::new("Invalid Content-Type, expected application/json"));
            }
        }

        // Consume body as Bytes first
        let bytes = Bytes::from_request(req, env).await?;
        let value = serde_json::from_slice(&bytes)
            .map_err(|e| Error::msg(format!("JSON parse error: {}", e)))?;
        Ok(Json(value))
    }
}

// --- Path Extractor ---

pub struct Path<T>(pub T);

impl<E, T: DeserializeOwned> FromRequest<E> for Path<T> {
    async fn from_request(req: &mut FeuRequest, _env: &E) -> Result<Self, Error> {
        let params = req
            .extensions()
            .get::<crate::types::Params>()
            .ok_or_else(|| Error::msg("No path parameters found"))?;

        // Round-trip through serde_urlencoded to handle type conversion from string values (e.g. "42" -> 42)
        let qs = serde_urlencoded::to_string(&params.0)
            .map_err(|e| Error::msg(format!("Failed to serialize params: {}", e)))?;
        let value = serde_urlencoded::from_str(&qs)
            .map_err(|e| Error::msg(format!("Failed to deserialize path params: {}", e)))?;
        Ok(Path(value))
    }
}

impl<E> FromRequest<E> for http::HeaderMap {
    async fn from_request(req: &mut FeuRequest, _env: &E) -> Result<Self, Error> {
        Ok(req.headers().clone())
    }
}
