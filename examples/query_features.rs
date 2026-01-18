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

    // Example 1: Basic WHERE clause query
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

    tracing::info!(
        feature_count = result.features().len(),
        "Found large cities"
    );

    for feature in result.features().iter().take(3) {
        let city = feature.attributes().get("CITY_NAME");
        let pop = feature.attributes().get("POP");
        let country = feature.attributes().get("CNTRY_NAME");
        tracing::info!(
            city = ?city,
            population = ?pop,
            country = ?country,
            has_geometry = feature.geometry().is_some(),
            "City details"
        );
    }

    // Example 2: Field filtering (attributes only, no geometry)
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

    tracing::info!(
        feature_count = result.features().len(),
        "Retrieved features (attributes only)"
    );

    for feature in result.features() {
        let has_geom = feature.geometry().is_some();
        tracing::debug!(
            has_geometry = has_geom,
            attributes = ?feature.attributes(),
            "Feature without geometry"
        );
    }

    // Example 3: Count-only query (no features returned)
    tracing::info!("\n=== Example 3: Count-Only Query ===");
    tracing::info!("Get count of features without retrieving data");

    let result = service
        .query(layer_id)
        .where_clause("POP > 100000")
        .count_only(true)
        .execute()
        .await?;

    if let Some(count) = result.count() {
        tracing::info!(
            count = count,
            features_returned = result.features().len(),
            "Cities with population > 100,000"
        );
    }

    // Example 4: Query by specific Object IDs
    tracing::info!("\n=== Example 4: Query by Object IDs ===");
    tracing::info!("Retrieve specific features by their ObjectID");

    let result = service
        .query(layer_id)
        .object_ids(&[ObjectId::new(1), ObjectId::new(2), ObjectId::new(3)])
        .out_fields(&["*"]) // All fields
        .execute()
        .await?;

    tracing::info!(
        requested = 3,
        returned = result.features().len(),
        "Retrieved features by ObjectID"
    );

    // Example 5: Manual pagination
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

    // Example 6: Auto-pagination (execute_all)
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

    // Example 7: Test Alternative Formats
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

    tracing::info!("\n✅ All query examples completed successfully!");
    tracing::info!("💡 Tips:");
    tracing::info!("   - Use return_geometry(false) for better performance when you don't need geometry");
    tracing::info!("   - Use count_only(true) to get counts without retrieving features");
    tracing::info!("   - Use execute_all() for automatic pagination");
    tracing::info!("   - Use .geojson() for GeoJSON format (standard for web mapping)");
    tracing::info!("   - Use .pbf() for PBF format (3-5x faster for large datasets)");

    Ok(())
}
