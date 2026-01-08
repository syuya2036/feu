use crate::ctx::Ctx;
use crate::error::Result;
use crate::handler::Handler;
use crate::types::{FeuRequest, FeuResponse};
use feu_router::{HandlerId, MethodRouter};
use http::{Method, StatusCode};
use std::sync::Arc;

pub struct App {
    router: MethodRouter,
    handlers: Vec<Arc<dyn Handler>>,
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}

impl App {
    pub fn new() -> Self {
        Self {
            router: MethodRouter::new(),
            handlers: Vec::new(),
        }
    }

    fn register<H>(&mut self, method: Method, path: &str, handler: H)
    where
        H: Handler,
    {
        let id = HandlerId(self.handlers.len());
        self.handlers.push(Arc::new(handler));
        if let Err(e) = self.router.insert(method, path, id) {
            // Panic on route conflict/error in registration phase is typical for Rust web frameworks
            panic!("Failed to register route: {}", e);
        }
    }

    pub fn get<H>(mut self, path: &str, handler: H) -> Self
    where
        H: Handler,
    {
        self.register(Method::GET, path, handler);
        self
    }

    pub fn post<H>(mut self, path: &str, handler: H) -> Self
    where
        H: Handler,
    {
        self.register(Method::POST, path, handler);
        self
    }

    pub fn put<H>(mut self, path: &str, handler: H) -> Self
    where
        H: Handler,
    {
        self.register(Method::PUT, path, handler);
        self
    }

    pub fn delete<H>(mut self, path: &str, handler: H) -> Self
    where
        H: Handler,
    {
        self.register(Method::DELETE, path, handler);
        self
    }

    pub fn patch<H>(mut self, path: &str, handler: H) -> Self
    where
        H: Handler,
    {
        self.register(Method::PATCH, path, handler);
        self
    }

    pub async fn handle(&self, req: FeuRequest) -> Result<FeuResponse> {
        let method = req.method();
        let path = req.uri().path();

        if let Some(match_) = self.router.recognize(method, path) {
            let handler = &self.handlers[match_.handler_id.0];
            let ctx = Ctx::new(req, match_.params);
            handler.call(ctx).await
        } else {
            Ok(FeuResponse::text("Not Found").with_status(StatusCode::NOT_FOUND))
        }
    }
}
