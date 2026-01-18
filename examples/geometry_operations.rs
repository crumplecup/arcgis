//! Comprehensive Geometry Service example.
//!
//! This example demonstrates common geometric operations:
//! - Coordinate projection (transform between spatial references)
//! - Buffer creation (create areas around geometries)
//! - Distance calculation
//! - Multiple points projection (batch operations)
//! - Demonstrates builder pattern for parameters
//!
//! # Prerequisites
//!
//! - ArcGIS API key (required for geometry services)
//!
//! # Environment Variables
//!
//! Create a `.env` file with:
//! ```env
//! ARCGIS_API_KEY=your_api_key_here
//! ```
//!
//! # Running
//!
//! ```bash
//! cargo run --example geometry_operations
//! ```

use arcgis::{
    ApiKeyAuth, ArcGISClient, ArcGISGeometry, ArcGISPoint, ArcGISPolyline, GeometryServiceClient,
    LinearUnit,
};

/// ArcGIS Online Geometry Service URL
const GEOMETRY_SERVICE: &str =
    "https://utility.arcgisonline.com/arcgis/rest/services/Geometry/GeometryServer";

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing for structured logging
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    tracing::info!("📐 Geometry Service Examples");

    // Load API key from environment (.env file automatically loaded)
    let auth = ApiKeyAuth::from_env()?;
    let client = ArcGISClient::new(auth);
    let geom_service = GeometryServiceClient::new(GEOMETRY_SERVICE, &client);

    // Example 1: Coordinate Projection
    tracing::info!("\n=== Example 1: Coordinate Projection ===");
    tracing::info!("Project coordinates from WGS84 (4326) to Web Mercator (3857)");

    // San Francisco coordinates in WGS84 (latitude/longitude)
    let sf_point = ArcGISPoint {
        x: -122.4194, // Longitude
        y: 37.7749,   // Latitude
        z: None,
        m: None,
        spatial_reference: None,
    };

    tracing::info!(
        lon = sf_point.x,
        lat = sf_point.y,
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
            x = projected.x,
            y = projected.y,
            "Projected coordinates (Web Mercator)"
        );
    }

    // Example 2: Buffer Creation
    tracing::info!("\n=== Example 2: Buffer Creation ===");
    tracing::info!("Create a 1000-meter buffer around a point");

    let buffer_point = ArcGISPoint {
        x: -122.4194,
        y: 37.7749,
        z: None,
        m: None,
        spatial_reference: None,
    };

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
        "Buffer polygons created"
    );

    // Example 3: Distance Calculation
    tracing::info!("\n=== Example 3: Distance Calculation ===");
    tracing::info!("Calculate distance between two cities");

    // San Francisco
    let sf = ArcGISPoint {
        x: -122.4194,
        y: 37.7749,
        z: None,
        m: None,
        spatial_reference: None,
    };

    // Los Angeles
    let la = ArcGISPoint {
        x: -118.2437,
        y: 34.0522,
        z: None,
        m: None,
        spatial_reference: None,
    };

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
        "Distance between San Francisco and Los Angeles"
    );

    // Example 4: Multiple Projections
    tracing::info!("\n=== Example 4: Batch Projection ===");
    tracing::info!("Project multiple points at once");

    let cities = vec![
        ("San Francisco", -122.4194, 37.7749),
        ("Los Angeles", -118.2437, 34.0522),
        ("Seattle", -122.3321, 47.6062),
        ("Portland", -122.6765, 45.5231),
    ];

    let points: Vec<ArcGISGeometry> = cities
        .iter()
        .map(|(_, lon, lat)| {
            ArcGISGeometry::Point(ArcGISPoint {
                x: *lon,
                y: *lat,
                z: None,
                m: None,
                spatial_reference: None,
            })
        })
        .collect();

    tracing::info!(point_count = points.len(), "Projecting cities");

    let batch_result = geom_service
        .project(
            points,
            4326, // WGS84
            3857, // Web Mercator
        )
        .await?;

    for (i, (city_name, _, _)) in cities.iter().enumerate() {
        if let Some(ArcGISGeometry::Point(projected)) = batch_result.geometries().get(i) {
            tracing::info!(
                city = %city_name,
                x_web_mercator = format!("{:.2}", projected.x),
                y_web_mercator = format!("{:.2}", projected.y),
                "Projected city"
            );
        }
    }

    // Example 5: Create Route Line and Calculate Length
    tracing::info!("\n=== Example 5: Line Length Calculation ===");
    tracing::info!("Calculate length of a route between cities");

    // Create a simple polyline connecting SF -> LA
    let route_line = ArcGISPolyline {
        paths: vec![vec![
            [-122.4194, 37.7749], // SF
            [-118.2437, 34.0522], // LA
        ]],
        spatial_reference: None,
    };

    tracing::debug!("Creating route geometry");

    // First project to a suitable projection for visualization
    let projected_route = geom_service
        .project(
            vec![ArcGISGeometry::Polyline(route_line)],
            4326, // WGS84
            3857, // Web Mercator
        )
        .await?;

    if let Some(ArcGISGeometry::Polyline(line)) = projected_route.geometries().first() {
        tracing::info!(
            paths = line.paths.len(),
            points_in_first_path = line.paths.first().map(|p| p.len()).unwrap_or(0),
            "Route line created"
        );
    }

    tracing::info!("\n✅ All geometry operations completed successfully!");
    tracing::info!("💡 Tips:");
    tracing::info!("   - Use geodesic=true for accurate Earth-surface distances");
    tracing::info!("   - Project to appropriate coordinate systems for your use case");
    tracing::info!("   - Web Mercator (3857) is common for web maps");
    tracing::info!("   - WGS84 (4326) is standard for GPS coordinates");
    tracing::info!("   - Batch operations are more efficient than individual calls");

    Ok(())
}
