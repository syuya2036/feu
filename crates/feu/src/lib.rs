pub use feu_core::app::App;
pub use feu_core::ctx::Ctx;
pub use feu_core::error::{Error, Result};
pub use feu_core::rt;
pub use feu_core::types::{FeuBody, FeuRequest, FeuResponse};

// Re-export http types for convenience
pub use http::Method;
pub use http::StatusCode;

pub mod prelude {
    pub use super::{App, Ctx, FeuBody, FeuResponse, Method, StatusCode};
}

pub mod middleware {
    pub use feu_core::middleware::{Middleware, Next};
    pub use feu_middleware::*;
}

pub mod adapters {
    #[cfg(feature = "hyper")]
    pub use feu_adapter_hyper as hyper;

    #[cfg(feature = "lambda")]
    pub use feu_adapter_lambda as lambda;

    #[cfg(feature = "cloudflare-workers")]
    pub use feu_adapter_cloudflare_workers as cloudflare_workers;
}
