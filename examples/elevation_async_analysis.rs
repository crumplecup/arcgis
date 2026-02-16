//! 🏔️ Async Elevation Analysis - Advanced Terrain Analysis with Geoprocessing
//!
//! Demonstrates asynchronous elevation analysis using ArcGIS Elevation Service
//! geoprocessing tasks. Shows how to submit long-running terrain analysis jobs,
//! poll for completion, and extract typed results.
//!
//! # What You'll Learn
//!
//! - **Async geoprocessing**: Submit elevation jobs for background processing
//! - **Job monitoring**: Poll job status and track progress
//! - **Typed results**: Extract elevation statistics using type-safe parsers
//! - **Summarize elevation**: Get min/mean/max elevation, slope, and aspect statistics
//! - **Viewshed analysis**: Calculate visibility from observation points
//!
//! # Prerequisites
//!
//! - API key with **Location Platform** access and `premium:user:elevation` privileges
//! - Elevation Service credits (consumed per analysis request)
//!
//! ## Environment Variables
//!
//! Set in your `.env` file:
//!
//! ```env
//! ARCGIS_LOCATION_KEY=your_location_platform_api_key
//! ```
//!
//! Or use the legacy key:
//!
//! ```env
//! ARCGIS_API_KEY=your_api_key_with_premium_elevation
//! ```
//!
//! # Running
//!
//! ```bash
//! cargo run --example elevation_async_analysis
//!
//! # With debug logging to see all API calls:
//! RUST_LOG=debug cargo run --example elevation_async_analysis
//! ```
//!
//! # Real-World Use Cases
//!
//! - **Site planning**: Terrain statistics for construction feasibility
//! - **Viewshed analysis**: Cell tower placement and line-of-sight studies
//! - **Environmental studies**: Watershed elevation and slope analysis
//! - **Military planning**: Visibility analysis and terrain assessment
//! - **Solar analysis**: Aspect/slope for solar panel placement
//! - **Flood modeling**: Terrain statistics for watershed analysis
//!
//! # Async Elevation Services
//!
//! The Elevation Service provides async geoprocessing tasks for complex analysis:
//! - **SummarizeElevation**: Extract terrain statistics (elevation, slope, aspect)
//! - **Viewshed**: Calculate visibility from observation points
//!
//! Both operations:
//! - Submit job → Poll status → Retrieve typed results
//! - Support multiple DEM resolutions (FINEST, 10m, 30m, 90m)
//! - Return strongly-typed results (no raw JSON parsing)
//! - Include comprehensive terrain statistics
//!
//! # Credit Usage
//!
//! ⚠️ Premium elevation operations consume credits:
//! - **SummarizeElevation**: ~5 credits per request
//! - **Viewshed**: ~10 credits per request
//!
//! Monitor your ArcGIS Location Platform quota!
//!
//! # Differences from elevation_analysis.rs
//!
//! - **Profile** (sync): Elevation along a line, instant results
//! - **SummarizeElevation** (async): Statistics for polygon areas, job-based
//! - **Viewshed** (async): Visibility analysis, job-based

use anyhow::Result;
use arcgis::{
    ApiKeyAuth, ApiKeyTier, ArcGISClient, ElevationClient, SummarizeElevationParametersBuilder,
    ViewshedParametersBuilder,
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

    tracing::info!("🏔️  Async Elevation Analysis Examples");
    tracing::info!("Advanced terrain analysis with ArcGIS Elevation Geoprocessing Services");
    tracing::info!("");

    // Create authenticated client (automatically loads .env)
    tracing::debug!("Creating authenticated client");
    let auth = ApiKeyAuth::agol(ApiKeyTier::Location)?;
    let client = ArcGISClient::new(auth);
    let elevation = ElevationClient::new(&client);

    tracing::info!("✅ Authenticated with Location Platform API key");
    tracing::info!("");

    // Demonstrate async elevation operations
    demonstrate_summarize_elevation(&elevation).await?;
    demonstrate_viewshed(&elevation).await?;

    tracing::info!("\n✅ Async elevation examples completed successfully!");
    print_best_practices();

    Ok(())
}

