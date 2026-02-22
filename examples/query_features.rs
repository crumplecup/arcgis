//! Comprehensive Feature Service query example.
//!
//! This example demonstrates the most common Feature Service query operations:
//! - Basic WHERE clause queries
//! - Field filtering with out_fields
//! - Geometry inclusion control
//! - Pagination (manual and automatic)
//! - Count-only queries
//! - Object ID queries
//!
//! # Prerequisites
//!
//! - ArcGIS API key or OAuth credentials
//! - Access to a Feature Service (we use ESRI's public World Cities service)
//!
//! # Environment Variables
//!
//! Create a `.env` file with:
//! ```env
//! ARCGIS_API_KEY=your_api_key_here
//! ```
//!
//! Or use NoAuth for public services (demonstrated below).
//!
//! # Running
//!
//! ```bash
//! cargo run --example query_features
//! ```

use arcgis::{ArcGISClient, FeatureServiceClient, LayerId, NoAuth, ObjectId};

/// Public World Cities feature service (no auth required).
const WORLD_CITIES_SERVICE: &str =
    "https://services.arcgis.com/P3ePLMYs2RVChkJx/arcgis/rest/services/World_Cities/FeatureServer";

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing for structured logging
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    tracing::info!("🌍 Feature Service Query Examples");
    tracing::info!("Using ESRI's public World Cities service");

    // Create client with NoAuth (public service)
    let client = ArcGISClient::new(NoAuth);
    let service = FeatureServiceClient::new(WORLD_CITIES_SERVICE, &client);
    let layer_id = LayerId::new(0);

    // Demonstrate Feature Service query operations
    demonstrate_basic_where_query(&service, layer_id).await?;
    demonstrate_field_filtering(&service, layer_id).await?;
    demonstrate_count_only(&service, layer_id).await?;
    demonstrate_object_id_query(&service, layer_id).await?;
    demonstrate_manual_pagination(&service, layer_id).await?;
    demonstrate_auto_pagination(&service, layer_id).await?;
    demonstrate_alternative_formats(&service, layer_id).await?;

    tracing::info!("\n✅ All query examples completed successfully!");
    print_best_practices();

    Ok(())
}

/// Demonstrates basic WHERE clause queries.
async fn demonstrate_basic_where_query(
    service: &FeatureServiceClient<'_>,
    layer_id: LayerId,
) -> anyhow::Result<()> {
    tracing::info!("\n=== Example 1: Basic WHERE Clause Query ===");
    tracing::info!("Query cities with population > 5 million");

    let result = service
        .query(layer_id)
        .where_clause("POP > 5000000")
        .out_fields(&["CITY_NAME", "POP", "CNTRY_NAME"])
        .return_geometry(true)
        .limit(10)
        .execute()
        .await?;

    // Verify we got results
    assert!(
        !result.features().is_empty(),
        "Expected to find cities with population > 5 million"
    );

    tracing::info!(
        feature_count = result.features().len(),
        "Found large cities"
    );

    for feature in result.features().iter().take(3) {
        let city = feature.attributes().get("CITY_NAME");
        let pop = feature.attributes().get("POP");
        let country = feature.attributes().get("CNTRY_NAME");

        // Verify requested fields are present
        assert!(city.is_some(), "CITY_NAME field should be present");
        assert!(pop.is_some(), "POP field should be present");

        // Verify geometry was returned when requested
        assert!(
            feature.geometry().is_some(),
            "Geometry should be present when return_geometry=true"
        );

        tracing::info!(
            city = ?city,
            population = ?pop,
            country = ?country,
            has_geometry = feature.geometry().is_some(),
            "City details"
        );
    }

    Ok(())
}

