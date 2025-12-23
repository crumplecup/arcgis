//! Common utilities for integration tests.
//!
//! These tests target ArcGIS Online (AGOL) and require credentials
//! set in a `.env` file at the repository root.

use std::sync::OnceLock;

#[cfg(feature = "api")]
use arcgis::{ApiKeyAuth, ArcGISClient, Error, ErrorKind};

/// Load environment variables from .env file.
/// Only loads once, subsequent calls are no-ops.
pub fn load_env() {
    static INIT: OnceLock<()> = OnceLock::new();
    INIT.get_or_init(|| {
        dotenvy::dotenv().ok();
    });
}

/// Get an optional API key from environment.
/// Some tests may use API key instead of OAuth.
///
/// Available with the `api` feature.
#[cfg(feature = "api")]
pub fn api_key() -> Option<String> {
    load_env();
    std::env::var("ARCGIS_API_KEY").ok()
}

// TODO: Add OAuth helper functions when implementing Phase 2
// pub fn client_id() -> String { ... }
// pub fn client_secret() -> String { ... }

/// Create a test client with API key authentication.
///
/// # Errors
///
/// Returns an error if ARCGIS_API_KEY is not set in environment.
///
/// Available with the `api` feature.
#[cfg(feature = "api")]
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
///
/// Available with the `api` feature.
#[cfg(feature = "api")]
pub const SAMPLE_FEATURE_SERVICE: &str =
    "https://services.arcgis.com/P3ePLMYs2RVChkJx/arcgis/rest/services/World_Cities/FeatureServer";

/// Rate limiting helper to be polite to the API.
/// Sleeps for a short duration between requests.
///
/// Available with the `api` feature.
#[cfg(feature = "api")]
pub async fn rate_limit() {
    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_env_loading() {
        load_env();
        // Just verify it doesn't panic - credentials are optional for basic tests
    }
}
