//! Basic integration tests for ArcGIS SDK.
//!
//! These tests require credentials in a `.env` file and the `api` feature flag.
//! Run with: `cargo test --features api`

mod common;

#[tokio::test]
#[cfg(feature = "api")]
async fn test_client_creation_with_api_key() {
    let _client = common::create_api_key_client();
    // Client creation should succeed without panicking
    // Actual API calls will be tested in more specific tests
}

#[test]
#[cfg(feature = "api")]
fn test_credentials_available() {
    common::load_env();

    // API key should be available for testing
    // OAuth credentials will be checked when OAuth is implemented (Phase 2)
    let has_api_key = common::api_key().is_some();

    assert!(
        has_api_key,
        "ARCGIS_API_KEY must be set in .env for integration tests"
    );
}

#[tokio::test]
#[cfg(feature = "api")]
async fn test_public_feature_service_accessible() {
    // This test verifies we can reach a public AGOL service
    // without authentication (read-only)

    let client = reqwest::Client::new();
    let url = format!("{}?f=json", common::SAMPLE_FEATURE_SERVICE);

    common::rate_limit().await;

    let response = client
        .get(&url)
        .send()
        .await
        .expect("Failed to reach AGOL sample service");

    assert!(
        response.status().is_success(),
        "Sample feature service should be accessible"
    );

    let json: serde_json::Value = response
        .json()
        .await
        .expect("Response should be valid JSON");

    // Verify it's a feature service response
    assert!(
        json.get("layers").is_some() || json.get("tables").is_some(),
        "Response should contain layers or tables"
    );
}

// TODO: Add more integration tests as we implement features:
// - test_feature_query_with_auth
// - test_feature_edit_operations
// - test_oauth_flow
// - test_rate_limiting
