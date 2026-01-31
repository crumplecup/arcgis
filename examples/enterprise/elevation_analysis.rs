//! 🏔️ Elevation Analysis - Terrain Analysis with ArcGIS Elevation Services
//!
//! Demonstrates elevation and terrain analysis operations using the ArcGIS
//! Elevation Service. Shows how to generate elevation profiles, compute
//! terrain statistics, and perform viewshed visibility analysis.
//!
//! # What You'll Learn
//!
//! - **Elevation profiles**: Extract elevation along hiking trails or transects
//! - **Terrain statistics**: Calculate min/max/mean elevation for areas
//! - **Slope and aspect**: Terrain steepness and orientation analysis
//! - **Viewshed analysis**: Determine visible areas from observation points
//! - **DEM resolution**: Control analysis precision (10m, 30m, 90m)
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
//! - **Hiking apps**: Elevation profiles along trails
//! - **Site planning**: Terrain analysis for construction
//! - **Telecommunications**: Tower placement with viewshed analysis
//! - **Environmental studies**: Watershed elevation characteristics
//! - **Military operations**: Line-of-sight and visibility analysis
//! - **Solar analysis**: Terrain impact on solar exposure
//!
//! # Elevation Service
//!
//! The Elevation Service provides access to global DEM (Digital Elevation Model) data:
//! - **Global coverage**: SRTM, ASTER GDEM datasets
//! - **Multiple resolutions**: 10m, 30m, 90m, or FINEST (auto-select)
//! - **Terrain derivatives**: Slope, aspect, hillshade
//! - **Accurate calculations**: Geodetic distance and area computations
//!
//! # Credit Usage
//!
//! ⚠️ Elevation operations consume credits:
//! - **Profile**: ~0.5 credits per request
//! - **Summarize Elevation**: ~0.5 credits per request
//! - **Viewshed**: ~10 credits per request (varies by distance/area)
//!
//! Monitor your ArcGIS Online quota!

use anyhow::Result;
use arcgis::{
    ApiKeyAuth, ApiKeyTier, ArcGISClient, ElevationClient, ProfileParametersBuilder,
    SummarizeElevationParametersBuilder, ViewshedParametersBuilder,
};

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
    demonstrate_terrain_statistics(&elevation).await?;
    demonstrate_viewshed_analysis(&elevation).await?;

    tracing::info!("\n✅ All Elevation Service examples completed successfully!");
    print_best_practices();

    Ok(())
}

/// Demonstrates generating an elevation profile along a line.
async fn demonstrate_elevation_profile(elevation: &ElevationClient<'_>) -> Result<()> {
    tracing::info!("\n=== Example 1: Elevation Profile ===");
    tracing::info!("Extract elevation along a hiking trail transect");
    tracing::info!("");

    // Sierra Nevada trail segment (Yosemite area)
    // Coordinates: west to east transect across mountain terrain
    let trail_geometry = r#"{"paths":[[[-119.65,37.85],[-119.60,37.87],[-119.55,37.85]]],"spatialReference":{"wkid":4326}}"#;

    tracing::info!("   Trail transect: Yosemite National Park area");
    tracing::info!("     Start: -119.65°, 37.85° (west)");
    tracing::info!("     End: -119.55°, 37.85° (east)");
    tracing::info!("     Resolution: 30m DEM");
    tracing::info!("");

    let params = ProfileParametersBuilder::default()
        .input_geometry(trail_geometry)
        .geometry_type("esriGeometryPolyline")
        .dem_resolution("30m")
        .return_first_point(true)
        .return_last_point(true)
        .build()?;

    let result = elevation.profile(params).await?;

    tracing::info!("✅ Elevation profile generated");

    if let Some(first_z) = result.first_point_z() {
        tracing::info!("   Trail start elevation: {:.1} meters", first_z);
    }

    if let Some(last_z) = result.last_point_z() {
        tracing::info!("   Trail end elevation: {:.1} meters", last_z);

        if let Some(first_z) = result.first_point_z() {
            let gain = last_z - first_z;
            tracing::info!(
                "   Elevation change: {}{:.1} meters",
                if gain > 0.0 { "+" } else { "" },
                gain
            );
        }
    }

    tracing::info!("");
    tracing::info!("💡 Profile data contains:");
    tracing::info!("   • Elevation sampled at regular intervals");
    tracing::info!("   • Distance along path from start");
    tracing::info!("   • Useful for creating elevation charts");
    tracing::info!("   • output_profile contains full feature set");

    Ok(())
}

