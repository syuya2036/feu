// Modules are defined once here at the top
pub mod app;
pub mod ctx;
pub mod error;
pub mod extract;
pub mod handler;
pub mod response;
pub mod rt;
pub mod tracing;
pub mod types;

// Re-exports
pub use app::App;
pub use ctx::Ctx;
pub use error::{Error, ErrorKind, Result};
pub use response::IntoResponse;
pub use types::{FeuBody, FeuRequest, FeuResponse};

#[cfg(test)]
mod tests;
#[cfg(test)]
mod tests_extract;
#[cfg(test)]
mod tests_router;

pub mod middleware;
pub use middleware::{Middleware, Next};

#[cfg(test)]
mod tests_middleware;
