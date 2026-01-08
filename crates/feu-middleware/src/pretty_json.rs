use feu_core::ctx::Ctx;
use feu_core::middleware::{BoxFuture, Middleware, Next};
use feu_core::types::FeuBody;
use feu_core::types::FeuResponse;
use http::header::CONTENT_TYPE;

#[cfg(feature = "json")]
#[derive(Clone)]
pub struct PrettyJson {
    query_param: String,
}

#[cfg(feature = "json")]
impl PrettyJson {
    pub fn new() -> Self {
        Self {
            query_param: "pretty".to_string(),
        }
    }
}

#[cfg(feature = "json")]
impl Default for PrettyJson {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(feature = "json")]
impl<E: Clone + Send + Sync + 'static> Middleware<E> for PrettyJson {
    fn handle(
        &self,
        ctx: Ctx<E>,
        next: Next<E>,
    ) -> BoxFuture<feu_core::error::Result<FeuResponse>> {
        // Check if query param present?
        // Basic check for ?pretty
        let should_pretty = ctx
            .req
            .uri()
            .query()
            .map(|q| q.contains(&self.query_param))
            .unwrap_or(false);

        Box::pin(async move {
            let mut res = next.run(ctx).await?;

            if should_pretty {
                let is_json = res
                    .headers()
                    .get(CONTENT_TYPE)
                    .and_then(|v| v.to_str().ok())
                    .map(|v| v.contains("application/json"))
                    .unwrap_or(false);

                if is_json {
                    let body = std::mem::replace(&mut *res.body_mut(), FeuBody::Empty);
                    let new_body = match body {
                        FeuBody::Bytes(b) => {
                            if let Ok(val) = serde_json::from_slice::<serde_json::Value>(&b) {
                                if let Ok(pretty) = serde_json::to_string_pretty(&val) {
                                    FeuBody::Text(pretty)
                                } else {
                                    FeuBody::Bytes(b)
                                }
                            } else {
                                FeuBody::Bytes(b)
                            }
                        }
                        FeuBody::Text(s) => {
                            if let Ok(val) = serde_json::from_str::<serde_json::Value>(&s) {
                                if let Ok(pretty) = serde_json::to_string_pretty(&val) {
                                    FeuBody::Text(pretty)
                                } else {
                                    FeuBody::Text(s)
                                }
                            } else {
                                FeuBody::Text(s)
                            }
                        }
                        b => b,
                    };
                    *res.body_mut() = new_body;
                }
            }

            Ok(res)
        })
    }
}
