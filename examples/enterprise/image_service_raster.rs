//! 🌍 Image Service - Raster Data Access and Analysis
//!
//! Demonstrates raster imagery operations using ArcGIS Image Services.
//! Learn how to export raster images, identify pixel values, sample along
//! transects, compute histograms, and query raster metadata.
//!
//! # What You'll Learn
//!
//! - **Export images**: Render raster data with custom extent and format
//! - **Identify pixels**: Get pixel values at specific locations
//! - **Sample transects**: Extract pixel values along a line
//! - **Compute histograms**: Analyze pixel value distributions
//! - **Raster metadata**: Query band count, pixel type, and extent information
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
//! cargo run --example image_service_raster
//!
//! # With debug logging to see all requests:
//! RUST_LOG=debug cargo run --example image_service_raster
//! ```
//!
//! # Real-World Use Cases
//!
//! - **Land cover analysis**: Classify and analyze terrain types
//! - **Environmental monitoring**: Track vegetation, water, urban development
//! - **Elevation analysis**: Extract elevation profiles along routes
//! - **Change detection**: Compare imagery over time
//! - **Agricultural monitoring**: NDVI and crop health assessment
//! - **Disaster response**: Analyze satellite/aerial imagery for damage assessment
//!
//! # Output
//!
//! This example creates:
//! - `landcover_export.png` - Exported raster image of study area
//! - Console output showing pixel values, histograms, and metadata

use anyhow::Result;
use arcgis::{
    geo_types::{Geometry, LineString, Point},
    ArcGISClient, ArcGISGeometry, ExportImageParametersBuilder, HistogramParametersBuilder,
    ImageServiceClient, NoAuth, SampleParametersBuilder,
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

    tracing::info!("🌍 Image Service Examples");
    tracing::info!("Using NLCD Land Cover 2001 public service");

    // Create client with NoAuth (public service)
    let client = ArcGISClient::new(NoAuth);
    let image_service = ImageServiceClient::new(NLCD_IMAGE_SERVICE, &client);

    // Demonstrate Image Service operations
    demonstrate_raster_metadata(&image_service).await?;
    demonstrate_export_image(&image_service).await?;
    demonstrate_identify_pixel(&image_service).await?;
    demonstrate_sample_transect(&image_service).await?;
    demonstrate_compute_histograms(&image_service).await?;

    tracing::info!("\n✅ All image service examples completed successfully!");
    print_best_practices();

    Ok(())
}

/// Demonstrates retrieving raster metadata.
async fn demonstrate_raster_metadata(service: &ImageServiceClient<'_>) -> Result<()> {
    tracing::info!("\n=== Example 1: Raster Metadata ===");
    tracing::info!("Query raster information (bands, extent, pixel type)");

    let info = service.get_raster_info().await?;

    tracing::info!("📊 Raster Information:");

    if let Some(pixel_type) = info.pixel_type() {
        tracing::info!("   Pixel Type: {}", pixel_type);
    }

    if let Some(band_count) = info.band_count() {
        tracing::info!("   Band Count: {}", band_count);
    }

    if let Some(extent) = info.extent() {
        tracing::debug!("   Extent: {:?}", extent);
    }

    if let Some(cell_size_x) = info.pixel_size_x() {
        tracing::info!("   Pixel Size X: {}", cell_size_x);
    }
    if let Some(cell_size_y) = info.pixel_size_y() {
        tracing::info!("   Pixel Size Y: {}", cell_size_y);
    }

    Ok(())
}

