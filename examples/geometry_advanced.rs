//! Advanced Geometry Service operations demonstration.
//!
//! This example demonstrates advanced geometry operations using the ArcGIS Geometry Service:
//! - Simplifying geometries to fix topological issues
//! - Union operations to merge multiple polygons
//! - Area and length calculations for polygons
//! - Finding available datum transformations
//! - Projecting geometries with specific transformation parameters
//!
//! These operations are useful for:
//! - Cleaning up invalid geometries from user input
//! - Combining multiple parcels or zones
//! - Calculating accurate measurements across different projections
//! - Working with data in different coordinate systems
//!
//! # Service Requirements
//!
//! This example uses the public ArcGIS Geometry Service:
//! - URL: <https://utility.arcgisonline.com/arcgis/rest/services/Geometry/GeometryServer>
//! - Authentication: None required (public service)
//!
//! # Running the Example
//!
//! ```bash
//! cargo run --example geometry_advanced
//! ```

use anyhow::{Context, Result, ensure};
use arcgis::{
    ArcGISClient, ArcGISEnvelope, ArcGISGeometry, ArcGISPoint, ArcGISPolygon, AreaUnit,
    AreasAndLengthsParameters, CalculationType, GeometryServiceClient, LinearUnit, NoAuth,
    ProjectParameters, SimplifyParameters, SpatialReference, UnionParameters,
};
use tracing::{debug, info};
use tracing_subscriber::{EnvFilter, layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing for debugging
    tracing_subscriber::registry()
        .with(EnvFilter::from_default_env())
        .with(tracing_subscriber::fmt::layer())
        .init();

    info!("Starting advanced geometry operations demonstration");

    // Initialize client with public geometry service (no auth required)
    let auth = NoAuth;
    let client = ArcGISClient::new(auth);

    // Create geometry service client
    let service_url =
        "https://utility.arcgisonline.com/arcgis/rest/services/Geometry/GeometryServer";
    let service = GeometryServiceClient::new(service_url, &client);

    info!("Connected to ArcGIS Geometry Service");

    // Demonstrate each advanced operation
    demonstrate_simplify(&service).await?;
    demonstrate_union(&service).await?;
    demonstrate_areas_and_lengths(&service).await?;
    demonstrate_datum_transformations(&service).await?;

    info!("✓ All advanced geometry operations completed successfully");
    Ok(())
}

/// Demonstrates geometry simplification for fixing topological issues.
///
/// Simplify is useful for cleaning up geometries that may have:
/// - Self-intersections
/// - Duplicate vertices
/// - Invalid ring orientations
async fn demonstrate_simplify(service: &GeometryServiceClient<'_>) -> Result<()> {
    info!("\n=== Simplify Operation ===");

    // Create a complex polygon with potential issues
    let complex_polygon = ArcGISPolygon::new(vec![vec![
        vec![-117.0, 34.0],
        vec![-117.0, 34.1],
        vec![-116.9, 34.1],
        vec![-116.9, 34.0],
        vec![-117.0, 34.0], // Close the ring
    ]]);

    debug!(
        ring_count = complex_polygon.rings().len(),
        "Created polygon for simplification"
    );

    let params = SimplifyParameters::builder()
        .geometries(vec![ArcGISGeometry::Polygon(complex_polygon.clone())])
        .sr(4326)
        .build()
        .context("Failed to build simplify params")?;

    let result = service
        .simplify(params)
        .await
        .context("Simplify operation failed")?;

    // Assertions to verify results
    ensure!(
        !result.geometries().is_empty(),
        "Simplify returned empty result - expected simplified geometries"
    );

    ensure!(
        result.geometries().len() == 1,
        "Expected 1 simplified geometry, got {}",
        result.geometries().len()
    );

    // Verify we got a polygon back
    match &result.geometries()[0] {
        ArcGISGeometry::Polygon(simplified) => {
            ensure!(
                !simplified.rings().is_empty(),
                "Simplified polygon has no rings"
            );

            let original_ring_count = complex_polygon.rings().len();
            let simplified_ring_count = simplified.rings().len();

            debug!(
                original_rings = original_ring_count,
                simplified_rings = simplified_ring_count,
                "Simplification complete"
            );

            info!(
                "✓ Simplified polygon: {} ring(s) → {} ring(s)",
                original_ring_count, simplified_ring_count
            );
        }
        _ => anyhow::bail!("Expected polygon result from simplify, got different geometry type"),
    }

    Ok(())
}

