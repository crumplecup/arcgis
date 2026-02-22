//! Advanced spatial query example demonstrating spatial relationships.
//!
//! This example shows how to:
//! - Query features by spatial relationship (intersects, contains, within)
//! - Use geometry filters in queries
//! - Combine spatial and attribute queries
//! - Query statistics with spatial filters
//!
//! # Prerequisites
//!
//! - Uses ESRI's public World Cities service (no auth required)
//!
//! # Running
//!
//! ```bash
//! cargo run --example spatial_query
//! ```

use arcgis::{
    ArcGISClient, ArcGISEnvelope, ArcGISGeometry, ArcGISPolygon, FeatureServiceClient,
    GeometryType, LayerId, NoAuth, SpatialReference, SpatialRel,
};

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

    tracing::info!("🗺️  Spatial Query Examples");
    tracing::info!("Using ESRI's public World Cities service");

    // Create client with NoAuth (public service)
    let client = ArcGISClient::new(NoAuth);
    let service = FeatureServiceClient::new(WORLD_CITIES_SERVICE, &client);
    let layer_id = LayerId::new(0);

    // Demonstrate spatial query operations
    demonstrate_bounding_box_query(&service, layer_id).await?;
    demonstrate_polygon_query(&service, layer_id).await?;
    demonstrate_combined_spatial_attribute(&service, layer_id).await?;
    demonstrate_spatial_relationships(&service, layer_id).await?;
    demonstrate_large_area_pagination(&service, layer_id).await?;

    tracing::info!("\n✅ All spatial query examples completed successfully!");
    print_best_practices();

    Ok(())
}

/// Demonstrates bounding box (envelope) queries.
async fn demonstrate_bounding_box_query(
    service: &FeatureServiceClient<'_>,
    layer_id: LayerId,
) -> anyhow::Result<()> {
    tracing::info!("\n=== Example 1: Bounding Box Query ===");
    tracing::info!("Find cities within a geographic extent (California)");

    // Define a bounding box around California
    // IMPORTANT: Must specify spatial reference so service knows coordinate system
    let california_bbox = ArcGISEnvelope::new(
        -124.5, // xmin: West
        32.5,   // ymin: South
        -114.0, // xmax: East
        42.0,   // ymax: North
    )
    .with_spatial_reference(Some(SpatialReference::wgs84()));

    let bbox_result = service
        .query(layer_id)
        .spatial_filter(
            ArcGISGeometry::Envelope(california_bbox),
            GeometryType::Envelope,
            SpatialRel::Intersects,
        )
        .where_clause("POP > 100000") // Cities with pop > 100k
        .out_fields(&["CITY_NAME", "POP"])
        .return_geometry(true)
        .execute()
        .await?;

    // Verify spatial filter returned results
    assert!(
        !bbox_result.features().is_empty(),
        "Expected to find cities in California bounding box"
    );

    tracing::info!(
        cities_found = bbox_result.features().len(),
        "Cities in California bounding box"
    );

    for feature in bbox_result.features().iter().take(5) {
        let city = feature.attributes().get("CITY_NAME");
        let pop = feature.attributes().get("POP");

        // Verify requested fields are present
        assert!(city.is_some(), "CITY_NAME should be present");
        assert!(pop.is_some(), "POP should be present");

        // Verify geometry is present when requested
        assert!(
            feature.geometry().is_some(),
            "Geometry should be present when return_geometry=true"
        );

        tracing::info!(city = ?city, population = ?pop, "City in bbox");
    }

    Ok(())
}

/// Demonstrates polygon queries with complex shapes.
async fn demonstrate_polygon_query(
    service: &FeatureServiceClient<'_>,
    layer_id: LayerId,
) -> anyhow::Result<()> {
    tracing::info!("\n=== Example 2: Polygon Query ===");
    tracing::info!("Find cities within a custom polygon");

    // Define a polygon (simplified Pacific Northwest)
    let pacific_nw_polygon = ArcGISPolygon::new(vec![vec![
        vec![-125.0, 49.0], // NW corner (start)
        vec![-116.0, 49.0], // NE corner
        vec![-116.0, 42.0], // SE corner
        vec![-125.0, 42.0], // SW corner
        vec![-125.0, 49.0], // Close the ring back to start
    ]])
    .with_spatial_reference(Some(SpatialReference::wgs84()));

    let polygon_result = service
        .query(layer_id)
        .spatial_filter(
            ArcGISGeometry::Polygon(pacific_nw_polygon),
            GeometryType::Polygon,
            SpatialRel::Intersects, // Cities that intersect the polygon
        )
        .where_clause("POP > 50000")
        .out_fields(&["CITY_NAME", "POP", "CNTRY_NAME"])
        .return_geometry(false)
        .execute()
        .await?;

    // Verify polygon filter returned results
    assert!(
        !polygon_result.features().is_empty(),
        "Expected to find cities in Pacific Northwest polygon"
    );

    tracing::info!(
        cities_found = polygon_result.features().len(),
        "Cities within Pacific Northwest polygon"
    );

    for feature in polygon_result.features() {
        let city = feature.attributes().get("CITY_NAME");
        let country = feature.attributes().get("CNTRY_NAME");

        // Verify geometry is NOT present when return_geometry=false
        assert!(
            feature.geometry().is_none(),
            "Geometry should be None when return_geometry=false"
        );

        tracing::info!(city = ?city, country = ?country, "City in polygon");
    }

    Ok(())
}

