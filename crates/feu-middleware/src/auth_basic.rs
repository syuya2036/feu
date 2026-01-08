use base64::prelude::*;
use feu_core::ctx::Ctx;
use feu_core::middleware::{BoxFuture, Middleware, Next};
use feu_core::types::FeuResponse;
use http::header::{AUTHORIZATION, WWW_AUTHENTICATE};
use http::StatusCode;
use std::sync::Arc;

#[derive(Clone)]
pub struct BasicAuth {
    validator: Arc<dyn Fn(&str, &str) -> bool + Send + Sync>,
    realm: String,
}

impl BasicAuth {
    pub fn new(validator: impl Fn(&str, &str) -> bool + Send + Sync + 'static) -> Self {
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

impl Middleware for BasicAuth {
    fn handle(&self, ctx: Ctx, next: Next) -> BoxFuture<feu_core::error::Result<FeuResponse>> {
        let auth_header = ctx
            .req
            .headers()
            .get(AUTHORIZATION)
            .and_then(|h| h.to_str().ok());

        let valid = if let Some(header) = auth_header {
            if header.starts_with("Basic ") {
                let token = &header[6..];
                if let Ok(decoded) = BASE64_STANDARD.decode(token) {
                    if let Ok(cred) = String::from_utf8(decoded) {
                        if let Some((user, pass)) = cred.split_once(':') {
                            (self.validator)(user, pass)
                        } else {
                            false
                        }
                    } else {
                        false
                    }
                } else {
                    false
                }
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
                    format!("Basic realm=\"{}\"", realm).parse().unwrap(),
                );
                Ok(res)
            })
        }
    }
}
