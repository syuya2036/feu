use feu_core::ctx::Ctx;
use feu_core::middleware::{BoxFuture, Middleware, Next};
use feu_core::types::{FeuBody, FeuResponse};
#[cfg(feature = "streaming")]
use futures_util::stream::StreamExt;
use http::StatusCode;

#[derive(Clone, Copy)]
pub struct BodyLimit {
    limit: u64,
}

impl BodyLimit {
    pub fn new(limit: u64) -> Self {
        Self { limit }
    }
}

impl Middleware for BodyLimit {
    fn handle(&self, mut ctx: Ctx, next: Next) -> BoxFuture<feu_core::error::Result<FeuResponse>> {
        let limit = self.limit;

        // 1. Check Content-Length if present
        if let Some(cl_val) = ctx.req.headers().get(http::header::CONTENT_LENGTH) {
            if let Ok(cl_str) = cl_val.to_str() {
                if let Ok(cl) = cl_str.parse::<u64>() {
                    if cl > limit {
                        return Box::pin(async move {
                            Ok(FeuResponse::text("Payload Too Large")
                                .with_status(StatusCode::PAYLOAD_TOO_LARGE))
                        });
                    }
                }
            }
        }

        // 2. Wrap body
        // We need to take body out of request, wrap it, put back.
        // req.body_mut() gives &mut FeuBody.
        let body = std::mem::replace(ctx.req.body_mut(), FeuBody::Empty);

        let new_body = match body {
            FeuBody::Empty => FeuBody::Empty,
            FeuBody::Bytes(b) => {
                if (b.len() as u64) > limit {
                    return Box::pin(async move {
                        Ok(FeuResponse::text("Payload Too Large")
                            .with_status(StatusCode::PAYLOAD_TOO_LARGE))
                    });
                }
                FeuBody::Bytes(b)
            }
            FeuBody::Text(s) => {
                if (s.len() as u64) > limit {
                    return Box::pin(async move {
                        Ok(FeuResponse::text("Payload Too Large")
                            .with_status(StatusCode::PAYLOAD_TOO_LARGE))
                    });
                }
                FeuBody::Text(s)
            }
            #[cfg(feature = "streaming")]
            FeuBody::Stream(s) => {
                // Wrap stream
                let s = s.scan(0u64, move |count, item| {
                    let res = match item {
                        Ok(bytes) => {
                            *count += bytes.len() as u64;
                            if *count > limit {
                                Some(Err(feu_core::error::Error::msg("Payload Too Large")))
                            } else {
                                Some(Ok(bytes))
                            }
                        }
                        Err(e) => Some(Err(e)),
                    };
                    futures_util::future::ready(res)
                });
                FeuBody::Stream(Box::pin(s))
            }
        };

        *ctx.req.body_mut() = new_body;

        Box::pin(async move { next.run(ctx).await })
    }
}