/// Demonstrates field filtering without geometry for better performance.
async fn demonstrate_field_filtering(
    service: &FeatureServiceClient<'_>,
    layer_id: LayerId,
) -> anyhow::Result<()> {
    tracing::info!("\n=== Example 2: Field Filtering (No Geometry) ===");
    tracing::info!("Query specific fields without geometry for faster response");

    let result = service
        .query(layer_id)
        .where_clause("POP > 1000000")
        .out_fields(&["CITY_NAME", "POP"])
        .return_geometry(false) // Skip geometry for better performance
        .limit(5)
        .execute()
        .await?;

    // Verify we got features
    assert!(
        !result.features().is_empty(),
        "Expected to find cities with population > 1 million"
    );

    tracing::info!(
        feature_count = result.features().len(),
        "Retrieved features (attributes only)"
    );

    for feature in result.features() {
        // Verify geometry was NOT returned when return_geometry=false
        assert!(
            feature.geometry().is_none(),
            "Geometry should be None when return_geometry=false"
        );

        // Verify requested fields are present
        assert!(
            feature.attributes().get("CITY_NAME").is_some(),
            "CITY_NAME should be present"
        );

        let has_geom = feature.geometry().is_some();
        tracing::debug!(
            has_geometry = has_geom,
            attributes = ?feature.attributes(),
            "Feature without geometry"
        );
    }

    Ok(())
}

/// Demonstrates count-only queries without returning features.
async fn demonstrate_count_only(
    service: &FeatureServiceClient<'_>,
    layer_id: LayerId,
) -> anyhow::Result<()> {
    tracing::info!("\n=== Example 3: Count-Only Query ===");
    tracing::info!("Get count of features without retrieving data");

    let result = service
        .query(layer_id)
        .where_clause("POP > 100000")
        .count_only(true)
        .execute()
        .await?;

    // Verify count was returned
    assert!(
        result.count().is_some(),
        "Count should be returned when count_only=true"
    );

    if let Some(count) = result.count() {
        // Verify count is reasonable (should be > 0 for cities with pop > 100k)
        assert!(*count > 0, "Expected at least some cities with pop > 100k");

        tracing::info!(
            count = count,
            features_returned = result.features().len(),
            "Cities with population > 100,000"
        );
    }

    Ok(())
}

/// Demonstrates querying by specific Object IDs.
async fn demonstrate_object_id_query(
    service: &FeatureServiceClient<'_>,
    layer_id: LayerId,
) -> anyhow::Result<()> {
    tracing::info!("\n=== Example 4: Query by Object IDs ===");
    tracing::info!("Retrieve specific features by their ObjectID");

    let result = service
        .query(layer_id)
        .object_ids(&[ObjectId::new(1), ObjectId::new(2), ObjectId::new(3)])
        .out_fields(&["*"]) // All fields
        .execute()
        .await?;

    // Verify we got features for the requested ObjectIDs
    assert!(
        !result.features().is_empty(),
        "Expected features for ObjectIDs 1, 2, 3"
    );

    tracing::info!(
        requested = 3,
        returned = result.features().len(),
        "Retrieved features by ObjectID"
    );

    Ok(())
}

/// Demonstrates manual pagination using offset and limit.
async fn demonstrate_manual_pagination(
    service: &FeatureServiceClient<'_>,
    layer_id: LayerId,
) -> anyhow::Result<()> {
    tracing::info!("\n=== Example 5: Manual Pagination ===");
    tracing::info!("Fetch results in pages using offset/limit");

    let page_size = 5;
    let mut total_fetched = 0;

    for page in 0..3 {
        let offset = page * page_size;
        tracing::debug!(page = page, offset = offset, "Fetching page");

        let result = service
            .query(layer_id)
            .where_clause("POP > 500000")
            .out_fields(&["CITY_NAME", "POP"])
            .return_geometry(false)
            .limit(page_size)
            .offset(offset)
            .execute()
            .await?;

        let count = result.features().len();
        total_fetched += count;

        tracing::info!(
            page = page,
            features_in_page = count,
            total_so_far = total_fetched,
            "Page fetched"
        );

        if count < page_size as usize {
            tracing::debug!("Reached last page");
            break;
        }
    }

    // Verify pagination returned features
    assert!(
        total_fetched > 0,
        "Pagination should have returned at least some features"
    );

    Ok(())
}

