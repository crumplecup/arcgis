//! 🗺️ Map Service Basics - Render Maps and Identify Features
//!
//! Real-world map service scenarios using ESRI's public World Street Map:
//! Export static map images, identify features at clicked locations, search
//! by keyword, and retrieve legend information for visualization!
//!
//! # What You'll Learn
//!
//! - **Map export**: Render static maps with custom extents and formats
//! - **Transparent backgrounds**: Create overlay-ready images
//! - **High DPI rendering**: Export print-quality maps
//! - **Feature identification**: Click-to-query features at a point
//! - **Text search**: Find features by keyword across layers
//! - **Legend retrieval**: Get symbology for map layers
//!
//! # Prerequisites
//!
//! - No authentication required (uses public ESRI World Street Map service)
//!
//! # Running
//!
//! ```bash
//! cargo run --example map_service_basics
//!
//! # With debug logging to see all requests:
//! RUST_LOG=debug cargo run --example map_service_basics
//! ```
//!
//! # Real-World Use Cases
//!
//! - **Static map generation**: Create map images for reports, presentations
//! - **Web application backends**: Generate map tiles on demand
//! - **Click-to-query**: Interactive feature info (like "what's here?")
//! - **Search interfaces**: Find addresses, landmarks, features by name
//! - **Legend generation**: Build map keys and symbology references
//!
//! # Output
//!
//! This example creates several PNG files in the current directory:
//! - `map_basic.png` - Basic map of San Francisco Bay Area
//! - `map_transparent.png` - Transparent overlay-ready map
//! - `map_high_dpi.png` - High-resolution print-quality map

use anyhow::Result;
use arcgis::{
    ArcGISClient, ExportTarget, GeometryType, IdentifyParams, ImageFormat, LayerSelection,
    MapServiceClient, NoAuth,
};

/// Public ESRI World Street Map service (no auth required).
const WORLD_STREET_MAP: &str =
    "https://services.arcgisonline.com/ArcGIS/rest/services/World_Street_Map/MapServer";

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing for structured logging
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    tracing::info!("🗺️ Map Service Examples");
    tracing::info!("Using ESRI's public World Street Map service");

    // Create client with NoAuth (public service)
    let client = ArcGISClient::new(NoAuth);
    let map_service = MapServiceClient::new(WORLD_STREET_MAP, &client);

    // Demonstrate Map Service operations
    demonstrate_basic_map_export(&map_service).await?;
    demonstrate_transparent_export(&map_service).await?;
    demonstrate_high_dpi_export(&map_service).await?;
    demonstrate_identify_features(&map_service).await?;
    demonstrate_find_by_text(&map_service).await?;
    demonstrate_legend_retrieval(&map_service).await?;

    tracing::info!("\n✅ All map service examples completed successfully!");
    print_best_practices();

    Ok(())
}

/// Demonstrates basic map export with custom extent.
async fn demonstrate_basic_map_export(service: &MapServiceClient<'_>) -> Result<()> {
    tracing::info!("\n=== Example 1: Basic Map Export ===");
    tracing::info!("Export map of San Francisco Bay Area");

    // Define extent: San Francisco Bay Area
    // Format: "xmin,ymin,xmax,ymax" in WGS84 (EPSG:4326)
    let sf_extent = "-122.5,37.6,-122.3,37.9"; // West, South, East, North

    let result = service
        .export()
        .bbox(sf_extent)
        .size(800, 600)
        .format(ImageFormat::Png)
        .execute(ExportTarget::to_path("map_basic.png"))
        .await?;

    if let Some(path) = result.path() {
        tracing::info!(
            path = %path.display(),
            "✅ Map exported successfully"
        );
        tracing::info!("   Extent: San Francisco Bay Area");
        tracing::info!("   Size: 800x600 pixels");
        tracing::info!("   Format: PNG");
    }

    Ok(())
}

