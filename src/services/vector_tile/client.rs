//! Vector tile service client implementation.

use crate::{ArcGISClient, Result, TileCoordinate};
use tracing::instrument;

use super::types::{FontStack, GlyphRange, VectorTileStyle};

/// Client for interacting with ArcGIS Vector Tile Services.
///
/// Vector tile services provide Mapbox Vector Tile (MVT) format tiles for
/// efficient, scalable web mapping. Unlike raster tiles, vector tiles contain
/// geometric data that can be styled dynamically on the client.
///
/// # Example
///
/// ```no_run
/// use arcgis::{ApiKeyAuth, ArcGISClient, VectorTileServiceClient, TileCoordinate};
///
/// # async fn example() -> arcgis::Result<()> {
/// let auth = ApiKeyAuth::new("YOUR_API_KEY");
/// let client = ArcGISClient::new(auth);
/// let vt_service = VectorTileServiceClient::new(
///     "https://basemaps.arcgis.com/arcgis/rest/services/World_Basemap_v2/VectorTileServer",
///     &client
/// );
///
/// // Get a vector tile
/// let tile = TileCoordinate::new(10, 512, 256);
/// let tile_data = vt_service.get_tile(&tile).await?;
/// # Ok(())
/// # }
/// ```
#[derive(Clone)]
pub struct VectorTileServiceClient<'a> {
    /// Base URL of the vector tile service.
    url: String,

    /// Reference to the ArcGIS client.
    client: &'a ArcGISClient,
}

impl<'a> VectorTileServiceClient<'a> {
    /// Creates a new vector tile service client.
    ///
    /// # Arguments
    ///
    /// * `url` - Base URL of the vector tile service (e.g., `https://basemaps.arcgis.com/arcgis/rest/services/World_Basemap_v2/VectorTileServer`)
    /// * `client` - Reference to an [`ArcGISClient`] for making requests
    ///
    /// # Example
    ///
    /// ```no_run
    /// use arcgis::{ApiKeyAuth, ArcGISClient, VectorTileServiceClient};
    ///
    /// let auth = ApiKeyAuth::new("YOUR_API_KEY");
    /// let client = ArcGISClient::new(auth);
    /// let vt_service = VectorTileServiceClient::new(
    ///     "https://basemaps.arcgis.com/arcgis/rest/services/World_Basemap_v2/VectorTileServer",
    ///     &client
    /// );
    /// ```
    pub fn new(url: impl Into<String>, client: &'a ArcGISClient) -> Self {
        VectorTileServiceClient {
            url: url.into(),
            client,
        }
    }

    /// Retrieves a vector tile in Mapbox Vector Tile (MVT) format.
    ///
    /// Returns the raw tile bytes in Protocol Buffer format. To decode the tile,
    /// use an MVT parsing library like `mapbox-vector-tile`.
    ///
    /// # Arguments
    ///
    /// * `tile` - Tile coordinate (zoom level, row, column)
    ///
    /// # Returns
    ///
    /// Raw MVT tile data as bytes.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use arcgis::{ApiKeyAuth, ArcGISClient, VectorTileServiceClient, TileCoordinate};
    ///
    /// # async fn example() -> arcgis::Result<()> {
    /// let auth = ApiKeyAuth::new("YOUR_API_KEY");
    /// let client = ArcGISClient::new(auth);
    /// let vt_service = VectorTileServiceClient::new(
    ///     "https://basemaps.arcgis.com/arcgis/rest/services/World_Basemap_v2/VectorTileServer",
    ///     &client
    /// );
    ///
    /// let tile = TileCoordinate::new(10, 512, 256);
    /// let tile_data = vt_service.get_tile(&tile).await?;
    /// tracing::info!(size = tile_data.len(), "Retrieved tile");
    /// # Ok(())
    /// # }
    /// ```
    #[instrument(skip(self), fields(level = tile.level(), row = tile.row(), col = tile.col()))]
    pub async fn get_tile(&self, tile: &TileCoordinate) -> Result<Vec<u8>> {
        tracing::debug!("Retrieving vector tile");

        // Tile URL format: {baseUrl}/tile/{z}/{y}/{x}.pbf
        let tile_url = format!(
            "{}/tile/{}/{}/{}.pbf",
            self.url,
            tile.level(),
            tile.row(),
            tile.col()
        );

        let response = self.client.http().get(&tile_url).send().await?;

        let bytes = response.bytes().await?;

        tracing::debug!(size = bytes.len(), "Tile retrieved");

        Ok(bytes.to_vec())
    }

