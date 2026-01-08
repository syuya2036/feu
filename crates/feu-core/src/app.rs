use crate::ctx::Ctx;
use crate::error::Result;
use crate::handler::Handler;
use crate::types::{FeuRequest, FeuResponse};
use http::{Method, StatusCode};
use std::sync::Arc;

pub struct App {
    // Temporary naive vector routing for Phase 1
    routes: Vec<(Method, String, Arc<dyn Handler>)>,
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}

impl App {
    pub fn new() -> Self {
        Self { routes: Vec::new() }
    }

    pub fn get<H>(mut self, path: &str, handler: H) -> Self
    where
        H: Handler,
    {
        self.routes
            .push((Method::GET, path.to_string(), Arc::new(handler)));
        self
    }

    // Minimal entrypoint
    pub async fn handle(&self, req: FeuRequest) -> Result<FeuResponse> {
        let method = req.method().clone();
        let path = req.uri().path().to_string();

        // 1. Matched route?
        if let Some((_, _, handler)) = self
            .routes
            .iter()
            .find(|(m, p, _)| *m == method && p == &path)
        {
            let ctx = Ctx::new(req);
            handler.call(ctx).await
        } else {
            // 404
            Ok(FeuResponse::text("Not Found").with_status(StatusCode::NOT_FOUND))
        }
    }
}