/// Demonstrates exporting a raster image.
async fn demonstrate_export_image(service: &ImageServiceClient<'_>) -> Result<()> {
    tracing::info!("\n=== Example 2: Export Raster Image ===");
    tracing::info!("Export land cover data for San Francisco Bay Area");

    // San Francisco Bay Area extent (Web Mercator EPSG:3857)
    let bbox = "-13658209,4525874,-13598209,4565874";

    let params = ExportImageParametersBuilder::default()
        .bbox(bbox)
        .bbox_sr(3857u32)  // Specify bbox is in Web Mercator
        .size("800,600")
        .format("png")
        .build()
        .expect("Valid export parameters");

    let result = service.export_image(params).await?;

    tracing::info!(
        href = %result.href(),
        "✅ Image exported successfully"
    );

    if let Some(width) = result.width() {
        if let Some(height) = result.height() {
            tracing::info!("   Dimensions: {}x{} pixels", width, height);
        }
    }

    tracing::info!("   Download URL: {}", result.href());
    tracing::info!("   Use case: Display land cover classification on map");

    Ok(())
}

/// Demonstrates identifying pixel values at a point.
async fn demonstrate_identify_pixel(service: &ImageServiceClient<'_>) -> Result<()> {
    tracing::info!("\n=== Example 3: Identify Pixel Values ===");
    tracing::info!("Get land cover classification at specific location");

    // Redlands, California (where ESRI headquarters is located)
    let point = Point::new(-117.1825, 34.0555);
    let geom: Geometry = point.into();
    let geometry = ArcGISGeometry::from_geo_types(&geom)?;

    let result = service.identify(&geometry).await?;

    tracing::info!("📍 Pixel Values at Redlands, CA:");
    if let Some(value) = result.value() {
        tracing::info!("   Land Cover Class: {}", value);
        tracing::info!("   Location: Lon {}, Lat {}",
            point.x(),
            point.y()
        );

        // NLCD class codes (common values):
        // 21 = Developed, Open Space
        // 22 = Developed, Low Intensity
        // 23 = Developed, Medium Intensity
        // 24 = Developed, High Intensity
        // 41 = Deciduous Forest
        // 42 = Evergreen Forest
        // 52 = Shrub/Scrub
        // 71 = Grassland/Herbaceous
        // 81 = Pasture/Hay
        // 82 = Cultivated Crops
        // 90 = Woody Wetlands
        // 95 = Emergent Herbaceous Wetlands

        tracing::info!("   Note: NLCD class codes represent land cover types");
        tracing::info!("   (21-24=Developed, 41-43=Forest, 52=Shrub, 71=Grass, etc.)");
    }

    if let Some(properties) = result.properties() {
        tracing::debug!("   Properties: {:?}", properties);
    }

    Ok(())
}

/// Demonstrates sampling pixel values along a transect.
async fn demonstrate_sample_transect(service: &ImageServiceClient<'_>) -> Result<()> {
    tracing::info!("\n=== Example 4: Sample Along Transect ===");
    tracing::info!("Extract pixel values along a line (e.g., hiking trail)");

    // Create a line from coast to inland (Santa Barbara area)
    let line = LineString::from(vec![
        (-119.7, 34.4),   // Pacific coast
        (-119.5, 34.45),  // Mid-point
        (-119.3, 34.5),   // Inland
    ]);

    let geom: Geometry = line.into();
    let geometry = ArcGISGeometry::from_geo_types(&geom)?;

    // Serialize geometry to JSON string
    let geometry_json = serde_json::to_string(&geometry)?;

    let params = SampleParametersBuilder::default()
        .geometry(geometry_json)
        .geometry_type("esriGeometryPolyline")
        .build()
        .expect("Valid sample parameters");

    let result = service.get_samples(params).await?;

    tracing::info!("🔬 Sampled Pixel Values:");
    let samples = result.samples();
    for (i, sample) in samples.iter().enumerate().take(10) {
        // Sample is a Value, so we need to access it as such
        if let Some(value) = sample.get("value") {
            if let Some(value_str) = value.as_str() {
                tracing::info!("   Sample {}: Land cover class = {}", i + 1, value_str);
            } else if let Some(arr) = value.as_array() {
                if let Some(first) = arr.first() {
                    tracing::info!("   Sample {}: Land cover class = {}", i + 1, first);
                }
            }
        }
        if let Some(location) = sample.get("location") {
            tracing::debug!("      Location: {:?}", location);
        }
    }

    if samples.len() > 10 {
        tracing::info!("   ... and {} more samples", samples.len() - 10);
    }

    tracing::info!("   Use case: Analyze land cover changes along transportation corridor");

    Ok(())
}

