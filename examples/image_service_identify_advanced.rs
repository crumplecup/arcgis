//! 🔍 Advanced Image Service Identification - Custom Parameters & Options
//!
//! Demonstrates advanced identify operations with custom parameters for fine-grained
//! control over pixel value queries. Learn how to use return options, spatial references,
//! and mosaic rules to customize identify behavior.
//!
//! # What You'll Learn
//!
//! - **identify_with_params**: Use IdentifyParameters for advanced options
//! - **Return options**: Control geometry and catalog item returns
//! - **Spatial references**: Specify input geometry coordinate systems
//! - **Custom parameters**: Fine-tune identify behavior for your use case
//!
//! # Prerequisites
//!
//! - No authentication required (uses public NLCD sample service)
//! - For enterprise image services, set ARCGIS_API_KEY in `.env`
//!
//! ## Environment Variables (Optional)
//!
//! ```env
//! ARCGIS_API_KEY=your_api_key_here  # Optional: for enterprise services
//! ```
//!
//! # Running
//!
//! ```bash
//! cargo run --example image_service_identify_advanced
//!
//! # With debug logging to see all requests:
//! RUST_LOG=debug cargo run --example image_service_identify_advanced
//! ```
//!
//! # Real-World Use Cases
//!
//! - **Multi-image catalogs**: Query catalog items for mosaic datasets
//! - **Coordinate system handling**: Work with data in different projections
//! - **Geometry returns**: Get exact pixel locations for downstream analysis
//! - **Custom mosaic rules**: Control which images are used in multi-image services
//! - **Rendering rules**: Apply on-the-fly analysis (NDVI, slope) before identify

use anyhow::Result;
use arcgis::{
    ArcGISClient, ArcGISGeometry, IdentifyParametersBuilder, ImageServiceClient, NoAuth,
    geo_types::{Geometry, Point},
};

/// Public NLCD Land Cover 2001 Image Service (no auth required).
///
/// This service contains the National Land Cover Database (NLCD) 2001
/// land cover classification for the contiguous United States.
const NLCD_IMAGE_SERVICE: &str =
    "https://sampleserver6.arcgisonline.com/arcgis/rest/services/NLCDLandCover2001/ImageServer";

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing for structured logging
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    tracing::info!("🔍 Advanced Image Service Identification Examples");
    tracing::info!("Using NLCD Land Cover 2001 public service");
    tracing::info!("");

    // Create client with NoAuth (public service)
    let client = ArcGISClient::new(NoAuth);
    let image_service = ImageServiceClient::new(NLCD_IMAGE_SERVICE, &client);

    // Demonstrate advanced identify operations
    demonstrate_basic_params(&image_service).await?;
    demonstrate_return_options(&image_service).await?;
    demonstrate_spatial_reference(&image_service).await?;

    tracing::info!("\n✅ All advanced identify examples completed successfully!");
    print_best_practices();

    Ok(())
}

/// Demonstrates basic identify_with_params usage.
async fn demonstrate_basic_params(service: &ImageServiceClient<'_>) -> Result<()> {
    tracing::info!("=== Example 1: Basic identify_with_params ===");
    tracing::info!("Identify with custom parameters for more control");
    tracing::info!("");

    // Redlands, California (where ESRI headquarters is located)
    let point = Point::new(-117.1825, 34.0555);
    let geom: Geometry = point.into();
    let geometry: ArcGISGeometry = geom.into();
    let geometry_json = serde_json::to_string(&geometry)?;

    let params = IdentifyParametersBuilder::default()
        .geometry(geometry_json)
        .geometry_type("esriGeometryPoint")
        .build()
        .expect("Valid identify parameters");

    let result = service.identify_with_params(params).await?;

    // Validate we got a result
    anyhow::ensure!(
        result.value().is_some(),
        "Identify should return a pixel value"
    );

    tracing::info!("📍 Pixel Value at Redlands, CA:");
    if let Some(value) = result.value() {
        tracing::info!("   Land Cover Class: {}", value);
        tracing::info!("   Location: Lon {}, Lat {}", point.x(), point.y());

        // Validate value is a reasonable NLCD class code
        if let Ok(class_code) = value.parse::<i32>() {
            anyhow::ensure!(
                (11..=95).contains(&class_code),
                "NLCD class code should be 11-95, got {}",
                class_code
            );
            tracing::info!("   ✅ Valid NLCD class code: {}", class_code);
        }
    }

    tracing::info!("");
    tracing::info!("💡 Use case: Basic identify with explicit parameters");
    tracing::info!("   • Full control over request parameters");
    tracing::info!("   • Same as basic identify() but extensible");

    Ok(())
}

