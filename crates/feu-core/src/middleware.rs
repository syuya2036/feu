use crate::ctx::Ctx;
use crate::error::Result;
use crate::response::IntoResponse;
use crate::types::FeuResponse;
use std::future::Future;
use std::pin::Pin;

pub type BoxFuture<T> = Pin<Box<dyn Future<Output = T> + Send>>;

/// Represents the "remainder" of the middleware pipeline.
/// Invoking `run(ctx)` executes the next middleware (or the final handler).
pub struct Next {
    pub(crate) endpoint: Box<dyn FnOnce(Ctx) -> BoxFuture<Result<FeuResponse>> + Send>,
}

impl Next {
    pub async fn run(self, ctx: Ctx) -> Result<FeuResponse> {
        (self.endpoint)(ctx).await
    }
}

/// A middleware is an async function that takes (Ctx, Next) and returns a Result<FeuResponse>.
pub trait Middleware: Send + Sync + 'static {
    fn handle(&self, ctx: Ctx, next: Next) -> BoxFuture<Result<FeuResponse>>;
}

impl<F, Fut, Res> Middleware for F
where
    F: Fn(Ctx, Next) -> Fut + Send + Sync + 'static,
    Fut: Future<Output = Result<Res>> + Send + 'static,
    Res: IntoResponse,
{
    fn handle(&self, ctx: Ctx, next: Next) -> BoxFuture<Result<FeuResponse>> {
        let fut = (self)(ctx, next);
        Box::pin(async move {
            let res = fut.await?;
            Ok(res.into_response())
        })
    }
}
