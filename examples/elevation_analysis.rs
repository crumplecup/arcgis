//! 🏔️ Elevation Analysis - Terrain Analysis with ArcGIS Elevation Services
//!
//! Demonstrates elevation profile generation using the ArcGIS Elevation Service.
//! Shows how to extract elevation data along hiking trails or transects, compute
//! terrain statistics, and analyze elevation changes.
//!
//! # What You'll Learn
//!
//! - **Elevation profiles**: Extract elevation along hiking trails or transects
//! - **Profile analysis**: Calculate min/max elevation and steepest segments
//! - **Distance calculations**: Compute cumulative distance along paths
//! - **DEM resolution**: Control analysis precision (10m, 30m, 90m, FINEST)
//!
//! # Prerequisites
//!
//! - API key with location services privileges (Tier 2+)
//! - Elevation Service credits (consumed per analysis request)
//!
//! ## Environment Variables
//!
//! Set in your `.env` file:
//!
//! ```env
//! ARCGIS_LOCATION_KEY=your_api_key_with_location_privileges
//! ```
//!
//! Or use the legacy key:
//!
//! ```env
//! ARCGIS_API_KEY=your_api_key
//! ```
//!
//! # Running
//!
//! ```bash
//! cargo run --example elevation_analysis
//!
//! # With debug logging to see all API calls:
//! RUST_LOG=debug cargo run --example elevation_analysis
//! ```
//!
//! # Real-World Use Cases
//!
//! - **Hiking apps**: Elevation profiles and trail difficulty assessment
//! - **Route planning**: Grade calculations and elevation gain analysis
//! - **Site planning**: Terrain cross-sections for construction
//! - **Transportation**: Road profile analysis and grade optimization
//! - **Environmental studies**: Watershed elevation transects
//! - **Cycling apps**: Climb categorization and route elevation profiles
//!
//! # Elevation Service
//!
//! The Elevation Service provides access to global DEM (Digital Elevation Model) data:
//! - **Global coverage**: SRTM, ASTER GDEM datasets
//! - **Multiple resolutions**: 10m, 30m, 90m, or FINEST (auto-select)
//! - **Profile generation**: Extract elevation along lines and transects
//! - **Accurate calculations**: Geodetic distance computations
//!
//! # Credit Usage
//!
//! ⚠️ Profile operations consume credits:
//! - **Profile**: ~0.5 credits per request
//!
//! Monitor your ArcGIS Online quota!
//!
//! # Note on Advanced Operations
//!
//! SummarizeElevation and Viewshed operations require ArcGIS Location Platform
//! with `premium:user:elevation` privileges and are not included in this example.

use anyhow::Result;
use arcgis::{ApiKeyAuth, ApiKeyTier, ArcGISClient, ElevationClient, ProfileParametersBuilder};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing for structured logging
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    tracing::info!("🏔️  Elevation Analysis Examples");
    tracing::info!("Terrain analysis with ArcGIS Elevation Services");
    tracing::info!("");

    // Create authenticated client (automatically loads .env)
    tracing::debug!("Creating authenticated client");
    let auth = ApiKeyAuth::from_env(ApiKeyTier::Location)?;
    let client = ArcGISClient::new(auth);
    let elevation = ElevationClient::new(&client);

    tracing::info!("✅ Authenticated with API key (ARCGIS_LOCATION_KEY)");
    tracing::info!("");

    // Demonstrate elevation operations
    demonstrate_elevation_profile(&elevation).await?;

    tracing::info!("\n✅ Elevation Profile example completed successfully!");
    print_best_practices();

    Ok(())
}

