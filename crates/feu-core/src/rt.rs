use crate::middleware::BoxFuture;

/// Abstract interface for usage of runtime-specific features.
pub trait RuntimeCtx: Send + Sync + 'static {
    /// Registers a background task to be awaited after the response is sent.
    /// (e.g., Cloudflare `ctx.waitUntil`, AWS Lambda extension).
    fn wait_until(&self, fut: BoxFuture<()>);
}

/// No-op implementation for testing or simple runtimes.
pub struct NoOpRuntimeCtx;

impl RuntimeCtx for NoOpRuntimeCtx {
    fn wait_until(&self, _fut: BoxFuture<()>) {
        // Do nothing or spawn?
        // In local/std environment, likely we want to spawn.
        // But "NoOp" usually implies ignore or sync?
        // Actually, preventing standard tokio spawn isn't the goal.
        // This is specifically for platform hooks.
        // Let's just drop it for "NoOp".
    }
}