    /// Retrieves multiple vector tiles in batch.
    ///
    /// More efficient than calling `get_tile` multiple times individually.
    ///
    /// # Arguments
    ///
    /// * `tiles` - Vector of tile coordinates to retrieve
    ///
    /// # Returns
    ///
    /// Vector of tile data in the same order as the input tiles.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use arcgis::{ApiKeyAuth, ArcGISClient, VectorTileServiceClient, TileCoordinate};
    ///
    /// # async fn example() -> arcgis::Result<()> {
    /// let auth = ApiKeyAuth::new("YOUR_API_KEY");
    /// let client = ArcGISClient::new(auth);
    /// let vt_service = VectorTileServiceClient::new(
    ///     "https://basemaps.arcgis.com/arcgis/rest/services/World_Basemap_v2/VectorTileServer",
    ///     &client
    /// );
    ///
    /// let tiles = vec![
    ///     TileCoordinate::new(10, 512, 256),
    ///     TileCoordinate::new(10, 512, 257),
    ///     TileCoordinate::new(10, 513, 256),
    /// ];
    ///
    /// let tile_data = vt_service.get_tiles(&tiles).await?;
    /// tracing::info!(count = tile_data.len(), "Retrieved tiles");
    /// # Ok(())
    /// # }
    /// ```
    #[instrument(skip(self), fields(tile_count = tiles.len()))]
    pub async fn get_tiles(&self, tiles: &[TileCoordinate]) -> Result<Vec<Vec<u8>>> {
        tracing::debug!("Retrieving multiple tiles");

        let mut results = Vec::with_capacity(tiles.len());

        // Fetch tiles concurrently
        let futures: Vec<_> = tiles.iter().map(|tile| self.get_tile(tile)).collect();

        for future in futures {
            results.push(future.await?);
        }

        tracing::debug!(count = results.len(), "All tiles retrieved");

        Ok(results)
    }

    /// Retrieves the Mapbox GL style JSON for this vector tile service.
    ///
    /// The style defines how vector tiles should be rendered, including layers,
    /// colors, fonts, and sprites.
    ///
    /// # Returns
    ///
    /// Vector tile style document.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use arcgis::{ApiKeyAuth, ArcGISClient, VectorTileServiceClient};
    ///
    /// # async fn example() -> arcgis::Result<()> {
    /// let auth = ApiKeyAuth::new("YOUR_API_KEY");
    /// let client = ArcGISClient::new(auth);
    /// let vt_service = VectorTileServiceClient::new(
    ///     "https://basemaps.arcgis.com/arcgis/rest/services/World_Basemap_v2/VectorTileServer",
    ///     &client
    /// );
    ///
    /// let style = vt_service.get_style().await?;
    /// tracing::info!(
    ///     version = style.version(),
    ///     layer_count = style.layers().len(),
    ///     "Retrieved style"
    /// );
    /// # Ok(())
    /// # }
    /// ```
    #[instrument(skip(self))]
    pub async fn get_style(&self) -> Result<VectorTileStyle> {
        tracing::debug!("Retrieving vector tile style");

        let style_url = format!("{}/resources/styles/root.json", self.url);

        let response = self.client.http().get(&style_url).send().await?;

        let style: VectorTileStyle = response.json().await?;

        tracing::debug!(
            version = style.version(),
            layer_count = style.layers().len(),
            "Style retrieved"
        );

        Ok(style)
    }

