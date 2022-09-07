use once_cell::sync::Lazy;
use tracing::subscriber::set_global_default;
use tracing_forest::ForestLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter, Registry};

static TRACING: Lazy<()> = Lazy::new(setup_global_subscriber);

fn setup_global_subscriber() {
    Registry::default().with(ForestLayer::default()).init();
    let env_filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("dox=debug"));
    let subscriber = Registry::default()
        .with(env_filter)
        .with(ForestLayer::default());
    let _res = set_global_default(subscriber);
}

pub fn init_tracing() {
    Lazy::force(&TRACING);
}
