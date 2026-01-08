use http::Method;
use matchit::Router as MatchitRouter;
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct HandlerId(pub usize);

#[derive(Debug)]
pub struct RouteMatch {
    pub handler_id: HandlerId,
    pub params: Vec<(String, String)>,
}

#[derive(Clone)]
pub struct MethodRouter {
    routers: HashMap<Method, MatchitRouter<HandlerId>>,
    // Fallback? Not here, handled in App usually.
}

impl Default for MethodRouter {
    fn default() -> Self {
        Self::new()
    }
}

impl MethodRouter {
    pub fn new() -> Self {
        Self {
            routers: HashMap::new(),
        }
    }

    // Expose inner router for iteration/merging (simplified for Phase 2)
    // We can't easily iterate matchit::Router.
    // So we should probably store routes in a separate list if we want to support `route(prefix, app)` by merging.
    // Or we rely on `matchit` having a way to list routes? It doesn't seem to expose iteration easily.
    // Plan B: App stores a list of pending route specs `(Method, Path, Handler)` and builds the router lazily or incrementally?
    // Or we keep it simple: `route` takes a `prefix` and `sub_app`. We iterate `sub_app.routes`?
    // We need to capture routes in `App` or `MethodRouter` to support this.

    // Let's add `routes: Vec<(Method, String, HandlerId)>` to `MethodRouter` or `App` for introspection.
    pub fn insert(
        &mut self,
        method: Method,
        path: &str,
        handler_id: HandlerId,
    ) -> Result<(), String> {
        let router = self.routers.entry(method).or_default();
        router.insert(path, handler_id).map_err(|e| e.to_string())
    }

    pub fn recognize(&self, method: &Method, path: &str) -> Option<RouteMatch> {
        if let Some(router) = self.routers.get(method) {
            if let Ok(match_) = router.at(path) {
                let params = match_
                    .params
                    .iter()
                    .map(|(k, v)| (k.to_string(), v.to_string()))
                    .collect();
                return Some(RouteMatch {
                    handler_id: *match_.value,
                    params,
                });
            }
        }
        None
    }
}
