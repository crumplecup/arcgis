//! 📐 Geometry Operations - Spatial Analysis and Transformations
//!
//! Demonstrates common geometric operations using the ArcGIS Geometry Service.
//! Learn coordinate projections, buffer creation, distance calculations, and
//! batch processing of spatial data.
//!
//! # What You'll Learn
//!
//! - **Coordinate projection**: Transform between spatial reference systems
//! - **Buffer creation**: Create areas around geometries
//! - **Distance calculation**: Measure geodesic distances between points
//! - **Batch operations**: Process multiple geometries efficiently
//! - **Line length**: Calculate lengths of polylines
//! - **Builder patterns**: Use type-safe parameter builders
//!
//! # Prerequisites
//!
//! **None!** The ArcGIS Geometry Service is a free public utility that requires
//! no authentication. This makes it perfect for learning and experimentation.
//!
//! # Running
//!
//! ```bash
//! cargo run --example geometry_operations
//!
//! # With debug logging:
//! RUST_LOG=debug cargo run --example geometry_operations
//! ```
//!
//! # Real-World Use Cases
//!
//! - **Mapping applications**: Project coordinates for display on web maps
//! - **Proximity analysis**: Create buffers around points of interest
//! - **Distance measurement**: Calculate travel distances between locations
//! - **Coordinate conversion**: Transform GPS coordinates to map projections
//! - **Spatial queries**: Find features within a certain distance

use anyhow::Result;
use arcgis::{
    ArcGISClient, ArcGISGeometryV2 as ArcGISGeometry, ArcGISPointV2 as ArcGISPoint,
    ArcGISPolylineV2 as ArcGISPolyline, GeometryServiceClient, LinearUnit, NoAuth,
};

/// ArcGIS Online Geometry Service URL (free public utility)
const GEOMETRY_SERVICE: &str =
    "https://utility.arcgisonline.com/arcgis/rest/services/Geometry/GeometryServer";

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing for structured logging
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    tracing::info!("📐 ArcGIS Geometry Service Examples");
    tracing::info!("Demonstrating spatial analysis and transformations");

    // Create geometry service client (no auth required for public service)
    let auth = NoAuth;
    let client = ArcGISClient::new(auth);
    let geom_service = GeometryServiceClient::new(GEOMETRY_SERVICE, &client);

    // Demonstrate geometry operations
    demonstrate_coordinate_projection(&geom_service).await?;
    demonstrate_buffer_creation(&geom_service).await?;
    demonstrate_distance_calculation(&geom_service).await?;
    demonstrate_batch_projection(&geom_service).await?;
    demonstrate_line_length(&geom_service).await?;

    tracing::info!("\n✅ All geometry operations completed successfully!");
    print_best_practices();

    Ok(())
}

/// Demonstrates coordinate projection between spatial reference systems.
async fn demonstrate_coordinate_projection(geom_service: &GeometryServiceClient<'_>) -> Result<()> {
    tracing::info!("\n=== Example 1: Coordinate Projection ===");
    tracing::info!("Project coordinates from WGS84 (4326) to Web Mercator (3857)");

    // San Francisco coordinates in WGS84 (latitude/longitude)
    let sf_point = ArcGISPoint::new(
        -122.4194, // Longitude
        37.7749,   // Latitude
    );

    tracing::info!(
        lon = *sf_point.x(),
        lat = *sf_point.y(),
        "Original coordinates (WGS84)"
    );

    let result = geom_service
        .project(
            vec![ArcGISGeometry::Point(sf_point)],
            4326, // WGS84
            3857, // Web Mercator
        )
        .await?;

    if let Some(ArcGISGeometry::Point(projected)) = result.geometries().first() {
        tracing::info!(
            x = *projected.x(),
            y = *projected.y(),
            "✅ Projected coordinates (Web Mercator)"
        );
    }

    Ok(())
}

/// Demonstrates creating buffers around geometries.
async fn demonstrate_buffer_creation(geom_service: &GeometryServiceClient<'_>) -> Result<()> {
    tracing::info!("\n=== Example 2: Buffer Creation ===");
    tracing::info!("Create a 1000-meter buffer around a point");

    let buffer_point = ArcGISPoint::new(-122.4194, 37.7749);

    let buffer_params = arcgis::BufferParameters::builder()
        .geometries(vec![ArcGISGeometry::Point(buffer_point)])
        .in_sr(4326) // WGS84
        .distances(vec![1000.0]) // 1000 meters
        .unit(LinearUnit::Meters)
        .union_results(false)
        .geodesic(true) // Use geodesic (great circle) buffer
        .build()
        .map_err(|e| anyhow::anyhow!("Failed to build buffer parameters: {}", e))?;

    tracing::debug!("Creating 1000m geodesic buffer");
    let buffer_result = geom_service.buffer(buffer_params).await?;

    tracing::info!(
        buffer_count = buffer_result.geometries().len(),
        "✅ Buffer polygons created"
    );

    Ok(())
}

