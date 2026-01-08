use feu_core::ctx::Ctx;
use feu_core::middleware::{BoxFuture, Middleware, Next};
use feu_core::types::FeuResponse;
use http::header::HeaderName;
use uuid::Uuid;

#[derive(Clone)]
pub struct RequestId {
    header_name: HeaderName,
}

impl RequestId {
    pub fn new() -> Self {
        Self {
            header_name: HeaderName::from_static("x-request-id"),
        }
    }

    pub fn with_header_name(name: &str) -> Self {
        Self {
            header_name: HeaderName::from_bytes(name.as_bytes()).unwrap(), // Panic if invalid
        }
    }
}

impl Default for RequestId {
    fn default() -> Self {
        Self::new()
    }
}

impl Middleware for RequestId {
    fn handle(&self, mut ctx: Ctx, next: Next) -> BoxFuture<feu_core::error::Result<FeuResponse>> {
        let header_name = self.header_name.clone();
        Box::pin(async move {
            let existing = ctx.req.headers().get(&header_name);
            let id = if let Some(val) = existing {
                val.to_str().unwrap_or_default().to_string()
            } else {
                Uuid::new_v4().to_string()
            };

            // If generated, inject into request headers so handlers can see it
            if existing.is_none() {
                if let Ok(val) = http::HeaderValue::from_str(&id) {
                    ctx.req.headers_mut().insert(header_name.clone(), val);
                }
            }

            // TODO: Also extensions?

            let mut res = next.run(ctx).await;

            // Inject into response
            if let Ok(ref mut response) = res {
                if let Ok(val) = http::HeaderValue::from_str(&id) {
                    response.headers_mut().insert(header_name, val);
                }
            }

            res
        })
    }
}
