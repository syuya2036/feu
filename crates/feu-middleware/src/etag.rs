use feu_core::ctx::Ctx;
use feu_core::middleware::{BoxFuture, Middleware, Next};
use feu_core::types::{FeuBody, FeuResponse};
use http::StatusCode;
use sha1::{Digest, Sha1};

#[derive(Clone)]
pub struct Etag;

impl Etag {
    pub fn new() -> Self {
        Etag
    }
}

impl Default for Etag {
    fn default() -> Self {
        Self::new()
    }
}

impl Middleware for Etag {
    fn handle(&self, ctx: Ctx, next: Next) -> BoxFuture<feu_core::error::Result<FeuResponse>> {
        let if_none_match = ctx
            .req
            .headers()
            .get(http::header::IF_NONE_MATCH)
            .and_then(|v| v.to_str().ok())
            .map(|s| s.to_string());

        Box::pin(async move {
            let mut res = next.run(ctx).await?;

            if res.status() == StatusCode::OK {
                let body_bytes = match res.body() {
                    FeuBody::Bytes(b) => Some(b.as_ref()),
                    FeuBody::Text(s) => Some(s.as_bytes()),
                    _ => None,
                };

                if let Some(bytes) = body_bytes {
                    let mut hasher = Sha1::new();
                    hasher.update(bytes);
                    let result = hasher.finalize();
                    // Simple hex formatting for ETag
                    let etag = format!("\"{:x}\"", result);

                    if let Ok(val) = http::HeaderValue::from_str(&etag) {
                        res.headers_mut().insert(http::header::ETAG, val);
                    }

                    if let Some(req_etag) = if_none_match {
                        if req_etag == etag {
                            let (mut parts, _) = res.0.into_parts();
                            parts.status = StatusCode::NOT_MODIFIED;
                            res = FeuResponse(http::Response::from_parts(parts, FeuBody::Empty));
                        }
                    }
                }
            }
            Ok(res)
        })
    }
}
