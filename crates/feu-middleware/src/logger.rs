use feu_core::ctx::Ctx;
use feu_core::middleware::{BoxFuture, Middleware, Next};
use feu_core::types::FeuResponse;
use std::time::Instant;

/// Logger middleware that logs request start and end using `tracing`.
pub struct Logger;

impl Logger {
    pub fn new() -> Self {
        Logger
    }
}

impl Default for Logger {
    fn default() -> Self {
        Self::new()
    }
}

impl Middleware for Logger {
    fn handle(&self, ctx: Ctx, next: Next) -> BoxFuture<feu_core::error::Result<FeuResponse>> {
        Box::pin(async move {
            let start = Instant::now();
            let method = ctx.method().clone();
            let path = ctx.path().to_owned();

            tracing::info!(method = ?method, path = %path, "request started");

            let res = next.run(ctx).await;

            let latency = start.elapsed();
            match &res {
                Ok(r) => {
                    let status = r.status().as_u16();
                    tracing::info!(
                        method = ?method,
                        path = %path,
                        status = status,
                        latency = ?latency,
                        "request completed"
                    );
                }
                Err(e) => {
                    tracing::error!(
                        method = ?method,
                        path = %path,
                        error = ?e,
                        latency = ?latency,
                        "request failed"
                    );
                }
            }

            res
        })
    }
}
