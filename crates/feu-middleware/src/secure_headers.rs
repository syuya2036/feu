use feu_core::ctx::Ctx;
use feu_core::middleware::{BoxFuture, Middleware, Next};
use feu_core::types::FeuResponse;

#[derive(Clone)]
pub struct SecureHeaders;

impl SecureHeaders {
    pub fn new() -> Self {
        SecureHeaders
    }
}

impl Default for SecureHeaders {
    fn default() -> Self {
        Self::new()
    }
}

impl Middleware for SecureHeaders {
    fn handle(&self, ctx: Ctx, next: Next) -> BoxFuture<feu_core::error::Result<FeuResponse>> {
        Box::pin(async move {
            let mut res = next.run(ctx).await;

            if let Ok(ref mut response) = res {
                let h = response.headers_mut();
                h.insert("X-Frame-Options", "DENY".parse().unwrap());
                h.insert("X-Content-Type-Options", "nosniff".parse().unwrap());
                h.insert("X-XSS-Protection", "1; mode=block".parse().unwrap());
                h.insert(
                    "Strict-Transport-Security",
                    "max-age=63072000; includeSubDomains; preload"
                        .parse()
                        .unwrap(),
                );
                h.insert("Referrer-Policy", "same-origin".parse().unwrap());
            }

            res
        })
    }
}
