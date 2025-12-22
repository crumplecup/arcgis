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

#[tokio::test]
#[cfg(feature = "api")]
async fn test_feature_query_with_where_clause() {
    use arcgis::{FeatureQueryParams, FeatureServiceClient, LayerId};

    let client = common::create_api_key_client();

    // Use ESRI's World Cities sample service (layer 0)
    let feature_service = FeatureServiceClient::new(common::SAMPLE_FEATURE_SERVICE, &client);

    common::rate_limit().await;

    // Query for cities with population > 5 million
    let params = FeatureQueryParams::builder()
        .where_clause("POP > 5000000")
        .out_fields(vec!["CITY_NAME".to_string(), "POP".to_string()])
        .return_geometry(true)
        .result_record_count(10u32)
        .build()
        .expect("Failed to build query params");

    let result = feature_service
        .query(LayerId::new(0), params)
        .await
        .expect("Feature query failed");

    // Verify we got results
    assert!(
        !result.features.is_empty(),
        "Should have found cities with population > 5 million"
    );

    // Verify features have attributes
    let first_feature = &result.features[0];
    assert!(
        first_feature.attributes.contains_key("CITY_NAME"),
        "Feature should have CITY_NAME attribute"
    );
    assert!(
        first_feature.attributes.contains_key("POP"),
        "Feature should have POP attribute"
    );

    // Verify geometry is present
    assert!(
        first_feature.geometry.is_some(),
        "Feature should have geometry"
    );
}

#[tokio::test]
#[cfg(feature = "api")]
async fn test_feature_query_count_only() {
    use arcgis::{FeatureQueryParams, FeatureServiceClient, LayerId};

    let client = common::create_api_key_client();
    let feature_service = FeatureServiceClient::new(common::SAMPLE_FEATURE_SERVICE, &client);

    common::rate_limit().await;

    // Query for count of all features
    let params = FeatureQueryParams::builder()
        .where_clause("1=1")
        .return_count_only(true)
        .build()
        .expect("Failed to build query params");

    let _result = feature_service
        .query(LayerId::new(0), params)
        .await
        .expect("Feature count query failed");

    // When returnCountOnly is true, the query should succeed
    // The features array may or may not be empty depending on the API response format
    // The important thing is that the query didn't fail
}

#[tokio::test]
#[cfg(feature = "api")]
async fn test_feature_query_with_object_ids() {
    use arcgis::{FeatureQueryParams, FeatureServiceClient, LayerId, ObjectId};

    let client = common::create_api_key_client();
    let feature_service = FeatureServiceClient::new(common::SAMPLE_FEATURE_SERVICE, &client);

    common::rate_limit().await;

    // Query specific features by object ID
    let params = FeatureQueryParams::builder()
        .object_ids(vec![ObjectId::new(1), ObjectId::new(2)])
        .out_fields(vec!["*".to_string()])
        .return_geometry(false)
        .build()
        .expect("Failed to build query params");

    let result = feature_service
        .query(LayerId::new(0), params)
        .await
        .expect("Feature query by object IDs failed");

    // May or may not return features depending on if those IDs exist
    // Just verify the query succeeded without error
    assert!(
        result.features.len() <= 2,
        "Should return at most 2 features"
    );
}

// TODO: Add more integration tests as we implement features:
// - test_feature_edit_operations
// - test_oauth_flow
// - test_rate_limiting
// - test_spatial_queries
