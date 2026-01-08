use feu_core::ctx::Ctx;
use feu_core::middleware::{BoxFuture, Middleware, Next};
use feu_core::types::FeuResponse;
use http::StatusCode;
use tokio::time::Duration;

#[derive(Clone)]
pub struct Timeout {
    duration: Duration,
}

impl Timeout {
    pub fn new(duration: Duration) -> Self {
        Self { duration }
    }
}

impl Middleware for Timeout {
    fn handle(&self, ctx: Ctx, next: Next) -> BoxFuture<feu_core::error::Result<FeuResponse>> {
        let duration = self.duration;
        Box::pin(async move {
            match tokio::time::timeout(duration, next.run(ctx)).await {
                Ok(res) => res,
                Err(_) => {
                    // Timed out
                    Ok(FeuResponse::empty(StatusCode::GATEWAY_TIMEOUT))
                }
            }
        })
    }
}
