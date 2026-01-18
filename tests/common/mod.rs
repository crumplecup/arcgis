//! Common utilities for integration tests.
//!
//! These tests target ArcGIS Online (AGOL) and require credentials
//! set in a `.env` file at the repository root.

use std::sync::OnceLock;
use tracing::instrument;

/// Load environment variables from .env file.
/// Only loads once, subsequent calls are no-ops.
#[instrument]
pub fn load_env() {
    static INIT: OnceLock<()> = OnceLock::new();
    INIT.get_or_init(|| {
        dotenvy::dotenv().ok();
    });
}

/// Initialize tracing subscriber for tests.
///
/// Reads RUST_LOG from environment (via .env file), defaulting to "info" level.
/// Only initializes once, subsequent calls are no-ops.
///
/// Call this at the start of every test function to enable logging.
///
/// # Example
///
/// ```no_run
/// #[tokio::test]
/// async fn test_something() -> anyhow::Result<()> {
///     common::init_tracing();
///     tracing::info!("Starting test");
///     // ... test code
///     Ok(())
/// }
/// ```
#[instrument]
pub fn init_tracing() {
    use tracing_subscriber::EnvFilter;

    static INIT: OnceLock<()> = OnceLock::new();
    INIT.get_or_init(|| {
        load_env();

        let log_level = std::env::var("RUST_LOG").unwrap_or_else(|_| "info".to_string());

        let env_filter =
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(log_level));

        tracing_subscriber::fmt()
            .with_env_filter(env_filter)
            .with_test_writer()
            .with_target(true)
            .with_thread_ids(true)
            .with_line_number(true)
            .init();
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_env_loading() {
        init_tracing();
        load_env();
        // Just verify it doesn't panic - credentials are optional for basic tests
    }
}
