use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

pub fn init() {
    let filter =
        tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into());

    #[cfg(debug_assertions)]
    let fmt_layer = tracing_subscriber::fmt::layer().pretty();

    #[cfg(not(debug_assertions))]
    let fmt_layer = tracing_subscriber::fmt::layer().json();

    tracing_subscriber::registry()
        .with(filter)
        .with(fmt_layer)
        .init();
}
