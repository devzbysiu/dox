use once_cell::sync::Lazy;
use tracing::subscriber::set_global_default;
use tracing_forest::ForestLayer;
use tracing_subscriber::{layer::SubscriberExt, EnvFilter, Registry};

static TRACING: Lazy<()> = Lazy::new(setup_global_subscriber);

fn setup_global_subscriber() {
    let env_filter = EnvFilter::from_default_env();
    let subscriber = Registry::default()
        .with(env_filter)
        .with(ForestLayer::default());
    set_global_default(subscriber).expect("failed to set flobal default");
}

pub fn init_tracing() {
    Lazy::force(&TRACING);
}
