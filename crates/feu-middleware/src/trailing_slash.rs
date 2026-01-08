use feu_core::ctx::Ctx;
use feu_core::middleware::{BoxFuture, Middleware, Next};
use feu_core::types::FeuResponse;
use http::StatusCode;

#[derive(Clone, Copy, Debug)]
pub struct TrailingSlash;

impl TrailingSlash {
    pub fn new() -> Self {
        TrailingSlash
    }
}

impl Default for TrailingSlash {
    fn default() -> Self {
        Self::new()
    }
}

impl Middleware for TrailingSlash {
    fn handle(&self, mut ctx: Ctx, next: Next) -> BoxFuture<feu_core::error::Result<FeuResponse>> {
        let path = ctx.req.uri().path();
        if path.len() > 1 && path.ends_with('/') {
            let new_path = path.trim_end_matches('/').to_string();
            let query = ctx
                .req
                .uri()
                .query()
                .map(|q| format!("?{}", q))
                .unwrap_or_default();
            let location = format!("{}{}", new_path, query);

            return Box::pin(async move {
                ctx.status(StatusCode::PERMANENT_REDIRECT);
                Ok(ctx.redirect(location))
            });
        }

        Box::pin(async move { next.run(ctx).await })
    }
}