/// Demonstrates asynchronous terrain statistics extraction.
async fn demonstrate_summarize_elevation(elevation: &ElevationClient<'_>) -> Result<()> {
    tracing::info!("\n=== Example 1: Summarize Elevation (Async) ===");
    tracing::info!("Extract terrain statistics for a polygon area");
    tracing::info!("");
    tracing::info!("📊 What is SummarizeElevation?");
    tracing::info!("   Computes comprehensive terrain statistics for polygon areas:");
    tracing::info!("   • Elevation: min/mean/max values");
    tracing::info!("   • Slope: min/mean/max gradient (degrees)");
    tracing::info!("   • Aspect: mean orientation (degrees from north)");
    tracing::info!("");
    tracing::info!("🎯 Analysis Target:");
    tracing::info!("   Area: Yosemite Valley region");
    tracing::info!("   Purpose: Terrain assessment for trail planning");
    tracing::info!("   Resolution: FINEST available DEM");

    // Yosemite Valley area polygon
    let valley_polygon = r#"{
        "geometryType": "esriGeometryPolygon",
        "features": [{
            "geometry": {
                "rings": [
                    [
                        [-119.65, 37.72],
                        [-119.50, 37.72],
                        [-119.50, 37.77],
                        [-119.65, 37.77],
                        [-119.65, 37.72]
                    ]
                ],
                "spatialReference": {"wkid": 4326}
            },
            "attributes": {
                "Name": "Yosemite Valley Study Area"
            }
        }],
        "spatialReference": {"wkid": 4326}
    }"#;

    tracing::info!("");
    tracing::info!("📤 Submitting SummarizeElevation job to server...");

    let params = SummarizeElevationParametersBuilder::default()
        .input_features(valley_polygon)
        .dem_resolution("FINEST")
        .include_slope_aspect(true)
        .build()?;

    let job_info = elevation.submit_summarize_elevation(params).await?;

    tracing::info!(
        job_id = %job_info.job_id(),
        status = ?job_info.job_status(),
        "✅ Job submitted"
    );

    // Poll until complete
    tracing::info!("⏳ Polling job status until completion...");
    tracing::info!("   (Terrain analysis typically takes 10-30 seconds)");

    let start_time = std::time::Instant::now();
    let result = elevation
        .poll_summarize_elevation(job_info.job_id(), Some(120000))
        .await?;
    let elapsed = start_time.elapsed();

    tracing::info!("✅ Job completed in {:.1} seconds", elapsed.as_secs_f64());
    tracing::info!("");

    // Display typed results
    tracing::info!("📊 Terrain Statistics for Yosemite Valley:");

    if let Some(min) = result.min_elevation() {
        tracing::info!("   Elevation (meters):");
        tracing::info!("      • Minimum: {:.1}m", min);
    }
    if let Some(mean) = result.mean_elevation() {
        tracing::info!("      • Mean: {:.1}m", mean);
    }
    if let Some(max) = result.max_elevation() {
        tracing::info!("      • Maximum: {:.1}m", max);
    }

    if let (Some(min), Some(max)) = (result.min_elevation(), result.max_elevation()) {
        let relief = max - min;
        tracing::info!("      • Relief (max - min): {:.1}m", relief);
    }

    if result.min_slope().is_some() {
        tracing::info!("");
        tracing::info!("   Slope (degrees):");
        if let Some(min) = result.min_slope() {
            tracing::info!("      • Minimum: {:.1}°", min);
        }
        if let Some(mean) = result.mean_slope() {
            tracing::info!("      • Mean: {:.1}°", mean);
        }
        if let Some(max) = result.max_slope() {
            tracing::info!("      • Maximum: {:.1}°", max);
        }
    }

    if let Some(aspect) = result.mean_aspect() {
        tracing::info!("");
        tracing::info!("   Aspect:");
        tracing::info!("      • Mean direction: {:.1}° from north", aspect);

        let cardinal = match *aspect {
            a if !(22.5..337.5).contains(&a) => "North",
            a if (22.5..67.5).contains(&a) => "Northeast",
            a if (67.5..112.5).contains(&a) => "East",
            a if (112.5..157.5).contains(&a) => "Southeast",
            a if (157.5..202.5).contains(&a) => "South",
            a if (202.5..247.5).contains(&a) => "Southwest",
            a if (247.5..292.5).contains(&a) => "West",
            a if (292.5..337.5).contains(&a) => "Northwest",
            _ => "Unknown",
        };
        tracing::info!("      • Cardinal direction: {}", cardinal);
    }

    tracing::info!("");
    tracing::info!("💡 Interpreting These Statistics:");
    tracing::info!("   • High relief = mountainous terrain (challenging trails)");
    tracing::info!("   • Mean slope > 15° = steep terrain (advanced trails only)");
    tracing::info!("   • Aspect affects sun exposure, snow retention, vegetation");
    tracing::info!("   • North-facing slopes retain snow longer (ski areas)");
    tracing::info!("   • South-facing slopes get more sun (solar placement)");

    tracing::info!("");
    tracing::info!("🎯 Use Cases:");
    tracing::info!("   ✓ Trail planning: Identify suitable gradients");
    tracing::info!("   ✓ Construction: Site feasibility assessment");
    tracing::info!("   ✓ Solar analysis: Optimal panel orientation");
    tracing::info!("   ✓ Watershed: Runoff and drainage modeling");

    Ok(())
}

