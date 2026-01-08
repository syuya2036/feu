use crate::ctx::Ctx;
use crate::error::Result;
use crate::handler::Handler;
use crate::types::{FeuRequest, FeuResponse};
use feu_router::{HandlerId, MethodRouter};
use http::{Method, StatusCode};
use std::sync::Arc;

use crate::middleware::{BoxFuture, Middleware, Next};

pub struct App {
    pub(crate) router: Arc<MethodRouter>,
    pub(crate) handlers: Arc<Vec<Arc<dyn Handler>>>, // Global handler registry, index is handlerID
    // For composition: store definitions so we can merge them.
    // Actually, if we merge, we have to register handlers into `self.handlers` producing NEW IDs,
    // and then insert into `self.router`.
    // So we just need the list of (Method, Path, Handler).
    // Type alias for route matching info: Method, Path, Handler
    #[allow(clippy::type_complexity)]
    routes: Arc<Vec<(Method, String, Arc<dyn Handler>)>>,
    middlewares: Arc<Vec<Arc<dyn Middleware>>>,
    base_path: String,
    not_found_handler: Arc<Option<Arc<dyn Handler>>>,
    error_handler: Arc<Option<Arc<dyn Handler>>>,
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}

struct PathMiddleware<M> {
    prefix: String,
    inner: M,
}

impl<M: Middleware> Middleware for PathMiddleware<M> {
    fn handle(&self, ctx: Ctx, next: Next) -> BoxFuture<Result<FeuResponse>> {
        let path = ctx.req.uri().path().to_string();
        if path.starts_with(&self.prefix) {
            self.inner.handle(ctx, next)
        } else {
            Box::pin(next.run(ctx))
        }
    }
}

impl App {
    pub fn new() -> Self {
        Self {
            router: Arc::new(MethodRouter::new()),
            handlers: Arc::new(Vec::new()),
            routes: Arc::new(Vec::new()),
            middlewares: Arc::new(Vec::new()),
            base_path: String::new(),
            not_found_handler: Arc::new(None),
            error_handler: Arc::new(None),
        }
    }

    pub fn base_path(mut self, path: &str) -> Self {
        self.base_path = path.trim_end_matches('/').to_string();
        self
    }

    pub fn use_mw<M>(mut self, middleware: M) -> Self
    where
        M: Middleware,
    {
        Arc::make_mut(&mut self.middlewares).push(Arc::new(middleware));
        self
    }

    /// Registers a middleware that only runs if the path starts with `prefix`.
    pub fn use_at<M>(mut self, prefix: &str, middleware: M) -> Self
    where
        M: Middleware,
    {
        let mw = PathMiddleware {
            prefix: prefix.to_string(),
            inner: middleware,
        };
        self.use_mw(mw)
    }

    fn register_arc(&mut self, method: Method, path: &str, handler: Arc<dyn Handler>) {
        let handlers = Arc::make_mut(&mut self.handlers);
        let id = HandlerId(handlers.len());
        handlers.push(handler.clone());

        let full_path = if self.base_path.is_empty() {
            path.to_string()
        } else {
            let p = if path == "/" { "" } else { path };
            format!("{}{}", self.base_path, p)
        };
        let full_path = if full_path.starts_with('/') {
            full_path
        } else {
            format!("/{}", full_path)
        };

        Arc::make_mut(&mut self.routes).push((method.clone(), full_path.clone(), handler));

        if let Err(e) = Arc::make_mut(&mut self.router).insert(method, &full_path, id) {
            panic!("Failed to register route: {}", e);
        }
    }

    pub fn get<H>(mut self, path: &str, handler: H) -> Self
    where
        H: Handler,
    {
        let h = Arc::new(handler);
        self.register_arc(Method::GET, path, h);
        self
    }

    pub fn post<H>(mut self, path: &str, handler: H) -> Self
    where
        H: Handler,
    {
        let h = Arc::new(handler);
        self.register_arc(Method::POST, path, h);
        self
    }

    pub fn put<H>(mut self, path: &str, handler: H) -> Self
    where
        H: Handler,
    {
        let h = Arc::new(handler);
        self.register_arc(Method::PUT, path, h);
        self
    }

    pub fn delete<H>(mut self, path: &str, handler: H) -> Self
    where
        H: Handler,
    {
        let h = Arc::new(handler);
        self.register_arc(Method::DELETE, path, h);
        self
    }

    pub fn patch<H>(mut self, path: &str, handler: H) -> Self
    where
        H: Handler,
    {
        let h = Arc::new(handler);
        self.register_arc(Method::PATCH, path, h);
        self
    }

    pub fn head<H>(mut self, path: &str, handler: H) -> Self
    where
        H: Handler,
    {
        let h = Arc::new(handler);
        self.register_arc(Method::HEAD, path, h);
        self
    }

    pub fn options<H>(mut self, path: &str, handler: H) -> Self
    where
        H: Handler,
    {
        let h = Arc::new(handler);
        self.register_arc(Method::OPTIONS, path, h);
        self
    }

    pub fn any<H>(mut self, path: &str, handler: H) -> Self
    where
        H: Handler,
    {
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

        // Iterate sub_app.routes (which is Arc<Vec>)
        for (method, sub_path, handler) in sub_app.routes.iter() {
            let new_path = if sub_path == "/" {
                prefix.to_string()
            } else {
                format!("{}{}", prefix, sub_path)
            };
            let new_path = if new_path.is_empty() {
                "/".to_string()
            } else {
                new_path
            };

            self.register_arc(method.clone(), &new_path, handler.clone());
        }
        self
    }

    pub fn not_found<H>(mut self, handler: H) -> Self
    where
        H: Handler,
    {
        *Arc::make_mut(&mut self.not_found_handler) = Some(Arc::new(handler));
        self
    }

    pub fn on_error<H>(mut self, handler: H) -> Self
    where
        H: Handler,
    {
        *Arc::make_mut(&mut self.error_handler) = Some(Arc::new(handler));
        self
    }

    pub async fn handle(&self, req: FeuRequest) -> Result<FeuResponse> {
        let router = self.router.clone();
        let handlers = self.handlers.clone();
        let not_found = self.not_found_handler.clone();

        let router_dispatch = move |req: Ctx| {
            let router = router.clone();
            let handlers = handlers.clone();
            let not_found = not_found.clone();

            Box::pin(async move {
                let method = req.req.method();
                let path = req.req.uri().path();

                if let Some(match_) = router.recognize(method, path) {
                    let handler = &handlers[match_.handler_id.0];
                    let mut ctx = req;
                    ctx.params = match_.params;
                    handler.call(ctx).await
                } else if let Some(h) = not_found.as_ref() {
                    h.call(req).await
                } else {
                    Ok(FeuResponse::text("Not Found").with_status(StatusCode::NOT_FOUND))
                }
            }) as BoxFuture<Result<FeuResponse>>
        };

        let mut next = Next {
            endpoint: Box::new(router_dispatch),
        };

        for mw in self.middlewares.iter().rev() {
            let mw = mw.clone();
            let current_next = next;
            let endpoint = move |ctx: Ctx| mw.handle(ctx, current_next);
            next = Next {
                endpoint: Box::new(endpoint),
            };
        }

        let ctx = Ctx::new(req, vec![]);
        let res = next.run(ctx).await;

        // Error handling
        match res {
            Ok(r) => Ok(r),
            Err(e) => {
                if let Some(_h) = self.error_handler.as_ref() {
                    // TODO: Implement error handler logic
                    Err(e)
                } else {
                    Err(e)
                }
            }
        }
    }
}
