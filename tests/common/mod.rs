//! Common utilities for integration tests.
//!
//! These tests target ArcGIS Online (AGOL) and require credentials
//! set in a `.env` file at the repository root.

use std::sync::OnceLock;
use tracing::instrument;

#[cfg(any(
    feature = "test-public",
    feature = "test-location",
    feature = "test-portal",
    feature = "test-publishing"
))]
use arcgis::{ApiKeyAuth, ArcGISClient, Error, ErrorKind};

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

/// Get an optional API key from environment.
/// Some tests may use API key instead of OAuth.
#[instrument]
#[cfg(any(
    feature = "test-public",
    feature = "test-location",
    feature = "test-portal",
    feature = "test-publishing"
))]
pub fn api_key() -> Option<String> {
    load_env();
    std::env::var("ARCGIS_API_KEY").ok()
}

/// Create a test client with API key authentication.
///
/// # Errors
///
/// Returns an error if ARCGIS_API_KEY is not set in environment.
#[instrument]
#[cfg(any(
    feature = "test-public",
    feature = "test-location",
    feature = "test-portal",
    feature = "test-publishing"
))]
pub fn create_api_key_client() -> Result<ArcGISClient, Error> {
    let key = api_key().ok_or_else(|| {
        Error::from(ErrorKind::Validation(
            "ARCGIS_API_KEY not found in environment. Add to .env file".to_string(),
        ))
    })?;
    let auth = ApiKeyAuth::new(key);
    Ok(ArcGISClient::new(auth))
}

/// Public ArcGIS Online feature service for testing (read-only).
/// This is ESRI's World Cities sample service.
#[cfg(any(
    feature = "test-public",
    feature = "test-location",
    feature = "test-portal",
    feature = "test-publishing"
))]
pub const SAMPLE_FEATURE_SERVICE: &str =
    "https://services.arcgis.com/P3ePLMYs2RVChkJx/arcgis/rest/services/World_Cities/FeatureServer";

/// Rate limiting helper to be polite to the API.
/// Sleeps for a short duration between requests.
#[instrument]
#[cfg(any(
    feature = "test-public",
    feature = "test-location",
    feature = "test-portal",
    feature = "test-publishing"
))]
pub async fn rate_limit() {
    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
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
