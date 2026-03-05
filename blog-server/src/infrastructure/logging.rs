use tracing_subscriber::{EnvFilter, fmt};

pub fn init_logging() {
    let filter = EnvFilter::try_from_default_env()
        .or_else(|_| EnvFilter::try_new("info"))
        .unwrap();

    let subscriber = fmt()
        .with_env_filter(filter)
        .with_target(false)
        .with_level(true)
        .with_timer(fmt::time::UtcTime::rfc_3339())
        .json()
        .finish();

    let _ = tracing::subscriber::set_global_default(subscriber);
}
