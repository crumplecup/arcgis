//! Common utilities for integration tests.
//!
//! These tests target ArcGIS Online (AGOL) and require credentials
//! set in a `.env` file at the repository root.

use config::{Config, File};
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

/// Get the API key for the current test tier.
///
/// Reads the appropriate environment variable based on the active test feature flag:
/// - `test-public` → `ARCGIS_PUBLIC_KEY`
/// - `test-location` → `ARCGIS_LOCATION_KEY`
/// - `test-content` → `ARCGIS_CONTENT_KEY`
/// - `test-features` → `ARCGIS_FEATURES_KEY`
/// - (no feature) → `ARCGIS_API_KEY` (default)
///
/// The mapping is defined in `config/test-tiers.toml`.
///
/// # Errors
///
/// Returns an error if:
/// - The tier configuration cannot be loaded
/// - The environment variable for the tier is not set
///
/// # Example
///
/// ```no_run
/// #[cfg(feature = "test-location")]
/// #[tokio::test]
/// async fn test_geocoding() -> anyhow::Result<()> {
///     let key = common::api_key()?;  // Reads ARCGIS_LOCATION_KEY
///     let client = ArcGISClient::new(ApiKeyAuth::new(key));
///     // ... test code
///     Ok(())
/// }
/// ```
#[instrument]
pub fn api_key() -> anyhow::Result<String> {
    use anyhow::Context;

    load_env();

    // Load tier configuration
    let settings = Config::builder()
        .add_source(File::with_name("config/test-tiers"))
        .build()
        .context("Failed to load config/test-tiers.toml")?;

    // Determine active tier from feature flags
    let tier = active_tier();
    tracing::debug!(tier = %tier, "Detected active test tier");

    // Get environment variable name for this tier
    let env_var_key = format!("tiers.{}.env_var", tier);
    let env_var_name = settings
        .get_string(&env_var_key)
        .with_context(|| format!("No env_var configured for tier: {}", tier))?;

    tracing::debug!(
        env_var = %env_var_name,
        tier = %tier,
        "Looking up API key from environment"
    );

    // Read the API key from environment
    std::env::var(&env_var_name).with_context(|| {
        format!(
            "Environment variable {} not found (required for tier: {}). \
             Add to .env file or set in environment.",
            env_var_name, tier
        )
    })
}

/// Determine the active test tier from compile-time feature flags.
fn active_tier() -> &'static str {
    if cfg!(feature = "test-public") {
        "public"
    } else if cfg!(feature = "test-location") {
        "location"
    } else if cfg!(feature = "test-content") {
        "content"
    } else if cfg!(feature = "test-features") {
        "features"
    } else {
        "public" // Default tier for unit tests and examples
    }
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

    #[test]
    fn test_api_key_helper() {
        init_tracing();

        // Test that api_key() helper works (may return error if env var not set)
        let result = api_key();

        // We don't assert success because keys may not be configured
        // Just verify the function executes without panicking
        tracing::debug!(
            has_key = result.is_ok(),
            tier = active_tier(),
            "API key lookup completed"
        );
    }
}
