use feu_core::ctx::Ctx;
use feu_core::middleware::{BoxFuture, Middleware, Next};
use feu_core::types::FeuResponse;
use http::header::{AUTHORIZATION, WWW_AUTHENTICATE};
use http::StatusCode;
use std::sync::Arc;

#[derive(Clone)]
pub struct BearerAuth {
    validator: Arc<dyn Fn(&str) -> bool + Send + Sync>,
    realm: String,
}

impl BearerAuth {
    pub fn new(validator: impl Fn(&str) -> bool + Send + Sync + 'static) -> Self {
        Self {
            validator: Arc::new(validator),
            realm: "Restricted".to_string(),
        }
    }

    pub fn with_realm(mut self, realm: impl Into<String>) -> Self {
        self.realm = realm.into();
        self
    }
}

impl Middleware for BearerAuth {
    fn handle(&self, ctx: Ctx, next: Next) -> BoxFuture<feu_core::error::Result<FeuResponse>> {
        let auth_header = ctx
            .req
            .headers()
            .get(AUTHORIZATION)
            .and_then(|h| h.to_str().ok());

        let valid = if let Some(header) = auth_header {
            if let Some(token) = header.strip_prefix("Bearer ") {
                (self.validator)(token)
            } else {
                false
            }
        } else {
            false
        };

        if valid {
            Box::pin(async move { next.run(ctx).await })
        } else {
            let realm = self.realm.clone();
            Box::pin(async move {
                let mut res =
                    FeuResponse::text("Unauthorized").with_status(StatusCode::UNAUTHORIZED);
                res.headers_mut().insert(
                    WWW_AUTHENTICATE,
                    format!("Bearer realm=\"{}\"", realm).parse().unwrap(),
                );
                Ok(res)
            })
        }
    }
}
