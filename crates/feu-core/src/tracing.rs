use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

/// Initialize a default tracing subscriber with EnvFilter and Fmt layer.
/// This logs to stdout by default.
/// The default log level is `debug` if `RUST_LOG` is not set.
pub fn init_tracing() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();
}