/// Demonstrates return options (geometry, catalog items).
async fn demonstrate_return_options(service: &ImageServiceClient<'_>) -> Result<()> {
    tracing::info!("\n=== Example 2: Return Options ===");
    tracing::info!("Control what information is returned in identify results");
    tracing::info!("");

    // San Francisco location
    let point = Point::new(-122.4194, 37.7749);
    let geom: Geometry = point.into();
    let geometry: ArcGISGeometry = geom.into();
    let geometry_json = serde_json::to_string(&geometry)?;

    // Request with return_geometry enabled
    let params_with_geom = IdentifyParametersBuilder::default()
        .geometry(geometry_json.clone())
        .geometry_type("esriGeometryPoint")
        .return_geometry(true)
        .build()
        .expect("Valid identify parameters");

    let result_with_geom = service.identify_with_params(params_with_geom).await?;

    anyhow::ensure!(
        result_with_geom.value().is_some(),
        "Should return pixel value"
    );

    tracing::info!("📐 Result with return_geometry=true:");
    if let Some(value) = result_with_geom.value() {
        tracing::info!("   Land Cover Class: {}", value);
    }

    if let Some(location) = result_with_geom.location() {
        tracing::info!("   ✅ Location geometry returned: {:?}", location);
    } else {
        tracing::info!("   ℹ️  No location geometry (service may not support)");
    }

    // Request without return_geometry
    let params_no_geom = IdentifyParametersBuilder::default()
        .geometry(geometry_json)
        .geometry_type("esriGeometryPoint")
        .return_geometry(false)
        .build()
        .expect("Valid identify parameters");

    let result_no_geom = service.identify_with_params(params_no_geom).await?;

    anyhow::ensure!(
        result_no_geom.value().is_some(),
        "Should return pixel value"
    );

    tracing::info!("");
    tracing::info!("📊 Result with return_geometry=false:");
    if let Some(value) = result_no_geom.value() {
        tracing::info!("   Land Cover Class: {}", value);
    }
    if result_no_geom.location().is_some() {
        tracing::info!("   ℹ️  Location returned even though disabled");
    } else {
        tracing::info!("   ✅ No location geometry (as requested)");
    }

    tracing::info!("");
    tracing::info!("💡 Use cases:");
    tracing::info!("   • return_geometry=true: Need exact pixel location for mapping");
    tracing::info!("   • return_geometry=false: Only pixel values needed (performance)");
    tracing::info!("   • return_catalog_items=true: Multi-image services (mosaics)");

    Ok(())
}