/// Demonstrates calculating geodesic distance between points.
async fn demonstrate_distance_calculation(geom_service: &GeometryServiceClient<'_>) -> Result<()> {
    tracing::info!("\n=== Example 3: Distance Calculation ===");
    tracing::info!("Calculate geodesic distance between San Francisco and Los Angeles");

    // San Francisco
    let sf = ArcGISPoint::new(-122.4194, 37.7749);

    // Los Angeles
    let la = ArcGISPoint::new(-118.2437, 34.0522);

    let distance_params = arcgis::DistanceParameters::builder()
        .sr(4326) // WGS84
        .geometry1(ArcGISGeometry::Point(sf))
        .geometry2(ArcGISGeometry::Point(la))
        .distance_unit(LinearUnit::Meters)
        .geodesic(true) // Use geodesic (great circle) distance
        .build()
        .map_err(|e| anyhow::anyhow!("Failed to build distance parameters: {}", e))?;

    tracing::debug!("Calculating geodesic distance between SF and LA");
    let distance_result = geom_service.distance(distance_params).await?;

    let distance_km = distance_result.distance() / 1000.0;
    let distance_mi = distance_km * 0.621371;

    tracing::info!(
        distance_meters = distance_result.distance(),
        distance_km = format!("{:.2}", distance_km),
        distance_miles = format!("{:.2}", distance_mi),
        "✅ Distance calculated"
    );

    Ok(())
}

/// Demonstrates batch projection of multiple geometries.
async fn demonstrate_batch_projection(geom_service: &GeometryServiceClient<'_>) -> Result<()> {
    tracing::info!("\n=== Example 4: Batch Projection ===");
    tracing::info!("Project multiple cities at once (more efficient than individual calls)");

    let cities = [
        ("San Francisco", -122.4194, 37.7749),
        ("Los Angeles", -118.2437, 34.0522),
        ("Seattle", -122.3321, 47.6062),
        ("Portland", -122.6765, 45.5231),
    ];

    let points: Vec<ArcGISGeometry> = cities
        .iter()
        .map(|(_, lon, lat)| ArcGISGeometry::Point(ArcGISPoint::new(*lon, *lat)))
        .collect();

    tracing::info!(point_count = points.len(), "Projecting cities in batch");

    let batch_result = geom_service
        .project(
            points, 4326, // WGS84
            3857, // Web Mercator
        )
        .await?;

    tracing::info!("✅ Batch projection completed");

    for (i, (city_name, _, _)) in cities.iter().enumerate() {
        if let Some(ArcGISGeometry::Point(projected)) = batch_result.geometries().get(i) {
            tracing::debug!(
                city = %city_name,
                x_web_mercator = format!("{:.2}", projected.x()),
                y_web_mercator = format!("{:.2}", projected.y()),
                "Projected city"
            );
        }
    }

    Ok(())
}

/// Demonstrates creating and projecting a polyline.
async fn demonstrate_line_length(geom_service: &GeometryServiceClient<'_>) -> Result<()> {
    tracing::info!("\n=== Example 5: Polyline Creation ===");
    tracing::info!("Create and project a route line between cities");

    // Create a simple polyline connecting SF -> LA
    let route_line = ArcGISPolyline::new(vec![vec![
        vec![-122.4194, 37.7749], // SF
        vec![-118.2437, 34.0522], // LA
    ]]);

    tracing::debug!("Creating route geometry");

    // Project to Web Mercator for visualization
    let projected_route = geom_service
        .project(
            vec![ArcGISGeometry::Polyline(route_line)],
            4326, // WGS84
            3857, // Web Mercator
        )
        .await?;

    if let Some(ArcGISGeometry::Polyline(line)) = projected_route.geometries().first() {
        tracing::info!(
            paths = line.paths().len(),
            points_in_first_path = line.paths().first().map(|p| p.len()).unwrap_or(0),
            "✅ Route line created and projected"
        );
    }

    Ok(())
}

/// Prints best practices for geometry operations.
fn print_best_practices() {
    tracing::info!("\n💡 Geometry Operations Best Practices:");
    tracing::info!("   - Use geodesic=true for accurate Earth-surface distances and buffers");
    tracing::info!("   - Batch operations are more efficient than individual calls");
    tracing::info!("   - Project to appropriate coordinate systems for your use case");
    tracing::info!("   - Use builder patterns for type-safe parameter construction");
    tracing::info!("   - The Geometry Service is free and requires no authentication");
    tracing::info!("");
    tracing::info!("📍 Common Spatial Reference Systems:");
    tracing::info!("   - WKID 4326 (WGS84): Standard GPS coordinates (lat/lon)");
    tracing::info!("   - WKID 3857 (Web Mercator): Most web maps (Google, Bing, OSM)");
    tracing::info!("   - WKID 4269 (NAD83): North American maps");
    tracing::info!("   - WKID 2163 (US National Atlas): Equal area for US statistics");
    tracing::info!("");
    tracing::info!("⚡ Performance Optimization:");
    tracing::info!("   - Project once, use many times (cache projected coordinates)");
    tracing::info!("   - Batch geometries when possible (single API call)");
    tracing::info!("   - Use appropriate spatial reference for operations:");
    tracing::info!("     • WGS84 (4326) for distance/buffer with geodesic=true");
    tracing::info!("     • Projected (3857, UTM) for planar calculations");
    tracing::info!("");
    tracing::info!("🎯 When to Use Each Operation:");
    tracing::info!("   - project(): Display coordinates on maps, convert GPS to screen coords");
    tracing::info!("   - buffer(): Proximity analysis, service areas, impact zones");
    tracing::info!("   - distance(): Straight-line distances, as-the-crow-flies measurements");
    tracing::info!("   - Note: Use routing services for actual travel distances on roads");
}
