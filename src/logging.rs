//! Logging initialization and configuration.

use tracing_subscriber::{
    EnvFilter,
    fmt::{self, time::UtcTime},
    layer::SubscriberExt,
    util::SubscriberInitExt,
};

use crate::config::{LogFormat, LoggingConfig};

/// Initialize the logging system based on configuration.
pub fn init(config: &LoggingConfig) {
    // Build filter with noisy crates silenced
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| {
        EnvFilter::new(&config.level)
            // Silence verbose database driver logs
            .add_directive("tokio_postgres=warn".parse().expect("valid directive"))
            .add_directive("diesel=warn".parse().expect("valid directive"))
            .add_directive("diesel_async=warn".parse().expect("valid directive"))
            // Silence other noisy crates
            .add_directive("hyper=warn".parse().expect("valid directive"))
            .add_directive("hyper_util=warn".parse().expect("valid directive"))
            .add_directive("reqwest=warn".parse().expect("valid directive"))
            .add_directive("rustls=warn".parse().expect("valid directive"))
            .add_directive("h2=warn".parse().expect("valid directive"))
    });

    let registry = tracing_subscriber::registry().with(filter);

    match config.format {
        LogFormat::Json => {
            let layer = fmt::layer()
                .json()
                .with_timer(UtcTime::rfc_3339())
                .with_target(true)
                .with_thread_ids(false)
                .with_file(false)
                .with_line_number(false);
            registry.with(layer).init();
        }
        LogFormat::Text => {
            let layer = fmt::layer()
                .with_timer(UtcTime::rfc_3339())
                .with_target(true)
                .with_thread_ids(false)
                .with_file(false)
                .with_line_number(false);
            registry.with(layer).init();
        }
    }

    tracing::debug!(format = ?config.format, level = %config.level, "Logging initialized");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_log_format_default() {
        let config = LoggingConfig::default();
        assert_eq!(config.format, LogFormat::Text);
    }
}