/// Demonstrates computing pixel value histograms.
async fn demonstrate_compute_histograms(service: &ImageServiceClient<'_>) -> Result<()> {
    tracing::info!("\n=== Example 5: Compute Histograms ===");
    tracing::info!("Analyze pixel value distribution in study area");

    // Study area: Los Angeles region (Web Mercator)
    let geometry_str = r#"{
        "rings": [[
            [-13211000, 4020000],
            [-13211000, 4060000],
            [-13171000, 4060000],
            [-13171000, 4020000],
            [-13211000, 4020000]
        ]],
        "spatialReference": {"wkid": 102100}
    }"#;

    let params = HistogramParametersBuilder::default()
        .geometry(geometry_str)
        .geometry_type("esriGeometryPolygon")
        .build()
        .expect("Valid histogram parameters");

    let result = service.compute_histograms(params).await?;

    tracing::info!("📊 Histogram Results:");
    let histograms = result.histograms();
    for (i, histogram) in histograms.iter().enumerate() {
        tracing::info!("   Band {}: {} bins", i, histogram.counts().len());

        // Show the most common values
        if let Some(max_count_idx) = histogram.counts()
            .iter()
            .enumerate()
            .max_by_key(|(_, count)| *count)
            .map(|(idx, _)| idx)
        {
            let count = histogram.counts()[max_count_idx];

            if let (Some(min), Some(max)) = (histogram.min(), histogram.max()) {
                let value = min + (max_count_idx as f64 * (max - min) / histogram.counts().len() as f64);
                tracing::info!("      Most common value ≈ {:.0} (count: {})", value, count);
            } else {
                tracing::info!("      Most common bin {} (count: {})", max_count_idx, count);
            }
        }

        if let Some(mean) = histogram.mean() {
            tracing::info!("      Mean: {:.2}", mean);
        }
        if let Some(std_dev) = histogram.std_dev() {
            tracing::info!("      Std Dev: {:.2}", std_dev);
        }
    }

    tracing::info!("   Use case: Understand land cover type distribution in urban area");

    Ok(())
}

/// Prints best practices for working with Image Services.
fn print_best_practices() {
    tracing::info!("\n💡 Image Service Best Practices:");
    tracing::info!("   - Use appropriate bbox for your study area to minimize data transfer");
    tracing::info!("   - Specify output size to control resolution vs. file size");
    tracing::info!("   - Use mosaic rules when working with multi-image services");
    tracing::info!("   - Apply rendering rules for on-the-fly analysis (NDVI, hillshade, etc.)");
    tracing::info!("   - Cache exported images when extent/parameters don't change");
    tracing::info!("");
    tracing::info!("🎯 Format Selection:");
    tracing::info!("   - PNG: Good for classification data, supports transparency");
    tracing::info!("   - JPG: Smaller files for continuous data (elevation, imagery)");
    tracing::info!("   - TIFF: Best for analysis, preserves pixel values exactly");
    tracing::info!("   - LERC: Efficient compression for elevation/scientific data");
    tracing::info!("");
    tracing::info!("⚡ Performance Tips:");
    tracing::info!("   - Use histograms to understand data before processing");
    tracing::info!("   - Sample along transects instead of dense grids when possible");
    tracing::info!("   - Consider pixel size when setting bbox and size parameters");
    tracing::info!("   - Use appropriate spatial reference for your region");
    tracing::info!("");
    tracing::info!("📐 Common Pixel Types:");
    tracing::info!("   - U8: Land cover classifications (0-255 classes)");
    tracing::info!("   - S16: Elevation data (signed values)");
    tracing::info!("   - F32: Scientific measurements (temperature, NDVI)");
    tracing::info!("   - RGB: True-color imagery");
}