/// Demonstrates asynchronous viewshed (visibility) analysis.
async fn demonstrate_viewshed(elevation: &ElevationClient<'_>) -> Result<()> {
    tracing::info!("\n=== Example 2: Viewshed Analysis (Async) ===");
    tracing::info!("Calculate visible areas from observation points");
    tracing::info!("");
    tracing::info!("📊 What is Viewshed?");
    tracing::info!("   Identifies terrain visible from observation point(s):");
    tracing::info!("   • Cell tower placement: Maximize coverage area");
    tracing::info!("   • Scenic overlooks: Quantify view quality");
    tracing::info!("   • Military: Line-of-sight analysis");
    tracing::info!("   • Environmental: Visual impact assessments");
    tracing::info!("");
    tracing::info!("🎯 Analysis Parameters:");
    tracing::info!("   Observer: Mountain summit viewpoint");
    tracing::info!("   Location: Yosemite high country");
    tracing::info!("   Max distance: 10 km radius");
    tracing::info!("   Observer height: 2m above ground (person standing)");

    // Observation point (Yosemite high country)
    let observation_point = r#"{
        "geometryType": "esriGeometryPoint",
        "features": [{
            "geometry": {
                "x": -119.60,
                "y": 37.75,
                "spatialReference": {"wkid": 4326}
            },
            "attributes": {
                "Name": "Summit Viewpoint"
            }
        }],
        "spatialReference": {"wkid": 4326}
    }"#;

    tracing::info!("");
    tracing::info!("📤 Submitting Viewshed job to server...");

    let params = ViewshedParametersBuilder::default()
        .input_points(observation_point)
        .maximum_distance(10.0)
        .maximum_distance_units("Kilometers")
        .dem_resolution("FINEST")
        .observer_height(2.0)
        .observer_height_units("Meters")
        .generalize_viewshed_polygons(true)
        .build()?;

    let job_info = elevation.submit_viewshed(params).await?;

    tracing::info!(
        job_id = %job_info.job_id(),
        status = ?job_info.job_status(),
        "✅ Job submitted"
    );

    // Poll until complete
    tracing::info!("⏳ Polling job status until completion...");
    tracing::info!("   (Viewshed analysis typically takes 20-60 seconds)");

    let start_time = std::time::Instant::now();
    let result = elevation
        .poll_viewshed(job_info.job_id(), Some(180000))
        .await?;
    let elapsed = start_time.elapsed();

    tracing::info!("✅ Job completed in {:.1} seconds", elapsed.as_secs_f64());
    tracing::info!("");

    // Display viewshed results
    tracing::info!("📊 Viewshed Analysis Results:");

    let viewshed_features = result.output_viewshed();
    let feature_count = viewshed_features.features().len();

    tracing::info!("   Features returned: {}", feature_count);

    if feature_count > 0 {
        tracing::info!("");
        tracing::info!("   ✅ Viewshed polygon generated successfully");
        tracing::info!("   💡 The returned polygon shows:");
        tracing::info!("      • All terrain visible from observation point");
        tracing::info!("      • Areas within 10km that have line-of-sight");
        tracing::info!("      • Accounts for Earth's curvature");
        tracing::info!("      • Considers terrain elevation blocking");

        // Calculate approximate visible area if polygon has attributes
        if let Some(first_feature) = viewshed_features.features().first() {
            if let Some(shape_area) = first_feature
                .attributes()
                .get("Shape_Area")
                .and_then(|v| v.as_f64())
            {
                tracing::info!("");
                tracing::info!("   📏 Visibility Statistics:");
                tracing::info!("      • Visible area: {:.2} sq meters", shape_area);
                tracing::info!(
                    "      • Visible area: {:.2} sq kilometers",
                    shape_area / 1_000_000.0
                );

                // Calculate percentage of analysis area (circle with 10km radius)
                let analysis_area = std::f64::consts::PI * 10000.0 * 10000.0; // π * r²
                let visibility_pct = (shape_area / analysis_area) * 100.0;
                tracing::info!(
                    "      • Visibility coverage: {:.1}% of 10km radius",
                    visibility_pct
                );
            }
        }
    } else {
        tracing::warn!("   ⚠️  No viewshed features returned");
        tracing::warn!("      Possible reasons:");
        tracing::warn!("      • Observation point too low");
        tracing::warn!("      • Maximum distance too small");
        tracing::warn!("      • Terrain completely blocks view");
    }

    tracing::info!("");
    tracing::info!("💡 Interpreting Viewshed Results:");
    tracing::info!("   • Large visible area = Good observation point");
    tracing::info!("   • Small visible area = Terrain obstructs views");
    tracing::info!("   • Fragmented polygons = Complex terrain (valleys/ridges)");
    tracing::info!("   • Use for tower placement: Maximize coverage");
    tracing::info!("   • Increase observer_height for radio towers (30-50m)");

    tracing::info!("");
    tracing::info!("🎯 Real-World Applications:");
    tracing::info!("   ✓ Cell tower placement: Maximize coverage area");
    tracing::info!("   ✓ Wind turbines: Minimize visual impact");
    tracing::info!("   ✓ Fire lookout towers: Optimize detection range");
    tracing::info!("   ✓ Scenic overlooks: Quantify view quality");
    tracing::info!("   ✓ Security: Surveillance camera placement");

    Ok(())
}

