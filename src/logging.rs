use tracing_subscriber::{
    EnvFilter,
    fmt::{self, time::UtcTime},
    layer::SubscriberExt,
    util::SubscriberInitExt,
};

use crate::config::{LogFormat, LoggingConfig};

pub fn init(config: &LoggingConfig) {
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| {
        let mut filter = EnvFilter::new(&config.level);
        for directive in &config.filters {
            filter = filter.add_directive(directive.parse().expect("valid log filter directive"));
        }
        filter
    });

    let registry = tracing_subscriber::registry().with(filter);

    macro_rules! configure_layer {
        ($layer:expr) => {
            $layer
                .with_timer(UtcTime::rfc_3339())
                .with_target(true)
                .with_thread_ids(false)
                .with_file(false)
                .with_line_number(false)
        };
    }

    match config.format {
        LogFormat::Json => {
            registry.with(configure_layer!(fmt::layer().json())).init();
        }
        LogFormat::Text => {
            registry.with(configure_layer!(fmt::layer())).init();
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
