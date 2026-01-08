use crate::ctx::Ctx;
use crate::error::Result;
use crate::handler::Handler;
use crate::types::{FeuRequest, FeuResponse};
use feu_router::{HandlerId, MethodRouter};
use http::{Method, StatusCode};
use std::sync::Arc;

pub struct App {
    router: MethodRouter,
    handlers: Vec<Arc<dyn Handler>>, // Global handler registry, index is handlerID
    // For composition: store definitions so we can merge them.
    // Actually, if we merge, we have to register handlers into `self.handlers` producing NEW IDs,
    // and then insert into `self.router`.
    // So we just need the list of (Method, Path, Handler).
    routes: Vec<(Method, String, Arc<dyn Handler>)>,
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
            routes: Vec::new(),
        }
    }

    fn register<H>(&mut self, method: Method, path: &str, handler: H)
    where
        H: Handler,
    {
        let h = Arc::new(handler);
        self.register_arc(method, path, h)
    }

    fn register_arc(&mut self, method: Method, path: &str, handler: Arc<dyn Handler>) {
        let id = HandlerId(self.handlers.len());
        self.handlers.push(handler.clone());
        self.routes
            .push((method.clone(), path.to_string(), handler));

        if let Err(e) = self.router.insert(method, path, id) {
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

    pub fn head<H>(mut self, path: &str, handler: H) -> Self
    where
        H: Handler,
    {
        self.register(Method::HEAD, path, handler);
        self
    }

    pub fn options<H>(mut self, path: &str, handler: H) -> Self
    where
        H: Handler,
    {
        self.register(Method::OPTIONS, path, handler);
        self
    }

    pub fn any<H>(mut self, path: &str, handler: H) -> Self
    where
        H: Handler,
    {
        // Removed Clone requirement as it's wrapped in Arc
        // Register for all standard methods
        // Note: Handler must be Clone to register multiple times if we do it closely.
        // Or we wrap it in Arc first.
        let h = Arc::new(handler);
        for method in &[
            Method::GET,
            Method::POST,
            Method::PUT,
            Method::DELETE,
            Method::PATCH,
            Method::HEAD,
            Method::OPTIONS,
        ] {
            self.register_arc(method.clone(), path, h.clone());
        }
        self
    }

    pub fn on<H>(mut self, methods: Vec<Method>, path: &str, handler: H) -> Self
    where
        H: Handler,
    {
        let h = Arc::new(handler);
        for method in methods {
            self.register_arc(method, path, h.clone());
        }
        self
    }

    // Composition / Nesting
    pub fn route(mut self, path: &str, sub_app: App) -> Self {
        // Prefix logic: path must start with /?
        // Logic: for each route in sub_app, join prefix + route.path

        let prefix = path.trim_end_matches('/');

        for (method, sub_path, handler) in sub_app.routes {
            let new_path = if sub_path == "/" {
                prefix.to_string()
            } else {
                format!("{}{}", prefix, sub_path)
            };

            // Fix double slash issue if prefix is "/"
            let new_path = if new_path.is_empty() {
                "/".to_string()
            } else {
                new_path
            };

            self.register_arc(method, &new_path, handler);
        }
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
