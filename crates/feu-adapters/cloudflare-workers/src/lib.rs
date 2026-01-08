use feu_core::app::App;
use feu_core::middleware::BoxFuture;
use feu_core::rt::RuntimeCtx;
use feu_core::types::FeuBody;
use std::sync::Arc;
use worker::{Context, Request as WorkerRequest, Response as WorkerResponse};

pub struct CloudflareRuntime {
    ctx: Context,
}

impl CloudflareRuntime {
    pub fn new(ctx: Context) -> Self {
        Self { ctx }
    }
}

// Implement RuntimeCtx for Cloudflare context
impl RuntimeCtx for CloudflareRuntime {
    fn wait_until(&self, fut: BoxFuture<()>) {
        self.ctx.wait_until(fut);
    }
}

pub async fn run<E>(
    app: &App<E>,
    req: WorkerRequest,
    env: E,
    ctx: Context,
) -> worker::Result<WorkerResponse>
where
    E: Clone + Send + Sync + 'static,
{
    // worker::Request doesn't impl into_parts.
    // We access fields directly.
    let method = req.method();
    let uri = req
        .url()
        .map_err(|e| worker::Error::RustError(e.to_string()))?;

    // We construct http::Request
    let mut builder = http::Request::builder()
        .method(http::Method::from_bytes(method.to_string().as_bytes()).unwrap())
        .uri(uri.as_str());

    for (k, v) in req.headers() {
        if let (Ok(val), Ok(name)) = (
            http::HeaderValue::from_str(&v),
            http::HeaderName::from_bytes(k.as_bytes()),
        ) {
            builder = builder.header(name, val);
        }
    }

    // Body?
    let feu_body = FeuBody::Empty;

    let feu_req = builder
        .body(feu_body)
        .map_err(|e| worker::Error::RustError(e.to_string()))?;

    // Wrap Context
    let runtime = Arc::new(CloudflareRuntime::new(ctx));

    // RUN
    let res = app
        .handle_with_runtime(feu_req, env, runtime)
        .await
        .map_err(|e| worker::Error::RustError(e.to_string()))?;

    // Convert FeuResponse to WorkerResponse
    let (parts, body) = res.0.into_parts();

    // Convert body
    let worker_body = match body {
        FeuBody::Text(s) => worker::ResponseBody::Body(s.into_bytes()),
        FeuBody::Empty => worker::ResponseBody::Empty,
        FeuBody::Bytes(b) => worker::ResponseBody::Body(b.to_vec()),
        // Handle Stream or future variants
        _ => {
            return Err(worker::Error::RustError(
                "Body type not supported in this adapter (e.g. Stream)".into(),
            ));
        }
    };

    let w_res = WorkerResponse::from_body(worker_body)?;
    let mut w_res = w_res.with_status(parts.status.as_u16());

    let h = w_res.headers_mut();
    for (k, v) in parts.headers.iter() {
        h.set(k.as_str(), v.to_str().unwrap())
            .map_err(|e| worker::Error::RustError(e.to_string()))?;
    }

    Ok(w_res)
}
