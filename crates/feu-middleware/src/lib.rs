use feu_core::ctx::Ctx;
use feu_core::middleware::{BoxFuture, Middleware, Next};
use feu_core::types::FeuResponse;

pub mod logger {
    use super::*;
    pub fn default() -> impl Middleware {
        struct Logger;
        impl Middleware for Logger {
            fn handle(
                &self,
                ctx: Ctx,
                next: Next,
            ) -> BoxFuture<feu_core::error::Result<FeuResponse>> {
                Box::pin(next.run(ctx))
            }
        }
        Logger
    }
}

pub mod cors {
    use super::*;
    pub struct Cors;
    impl Cors {
        pub fn permissive() -> impl Middleware {
            struct C;
            impl Middleware for C {
                fn handle(
                    &self,
                    ctx: Ctx,
                    next: Next,
                ) -> BoxFuture<feu_core::error::Result<FeuResponse>> {
                    Box::pin(next.run(ctx))
                }
            }
            C
        }
    }
}