/// Demonstrates terrain statistics within a polygon area.
async fn demonstrate_terrain_statistics(elevation: &ElevationClient<'_>) -> Result<()> {
    tracing::info!("\n=== Example 2: Terrain Statistics ===");
    tracing::info!("Calculate elevation statistics for a wilderness area");
    tracing::info!("");

    // Mountain valley polygon in Sierra Nevada
    let valley_geometry = r#"{"rings":[[[-119.60,37.80],[-119.50,37.80],[-119.50,37.90],[-119.60,37.90],[-119.60,37.80]]],"spatialReference":{"wkid":4326}}"#;

    tracing::info!("   Analysis area: Sierra Nevada wilderness");
    tracing::info!("     Bounding box: ~10km × ~10km");
    tracing::info!("     Resolution: 30m DEM");
    tracing::info!("     Including slope and aspect");
    tracing::info!("");

    let params = SummarizeElevationParametersBuilder::default()
        .input_geometry(valley_geometry)
        .geometry_type("esriGeometryPolygon")
        .dem_resolution("30m")
        .include_slope(true)
        .include_aspect(true)
        .build()?;

    let result = elevation.summarize_elevation(params).await?;

    tracing::info!("✅ Terrain statistics calculated");
    tracing::info!("");

    if let Some(min) = result.min_elevation() {
        tracing::info!("   Minimum elevation: {:.1} meters", min);
    }

    if let Some(max) = result.max_elevation() {
        tracing::info!("   Maximum elevation: {:.1} meters", max);
    }

    if let Some(mean) = result.mean_elevation() {
        tracing::info!("   Mean elevation: {:.1} meters", mean);
    }

    if let (Some(min), Some(max)) = (result.min_elevation(), result.max_elevation()) {
        tracing::info!("   Elevation range: {:.1} meters", max - min);
    }

    if let Some(area) = result.area() {
        tracing::info!(
            "   Area analyzed: {:.2} square kilometers",
            area / 1_000_000.0
        );
    }

    tracing::info!("");
    tracing::info!("💡 Summary statistics include:");
    tracing::info!("   • Min/max/mean elevation");
    tracing::info!("   • Area in square meters");
    tracing::info!("   • Optional: slope statistics (degrees)");
    tracing::info!("   • Optional: aspect statistics (compass direction)");
    tracing::info!("   • Useful for site suitability analysis");

    Ok(())
}

/// Demonstrates viewshed visibility analysis.
async fn demonstrate_viewshed_analysis(elevation: &ElevationClient<'_>) -> Result<()> {
    tracing::info!("\n=== Example 3: Viewshed Analysis ===");
    tracing::info!("Compute visible areas from a mountain lookout point");
    tracing::info!("");

    // Mountain peak observation point (Sierra Nevada)
    let lookout_geometry = r#"{"points":[[-119.58,37.85]],"spatialReference":{"wkid":4326}}"#;

    tracing::info!("   Observer location: Sierra Nevada peak");
    tracing::info!("     Coordinates: -119.58°, 37.85°");
    tracing::info!("     Observer height: 2.0 meters (6.5 feet)");
    tracing::info!("     Maximum distance: 10,000 meters (10 km)");
    tracing::info!("     Resolution: 90m DEM (faster analysis)");
    tracing::info!("");

    let params = ViewshedParametersBuilder::default()
        .input_points(lookout_geometry)
        .geometry_type("esriGeometryMultipoint")
        .maximum_distance(10000.0)
        .observer_height(2.0)
        .dem_resolution("90m")
        .generalize(true)
        .build()?;

    let result = elevation.viewshed(params).await?;

    tracing::info!("✅ Viewshed analysis completed");
    tracing::info!("");

    if let Some(visible_area) = result.visible_area() {
        tracing::info!(
            "   Visible area: {:.2} square kilometers",
            visible_area / 1_000_000.0
        );
    }

    if let Some(total_area) = result.total_area() {
        tracing::info!(
            "   Total analyzed area: {:.2} square kilometers",
            total_area / 1_000_000.0
        );
    }

    if let (Some(visible), Some(total)) = (result.visible_area(), result.total_area()) {
        let percent = (visible / total) * 100.0;
        tracing::info!("   Visibility coverage: {:.1}%", percent);
    }

    tracing::info!("");
    tracing::info!("💡 Viewshed analysis provides:");
    tracing::info!("   • Visible area polygons (output_viewshed)");
    tracing::info!("   • Visible area in square meters");
    tracing::info!("   • Total area analyzed");
    tracing::info!("   • Useful for tower placement, surveillance, scenic viewpoints");
    tracing::info!("   • Can analyze multiple observer points simultaneously");

    Ok(())
}

/// Prints best practices for Elevation Service usage.
fn print_best_practices() {
    tracing::info!("\n💡 Elevation Service Best Practices:");
    tracing::info!("   - Choose appropriate DEM resolution for your scale");
    tracing::info!("   - Use FINEST for small areas, 90m for regional analysis");
    tracing::info!("   - Limit viewshed distance to reduce credit consumption");
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
    tracing::info!("   - Use generalize=true for viewsheds (simpler polygons)");
    tracing::info!("   - Profile short segments instead of entire trails");
    tracing::info!("   - Batch multiple points for viewshed (single request)");
    tracing::info!("");
    tracing::info!("💰 Credit Conservation:");
    tracing::info!("   - Profile: ~0.5 credits per request");
    tracing::info!("   - Summarize Elevation: ~0.5 credits per request");
    tracing::info!("   - Viewshed: ~10 credits (varies by distance/area)");
    tracing::info!("   - Viewshed distance impacts credits significantly");
    tracing::info!("   - Use lower resolution for cost-sensitive applications");
    tracing::info!("");
    tracing::info!("📐 Coordinate Systems:");
    tracing::info!("   - Input geometries must include spatialReference");
    tracing::info!("   - WGS84 (4326) is common for global coordinates");
    tracing::info!("   - Web Mercator (3857) for web mapping");
    tracing::info!("   - Results returned in same coordinate system as input");
    tracing::info!("");
    tracing::info!("🔍 Common Use Cases:");
    tracing::info!("   - Profile: Trail difficulty assessment, cross-sections");
    tracing::info!("   - Summarize: Watershed characteristics, site selection");
    tracing::info!("   - Viewshed: Tower coverage, scenic overlooks, surveillance");
}