/// Demonstrates transparent background for overlay maps.
async fn demonstrate_transparent_export(service: &MapServiceClient<'_>) -> Result<()> {
    tracing::info!("\n=== Example 2: Transparent Background Export ===");
    tracing::info!("Export map with transparency for overlay use");

    // Downtown Los Angeles
    let la_extent = "-118.3,34.0,-118.2,34.1";

    let result = service
        .export()
        .bbox(la_extent)
        .size(1024, 768)
        .format(ImageFormat::Png32) // PNG32 supports transparency
        .transparent(true) // Enable transparency
        .execute(ExportTarget::to_path("map_transparent.png"))
        .await?;

    if let Some(path) = result.path() {
        tracing::info!(
            path = %path.display(),
            "✅ Transparent map exported"
        );
        tracing::info!("   Background: Transparent (alpha channel)");
        tracing::info!("   Use case: Perfect for overlaying on other imagery");
    }

    Ok(())
}

/// Demonstrates high DPI export for print quality.
async fn demonstrate_high_dpi_export(service: &MapServiceClient<'_>) -> Result<()> {
    tracing::info!("\n=== Example 3: High DPI Export ===");
    tracing::info!("Export high-resolution map for print");

    // Seattle downtown
    let seattle_extent = "-122.35,47.58,-122.30,47.63";

    let result = service
        .export()
        .bbox(seattle_extent)
        .size(1600, 1200) // Larger dimensions
        .format(ImageFormat::Png32)
        .dpi(300) // Print quality (default is 96)
        .execute(ExportTarget::to_path("map_high_dpi.png"))
        .await?;

    if let Some(path) = result.path() {
        tracing::info!(
            path = %path.display(),
            "✅ High-DPI map exported"
        );
        tracing::info!("   Resolution: 300 DPI (print quality)");
        tracing::info!("   Size: 1600x1200 pixels");
        tracing::info!("   Use case: Professional printing, large displays");
    }

    Ok(())
}

/// Demonstrates identifying features at a clicked point.
async fn demonstrate_identify_features(service: &MapServiceClient<'_>) -> Result<()> {
    tracing::info!("\n=== Example 4: Identify Features at Point ===");
    tracing::info!("Identify features at clicked location (Golden Gate Bridge)");

    // Golden Gate Bridge location
    let point_x = -122.4783;
    let point_y = 37.8199;

    // Build geometry JSON for point
    let geometry = format!("{{\"x\":{},\"y\":{}}}", point_x, point_y);

    // Map extent for context (San Francisco area)
    let map_extent = "-122.5,37.7,-122.3,37.9";

    // Image display format: "width,height,dpi"
    let image_display = "800,600,96";

    let params = IdentifyParams::builder()
        .geometry(geometry)
        .geometry_type(GeometryType::Point)
        .map_extent(map_extent.to_string())
        .image_display(image_display.to_string())
        .layers(LayerSelection::Visible) // All visible layers
        .tolerance(5) // 5 pixel tolerance around click
        .return_geometry(true)
        .build()
        .expect("Valid identify params");

    let response = service.identify(params).await?;

    tracing::info!(
        result_count = response.results().len(),
        "✅ Identify completed"
    );

    if response.results().is_empty() {
        tracing::info!("   No features found at this location");
        tracing::info!("   Try different coordinates or increase tolerance");
    } else {
        tracing::info!("📍 Features identified:");
        for (i, result) in response.results().iter().take(5).enumerate() {
            tracing::info!("   {}. Layer {}: {}", i + 1, result.layer_id(), result.layer_name());

            // Show a few attributes
            if !result.attributes().is_empty() {
                let attrs: Vec<String> = result
                    .attributes()
                    .iter()
                    .take(3)
                    .map(|(k, v)| format!("{}: {:?}", k, v))
                    .collect();
                tracing::info!("      Attributes: {}", attrs.join(", "));
            }
        }
    }

    Ok(())
}