/// Demonstrates spatial reference handling.
async fn demonstrate_spatial_reference(service: &ImageServiceClient<'_>) -> Result<()> {
    tracing::info!("\n=== Example 3: Spatial Reference Handling ===");
    tracing::info!("Specify input geometry coordinate system explicitly");
    tracing::info!("");

    // Point in Web Mercator (EPSG:3857) - Redlands, CA
    // Convert from WGS84 (-117.1825, 34.0555) to Web Mercator
    let point_wm = Point::new(-13056000.0, 4039000.0);
    let geom: Geometry = point_wm.into();
    let geometry: ArcGISGeometry = geom.into();
    let geometry_json = serde_json::to_string(&geometry)?;

    // Identify with explicit Web Mercator SR
    let params = IdentifyParametersBuilder::default()
        .geometry(geometry_json)
        .geometry_type("esriGeometryPoint")
        .geometry_sr(3857u32) // Web Mercator
        .build()
        .expect("Valid identify parameters");

    let result = service.identify_with_params(params).await?;

    anyhow::ensure!(
        result.value().is_some(),
        "Should return pixel value for Web Mercator input"
    );

    tracing::info!("🌐 Result with Web Mercator input (EPSG:3857):");
    if let Some(value) = result.value() {
        tracing::info!("   Land Cover Class: {}", value);
        tracing::info!(
            "   Input Coords (Web Mercator): x={}, y={}",
            point_wm.x(),
            point_wm.y()
        );
        tracing::info!("   ✅ geometry_sr parameter accepted by service");

        // NoData is valid for out-of-bounds, but numeric classes should be in range
        if value != "NoData" {
            if let Ok(class_code) = value.parse::<i32>() {
                anyhow::ensure!(
                    (11..=95).contains(&class_code),
                    "NLCD class code should be 11-95, got {}",
                    class_code
                );
            }
        }
    }

    // Now test with WGS84 (EPSG:4326) - same location
    let point_wgs84 = Point::new(-118.25, 34.05);
    let geom2: Geometry = point_wgs84.into();
    let geometry2: ArcGISGeometry = geom2.into();
    let geometry_json2 = serde_json::to_string(&geometry2)?;

    let params2 = IdentifyParametersBuilder::default()
        .geometry(geometry_json2)
        .geometry_type("esriGeometryPoint")
        .geometry_sr(4326u32) // WGS84
        .build()
        .expect("Valid identify parameters");

    let result2 = service.identify_with_params(params2).await?;

    anyhow::ensure!(
        result2.value().is_some(),
        "Should return pixel value for WGS84 input"
    );

    tracing::info!("");
    tracing::info!("🌍 Result with WGS84 input (EPSG:4326):");
    if let Some(value) = result2.value() {
        tracing::info!("   Land Cover Class: {}", value);
        tracing::info!(
            "   Input Coords (WGS84): lon={}, lat={}",
            point_wgs84.x(),
            point_wgs84.y()
        );
    }

    tracing::info!("");
    tracing::info!("💡 Use cases:");
    tracing::info!("   • Specify geometry_sr when input is not WGS84");
    tracing::info!("   • Essential for Web Mercator data (web maps)");
    tracing::info!("   • Ensures correct coordinate transformation");
    tracing::info!("   • Prevents misinterpretation of coordinates");

    Ok(())
}

/// Prints best practices for advanced identify operations.
fn print_best_practices() {
    tracing::info!("\n💡 Advanced Identify Best Practices:");
    tracing::info!("   - Use identify_with_params() when you need fine-grained control");
    tracing::info!("   - Basic identify() is a convenience wrapper (WGS84 only)");
    tracing::info!("   - Always specify geometry_sr for non-WGS84 inputs");
    tracing::info!("   - Set return_geometry=false if you don't need location geometry");
    tracing::info!("   - Use return_catalog_items=true for mosaic datasets");
    tracing::info!("");
    tracing::info!("🎯 Parameter Selection:");
    tracing::info!("   - geometry_sr: Match your input coordinate system");
    tracing::info!("   - return_geometry: true = locations, false = values only");
    tracing::info!("   - return_catalog_items: true for multi-image services");
    tracing::info!("   - mosaic_rule: Control which images are queried");
    tracing::info!("   - rendering_rule: Apply analysis before identify");
    tracing::info!("");
    tracing::info!("⚡ Performance Tips:");
    tracing::info!("   - return_geometry=false reduces response size");
    tracing::info!("   - Batch multiple nearby points when possible");
    tracing::info!("   - Use appropriate spatial reference for your region");
    tracing::info!("   - Consider caching results for static imagery");
    tracing::info!("");
    tracing::info!("🔍 When to Use Advanced Parameters:");
    tracing::info!("   • Working with Web Mercator or other projections");
    tracing::info!("   • Multi-image mosaic datasets (need catalog info)");
    tracing::info!("   • Performance-critical applications (minimize returns)");
    tracing::info!("   • Custom rendering rules (NDVI, slope, hillshade)");
}