/// Demonstrates automatic pagination using execute_all().
async fn demonstrate_auto_pagination(
    service: &FeatureServiceClient<'_>,
    layer_id: LayerId,
) -> anyhow::Result<()> {
    tracing::info!("\n=== Example 6: Auto-Pagination ===");
    tracing::info!("Let the SDK handle pagination automatically");

    let result = service
        .query(layer_id)
        .where_clause("POP > 200000")
        .out_fields(&["CITY_NAME", "POP"])
        .return_geometry(false)
        .limit(5) // Small page size to force multiple requests
        .execute_all() // Automatically paginate through all results
        .await?;

    tracing::info!(
        total_features = result.features().len(),
        exceeded_limit = result.exceeded_transfer_limit(),
        "Auto-pagination completed"
    );

    Ok(())
}

/// Demonstrates alternative response formats (GeoJSON and PBF).
async fn demonstrate_alternative_formats(
    service: &FeatureServiceClient<'_>,
    layer_id: LayerId,
) -> anyhow::Result<()> {
    tracing::info!("\n=== Example 7: Alternative Response Formats ===");
    tracing::info!("Testing GeoJSON and PBF format support");

    // Test GeoJSON format
    let geojson_result = service
        .query(layer_id)
        .where_clause("POP > 5000000")
        .out_fields(&["CITY_NAME", "POP"])
        .return_geometry(false)
        .limit(3)
        .geojson()
        .execute()
        .await?;

    tracing::info!(
        feature_count = geojson_result.features().len(),
        "GeoJSON query completed"
    );

    for feature in geojson_result.features() {
        let city = feature.attributes().get("CITY_NAME");
        let pop = feature.attributes().get("POP");
        tracing::info!(city = ?city, population = ?pop, "City from GeoJSON");
    }

    // Test PBF format
    let pbf_result = service
        .query(layer_id)
        .where_clause("POP > 5000000")
        .out_fields(&["CITY_NAME", "POP"])
        .return_geometry(false)
        .limit(3)
        .pbf()
        .execute()
        .await?;

    tracing::info!(
        feature_count = pbf_result.features().len(),
        "PBF query completed"
    );

    for feature in pbf_result.features() {
        let city = feature.attributes().get("CITY_NAME");
        let pop = feature.attributes().get("POP");
        tracing::info!(city = ?city, population = ?pop, "City from PBF");
    }

    Ok(())
}

/// Prints best practices for Feature Service queries.
fn print_best_practices() {
    tracing::info!("\n💡 Query Best Practices:");
    tracing::info!(
        "   - Use return_geometry(false) for better performance when you don't need geometry"
    );
    tracing::info!("   - Use count_only(true) to get counts without retrieving features");
    tracing::info!("   - Use execute_all() for automatic pagination");
    tracing::info!("   - Narrow queries with specific WHERE clauses to reduce data transfer");
    tracing::info!("   - Request only needed fields with out_fields() instead of all fields");
    tracing::info!("");
    tracing::info!("🎯 Format Selection:");
    tracing::info!("   - Default JSON: Universal compatibility, moderate performance");
    tracing::info!("   - GeoJSON: Standard for web mapping libraries (Leaflet, Mapbox, etc.)");
    tracing::info!("   - PBF: 3-5x faster for large datasets, binary format");
    tracing::info!("");
    tracing::info!("⚡ Performance Tips:");
    tracing::info!("   - Skip geometry when doing attribute-only analysis");
    tracing::info!("   - Use count_only for checking dataset size before full query");
    tracing::info!("   - Paginate large result sets instead of fetching all at once");
    tracing::info!("   - Consider spatial queries (bounding box) to limit results geographically");
}