/// Demonstrates finding features by text search.
async fn demonstrate_find_by_text(service: &MapServiceClient<'_>) -> Result<()> {
    tracing::info!("\n=== Example 5: Find Features by Text ===");
    tracing::info!("Search for features containing 'Park' across all layers");

    let params = arcgis::FindParams::builder()
        .search_text("Park")
        .contains(true) // Match partial text
        .return_geometry(false) // Don't need geometry for this example
        .build()
        .expect("Valid find params");

    let response = service.find(params).await?;

    tracing::info!(
        result_count = response.results().len(),
        "✅ Find completed"
    );

    if response.results().is_empty() {
        tracing::info!("   No features found matching 'Park'");
    } else {
        tracing::info!("🔍 Features found:");
        for (i, result) in response.results().iter().take(5).enumerate() {
            tracing::info!(
                "   {}. Layer {}: {}",
                i + 1,
                result.layer_id(),
                result.layer_name()
            );

            // Show the found value
            if !result.found_field_name().is_empty() {
                tracing::info!(
                    "      Match in field '{}': {}",
                    result.found_field_name(),
                    result.value()
                );
            }
        }

        if response.results().len() > 5 {
            tracing::info!("   ... and {} more results", response.results().len() - 5);
        }
    }

    Ok(())
}

/// Demonstrates retrieving map legend information.
async fn demonstrate_legend_retrieval(service: &MapServiceClient<'_>) -> Result<()> {
    tracing::info!("\n=== Example 6: Legend Retrieval ===");
    tracing::info!("Get legend symbols and labels for all layers");

    let legend = service.get_legend().await?;

    tracing::info!(
        layer_count = legend.layers().len(),
        "✅ Legend retrieved"
    );

    tracing::info!("🎨 Legend information:");
    for layer in legend.layers().iter().take(5) {
        tracing::info!(
            "   Layer {}: {}",
            layer.layer_id(),
            layer.layer_name()
        );

        // Show legend items (symbols)
        if !layer.legend().is_empty() {
            tracing::info!("      Symbols:");
            for (i, symbol) in layer.legend().iter().take(3).enumerate() {
                tracing::info!("         {}. {}", i + 1, symbol.label());
                if let Some(url) = symbol.url() {
                    tracing::debug!("            Icon URL: {}", url);
                }
            }

            if layer.legend().len() > 3 {
                tracing::info!("         ... and {} more symbols", layer.legend().len() - 3);
            }
        }
    }

    if legend.layers().len() > 5 {
        tracing::info!("   ... and {} more layers", legend.layers().len() - 5);
    }

    Ok(())
}

/// Prints best practices for Map Service operations.
fn print_best_practices() {
    tracing::info!("\n💡 Map Service Best Practices:");
    tracing::info!("   - Use appropriate image formats (PNG for transparency, JPG for photos)");
    tracing::info!("   - Request only the extent you need to minimize data transfer");
    tracing::info!("   - Use DPI setting (96 for screen, 300 for print)");
    tracing::info!("   - Cache exported maps when extent/params don't change");
    tracing::info!("   - Set appropriate tolerance for identify operations");
    tracing::info!("");
    tracing::info!("🎯 Format Selection:");
    tracing::info!("   - PNG32: Transparency support, best for overlays");
    tracing::info!("   - PNG24: No transparency, smaller than PNG32");
    tracing::info!("   - JPG: Smallest file size, no transparency, best for photos");
    tracing::info!("   - PDF/SVG: Vector formats for scalable graphics");
    tracing::info!("");
    tracing::info!("⚡ Performance Tips:");
    tracing::info!("   - Stream large exports to files instead of loading into memory");
    tracing::info!("   - Use cached tile services when available (export_tile)");
    tracing::info!("   - Limit identify tolerance to reduce processing time");
    tracing::info!("   - Use contains=true for partial text matching in find operations");
    tracing::info!("");
    tracing::info!("📐 Extent Format:");
    tracing::info!("   - Bbox format: 'xmin,ymin,xmax,ymax' (west,south,east,north)");
    tracing::info!("   - Use WGS84 (EPSG:4326) for lat/lon coordinates");
    tracing::info!("   - Use Web Mercator (EPSG:3857) for web mapping");
    tracing::info!("   - Match service spatial reference for best results");
}