/// Demonstrates union operation to merge multiple polygons.
///
/// Union combines multiple geometries into a single geometry,
/// useful for merging adjacent parcels or combining zones.
async fn demonstrate_union(service: &GeometryServiceClient<'_>) -> Result<()> {
    info!("\n=== Union Operation ===");

    // Create two adjacent polygons
    let polygon1 = ArcGISPolygon::new(vec![vec![
        vec![-118.0, 34.0],
        vec![-118.0, 34.05],
        vec![-117.95, 34.05],
        vec![-117.95, 34.0],
        vec![-118.0, 34.0],
    ]]);

    let polygon2 = ArcGISPolygon::new(vec![vec![
        vec![-117.95, 34.0],
        vec![-117.95, 34.05],
        vec![-117.90, 34.05],
        vec![-117.90, 34.0],
        vec![-117.95, 34.0],
    ]]);

    debug!("Created 2 adjacent polygons for union");

    let params = UnionParameters::builder()
        .geometries(vec![
            ArcGISGeometry::Polygon(polygon1),
            ArcGISGeometry::Polygon(polygon2),
        ])
        .sr(4326)
        .build()
        .context("Failed to build union params")?;

    let result = service
        .union(params)
        .await
        .context("Union operation failed")?;

    // Assertions to verify results
    match result.geometry() {
        ArcGISGeometry::Polygon(union_polygon) => {
            ensure!(
                !union_polygon.rings().is_empty(),
                "Union polygon has no rings"
            );

            // A union of two adjacent rectangles should produce one ring
            let ring_count = union_polygon.rings().len();
            debug!(ring_count = ring_count, "Union complete");

            ensure!(
                ring_count >= 1,
                "Expected at least 1 ring in union result, got {}",
                ring_count
            );

            info!(
                "✓ Union successful: 2 polygons merged into {} ring(s)",
                ring_count
            );
        }
        _ => anyhow::bail!("Expected polygon result from union, got different geometry type"),
    }

    Ok(())
}

/// Demonstrates calculating areas and lengths for polygons.
///
/// This operation is useful for accurate measurements, especially
/// when working with data in different coordinate systems.
async fn demonstrate_areas_and_lengths(service: &GeometryServiceClient<'_>) -> Result<()> {
    info!("\n=== Areas and Lengths Calculation ===");

    // Create a polygon representing approximately 1 degree by 1 degree square
    // (roughly 111km x 111km at the equator, smaller at higher latitudes)
    let test_polygon = ArcGISPolygon::new(vec![vec![
        vec![-118.0, 34.0],
        vec![-118.0, 35.0],
        vec![-117.0, 35.0],
        vec![-117.0, 34.0],
        vec![-118.0, 34.0],
    ]]);

    debug!("Created 1°×1° test polygon for area/length calculation");

    let params = AreasAndLengthsParameters::builder()
        .polygons(vec![ArcGISGeometry::Polygon(test_polygon)])
        .sr(4326)
        .length_unit(LinearUnit::Kilometers)
        .area_unit(AreaUnit::SquareKilometers)
        .calculation_type(CalculationType::Planar)
        .build()
        .context("Failed to build areas and lengths params")?;

    let result = service
        .areas_and_lengths(params)
        .await
        .context("Areas and lengths operation failed")?;

    // Assertions to verify results
    ensure!(
        !result.areas().is_empty(),
        "No areas returned - expected at least one area calculation"
    );

    ensure!(
        !result.lengths().is_empty(),
        "No lengths returned - expected at least one perimeter calculation"
    );

    ensure!(
        result.areas().len() == 1,
        "Expected 1 area, got {}",
        result.areas().len()
    );

    let area = result.areas()[0];
    let perimeter = result.lengths()[0];

    // Verify area is reasonable (should be positive and within expected range)
    ensure!(area > 0.0, "Area must be positive, got {}", area);

    // For a 1° square at ~34°N latitude:
    // - Expected area: roughly 111km * 92km = ~10,200 km²
    // - We'll allow a wide range since this is planar calculation
    ensure!(
        area > 1000.0 && area < 50000.0,
        "Area {} km² outside expected range (1,000-50,000) for 1° square",
        area
    );

    // Verify perimeter is reasonable
    ensure!(
        perimeter > 0.0,
        "Perimeter must be positive, got {}",
        perimeter
    );

    // Expected perimeter: roughly 2*(111km + 92km) = ~406 km
    ensure!(
        perimeter > 200.0 && perimeter < 1000.0,
        "Perimeter {} km outside expected range (200-1,000) for 1° square",
        perimeter
    );

    info!("✓ Area: {:.2} km², Perimeter: {:.2} km", area, perimeter);

    Ok(())
}

