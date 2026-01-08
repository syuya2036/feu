use crate::ctx::Ctx;
use crate::error::Result;
use crate::response::IntoResponse;
use crate::types::FeuResponse;
use std::future::Future;
use std::pin::Pin;

pub type BoxFuture<T> = Pin<Box<dyn Future<Output = T> + Send>>;

/// Represents the "remainder" of the middleware pipeline.
/// Invoking `run(ctx)` executes the next middleware (or the final handler).
pub struct Next<E = ()> {
    pub(crate) endpoint: Box<dyn FnOnce(Ctx<E>) -> BoxFuture<Result<FeuResponse>> + Send>,
}

impl<E> Next<E> {
    pub async fn run(self, ctx: Ctx<E>) -> Result<FeuResponse> {
        (self.endpoint)(ctx).await
    }
}

/// A middleware is an async function that takes (Ctx, Next) and returns a Result<FeuResponse>.
pub trait Middleware<E = ()>: Send + Sync + 'static {
    fn handle(&self, ctx: Ctx<E>, next: Next<E>) -> BoxFuture<Result<FeuResponse>>;
}

impl<F, Fut, Res, E> Middleware<E> for F
where
    F: Fn(Ctx<E>, Next<E>) -> Fut + Send + Sync + 'static,
    Fut: Future<Output = Result<Res>> + Send + 'static,
    Res: IntoResponse,
    E: Send + Sync + 'static,
{
    fn handle(&self, ctx: Ctx<E>, next: Next<E>) -> BoxFuture<Result<FeuResponse>> {
        let fut = (self)(ctx, next);
        Box::pin(async move {
            let res = fut.await?;
            Ok(res.into_response())
        })
    }
}
