//! Public feature service query tests (no authentication required).
//!
//! These tests use ESRI's public World Cities sample service and run in CI.
//! No API key or credentials needed.

mod common;

#[cfg(feature = "test-public")]
use arcgis::{FeatureServiceClient, LayerId, ObjectId};

/// Public World Cities feature service (ESRI sample data).
#[cfg(feature = "test-public")]
const WORLD_CITIES_SERVICE: &str =
    "https://services.arcgis.com/P3ePLMYs2RVChkJx/arcgis/rest/services/World_Cities/FeatureServer";

/// Rate limiting helper to be polite to public services.
#[cfg(feature = "test-public")]
async fn rate_limit() {
    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
}

/// Create an unauthenticated client for public service access.
#[cfg(feature = "test-public")]
fn create_public_client() -> arcgis::ArcGISClient {
    use arcgis::NoAuth;
    arcgis::ArcGISClient::new(NoAuth)
}

#[tokio::test]
#[cfg(feature = "test-public")]
async fn test_public_service_metadata() -> anyhow::Result<()> {
    common::init_tracing();
    tracing::info!("test_public_service_metadata: Starting");

    // Verify we can access service metadata without authentication
    let client = reqwest::Client::new();
    let url = format!("{}?f=json", WORLD_CITIES_SERVICE);
    tracing::info!(url = %url, "test_public_service_metadata: Fetching service metadata");

    rate_limit().await;

    let response = client.get(&url).send().await?;

    assert!(
        response.status().is_success(),
        "Public service should be accessible"
    );
    tracing::info!("test_public_service_metadata: Service accessible");

    let json: serde_json::Value = response.json().await?;

    // Verify it's a feature service
    assert!(
        json.get("layers").is_some() || json.get("tables").is_some(),
        "Response should contain layers or tables"
    );

    // Verify serviceDescription field exists (may be empty)
    assert!(
        json.get("serviceDescription").is_some(),
        "Service should have serviceDescription field"
    );
    tracing::info!("test_public_service_metadata: Completed");
    Ok(())
}

#[tokio::test]
#[cfg(feature = "test-public")]
async fn test_query_where_clause() -> anyhow::Result<()> {
    common::init_tracing();
    tracing::info!("test_query_where_clause: Starting");

    let client = create_public_client();
    let service = FeatureServiceClient::new(WORLD_CITIES_SERVICE, &client);

    rate_limit().await;

    // Query for large cities (population > 5 million)
    tracing::info!("test_query_where_clause: Executing query for cities with POP > 5000000");
    let result = service
        .query(LayerId::new(0))
        .where_clause("POP > 5000000")
        .out_fields(&["CITY_NAME", "POP", "CNTRY_NAME"])
        .return_geometry(true)
        .limit(10)
        .execute()
        .await?;

    // Should find major world cities
    tracing::info!(
        feature_count = result.features().len(),
        "test_query_where_clause: Received features"
    );
    assert!(
        !result.features().is_empty(),
        "Should find cities with population > 5 million"
    );

    // Verify feature structure
    let first = &result.features()[0];
    assert!(
        first.attributes().contains_key("CITY_NAME"),
        "Feature should have CITY_NAME"
    );
    assert!(
        first.attributes().contains_key("POP"),
        "Feature should have POP"
    );
    assert!(
        first.geometry().is_some(),
        "Feature should have geometry when requested"
    );

    tracing::info!("test_query_where_clause: Completed");
    Ok(())
}

#[tokio::test]
#[cfg(feature = "test-public")]
async fn test_query_count_only() -> anyhow::Result<()> {
    common::init_tracing();
    tracing::info!("test_query_count_only: Starting");

    let client = create_public_client();
    let service = FeatureServiceClient::new(WORLD_CITIES_SERVICE, &client);

    rate_limit().await;

    // Get count of all cities
    tracing::info!("test_query_count_only: Executing count-only query");
    let result = service
        .query(LayerId::new(0))
        .where_clause("1=1")
        .count_only(true)
        .execute()
        .await?;

    // Should have a count
    tracing::info!(
        count = ?result.count(),
        "test_query_count_only: Received count"
    );
    assert!(
        result.count().is_some(),
        "Count-only query should return count"
    );
    assert!(
        result.count().unwrap() > 1000,
        "World Cities dataset should have many cities"
    );

    // Should not return features
    assert!(
        result.features().is_empty(),
        "Count-only query should not return features"
    );

    tracing::info!("test_query_count_only: Completed");
    Ok(())
}