/// Demonstrates combining spatial and attribute queries.
async fn demonstrate_combined_spatial_attribute(
    service: &FeatureServiceClient<'_>,
    layer_id: LayerId,
) -> anyhow::Result<()> {
    tracing::info!("\n=== Example 3: Combined Spatial + Attribute Query ===");
    tracing::info!("Large cities on the West Coast");

    let west_coast_bbox = ArcGISEnvelope::new(-125.0, 32.0, -114.0, 50.0)
        .with_spatial_reference(Some(SpatialReference::wgs84()));

    let combined_result = service
        .query(layer_id)
        .spatial_filter(
            ArcGISGeometry::Envelope(west_coast_bbox),
            GeometryType::Envelope,
            SpatialRel::Intersects,
        )
        .where_clause("POP > 500000") // Large cities only
        .out_fields(&["CITY_NAME", "POP", "CNTRY_NAME"])
        .return_geometry(true)
        .limit(10)
        .execute()
        .await?;

    // Verify combined spatial + attribute filter works
    assert!(
        !combined_result.features().is_empty(),
        "Expected to find large cities on West Coast"
    );

    tracing::info!(
        cities_found = combined_result.features().len(),
        "Large cities on West Coast"
    );

    for feature in combined_result.features() {
        let city = feature.attributes().get("CITY_NAME");
        let pop = feature.attributes().get("POP");
        let country = feature.attributes().get("CNTRY_NAME");
        let has_geom = feature.geometry().is_some();

        tracing::info!(
            city = ?city,
            population = ?pop,
            country = ?country,
            has_geometry = has_geom,
            "Large West Coast city"
        );
    }

    Ok(())
}

/// Demonstrates different spatial relationship types.
async fn demonstrate_spatial_relationships(
    service: &FeatureServiceClient<'_>,
    layer_id: LayerId,
) -> anyhow::Result<()> {
    tracing::info!("\n=== Example 4: Different Spatial Relationships ===");
    tracing::info!("Demonstrating various spatial relationship types");

    let test_bbox = ArcGISEnvelope::new(-122.5, 37.5, -122.0, 38.0)
        .with_spatial_reference(Some(SpatialReference::wgs84()));

    // Test different spatial relationships
    let relationships = vec![
        (SpatialRel::Intersects, "Intersects"),
        (SpatialRel::Contains, "Contains"),
        (SpatialRel::Within, "Within"),
    ];

    for (spatial_rel, rel_name) in relationships {
        let result = service
            .query(layer_id)
            .spatial_filter(
                ArcGISGeometry::Envelope(test_bbox.clone()),
                GeometryType::Envelope,
                spatial_rel,
            )
            .where_clause("POP > 10000")
            .return_geometry(false)
            .count_only(true)
            .execute()
            .await?;

        // Verify count was returned
        assert!(
            result.count().is_some(),
            "Count should be returned for spatial relationship query: {}",
            rel_name
        );

        if let Some(count) = result.count() {
            tracing::info!(
                relationship = %rel_name,
                count = count,
                "Cities matching spatial relationship"
            );
        }
    }

    Ok(())
}

/// Demonstrates large area queries with automatic pagination.
async fn demonstrate_large_area_pagination(
    service: &FeatureServiceClient<'_>,
    layer_id: LayerId,
) -> anyhow::Result<()> {
    tracing::info!("\n=== Example 5: Large Area with Auto-Pagination ===");
    tracing::info!("Query entire US with automatic pagination");

    let us_bbox = ArcGISEnvelope::new(-125.0, 24.0, -66.0, 50.0)
        .with_spatial_reference(Some(SpatialReference::wgs84()));

    let us_result = service
        .query(layer_id)
        .spatial_filter(
            ArcGISGeometry::Envelope(us_bbox),
            GeometryType::Envelope,
            SpatialRel::Intersects,
        )
        .where_clause("POP > 100000")
        .out_fields(&["CITY_NAME", "POP"])
        .return_geometry(false)
        .limit(10) // Small page size
        .execute_all() // Auto-paginate
        .await?;

    // Verify pagination returned multiple cities
    assert!(
        !us_result.features().is_empty(),
        "Expected to find US cities with population > 100,000"
    );

    // With small page size (10) and execute_all(), should get many results
    assert!(
        us_result.features().len() > 10,
        "execute_all() should have paginated and returned > 10 results, got {}",
        us_result.features().len()
    );

    tracing::info!(
        total_cities = us_result.features().len(),
        exceeded_limit = us_result.exceeded_transfer_limit(),
        "US cities with population > 100,000"
    );

    Ok(())
}

/// Prints best practices for spatial queries.
fn print_best_practices() {
    tracing::info!("\n💡 Spatial Query Best Practices:");
    tracing::info!("   - Always specify spatial_reference on geometries (typically WGS84)");
    tracing::info!("   - Use Intersects for 'overlaps or touches' queries (most common)");
    tracing::info!("   - Use Contains for 'completely inside' queries");
    tracing::info!("   - Use Within for 'feature is inside geometry' queries");
    tracing::info!("   - Combine spatial filters with WHERE clauses for powerful queries");
    tracing::info!("");
    tracing::info!("🎯 Geometry Types:");
    tracing::info!("   - Envelope: Fast bounding box queries (rectangular areas)");
    tracing::info!("   - Polygon: Complex shapes, irregular boundaries");
    tracing::info!("   - Point: Distance-based queries, nearest neighbor");
    tracing::info!("   - Polyline: Route analysis, corridor queries");
    tracing::info!("");
    tracing::info!("⚡ Performance Tips:");
    tracing::info!("   - Use envelopes instead of polygons when possible (faster)");
    tracing::info!("   - Limit result set with WHERE clauses to reduce data transfer");
    tracing::info!("   - Use execute_all() for large spatial queries with pagination");
    tracing::info!("   - Skip geometry in results (return_geometry(false)) if not needed");
}
