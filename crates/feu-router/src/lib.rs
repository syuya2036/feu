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
