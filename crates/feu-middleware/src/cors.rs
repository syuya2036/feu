use feu_core::ctx::Ctx;
use feu_core::middleware::{BoxFuture, Middleware, Next};
use feu_core::types::FeuResponse;
use http::{Method, StatusCode};

#[derive(Clone)]
pub struct Cors;

impl Cors {
    pub fn permissive() -> Self {
        Cors
    }
}

impl Middleware for Cors {
    fn handle(&self, ctx: Ctx, next: Next) -> BoxFuture<feu_core::error::Result<FeuResponse>> {
        Box::pin(async move {
            let is_options = ctx.method() == Method::OPTIONS;

            if is_options {
                let mut res = FeuResponse::empty(StatusCode::NO_CONTENT);
                let h = res.headers_mut();
                h.insert("Access-Control-Allow-Origin", "*".parse().unwrap());
                h.insert(
                    "Access-Control-Allow-Methods",
                    "GET, POST, PUT, DELETE, PATCH, OPTIONS".parse().unwrap(),
                );
                h.insert("Access-Control-Allow-Headers", "*".parse().unwrap());
                h.insert("Access-Control-Max-Age", "86400".parse().unwrap());
                return Ok(res);
            }

            let mut res = next.run(ctx).await;

            if let Ok(ref mut response) = res {
                response
                    .headers_mut()
                    .insert("Access-Control-Allow-Origin", "*".parse().unwrap());
            }

            res
        })
    }
}
