use once_cell::sync::Lazy;
use tracing::subscriber::set_global_default;
use tracing_bunyan_formatter::{BunyanFormattingLayer, JsonStorageLayer};
use tracing_subscriber::registry::Registry;
use tracing_subscriber::{layer::SubscriberExt, EnvFilter};

static TRACING: Lazy<()> = Lazy::new(setup_global_subscriber);

fn setup_global_subscriber() {
    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("debug"));
    let formatting_layer = BunyanFormattingLayer::new("dox".into(), std::io::stdout);
    let subscriber = Registry::default()
        .with(env_filter)
        .with(JsonStorageLayer)
        .with(formatting_layer);
    set_global_default(subscriber).expect("Failed to set subscriber");
}

pub fn init_tracing() {
    Lazy::force(&TRACING);
}
