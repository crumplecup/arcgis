//! Common utilities for integration tests.
//!
//! These tests target ArcGIS Online (AGOL) and require credentials
//! set in a `.env` file at the repository root.

use arcgis::{ArcGISClient, auth::ApiKeyAuth};
use std::sync::OnceLock;

/// Load environment variables from .env file.
/// Only loads once, subsequent calls are no-ops.
pub fn load_env() {
    static INIT: OnceLock<()> = OnceLock::new();
    INIT.get_or_init(|| {
        dotenvy::dotenv().ok();
    });
}

/// Get CLIENT_ID from environment.
pub fn client_id() -> String {
    load_env();
    std::env::var("CLIENT_ID")
        .expect("CLIENT_ID not found in environment. Add to .env file")
}

/// Get CLIENT_SECRET from environment.
pub fn client_secret() -> String {
    load_env();
    std::env::var("CLIENT_SECRET")
        .expect("CLIENT_SECRET not found in environment. Add to .env file")
}

/// Get an optional API key from environment.
/// Some tests may use API key instead of OAuth.
pub fn api_key() -> Option<String> {
    load_env();
    std::env::var("ARCGIS_API_KEY").ok()
}

/// Create a test client with API key authentication.
///
/// # Panics
///
/// Panics if ARCGIS_API_KEY is not set in environment.
pub fn create_api_key_client() -> ArcGISClient {
    let key = api_key().expect("ARCGIS_API_KEY not found in environment. Add to .env file");
    let auth = ApiKeyAuth::new(key);
    ArcGISClient::new(auth)
}

/// Public ArcGIS Online feature service for testing (read-only).
/// This is ESRI's World Cities sample service.
pub const SAMPLE_FEATURE_SERVICE: &str =
    "https://services.arcgis.com/P3ePLMYs2RVChkJx/arcgis/rest/services/World_Cities/FeatureServer";

/// Rate limiting helper to be polite to the API.
/// Sleeps for a short duration between requests.
pub async fn rate_limit() {
    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_env_loading() {
        load_env();
        // Just verify it doesn't panic
        assert!(std::env::var("CLIENT_ID").is_ok() || std::env::var("ARCGIS_API_KEY").is_ok(),
                "Either CLIENT_ID or ARCGIS_API_KEY should be set");
    }
}