/// Demonstrates finding datum transformations and projecting with specific parameters.
///
/// This is crucial for accurate coordinate transformations between different
/// spatial reference systems, especially across different datums (e.g., NAD83 to WGS84).
async fn demonstrate_datum_transformations(service: &GeometryServiceClient<'_>) -> Result<()> {
    info!("\n=== Datum Transformations ===");

    // Find transformations from NAD83 (4269) to WGS84 (4326)
    // These are commonly used coordinate systems in North America
    let in_sr = 4269; // NAD83
    let out_sr = 4326; // WGS84
    let extent = ArcGISEnvelope::new(-120.0, 30.0, -115.0, 35.0).with_spatial_reference(Some(
        SpatialReference::builder()
            .wkid(4269u32)
            .build()
            .context("Failed to build spatial reference")?,
    ));

    debug!(
        in_sr = in_sr,
        out_sr = out_sr,
        "Finding available datum transformations"
    );

    let transformations = service
        .find_transformations(in_sr, out_sr, Some(extent))
        .await
        .context("Failed to find transformations")?;

    // Assertions to verify results
    ensure!(
        !transformations.is_empty(),
        "No transformations found between NAD83 (4269) and WGS84 (4326). \
         Expected at least one transformation method."
    );

    info!(
        "✓ Found {} transformation(s) from NAD83 to WGS84",
        transformations.len()
    );

    // Log available transformations
    for (i, transform) in transformations.iter().enumerate() {
        debug!(
            index = i,
            wkid = transform.wkid(),
            name = transform.name().as_deref().unwrap_or("unnamed"),
            "Available transformation"
        );
    }

    // Now demonstrate projecting with a specific transformation
    let test_point = ArcGISPoint::new(-118.0, 34.0).with_spatial_reference(Some(
        SpatialReference::builder()
            .wkid(4269u32)
            .build()
            .context("Failed to build NAD83 spatial reference")?,
    ));

    debug!(
        x = *test_point.x(),
        y = *test_point.y(),
        "Created NAD83 test point"
    );

    // Use the first available transformation
    let transformation_wkid = *transformations[0].wkid();

    let params = ProjectParameters::builder()
        .geometries(vec![ArcGISGeometry::Point(test_point.clone())])
        .in_sr(in_sr)
        .out_sr(out_sr)
        .transformation(transformation_wkid)
        .build()
        .context("Failed to build project params")?;

    let projected = service
        .project_with_params(params)
        .await
        .context("Project with params failed")?;

    // Assertions to verify projection results
    ensure!(
        !projected.geometries().is_empty(),
        "Projection returned no geometries - expected transformed point"
    );

    match &projected.geometries()[0] {
        ArcGISGeometry::Point(wgs84_point) => {
            let x_diff = (*wgs84_point.x() - *test_point.x()).abs();
            let y_diff = (*wgs84_point.y() - *test_point.y()).abs();

            // NAD83 to WGS84 transformations typically result in small shifts
            // (usually less than 1 meter, which is ~0.00001 degrees at this latitude)
            ensure!(
                x_diff < 0.01,
                "X coordinate shift {:.6}° too large - expected small datum shift",
                x_diff
            );

            ensure!(
                y_diff < 0.01,
                "Y coordinate shift {:.6}° too large - expected small datum shift",
                y_diff
            );

            // Verify coordinates are still in reasonable range (Southern California)
            ensure!(
                *wgs84_point.x() > -119.0 && *wgs84_point.x() < -117.0,
                "Projected X coordinate {:.6} outside expected range for Southern California",
                *wgs84_point.x()
            );

            ensure!(
                *wgs84_point.y() > 33.0 && *wgs84_point.y() < 35.0,
                "Projected Y coordinate {:.6} outside expected range for Southern California",
                *wgs84_point.y()
            );

            info!(
                "✓ Projected NAD83 ({:.6}, {:.6}) → WGS84 ({:.6}, {:.6})",
                *test_point.x(),
                *test_point.y(),
                *wgs84_point.x(),
                *wgs84_point.y()
            );

            info!(
                "  Shift: {:.8}° east, {:.8}° north (transformation WKID: {})",
                *wgs84_point.x() - *test_point.x(),
                *wgs84_point.y() - *test_point.y(),
                transformation_wkid
            );
        }
        _ => anyhow::bail!("Expected point result from projection, got different geometry type"),
    }

    Ok(())
}