/// Demonstrates generating an elevation profile along a line.
async fn demonstrate_elevation_profile(elevation: &ElevationClient<'_>) -> Result<()> {
    tracing::info!("\n=== Example 1: Elevation Profile ===");
    tracing::info!("Extract elevation along a hiking trail transect");
    tracing::info!("");

    // Sierra Nevada trail segment (Yosemite area)
    // Create a proper FeatureSet with line geometry
    let trail_features = r#"{
        "geometryType": "esriGeometryPolyline",
        "features": [{
            "geometry": {
                "paths": [[[-119.65,37.85],[-119.60,37.87],[-119.55,37.85]]],
                "spatialReference": {"wkid": 4326}
            }
        }],
        "spatialReference": {"wkid": 4326}
    }"#;

    tracing::info!("   Trail transect: Yosemite National Park area");
    tracing::info!("     Start: -119.65°, 37.85° (west)");
    tracing::info!("     End: -119.55°, 37.85° (east)");
    tracing::info!("     Resolution: FINEST DEM");
    tracing::info!("");

    let params = ProfileParametersBuilder::default()
        .input_line_features(trail_features)
        .dem_resolution("FINEST")
        .build()?;

    let result = elevation.profile(params).await?;

    // Verify profile was generated successfully
    assert!(
        result.first_point_z().is_ok(),
        "Profile should have first point elevation"
    );
    assert!(
        result.last_point_z().is_ok(),
        "Profile should have last point elevation"
    );

    tracing::info!("✅ Elevation profile generated");

    if let Ok(first_z) = result.first_point_z() {
        tracing::info!("   Trail start elevation: {:.1} meters", first_z);
    }

    if let Ok(last_z) = result.last_point_z() {
        tracing::info!("   Trail end elevation: {:.1} meters", last_z);

        if let Ok(first_z) = result.first_point_z() {
            let gain = last_z - first_z;
            tracing::info!(
                "   Elevation change: {}{:.1} meters",
                if gain > 0.0 { "+" } else { "" },
                gain
            );
        }
    }

    // Extract typed elevation points using helper method
    let points = result.elevation_points()?;

    // Verify we got elevation profile points
    assert!(
        !points.is_empty(),
        "Elevation profile should contain points"
    );
    assert!(
        points.len() >= 2,
        "Profile should have at least 2 points (start and end), got {}",
        points.len()
    );

    tracing::info!("");
    tracing::info!("📊 Profile analysis ({} points):", points.len());

    // Find min/max elevation
    let min_point = points.iter().min_by(|a, b| {
        a.elevation_meters()
            .partial_cmp(b.elevation_meters())
            .unwrap()
    });
    let max_point = points.iter().max_by(|a, b| {
        a.elevation_meters()
            .partial_cmp(b.elevation_meters())
            .unwrap()
    });

    // Verify min and max points exist
    assert!(
        min_point.is_some(),
        "Should be able to find minimum elevation point"
    );
    assert!(
        max_point.is_some(),
        "Should be able to find maximum elevation point"
    );

    if let Some(min) = min_point {
        tracing::info!(
            "   Lowest point: {:.1}m at {:.0}m from start",
            min.elevation_meters(),
            min.distance_meters()
        );
    }

    if let Some(max) = max_point {
        tracing::info!(
            "   Highest point: {:.1}m at {:.0}m from start",
            max.elevation_meters(),
            max.distance_meters()
        );
    }

    // Calculate total distance
    if let Some(last) = points.last() {
        // Verify distance is reasonable (non-zero for a trail segment)
        assert!(
            *last.distance_meters() > 0.0,
            "Total distance should be positive, got: {}",
            last.distance_meters()
        );

        tracing::info!(
            "   Total distance: {:.1} kilometers",
            last.distance_meters() / 1000.0
        );
    }

    // Find steepest segment (largest elevation change between consecutive points)
    let steepest = points
        .windows(2)
        .map(|pair| {
            let distance_delta = pair[1].distance_meters() - pair[0].distance_meters();
            let elevation_delta = pair[1].elevation_meters() - pair[0].elevation_meters();
            let grade = if distance_delta > 0.0 {
                (elevation_delta / distance_delta).abs() * 100.0
            } else {
                0.0
            };
            (grade, pair[0].distance_meters(), elevation_delta)
        })
        .max_by(|a, b| a.0.partial_cmp(&b.0).unwrap());

    // Verify we found a steepest segment (we have at least 2 points)
    assert!(
        steepest.is_some(),
        "Should be able to find steepest segment with at least 2 points"
    );

    if let Some((grade, distance, _elevation_delta)) = steepest {
        tracing::info!(
            "   Steepest segment: {:.1}% grade at {:.0}m",
            grade,
            distance
        );
    }

    tracing::info!("");
    tracing::info!("💡 Profile data provides:");
    tracing::info!("   • Type-safe elevation points (no raw JSON parsing)");
    tracing::info!("   • Distance and elevation for each sample point");
    tracing::info!("   • Ready for charts, analysis, or further processing");
    tracing::info!("   • Use elevation_points() helper to extract typed data");

    Ok(())
}

/// Prints best practices for Elevation Service usage.
fn print_best_practices() {
    tracing::info!("\n💡 Elevation Service Best Practices:");
    tracing::info!("   - Choose appropriate DEM resolution for your scale");
    tracing::info!("   - Use FINEST for small areas, 90m for regional analysis");
    tracing::info!("   - Cache results for repeated analyses");
    tracing::info!("   - Monitor credit usage (check ArcGIS Online dashboard)");
    tracing::info!("");
    tracing::info!("🎯 Resolution Guidelines:");
    tracing::info!("   - 10m: Detailed site analysis, urban planning");
    tracing::info!("   - 30m: General terrain analysis, hiking trails");
    tracing::info!("   - 90m: Regional studies, large area statistics");
    tracing::info!("   - FINEST: Automatic selection (uses best available)");
    tracing::info!("");
    tracing::info!("⚡ Performance Tips:");
    tracing::info!("   - Simplify input geometries before analysis");
    tracing::info!("   - Profile short segments instead of entire trails");
    tracing::info!("   - Use appropriate resolution for your analysis scale");
    tracing::info!("");
    tracing::info!("💰 Credit Conservation:");
    tracing::info!("   - Profile: ~0.5 credits per request");
    tracing::info!("   - Use lower resolution for cost-sensitive applications");
    tracing::info!("   - Profile shorter segments to reduce computation");
    tracing::info!("");
    tracing::info!("📐 Coordinate Systems:");
    tracing::info!("   - Input geometries must include spatialReference");
    tracing::info!("   - WGS84 (4326) is common for global coordinates");
    tracing::info!("   - Web Mercator (3857) for web mapping");
    tracing::info!("   - Results returned in same coordinate system as input");
    tracing::info!("");
    tracing::info!("🔍 Common Use Cases:");
    tracing::info!("   - Trail difficulty assessment and elevation profiles");
    tracing::info!("   - Terrain cross-sections for construction planning");
    tracing::info!("   - Hiking route analysis and grade calculations");
}
