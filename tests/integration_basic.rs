//! Basic integration tests for ArcGIS SDK.
//!
//! These tests require credentials in a `.env` file and the `api` feature flag.
//! Run with: `cargo test --features api`

mod common;

#[cfg(feature = "api")]
use arcgis::Result;

#[tokio::test]
#[cfg(feature = "api")]
async fn test_client_creation_with_api_key() -> Result<()> {
    let _client = common::create_api_key_client()?;
    // Client creation should succeed without panicking
    // Actual API calls will be tested in more specific tests
    Ok(())
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
async fn test_public_feature_service_accessible() -> Result<()> {
    // This test verifies we can reach a public AGOL service
    // without authentication (read-only)

    let client = reqwest::Client::new();
    let url = format!("{}?f=json", common::SAMPLE_FEATURE_SERVICE);

    common::rate_limit().await;

    let response = client.get(&url).send().await?;

    assert!(
        response.status().is_success(),
        "Sample feature service should be accessible"
    );

    let json: serde_json::Value = response.json().await?;

    // Verify it's a feature service response
    assert!(
        json.get("layers").is_some() || json.get("tables").is_some(),
        "Response should contain layers or tables"
    );
    Ok(())
}

#[tokio::test]
#[cfg(feature = "api")]
async fn test_feature_query_with_where_clause() -> Result<()> {
    use arcgis::{FeatureServiceClient, LayerId};

    let client = common::create_api_key_client()?;
    let feature_service = FeatureServiceClient::new(common::SAMPLE_FEATURE_SERVICE, &client);

    common::rate_limit().await;

    // Query for cities with population > 5 million using the fluent API
    let result = feature_service
        .query(LayerId::new(0))
        .where_clause("POP > 5000000")
        .out_fields(&["CITY_NAME", "POP"])
        .return_geometry(true)
        .limit(10)
        .execute()
        .await?;

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
    Ok(())
}

#[tokio::test]
#[cfg(feature = "api")]
async fn test_feature_query_count_only() -> Result<()> {
    use arcgis::{FeatureServiceClient, LayerId};

    let client = common::create_api_key_client()?;
    let feature_service = FeatureServiceClient::new(common::SAMPLE_FEATURE_SERVICE, &client);

    common::rate_limit().await;

    // Query for count of all features using the fluent API
    let result = feature_service
        .query(LayerId::new(0))
        .where_clause("1=1")
        .count_only(true)
        .execute()
        .await?;

    // When count_only is true, the response should include a count
    assert!(
        result.count.is_some(),
        "Count-only query should return a count"
    );
    assert!(
        result.count.unwrap() > 0,
        "Should have at least some features in the dataset"
    );

    // Features array should be empty for count-only queries
    assert!(
        result.features.is_empty(),
        "Count-only query should not return features"
    );
    Ok(())
}

#[tokio::test]
#[cfg(feature = "api")]
async fn test_feature_query_with_object_ids() -> Result<()> {
    use arcgis::{FeatureServiceClient, LayerId, ObjectId};

    let client = common::create_api_key_client()?;
    let feature_service = FeatureServiceClient::new(common::SAMPLE_FEATURE_SERVICE, &client);

    common::rate_limit().await;

    // Query specific features by object ID using the fluent API
    let result = feature_service
        .query(LayerId::new(0))
        .object_ids(&[ObjectId::new(1), ObjectId::new(2)])
        .out_fields(&["*"])
        .return_geometry(false)
        .execute()
        .await?;

    // May or may not return features depending on if those IDs exist
    // Just verify the query succeeded without error
    assert!(
        result.features.len() <= 2,
        "Should return at most 2 features"
    );
    Ok(())
}

#[tokio::test]
#[cfg(feature = "api")]
async fn test_feature_query_autopagination() -> Result<()> {
    use arcgis::{FeatureServiceClient, LayerId};

    let client = common::create_api_key_client()?;
    let feature_service = FeatureServiceClient::new(common::SAMPLE_FEATURE_SERVICE, &client);

    common::rate_limit().await;

    // Use execute_all() to automatically paginate through all results
    // Using a small page size to force multiple requests
    let result = feature_service
        .query(LayerId::new(0))
        .where_clause("POP > 100000") // Cities with population > 100k
        .out_fields(&["CITY_NAME", "POP"])
        .return_geometry(false)
        .limit(5) // Small page size to test pagination
        .execute_all()
        .await?;

    // Should have retrieved multiple pages of results
    // The actual count depends on the data, but should be > 5
    assert!(
        result.features.len() >= 5,
        "Auto-pagination should retrieve more than one page"
    );

    // exceeded_transfer_limit should be false after pagination completes
    assert!(
        !result.exceeded_transfer_limit,
        "Auto-pagination should retrieve all results"
    );
    Ok(())
}

// TODO: Add more integration tests as we implement features:
// - test_feature_edit_operations
// - test_oauth_flow
// - test_rate_limiting
// - test_spatial_queries
