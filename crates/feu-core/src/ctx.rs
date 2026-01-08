use crate::extract::FromRequest;
use crate::rt::RuntimeCtx;
use crate::types::{FeuBody, FeuRequest, FeuResponse};
use http::{HeaderName, HeaderValue, StatusCode};
use std::sync::Arc;

/// `Ctx` (Context) for the request/response lifecycle.
///
/// Follows the "A-plan" design:
/// - `status(...)` and `header(...)` set pending state.
/// - Response helpers like `text(...)` consume this pending state.
///
/// **Lifecycle Note**: Response helper methods consume `self` (`Ctx`), enforcing that only one response
/// is generated and preventing modification after the response is finalized. This is crucial for
/// correctness, especially with streaming bodies.
pub struct Ctx<E = ()> {
    pub req: FeuRequest,
    pub env: E,
    pub runtime: Arc<dyn RuntimeCtx>,
    pub error: Option<Box<dyn std::error::Error + Send + Sync>>,

    // Extensions and other fields will go here
    pub(crate) pending_status: Option<StatusCode>,
    pub(crate) pending_headers: http::HeaderMap,
    pub(crate) params: Vec<(String, String)>,
}

impl<E: Clone + Send + Sync + 'static> Ctx<E> {
    pub fn new(
        req: FeuRequest,
        env: E,
        runtime: Arc<dyn RuntimeCtx>,
        params: Vec<(String, String)>,
    ) -> Self {
        Self {
            req,
            env,
            runtime,
            error: None,
            pending_status: None,
            pending_headers: http::HeaderMap::new(),
            params,
        }
    }

    /// Access the request
    pub fn req(&self) -> &FeuRequest {
        &self.req
    }

    /// Access the request mutably
    pub fn req_mut(&mut self) -> &mut FeuRequest {
        &mut self.req
    }

    /// Get a path parameter by name
    pub fn param(&self, key: &str) -> Option<&str> {
        // Linear search is fine for small param sets
        for (k, v) in &self.params {
            if k == key {
                return Some(v);
            }
        }
        None
    }

    /// Access the request method
    pub fn method(&self) -> &http::Method {
        self.req.method()
    }

    /// Access the request path
    pub fn path(&self) -> &str {
        self.req.uri().path()
    }

    // --- Pending Response Configuration (A-plan) ---

    /// Sets the pending status code.
    pub fn status(&mut self, code: StatusCode) -> &mut Self {
        self.pending_status = Some(code);
        self
    }

    /// Appends a pending header.
    pub fn header<K, V>(&mut self, key: K, value: V) -> &mut Self
    where
        K: TryInto<HeaderName>,
        V: TryInto<HeaderValue>,
    {
        // We unwrap here for ergonomics, or we could return Result.
        // Hono/Axum usually panic or fail on invalid headers in builder.
        // Let's silently ignore or panic. Valid headers are expected.
        if let (Ok(k), Ok(v)) = (key.try_into(), value.try_into()) {
            self.pending_headers.append(k, v);
        }
        self
    }

    #[cfg(feature = "cookie")]
    pub fn set_cookie(&mut self, cookie: cookie::Cookie<'_>) -> &mut Self {
        if let Ok(val) = http::HeaderValue::from_str(&cookie.to_string()) {
            self.pending_headers.append(http::header::SET_COOKIE, val);
        }
        self
    }

    /// Consumes pending state and resets it.
    pub(crate) fn take_pending(&mut self) -> (Option<StatusCode>, http::HeaderMap) {
        let status = self.pending_status.take();
        // efficient move? http::HeaderMap doesn't implement Default cleanly for "take" pattern easily without mem::replace
        let headers = std::mem::take(&mut self.pending_headers);
        (status, headers)
    }

    // --- Response Helpers ---

    fn apply_pending(&mut self, mut res: FeuResponse) -> FeuResponse {
        let (status, headers) = self.take_pending();

        if let Some(s) = status {
            res = res.with_status(s);
        }

        // Append headers
        // Append headers (handle multi-value headers properly)
        let res_headers = res.0.headers_mut();
        let mut last_key: Option<http::HeaderName> = None;
        for (k, v) in headers {
            if let Some(key) = k {
                last_key = Some(key.clone());
                res_headers.append(key, v);
            } else if let Some(ref key) = last_key {
                res_headers.append(key, v);
            }
        }

        res
    }

    pub fn text(mut self, body: impl Into<String>) -> FeuResponse {
        let res = FeuResponse::text(body);
        self.apply_pending(res)
    }

    pub fn html(mut self, body: impl Into<String>) -> FeuResponse {
        let res = FeuResponse::html(body);
        self.apply_pending(res)
    }

    #[cfg(feature = "json")]
    pub fn json<T: serde::Serialize>(mut self, value: T) -> crate::error::Result<FeuResponse> {
        let body =
            serde_json::to_string(&value).map_err(|e| crate::error::Error::msg(e.to_string()))?;
        let mut res = http::Response::new(FeuBody::Text(body));
        res.headers_mut().insert(
            http::header::CONTENT_TYPE,
            http::HeaderValue::from_static("application/json"),
        );
        let res = FeuResponse(res);
        Ok(self.apply_pending(res))
    }

    pub async fn extract<T: FromRequest<E>>(&mut self) -> crate::error::Result<T> {
        T::from_request(&mut self.req, &self.env).await
    }

    // Manual raw body
    pub fn body(mut self, body: impl Into<FeuBody>) -> FeuResponse {
        let res = FeuResponse(http::Response::new(body.into()));
        self.apply_pending(res)
    }

    pub fn redirect(mut self, location: impl Into<String>) -> FeuResponse {
        // Default to 302 Found, unless pending status is set
        let status = self.pending_status.unwrap_or(StatusCode::FOUND);
        // Reset pending status so apply_pending doesn't override it effectively (or does it?)
        // Actually apply_pending overrides.
        // If user did c.status(301).redirect("/foo"), they expect 301.

        let res = FeuResponse::redirect(location, status);

        // apply_pending will apply headers.
        // implementation detail: if self.pending_status was set, apply_pending uses it.
        // valid: redirect constructor uses status param. apply_pending will overwrite it with same value if set.

        self.apply_pending(res)
    }
}
