//! Public feature service query tests (no authentication required).
//!
//! These tests use ESRI's public World Cities sample service and run in CI.
//! No API key or credentials needed.

use arcgis::{FeatureServiceClient, LayerId, ObjectId};

/// Public World Cities feature service (ESRI sample data).
const WORLD_CITIES_SERVICE: &str =
    "https://services.arcgis.com/P3ePLMYs2RVChkJx/arcgis/rest/services/World_Cities/FeatureServer";

/// Rate limiting helper to be polite to public services.
async fn rate_limit() {
    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
}

/// Create an unauthenticated client for public service access.
fn create_public_client() -> arcgis::ArcGISClient {
    use arcgis::NoAuth;
    arcgis::ArcGISClient::new(NoAuth)
}

#[tokio::test]
#[cfg(feature = "test-public")]
async fn test_public_service_metadata() {
    // Verify we can access service metadata without authentication
    let client = reqwest::Client::new();
    let url = format!("{}?f=json", WORLD_CITIES_SERVICE);

    rate_limit().await;

    let response = client.get(&url).send().await.expect("Request failed");

    assert!(
        response.status().is_success(),
        "Public service should be accessible"
    );

    let json: serde_json::Value = response.json().await.expect("Invalid JSON");

    // Verify it's a feature service
    assert!(
        json.get("layers").is_some() || json.get("tables").is_some(),
        "Response should contain layers or tables"
    );
    assert_eq!(
        json.get("serviceDescription").and_then(|v| v.as_str()),
        Some("World Cities"),
        "Service description should match"
    );
}

#[tokio::test]
#[cfg(feature = "test-public")]
async fn test_query_where_clause() {
    let client = create_public_client();
    let service = FeatureServiceClient::new(WORLD_CITIES_SERVICE, &client);

    rate_limit().await;

    // Query for large cities (population > 5 million)
    let result = service
        .query(LayerId::new(0))
        .where_clause("POP > 5000000")
        .out_fields(&["CITY_NAME", "POP", "CNTRY_NAME"])
        .return_geometry(true)
        .limit(10)
        .execute()
        .await
        .expect("Query failed");

    // Should find major world cities
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
}

#[tokio::test]
#[cfg(feature = "test-public")]
async fn test_query_count_only() {
    let client = create_public_client();
    let service = FeatureServiceClient::new(WORLD_CITIES_SERVICE, &client);

    rate_limit().await;

    // Get count of all cities
    let result = service
        .query(LayerId::new(0))
        .where_clause("1=1")
        .count_only(true)
        .execute()
        .await
        .expect("Count query failed");

    // Should have a count
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
}

#[tokio::test]
#[cfg(feature = "test-public")]
async fn test_query_specific_object_ids() {
    let client = create_public_client();
    let service = FeatureServiceClient::new(WORLD_CITIES_SERVICE, &client);

    rate_limit().await;

    // Query by specific object IDs (may or may not exist)
    let result = service
        .query(LayerId::new(0))
        .object_ids(&[ObjectId::new(1), ObjectId::new(2), ObjectId::new(3)])
        .out_fields(&["*"])
        .return_geometry(false)
        .execute()
        .await
        .expect("Object ID query failed");

    // Should return at most 3 features (if those IDs exist)
    assert!(
        result.features().len() <= 3,
        "Should return at most requested number of features"
    );
}

#[tokio::test]
#[cfg(feature = "test-public")]
async fn test_query_with_field_filtering() {
    let client = create_public_client();
    let service = FeatureServiceClient::new(WORLD_CITIES_SERVICE, &client);

    rate_limit().await;

    // Query with specific fields only
    let result = service
        .query(LayerId::new(0))
        .where_clause("POP > 1000000")
        .out_fields(&["CITY_NAME", "POP"])
        .return_geometry(false)
        .limit(5)
        .execute()
        .await
        .expect("Field filtering query failed");

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
}

#[tokio::test]
#[cfg(feature = "test-public")]
async fn test_query_without_geometry() {
    let client = create_public_client();
    let service = FeatureServiceClient::new(WORLD_CITIES_SERVICE, &client);

    rate_limit().await;

    // Query without geometry for faster response
    let result = service
        .query(LayerId::new(0))
        .where_clause("1=1")
        .return_geometry(false)
        .limit(10)
        .execute()
        .await
        .expect("No-geometry query failed");

    assert!(!result.features().is_empty(), "Should return features");

    // Verify no geometry
    for feature in result.features() {
        assert!(
            feature.geometry().is_none(),
            "Features should not have geometry when not requested"
        );
    }
}

#[tokio::test]
#[cfg(feature = "test-public")]
async fn test_query_with_limit() {
    let client = create_public_client();
    let service = FeatureServiceClient::new(WORLD_CITIES_SERVICE, &client);

    rate_limit().await;

    // Query with small limit
    let limit = 3u32;
    let result = service
        .query(LayerId::new(0))
        .where_clause("1=1")
        .limit(limit)
        .return_geometry(false)
        .execute()
        .await
        .expect("Limited query failed");

    // Should respect limit
    assert!(
        result.features().len() <= limit as usize,
        "Should not exceed requested limit"
    );
}

#[tokio::test]
#[cfg(feature = "test-public")]
async fn test_feature_count_method() {
    let client = create_public_client();
    let service = FeatureServiceClient::new(WORLD_CITIES_SERVICE, &client);

    rate_limit().await;

    // Use the dedicated count method
    let count = service
        .query_feature_count(LayerId::new(0), "POP > 100000")
        .await
        .expect("Count method failed");

    assert!(count > 0, "Should find cities with population > 100,000");
}
