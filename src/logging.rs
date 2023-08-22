use tracing_subscriber::{filter, prelude::*};

pub fn init_logging() {
    let stdout_log = tracing_subscriber::fmt::layer();
    
    tracing_subscriber::Registry::default()
        .with(stdout_log.with_filter(filter::LevelFilter::DEBUG)
            .with_filter(filter::filter_fn(|metadata| {
            metadata.target().starts_with("llm_router")
        }))).init();
}