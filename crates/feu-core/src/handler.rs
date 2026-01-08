use crate::ctx::Ctx;
use crate::error::Result;
use crate::response::IntoResponse;
use crate::types::FeuResponse;
use std::future::Future;
use std::pin::Pin;

pub type BoxFuture<T> = Pin<Box<dyn Future<Output = T> + Send>>;

pub trait Handler<E = ()>: Send + Sync + 'static {
    fn call(&self, ctx: Ctx<E>) -> BoxFuture<Result<FeuResponse>>;
}

impl<F, Fut, Res, E> Handler<E> for F
where
    F: Fn(Ctx<E>) -> Fut + Send + Sync + 'static,
    Fut: Future<Output = Result<Res>> + Send + 'static,
    Res: IntoResponse,
    E: Send + Sync + 'static,
{
    fn call(&self, ctx: Ctx<E>) -> BoxFuture<Result<FeuResponse>> {
        let fut = (self)(ctx);
        Box::pin(async move {
            let res = fut.await?;
            Ok(res.into_response())
        })
    }
}
