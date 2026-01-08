use crate::ctx::Ctx;
use crate::error::Result;
use crate::response::IntoResponse;
use crate::types::FeuResponse;
use std::future::Future;
use std::pin::Pin;

pub type BoxFuture<T> = Pin<Box<dyn Future<Output = T> + Send>>;

pub trait Handler: Send + Sync + 'static {
    fn call(&self, ctx: Ctx) -> BoxFuture<Result<FeuResponse>>;
}

impl<F, Fut, Res> Handler for F
where
    F: Fn(Ctx) -> Fut + Send + Sync + 'static,
    Fut: Future<Output = Result<Res>> + Send + 'static,
    Res: IntoResponse,
{
    fn call(&self, ctx: Ctx) -> BoxFuture<Result<FeuResponse>> {
        let fut = (self)(ctx);
        Box::pin(async move {
            let res = fut.await?;
            Ok(res.into_response())
        })
    }
}