    /// Retrieves font glyphs for text rendering.
    ///
    /// Glyphs are returned in Protocol Buffer format and contain the vector
    /// outlines for rendering text characters.
    ///
    /// # Arguments
    ///
    /// * `font_stack` - Font name (e.g., "Arial Regular")
    /// * `range` - Unicode character range (e.g., 0-255 for ASCII)
    ///
    /// # Returns
    ///
    /// Raw glyph data as bytes (PBF format).
    ///
    /// # Example
    ///
    /// ```no_run
    /// use arcgis::{ApiKeyAuth, ArcGISClient, VectorTileServiceClient, FontStack, GlyphRange};
    ///
    /// # async fn example() -> arcgis::Result<()> {
    /// let auth = ApiKeyAuth::new("YOUR_API_KEY");
    /// let client = ArcGISClient::new(auth);
    /// let vt_service = VectorTileServiceClient::new(
    ///     "https://basemaps.arcgis.com/arcgis/rest/services/World_Basemap_v2/VectorTileServer",
    ///     &client
    /// );
    ///
    /// let font = FontStack::new("Arial Regular");
    /// let range = GlyphRange::ascii();
    /// let glyphs = vt_service.get_fonts(&font, &range).await?;
    /// tracing::info!(size = glyphs.len(), "Retrieved glyphs");
    /// # Ok(())
    /// # }
    /// ```
    #[instrument(skip(self), fields(font = %font_stack.0, range = %range))]
    pub async fn get_fonts(&self, font_stack: &FontStack, range: &GlyphRange) -> Result<Vec<u8>> {
        tracing::debug!("Retrieving font glyphs");

        // Font URL format: {baseUrl}/resources/fonts/{fontstack}/{range}.pbf
        let font_url = format!(
            "{}/resources/fonts/{}/{}.pbf",
            self.url,
            font_stack.0,
            range.format()
        );

        let response = self.client.http().get(&font_url).send().await?;

        let bytes = response.bytes().await?;

        tracing::debug!(size = bytes.len(), "Glyphs retrieved");

        Ok(bytes.to_vec())
    }

    /// Retrieves sprite sheet metadata.
    ///
    /// Sprite sheets contain icons and symbols used in vector tile rendering.
    /// This returns JSON metadata describing sprite positions and sizes.
    ///
    /// # Returns
    ///
    /// Sprite metadata as JSON value.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use arcgis::{ApiKeyAuth, ArcGISClient, VectorTileServiceClient};
    ///
    /// # async fn example() -> arcgis::Result<()> {
    /// let auth = ApiKeyAuth::new("YOUR_API_KEY");
    /// let client = ArcGISClient::new(auth);
    /// let vt_service = VectorTileServiceClient::new(
    ///     "https://basemaps.arcgis.com/arcgis/rest/services/World_Basemap_v2/VectorTileServer",
    ///     &client
    /// );
    ///
    /// let sprite_meta = vt_service.get_sprite_metadata().await?;
    /// # Ok(())
    /// # }
    /// ```
    #[instrument(skip(self))]
    pub async fn get_sprite_metadata(&self) -> Result<serde_json::Value> {
        tracing::debug!("Retrieving sprite metadata");

        let sprite_url = format!("{}/resources/sprites/sprite.json", self.url);

        let response = self.client.http().get(&sprite_url).send().await?;

        let metadata: serde_json::Value = response.json().await?;

        tracing::debug!("Sprite metadata retrieved");

        Ok(metadata)
    }

    /// Retrieves sprite sheet image.
    ///
    /// Returns the PNG image containing all sprite icons.
    ///
    /// # Returns
    ///
    /// Raw PNG image data as bytes.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use arcgis::{ApiKeyAuth, ArcGISClient, VectorTileServiceClient};
    ///
    /// # async fn example() -> arcgis::Result<()> {
    /// let auth = ApiKeyAuth::new("YOUR_API_KEY");
    /// let client = ArcGISClient::new(auth);
    /// let vt_service = VectorTileServiceClient::new(
    ///     "https://basemaps.arcgis.com/arcgis/rest/services/World_Basemap_v2/VectorTileServer",
    ///     &client
    /// );
    ///
    /// let sprite_image = vt_service.get_sprite_image().await?;
    /// tracing::info!(size = sprite_image.len(), "Retrieved sprite sheet");
    /// # Ok(())
    /// # }
    /// ```
    #[instrument(skip(self))]
    pub async fn get_sprite_image(&self) -> Result<Vec<u8>> {
        tracing::debug!("Retrieving sprite image");

        let sprite_url = format!("{}/resources/sprites/sprite.png", self.url);

        let response = self.client.http().get(&sprite_url).send().await?;

        let bytes = response.bytes().await?;

        tracing::debug!(size = bytes.len(), "Sprite image retrieved");

        Ok(bytes.to_vec())
    }
}