#[tokio::test]
#[cfg(feature = "test-public")]
async fn test_query_specific_object_ids() -> anyhow::Result<()> {
    common::init_tracing();
    tracing::info!("test_query_specific_object_ids: Starting");

    let client = create_public_client();
    let service = FeatureServiceClient::new(WORLD_CITIES_SERVICE, &client);

    rate_limit().await;

    // Query by specific object IDs (may or may not exist)
    tracing::info!("test_query_specific_object_ids: Querying for object IDs 1, 2, 3");
    let result = service
        .query(LayerId::new(0))
        .object_ids(&[ObjectId::new(1), ObjectId::new(2), ObjectId::new(3)])
        .out_fields(&["*"])
        .return_geometry(false)
        .execute()
        .await?;

    // Should return at most 3 features (if those IDs exist)
    tracing::info!(
        feature_count = result.features().len(),
        "test_query_specific_object_ids: Received features"
    );
    assert!(
        result.features().len() <= 3,
        "Should return at most requested number of features"
    );

    tracing::info!("test_query_specific_object_ids: Completed");
    Ok(())
}

#[tokio::test]
#[cfg(feature = "test-public")]
async fn test_query_with_field_filtering() -> anyhow::Result<()> {
    common::init_tracing();
    tracing::info!("test_query_with_field_filtering: Starting");

    let client = create_public_client();
    let service = FeatureServiceClient::new(WORLD_CITIES_SERVICE, &client);

    rate_limit().await;

    // Query with specific fields only
    tracing::info!(
        "test_query_with_field_filtering: Querying with field filtering (CITY_NAME, POP)"
    );
    let result = service
        .query(LayerId::new(0))
        .where_clause("POP > 1000000")
        .out_fields(&["CITY_NAME", "POP"])
        .return_geometry(false)
        .limit(5)
        .execute()
        .await?;

    tracing::info!(
        feature_count = result.features().len(),
        "test_query_with_field_filtering: Received features"
    );
    assert!(
        !result.features().is_empty(),
        "Should find cities with population > 1 million"
    );

    // Verify only requested fields are present
    let first = &result.features()[0];
    assert!(
        first.attributes().contains_key("CITY_NAME"),
        "Should have requested CITY_NAME field"
    );
    assert!(
        first.attributes().contains_key("POP"),
        "Should have requested POP field"
    );
    assert!(
        first.geometry().is_none(),
        "Should not have geometry when not requested"
    );

    tracing::info!("test_query_with_field_filtering: Completed");
    Ok(())
}

#[tokio::test]
#[cfg(feature = "test-public")]
async fn test_query_without_geometry() -> anyhow::Result<()> {
    common::init_tracing();
    tracing::info!("test_query_without_geometry: Starting");

    let client = create_public_client();
    let service = FeatureServiceClient::new(WORLD_CITIES_SERVICE, &client);

    rate_limit().await;

    // Query without geometry for faster response
    tracing::info!("test_query_without_geometry: Querying without geometry");
    let result = service
        .query(LayerId::new(0))
        .where_clause("1=1")
        .return_geometry(false)
        .limit(10)
        .execute()
        .await?;

    tracing::info!(
        feature_count = result.features().len(),
        "test_query_without_geometry: Received features"
    );
    assert!(!result.features().is_empty(), "Should return features");

    // Verify no geometry
    for feature in result.features() {
        assert!(
            feature.geometry().is_none(),
            "Features should not have geometry when not requested"
        );
    }

    tracing::info!("test_query_without_geometry: Completed");
    Ok(())
}

#[tokio::test]
#[cfg(feature = "test-public")]
async fn test_query_with_limit() -> anyhow::Result<()> {
    common::init_tracing();
    tracing::info!("test_query_with_limit: Starting");

    let client = create_public_client();
    let service = FeatureServiceClient::new(WORLD_CITIES_SERVICE, &client);

    rate_limit().await;

    // Query with small limit
    let limit = 3u32;
    tracing::info!(limit = limit, "test_query_with_limit: Querying with limit");
    let result = service
        .query(LayerId::new(0))
        .where_clause("1=1")
        .limit(limit)
        .return_geometry(false)
        .execute()
        .await?;

    // Should respect limit
    tracing::info!(
        feature_count = result.features().len(),
        limit = limit,
        "test_query_with_limit: Received features"
    );
    assert!(
        result.features().len() <= limit as usize,
        "Should not exceed requested limit"
    );

    tracing::info!("test_query_with_limit: Completed");
    Ok(())
}

#[tokio::test]
#[cfg(feature = "test-public")]
async fn test_feature_count_method() -> anyhow::Result<()> {
    common::init_tracing();
    tracing::info!("test_feature_count_method: Starting");

    let client = create_public_client();
    let service = FeatureServiceClient::new(WORLD_CITIES_SERVICE, &client);

    rate_limit().await;

    // Use the dedicated count method
    tracing::info!("test_feature_count_method: Executing count query for POP > 100000");
    let count = service
        .query_feature_count(LayerId::new(0), "POP > 100000")
        .await?;

    tracing::info!(count = count, "test_feature_count_method: Received count");
    assert!(count > 0, "Should find cities with population > 100,000");

    tracing::info!("test_feature_count_method: Completed");
    Ok(())
}