/// Prints best practices for async elevation analysis.
fn print_best_practices() {
    tracing::info!("\n💡 Async Elevation Analysis Best Practices:");
    tracing::info!("   - Use async operations for large areas or complex analysis");
    tracing::info!("   - Poll with reasonable intervals (2-5 seconds)");
    tracing::info!("   - Set appropriate timeouts (2-3 minutes typical)");
    tracing::info!("   - Handle job failures gracefully");
    tracing::info!("   - Monitor credit usage (check Location Platform dashboard)");
    tracing::info!("");
    tracing::info!("🎯 Resolution Guidelines:");
    tracing::info!("   - FINEST: Best available resolution (automatic)");
    tracing::info!("   - 10m: Detailed site analysis, small areas");
    tracing::info!("   - 30m: General terrain analysis, medium areas");
    tracing::info!("   - 90m: Regional studies, large areas");
    tracing::info!("");
    tracing::info!("⚡ Performance Tips:");
    tracing::info!("   - Use appropriate resolution for your scale");
    tracing::info!("   - Smaller areas process faster");
    tracing::info!("   - Viewshed: Limit maximum distance to necessary range");
    tracing::info!("   - Cache results for repeated analyses");
    tracing::info!("");
    tracing::info!("💰 Credit Conservation:");
    tracing::info!("   - SummarizeElevation: ~5 credits per request");
    tracing::info!("   - Viewshed: ~10 credits per request");
    tracing::info!("   - Use lower resolution for cost-sensitive applications");
    tracing::info!("   - Analyze smaller areas when possible");
    tracing::info!("");
    tracing::info!("📐 Coordinate Systems:");
    tracing::info!("   - Input must include spatialReference");
    tracing::info!("   - WGS84 (4326) recommended for global data");
    tracing::info!("   - Results returned in input coordinate system");
    tracing::info!("");
    tracing::info!("🔍 Common Issues:");
    tracing::info!("   - Job timeout: Increase timeout_ms parameter");
    tracing::info!("   - Empty results: Check input geometry validity");
    tracing::info!("   - Credit errors: Verify Location Platform privileges");
    tracing::info!("   - Viewshed empty: Increase maximum_distance or observer_height");
}
