//! Basic integration tests for ArcGIS SDK.
//!
//! These tests require credentials in a `.env` file.
//! They are marked with `#[ignore]` by default to avoid hammering the API.
//! Run with: `cargo test --test integration_basic -- --ignored`

mod common;

#[tokio::test]
#[ignore = "Requires API key and hits live API"]
async fn test_client_creation_with_api_key() {
    let _client = common::create_api_key_client();
    // Client creation should succeed without panicking
    // Actual API calls will be tested in more specific tests
}

#[test]
fn test_credentials_available() {
    common::load_env();

    // Either API key OR OAuth credentials should be available
    let has_api_key = common::api_key().is_some();
    let has_oauth = std::env::var("CLIENT_ID").is_ok() &&
                    std::env::var("CLIENT_SECRET").is_ok();

    assert!(has_api_key || has_oauth,
            "Either ARCGIS_API_KEY or (CLIENT_ID + CLIENT_SECRET) must be set in .env");
}

#[tokio::test]
#[ignore = "Hits live AGOL API - run sparingly"]
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

    assert!(response.status().is_success(),
            "Sample feature service should be accessible");

    let json: serde_json::Value = response.json().await
        .expect("Response should be valid JSON");

    // Verify it's a feature service response
    assert!(json.get("layers").is_some() || json.get("tables").is_some(),
            "Response should contain layers or tables");
}

// TODO: Add more integration tests as we implement features:
// - test_feature_query_with_auth
// - test_feature_edit_operations
// - test_oauth_flow
// - test_rate_limiting
