//! Vector Tile Service.
//!
//! The Vector Tile Service provides operations for:
//! - **getTile**: Retrieve Mapbox Vector Tile (MVT) format tiles
//! - **getStyle**: Retrieve Mapbox GL style JSON
//! - **getFonts**: Retrieve font glyphs for text rendering
//!
//! Vector tiles are a modern alternative to raster tiles, providing:
//! - Smaller file sizes (compressed vector data)
//! - Scalable rendering at any zoom level
//! - Dynamic styling on the client
//! - Better performance for interactive maps
//!
//! # Example
//!
//! ```no_run
//! use arcgis::{ApiKeyAuth, ArcGISClient, VectorTileServiceClient, TileCoordinate};
//!
//! # async fn example() -> arcgis::Result<()> {
//! let auth = ApiKeyAuth::new("YOUR_API_KEY");
//! let client = ArcGISClient::new(auth);
//! let vt_service = VectorTileServiceClient::new(
//!     "https://basemaps.arcgis.com/arcgis/rest/services/World_Basemap_v2/VectorTileServer",
//!     &client
//! );
//!
//! // Get a tile at zoom level 10, row 512, column 256
//! let tile_coord = TileCoordinate::new(10, 512, 256);
//! let tile_data = vt_service.get_tile(&tile_coord).await?;
//!
//! // Get the style JSON
//! let style = vt_service.get_style().await?;
//! # Ok(())
//! # }
//! ```

mod client;
mod types;

pub use client::VectorTileServiceClient;
pub use types::{FontStack, GlyphRange, VectorTileStyle};
