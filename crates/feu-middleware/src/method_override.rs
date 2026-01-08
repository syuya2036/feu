use feu_core::ctx::Ctx;
use feu_core::middleware::{BoxFuture, Middleware, Next};
use feu_core::types::FeuResponse;
use http::Method;

#[derive(Clone, Debug)]
pub struct MethodOverride {
    header: &'static str,
}

impl MethodOverride {
    pub fn new() -> Self {
        Self {
            header: "x-http-method-override",
        }
    }
}

impl Default for MethodOverride {
    fn default() -> Self {
        Self::new()
    }
}

impl Middleware for MethodOverride {
    fn handle(&self, mut ctx: Ctx, next: Next) -> BoxFuture<feu_core::error::Result<FeuResponse>> {
        let new_method = if let Some(val) = ctx.req.headers().get(self.header) {
            val.to_str()
                .ok()
                .and_then(|s| Method::from_bytes(s.as_bytes()).ok())
        } else {
            None
        };

        if let Some(m) = new_method {
            *ctx.req.method_mut() = m;
        }

        Box::pin(async move { next.run(ctx).await })
    }
}
