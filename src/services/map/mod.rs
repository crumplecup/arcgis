//! Map Service.
//!
//! This module provides support for ArcGIS Map Services, which offer dynamic map
//! rendering, cached tile access, metadata retrieval, legend information, and
//! feature identification capabilities.
//!
//! # Operations
//!
//! - **Export Map** - Dynamically render and export map images with custom parameters
//! - **Export Tile** - Retrieve cached tiles from tiled/cached services
//! - **Get Legend** - Retrieve legend information for all layers
//! - **Get Metadata** - Retrieve service metadata (layers, extent, capabilities)
//! - **Identify** - Identify features at a specific location
//!
//! # Quick Start
//!
//! ```no_run
//! use arcgis::{ApiKeyAuth, ArcGISClient, MapServiceClient, ExportTarget, ImageFormat};
//!
//! # async fn example() -> arcgis::Result<()> {
//! let auth = ApiKeyAuth::new("YOUR_API_KEY");
//! let client = ArcGISClient::new(auth);
//!
//! let map_service = MapServiceClient::new(
//!     "https://services.arcgis.com/org/arcgis/rest/services/World/MapServer",
//!     &client,
//! );
//!
//! // Export a map with fluent builder
//! let result = map_service
//!     .export()
//!     .bbox("-118.0,34.0,-117.0,35.0")
//!     .size(800, 600)
//!     .format(ImageFormat::Png32)
//!     .transparent(true)
//!     .execute(ExportTarget::to_path("map.png"))
//!     .await?;
//!
//! // Get service metadata
//! let metadata = map_service.get_metadata().await?;
//! println!("Service has {} layers", metadata.layers().len());
//!
//! // Get legend
//! let legend = map_service.get_legend().await?;
//! for layer in &legend.layers {
//!     println!("Layer {}: {}", layer.layer_id(), layer.layer_name());
//! }
//! # Ok(())
//! # }
//! ```
//!
//! # Export Targets
//!
//! Maps and tiles can be exported to three different targets:
//!
//! - **Path** - Stream directly to a file (most efficient for large images)
//! - **Bytes** - Collect into memory as `Vec<u8>` (convenient for small images)
//! - **Writer** - Stream to any `AsyncWrite` implementation
//!
//! ```no_run
//! # use arcgis::{MapServiceClient, ExportTarget};
//! # async fn example(service: &MapServiceClient<'_>) -> arcgis::Result<()> {
//! // To file
//! let result = service
//!     .export()
//!     .bbox("-118,34,-117,35")
//!     .execute(ExportTarget::to_path("output.png"))
//!     .await?;
//!
//! // To bytes
//! let result = service
//!     .export()
//!     .bbox("-118,34,-117,35")
//!     .execute(ExportTarget::to_bytes())
//!     .await?;
//!
//! if let Some(bytes) = result.bytes() {
//!     // Process image bytes...
//! }
//! # Ok(())
//! # }
//! ```
//!
//! # Image Formats
//!
//! Supported image formats (via [`ImageFormat`]):
//!
//! - PNG variants: `Png`, `Png8`, `Png24`, `Png32` (supports transparency)
//! - `Jpg` - Compact, lossy compression
//! - `Gif` - Supports transparency
//! - Vector formats: `Pdf`, `Svg`, `Svgz`
//! - Others: `Bmp`, `Emf`, `Ps`
//!
//! # Layer Visibility
//!
//! Control which layers appear in exported maps using [`LayerOperation`]:
//!
//! ```no_run
//! # use arcgis::{MapServiceClient, ExportTarget, LayerOperation};
//! # async fn example(service: &MapServiceClient<'_>) -> arcgis::Result<()> {
//! // Show only specific layers
//! service
//!     .export()
//!     .bbox("-118,34,-117,35")
//!     .layer_visibility(LayerOperation::Show, &[0, 1, 2])
//!     .execute(ExportTarget::to_bytes())
//!     .await?;
//!
//! // Hide specific layers
//! service
//!     .export()
//!     .bbox("-118,34,-117,35")
//!     .layer_visibility(LayerOperation::Hide, &[3, 4])
//!     .execute(ExportTarget::to_bytes())
//!     .await?;
//! # Ok(())
//! # }
//! ```
//!
//! # Cached Tiles
//!
//! For cached/tiled services, retrieve individual tiles:
//!
//! ```no_run
//! # use arcgis::{MapServiceClient, TileCoordinate, ExportTarget};
//! # async fn example(service: &MapServiceClient<'_>) -> arcgis::Result<()> {
//! let coord = TileCoordinate::new(5, 10, 15);  // level, row, col
//! let result = service
//!     .export_tile(coord, ExportTarget::to_bytes())
//!     .await?;
//! # Ok(())
//! # }
//! ```
//!
//! # Feature Identification
//!
//! Identify features at a location on the map:
//!
//! ```no_run
//! # use arcgis::{MapServiceClient, IdentifyParamsBuilder, LayerSelection, GeometryType};
//! # async fn example(service: &MapServiceClient<'_>) -> arcgis::Result<()> {
//! let params = IdentifyParamsBuilder::default()
//!     .geometry("{\"x\":-118.0,\"y\":34.0}".to_string())
//!     .geometry_type(GeometryType::Point)
//!     .map_extent("-120,32,-116,36".to_string())
//!     .image_display("800,600,96".to_string())
//!     .layers(LayerSelection::Visible)
//!     .build()
//!     .expect("Valid params");
//!
//! let response = service.identify(params).await?;
//! for result in response.results() {
//!     println!("Found feature in layer {}", result.layer_id());
//! }
//! # Ok(())
//! # }
//! ```

mod client;
mod enums;
mod export;
mod types;

pub use client::MapServiceClient;
pub use enums::{ImageFormat, LayerOperation, LayerSelection, ResponseFormat, TimeRelation};
pub use export::ExportMapBuilder;
pub use types::{
    ClassBreakInfo, ExportExtent, ExportMapParams, ExportMapParamsBuilder, ExportMapResponse,
    ExportResult, ExportTarget, FindParams, FindParamsBuilder, FindResponse, FindResult,
    GenerateKmlParams, GenerateKmlParamsBuilder, GenerateRendererParams,
    GenerateRendererParamsBuilder, IdentifyParams, IdentifyParamsBuilder, IdentifyResponse,
    IdentifyResult, LayerDefinitions, LayerLegend, LegendResponse, LegendSymbol, LevelOfDetail,
    MapServiceMetadata, RendererResponse, ServiceLayer, SpatialReference, TileCoordinate, TileInfo,
    UniqueValueInfo,
};
