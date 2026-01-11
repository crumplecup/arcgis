//! Map service client implementation.

use super::ResponseFormat;
use crate::{
    ArcGISClient, ExportMapParams, ExportMapResponse, ExportResult, ExportTarget, IdentifyParams,
    IdentifyResponse, LegendResponse, MapServiceMetadata, Result, TileCoordinate,
};
use futures::StreamExt;
use tokio::io::AsyncWriteExt;
use tracing::instrument;

/// Client for interacting with an ArcGIS Map Service.
///
/// Map Services provide dynamic map rendering, cached tiles, metadata,
/// legend information, and feature identification capabilities.
///
/// # Example
///
/// ```no_run
/// use arcgis::{ApiKeyAuth, ArcGISClient, MapServiceClient, ExportTarget};
///
/// # async fn example() -> arcgis::Result<()> {
/// let auth = ApiKeyAuth::new("YOUR_API_KEY");
/// let client = ArcGISClient::new(auth);
///
/// let map_service = MapServiceClient::new(
///     "https://services.arcgis.com/org/arcgis/rest/services/MapName/MapServer",
///     &client,
/// );
///
/// // Export a map image
/// let result = map_service
///     .export()
///     .bbox("-180,-90,180,90")
///     .size(800, 600)
///     .execute(ExportTarget::to_path("map.png"))
///     .await?;
/// # Ok(())
/// # }
/// ```
pub struct MapServiceClient<'a> {
    /// Base URL of the map service.
    base_url: String,
    /// Reference to the ArcGIS client for HTTP operations.
    client: &'a ArcGISClient,
}

impl<'a> MapServiceClient<'a> {
    /// Creates a new Map Service client.
    ///
    /// # Arguments
    ///
    /// * `base_url` - The base URL of the map service (e.g., `https://services.arcgis.com/.../MapServer`)
    /// * `client` - Reference to an authenticated ArcGIS client
    ///
    /// # Example
    ///
    /// ```no_run
    /// use arcgis::{ApiKeyAuth, ArcGISClient, MapServiceClient};
    ///
    /// let auth = ApiKeyAuth::new("YOUR_API_KEY");
    /// let client = ArcGISClient::new(auth);
    ///
    /// let map_service = MapServiceClient::new(
    ///     "https://services.arcgis.com/org/arcgis/rest/services/MapName/MapServer",
    ///     &client,
    /// );
    /// ```
    #[instrument(skip(base_url, client))]
    pub fn new(base_url: impl Into<String>, client: &'a ArcGISClient) -> Self {
        let base_url = base_url.into();
        tracing::debug!(base_url = %base_url, "Creating MapServiceClient");
        Self { base_url, client }
    }

    /// Creates a fluent builder for exporting maps.
    ///
    /// This is the recommended way to export maps. It provides a more
    /// ergonomic API than manually constructing [`ExportMapParams`].
    ///
    /// # Example
    ///
    /// ```no_run
    /// use arcgis::{ApiKeyAuth, ArcGISClient, MapServiceClient, ExportTarget, ImageFormat};
    ///
    /// # async fn example() -> arcgis::Result<()> {
    /// let auth = ApiKeyAuth::new("YOUR_API_KEY");
    /// let client = ArcGISClient::new(auth);
    /// let service = MapServiceClient::new(
    ///     "https://services.arcgis.com/org/arcgis/rest/services/World/MapServer",
    ///     &client,
    /// );
    ///
    /// // Use the fluent export builder
    /// let result = service
    ///     .export()
    ///     .bbox("-118.0,34.0,-117.0,35.0")
    ///     .size(800, 600)
    ///     .format(ImageFormat::Png32)
    ///     .transparent(true)
    ///     .execute(ExportTarget::to_bytes())
    ///     .await?;
    ///
    /// if let Some(bytes) = result.bytes() {
    ///     println!("Exported {} bytes", bytes.len());
    /// }
    /// # Ok(())
    /// # }
    /// ```
    #[instrument(skip(self), fields(base_url = %self.base_url))]
    pub fn export(&'a self) -> super::ExportMapBuilder<'a> {
        tracing::debug!("Creating export builder");
        super::ExportMapBuilder::new(self)
    }

    /// Exports a map with pre-built parameters.
    ///
    /// For most use cases, prefer using the fluent [`export()`](Self::export) builder.
    /// This method is useful when you need to construct parameters programmatically
    /// or reuse the same parameters multiple times.
    ///
    /// # Arguments
    ///
    /// * `params` - Export parameters (bbox is required)
    /// * `target` - Where to write the exported image (Path, Bytes, or Writer)
    ///
    /// # Example
    ///
    /// ```no_run
    /// use arcgis::{
    ///     ApiKeyAuth, ArcGISClient, MapServiceClient, ExportMapParams,
    ///     ExportMapParamsBuilder, ExportTarget, ImageFormat,
    /// };
    ///
    /// # async fn example() -> arcgis::Result<()> {
    /// let auth = ApiKeyAuth::new("YOUR_API_KEY");
    /// let client = ArcGISClient::new(auth);
    /// let service = MapServiceClient::new("https://example.com/MapServer", &client);
    ///
    /// let params = ExportMapParamsBuilder::default()
    ///     .bbox("-118.0,34.0,-117.0,35.0".to_string())
    ///     .size("800,600".to_string())
    ///     .format(ImageFormat::Png32)
    ///     .build()
    ///     .expect("Valid params");
    ///
    /// let result = service
    ///     .export_map(params, ExportTarget::to_path("map.png"))
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    #[instrument(skip(self, params, target), fields(bbox = %params.bbox(), size = ?params.size()))]
    pub async fn export_map(
        &self,
        params: ExportMapParams,
        target: ExportTarget,
    ) -> Result<ExportResult> {
        tracing::debug!("Exporting map");

        // Validate required parameters
        if params.bbox().is_empty() {
            tracing::error!("bbox parameter is required but empty");
            return Err(crate::Error::from(crate::ErrorKind::Api {
                code: 400,
                message: "bbox parameter is required".to_string(),
            }));
        }

        let url = format!("{}/export", self.base_url);
        let token = self.client.auth().get_token().await?;

        tracing::debug!(url = %url, "Sending export request");

        // Handle different response formats
        match params.format_response() {
            ResponseFormat::Image => {
                // Direct binary response - stream immediately
                self.stream_export(&url, &params, &token, target).await
            }
            ResponseFormat::Json | ResponseFormat::PJson => {
                // JSON response with href - fetch image from href
                self.export_via_json(&url, &params, &token, target).await
            }
            _ => {
                tracing::error!(format = ?params.format_response(), "Unsupported response format");
                Err(crate::Error::from(crate::ErrorKind::Api {
                    code: 400,
                    message: format!(
                        "Unsupported response format: {:?}",
                        params.format_response()
                    ),
                }))
            }
        }
    }

    /// Exports a tile from a cached map service.
    ///
    /// Retrieves a specific tile from a tiled/cached map service.
    /// Not all map services support tiles - check service metadata first.
    ///
    /// # Arguments
    ///
    /// * `coord` - Tile coordinate (level, row, column)
    /// * `target` - Where to write the tile image (Path, Bytes, or Writer)
    ///
    /// # Example
    ///
    /// ```no_run
    /// use arcgis::{ApiKeyAuth, ArcGISClient, MapServiceClient, TileCoordinate, ExportTarget};
    ///
    /// # async fn example() -> arcgis::Result<()> {
    /// let auth = ApiKeyAuth::new("YOUR_API_KEY");
    /// let client = ArcGISClient::new(auth);
    /// let service = MapServiceClient::new("https://example.com/MapServer", &client);
    ///
    /// // Get tile at level 5, row 10, column 15
    /// let coord = TileCoordinate::new(5, 10, 15);
    /// let result = service
    ///     .export_tile(coord, ExportTarget::to_bytes())
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    #[instrument(skip(self, target), fields(level = coord.level(), row = coord.row(), col = coord.col()))]
    pub async fn export_tile(
        &self,
        coord: TileCoordinate,
        target: ExportTarget,
    ) -> Result<ExportResult> {
        tracing::debug!("Exporting tile");

        let url = format!(
            "{}/tile/{}/{}/{}",
            self.base_url,
            coord.level(),
            coord.row(),
            coord.col()
        );
        let token = self.client.auth().get_token().await?;

        tracing::debug!(url = %url, "Sending tile request");

        let response = self
            .client
            .http()
            .get(&url)
            .query(&[("token", token.as_str())])
            .send()
            .await?;

        let status = response.status();
        if !status.is_success() {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|e| format!("Failed to read error: {}", e));
            tracing::error!(status = %status, error = %error_text, "export_tile failed");
            return Err(crate::Error::from(crate::ErrorKind::Api {
                code: status.as_u16() as i32,
                message: format!("HTTP {}: {}", status, error_text),
            }));
        }

        // Stream tile to target
        self.stream_to_target(response, target).await
    }

    /// Retrieves the legend for all layers in the map service.
    ///
    /// The legend provides information about the symbols and labels
    /// used to represent features in each layer.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use arcgis::{ApiKeyAuth, ArcGISClient, MapServiceClient};
    ///
    /// # async fn example() -> arcgis::Result<()> {
    /// let auth = ApiKeyAuth::new("YOUR_API_KEY");
    /// let client = ArcGISClient::new(auth);
    /// let service = MapServiceClient::new("https://example.com/MapServer", &client);
    ///
    /// let legend = service.get_legend().await?;
    ///
    /// for layer in &legend.layers {
    ///     println!("Layer {}: {}", layer.layer_id(), layer.layer_name());
    ///     for symbol in layer.legend() {
    ///         println!("  - {}", symbol.label());
    ///     }
    /// }
    /// # Ok(())
    /// # }
    /// ```
    #[instrument(skip(self), fields(base_url = %self.base_url))]
    pub async fn get_legend(&self) -> Result<LegendResponse> {
        tracing::debug!("Retrieving legend");

        let url = format!("{}/legend", self.base_url);
        let token = self.client.auth().get_token().await?;

        tracing::debug!(url = %url, "Sending legend request");

        let response = self
            .client
            .http()
            .get(&url)
            .query(&[("f", "json"), ("token", token.as_str())])
            .send()
            .await?;

        let status = response.status();
        if !status.is_success() {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|e| format!("Failed to read error: {}", e));
            tracing::error!(status = %status, error = %error_text, "get_legend failed");
            return Err(crate::Error::from(crate::ErrorKind::Api {
                code: status.as_u16() as i32,
                message: format!("HTTP {}: {}", status, error_text),
            }));
        }

        let legend: LegendResponse = response.json().await?;

        tracing::info!(layer_count = legend.layers().len(), "Legend retrieved");

        Ok(legend)
    }

    /// Retrieves metadata about the map service.
    ///
    /// Provides comprehensive information about the service including
    /// layers, spatial reference, extent, tile info (if cached), and capabilities.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use arcgis::{ApiKeyAuth, ArcGISClient, MapServiceClient};
    ///
    /// # async fn example() -> arcgis::Result<()> {
    /// let auth = ApiKeyAuth::new("YOUR_API_KEY");
    /// let client = ArcGISClient::new(auth);
    /// let service = MapServiceClient::new("https://example.com/MapServer", &client);
    ///
    /// let metadata = service.get_metadata().await?;
    ///
    /// println!("Layers: {}", metadata.layers().len());
    /// if let Some(desc) = metadata.description() {
    ///     println!("Description: {}", desc);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    #[instrument(skip(self), fields(base_url = %self.base_url))]
    pub async fn get_metadata(&self) -> Result<MapServiceMetadata> {
        tracing::debug!("Retrieving metadata");

        let url = &self.base_url;
        let token = self.client.auth().get_token().await?;

        tracing::debug!(url = %url, "Sending metadata request");

        let response = self
            .client
            .http()
            .get(url)
            .query(&[("f", "json"), ("token", token.as_str())])
            .send()
            .await?;

        let status = response.status();
        if !status.is_success() {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|e| format!("Failed to read error: {}", e));
            tracing::error!(status = %status, error = %error_text, "get_metadata failed");
            return Err(crate::Error::from(crate::ErrorKind::Api {
                code: status.as_u16() as i32,
                message: format!("HTTP {}: {}", status, error_text),
            }));
        }

        let metadata: MapServiceMetadata = response.json().await?;

        tracing::info!(layers = metadata.layers().len(), "Metadata retrieved");

        Ok(metadata)
    }

    /// Identifies features at a specific location on the map.
    ///
    /// Returns information about features from visible layers at the
    /// specified geometry (typically a point, but can be other geometries).
    ///
    /// # Arguments
    ///
    /// * `params` - Identify parameters (geometry and image extent are required)
    ///
    /// # Example
    ///
    /// ```no_run
    /// use arcgis::{
    ///     ApiKeyAuth, ArcGISClient, MapServiceClient, IdentifyParams,
    ///     IdentifyParamsBuilder, LayerSelection, GeometryType,
    /// };
    ///
    /// # async fn example() -> arcgis::Result<()> {
    /// let auth = ApiKeyAuth::new("YOUR_API_KEY");
    /// let client = ArcGISClient::new(auth);
    /// let service = MapServiceClient::new("https://example.com/MapServer", &client);
    ///
    /// let params = IdentifyParamsBuilder::default()
    ///     .geometry("{\"x\":-118.0,\"y\":34.0}".to_string())
    ///     .geometry_type(GeometryType::Point)
    ///     .map_extent("-120,32,-116,36".to_string())
    ///     .image_display("800,600,96".to_string())
    ///     .layers(LayerSelection::Visible)
    ///     .build()
    ///     .expect("Valid params");
    ///
    /// let response = service.identify(params).await?;
    ///
    /// for result in response.results() {
    ///     println!("Found feature in layer {}", result.layer_id());
    /// }
    /// # Ok(())
    /// # }
    /// ```
    #[instrument(skip(self, params), fields(geometry_type = ?params.geometry_type()))]
    pub async fn identify(&self, params: IdentifyParams) -> Result<IdentifyResponse> {
        tracing::debug!("Identifying features");

        let url = format!("{}/identify", self.base_url);
        let token = self.client.auth().get_token().await?;

        tracing::debug!(url = %url, "Sending identify request");

        let response = self
            .client
            .http()
            .get(&url)
            .query(&params)
            .query(&[("token", token.as_str()), ("f", "json")])
            .send()
            .await?;

        let status = response.status();
        if !status.is_success() {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|e| format!("Failed to read error: {}", e));
            tracing::error!(status = %status, error = %error_text, "identify failed");
            return Err(crate::Error::from(crate::ErrorKind::Api {
                code: status.as_u16() as i32,
                message: format!("HTTP {}: {}", status, error_text),
            }));
        }

        let identify_response: IdentifyResponse = response.json().await?;

        tracing::info!(
            result_count = identify_response.results().len(),
            "Identify completed"
        );

        Ok(identify_response)
    }

    // === Helper methods ===

    /// Streams export response directly (for ResponseFormat::Image).
    #[instrument(skip(self, params, token, target))]
    async fn stream_export(
        &self,
        url: &str,
        params: &ExportMapParams,
        token: &str,
        target: ExportTarget,
    ) -> Result<ExportResult> {
        tracing::debug!("Streaming export (direct image response)");

        let response = self
            .client
            .http()
            .get(url)
            .query(&params)
            .query(&[("token", token)])
            .send()
            .await?;

        let status = response.status();
        if !status.is_success() {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|e| format!("Failed to read error: {}", e));
            tracing::error!(status = %status, error = %error_text, "stream_export failed");
            return Err(crate::Error::from(crate::ErrorKind::Api {
                code: status.as_u16() as i32,
                message: format!("HTTP {}: {}", status, error_text),
            }));
        }

        self.stream_to_target(response, target).await
    }

    /// Exports via JSON response (fetches href).
    #[instrument(skip(self, params, token, target))]
    async fn export_via_json(
        &self,
        url: &str,
        params: &ExportMapParams,
        token: &str,
        target: ExportTarget,
    ) -> Result<ExportResult> {
        tracing::debug!("Exporting via JSON (will fetch href)");

        let response = self
            .client
            .http()
            .get(url)
            .query(&params)
            .query(&[("token", token)])
            .send()
            .await?;

        let status = response.status();
        if !status.is_success() {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|e| format!("Failed to read error: {}", e));
            tracing::error!(status = %status, error = %error_text, "export_via_json failed");
            return Err(crate::Error::from(crate::ErrorKind::Api {
                code: status.as_u16() as i32,
                message: format!("HTTP {}: {}", status, error_text),
            }));
        }

        let export_response: ExportMapResponse = response.json().await?;

        tracing::debug!(
            href = %export_response.href(),
            width = export_response.width(),
            height = export_response.height(),
            "Received export response, fetching image"
        );

        // Fetch the actual image from href
        let image_response = self
            .client
            .http()
            .get(export_response.href())
            .query(&[("token", token)])
            .send()
            .await?;

        let status = image_response.status();
        if !status.is_success() {
            let error_text = image_response
                .text()
                .await
                .unwrap_or_else(|e| format!("Failed to read error: {}", e));
            tracing::error!(status = %status, error = %error_text, "Image fetch failed");
            return Err(crate::Error::from(crate::ErrorKind::Api {
                code: status.as_u16() as i32,
                message: format!("HTTP {}: {}", status, error_text),
            }));
        }

        self.stream_to_target(image_response, target).await
    }

    /// Streams response to target (Path, Bytes, or Writer).
    #[instrument(skip(self, response, target))]
    async fn stream_to_target(
        &self,
        response: reqwest::Response,
        target: ExportTarget,
    ) -> Result<ExportResult> {
        match target {
            ExportTarget::Path(path) => {
                tracing::debug!(path = %path.display(), "Streaming to file");

                let mut file = tokio::fs::File::create(&path).await?;
                let mut stream = response.bytes_stream();
                let mut total_bytes = 0u64;

                while let Some(chunk_result) = stream.next().await {
                    let chunk = chunk_result?;
                    file.write_all(&chunk).await?;
                    total_bytes += chunk.len() as u64;
                }

                file.flush().await?;

                tracing::info!(
                    path = %path.display(),
                    bytes = total_bytes,
                    "Streaming to file completed"
                );

                Ok(ExportResult::Path(path))
            }
            ExportTarget::Bytes => {
                tracing::debug!("Collecting bytes to memory");

                let mut stream = response.bytes_stream();
                let mut buffer = Vec::new();

                while let Some(chunk_result) = stream.next().await {
                    let chunk = chunk_result?;
                    buffer.extend_from_slice(&chunk);
                }

                tracing::info!(bytes = buffer.len(), "Streaming to bytes completed");

                Ok(ExportResult::Bytes(buffer))
            }
            ExportTarget::Writer(mut writer) => {
                tracing::debug!("Streaming to writer");

                let mut stream = response.bytes_stream();
                let mut total_bytes = 0u64;

                while let Some(chunk_result) = stream.next().await {
                    let chunk = chunk_result?;
                    writer.write_all(&chunk).await?;
                    total_bytes += chunk.len() as u64;
                }

                writer.flush().await?;

                tracing::info!(bytes = total_bytes, "Streaming to writer completed");

                Ok(ExportResult::Written(total_bytes))
            }
        }
    }

    /// Searches for features containing the specified text in a map service.
    ///
    /// The find operation searches for text in one or more fields across multiple layers.
    /// It returns features that contain the search text along with their attributes and geometries.
    ///
    /// # Arguments
    ///
    /// * `params` - Find parameters including search text, layers, and fields to search
    ///
    /// # Example
    ///
    /// ```no_run
    /// use arcgis::{ArcGISClient, ApiKeyAuth, MapServiceClient, FindParams};
    ///
    /// # async fn example() -> arcgis::Result<()> {
    /// let auth = ApiKeyAuth::new("YOUR_API_KEY");
    /// let client = ArcGISClient::new(auth);
    /// let service = MapServiceClient::new("https://example.com/MapServer", &client);
    ///
    /// let params = FindParams::builder()
    ///     .search_text("Main Street")
    ///     .layers(vec![0, 1])
    ///     .search_fields(vec!["NAME".to_string(), "STREET".to_string()])
    ///     .build()
    ///     .expect("Valid params");
    ///
    /// let result = service.find(params).await?;
    /// # Ok(())
    /// # }
    /// ```
    #[instrument(skip(self, params))]
    pub async fn find(&self, params: crate::FindParams) -> Result<crate::FindResponse> {
        tracing::debug!("Finding features by text search");

        let url = format!("{}/find", self.base_url);
        let token = self.client.auth().get_token().await?;

        tracing::debug!(url = %url, search_text = %params.search_text(), "Sending find request");

        let form = serde_urlencoded::to_string(&params)?;
        let response = self
            .client
            .http()
            .get(&url)
            .query(&[("token", token.as_str())])
            .query(&[("f", "json")])
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(form)
            .send()
            .await?;

        let status = response.status();
        if !status.is_success() {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|e| format!("Failed to read error: {}", e));
            tracing::error!(status = %status, error = %error_text, "find request failed");
            return Err(crate::Error::from(crate::ErrorKind::Api {
                code: status.as_u16() as i32,
                message: format!("HTTP {}: {}", status, error_text),
            }));
        }

        let result: crate::FindResponse = response.json().await?;

        tracing::info!(result_count = result.results().len(), "Find completed");

        Ok(result)
    }

    /// Generates KML (Keyhole Markup Language) output for the map service.
    ///
    /// This operation returns a KML representation of the map that can be used
    /// in Google Earth and other KML viewers.
    ///
    /// # Arguments
    ///
    /// * `params` - Parameters for KML generation including layers and image options
    ///
    /// # Example
    ///
    /// ```no_run
    /// use arcgis::{ArcGISClient, ApiKeyAuth, MapServiceClient, GenerateKmlParams};
    ///
    /// # async fn example() -> arcgis::Result<()> {
    /// let auth = ApiKeyAuth::new("YOUR_API_KEY");
    /// let client = ArcGISClient::new(auth);
    /// let service = MapServiceClient::new("https://example.com/MapServer", &client);
    ///
    /// let params = GenerateKmlParams::builder()
    ///     .doc_name("MyMap")
    ///     .layers(vec![0, 1])
    ///     .build()
    ///     .expect("Valid params");
    ///
    /// let kml = service.generate_kml(params).await?;
    /// # Ok(())
    /// # }
    /// ```
    #[instrument(skip(self, params))]
    pub async fn generate_kml(&self, params: crate::GenerateKmlParams) -> Result<String> {
        tracing::debug!("Generating KML");

        let url = format!("{}/generateKml", self.base_url);
        let token = self.client.auth().get_token().await?;

        tracing::debug!(url = %url, doc_name = %params.doc_name(), "Sending generateKml request");

        let form = serde_urlencoded::to_string(&params)?;
        let response = self
            .client
            .http()
            .get(&url)
            .query(&[("token", token.as_str())])
            .query(&[("f", "kmz")]) // KML format
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(form)
            .send()
            .await?;

        let status = response.status();
        if !status.is_success() {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|e| format!("Failed to read error: {}", e));
            tracing::error!(status = %status, error = %error_text, "generateKml request failed");
            return Err(crate::Error::from(crate::ErrorKind::Api {
                code: status.as_u16() as i32,
                message: format!("HTTP {}: {}", status, error_text),
            }));
        }

        let kml = response.text().await?;

        tracing::info!(kml_length = kml.len(), "KML generation completed");

        Ok(kml)
    }

    /// Generates a classification renderer for a layer.
    ///
    /// This operation generates renderer definitions (symbols, colors, class breaks)
    /// based on the data in a layer field. Useful for creating dynamic visualizations
    /// like choropleth maps, graduated symbols, etc.
    ///
    /// # Arguments
    ///
    /// * `layer_id` - The layer to generate renderer for
    /// * `params` - Parameters for renderer generation including classification method
    ///
    /// # Example
    ///
    /// ```no_run
    /// use arcgis::{ArcGISClient, ApiKeyAuth, MapServiceClient, GenerateRendererParams};
    ///
    /// # async fn example() -> arcgis::Result<()> {
    /// let auth = ApiKeyAuth::new("YOUR_API_KEY");
    /// let client = ArcGISClient::new(auth);
    /// let service = MapServiceClient::new("https://example.com/MapServer", &client);
    ///
    /// let params = GenerateRendererParams::builder()
    ///     .classification_field("POPULATION")
    ///     .classification_method("natural-breaks")
    ///     .break_count(5)
    ///     .build()
    ///     .expect("Valid params");
    ///
    /// let renderer = service.generate_renderer(0, params).await?;
    /// # Ok(())
    /// # }
    /// ```
    #[instrument(skip(self, params), fields(layer_id = layer_id))]
    pub async fn generate_renderer(
        &self,
        layer_id: i32,
        params: crate::GenerateRendererParams,
    ) -> Result<crate::RendererResponse> {
        tracing::debug!("Generating renderer for layer");

        let url = format!("{}/{}/generateRenderer", self.base_url, layer_id);
        let token = self.client.auth().get_token().await?;

        tracing::debug!(
            url = %url,
            classification_field = %params.classification_field(),
            "Sending generateRenderer request"
        );

        let params_json = serde_json::to_string(&params)?;
        let form = vec![
            ("classificationDef", params_json.as_str()),
            ("f", "json"),
            ("token", token.as_str()),
        ];

        let response = self.client.http().post(&url).form(&form).send().await?;

        let status = response.status();
        if !status.is_success() {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|e| format!("Failed to read error: {}", e));
            tracing::error!(status = %status, error = %error_text, "generateRenderer request failed");
            return Err(crate::Error::from(crate::ErrorKind::Api {
                code: status.as_u16() as i32,
                message: format!("HTTP {}: {}", status, error_text),
            }));
        }

        let result: crate::RendererResponse = response.json().await?;

        tracing::info!("Renderer generation completed");

        Ok(result)
    }

    /// Queries field domains and subtype information for map service layers.
    ///
    /// This operation retrieves domain definitions (coded values, ranges) and subtype
    /// information for one or more layers in the map service. Useful for understanding
    /// valid values and field constraints.
    ///
    /// # Arguments
    ///
    /// * `layers` - Vector of layer IDs to query domains for (empty for all layers)
    ///
    /// # Example
    ///
    /// ```no_run
    /// use arcgis::{ArcGISClient, ApiKeyAuth, MapServiceClient, LayerId};
    ///
    /// # async fn example() -> arcgis::Result<()> {
    /// let auth = ApiKeyAuth::new("YOUR_API_KEY");
    /// let client = ArcGISClient::new(auth);
    /// let service = MapServiceClient::new("https://example.com/MapServer", &client);
    ///
    /// // Query domains for specific layers
    /// let domains = service
    ///     .query_domains(vec![LayerId::new(0), LayerId::new(1)])
    ///     .await?;
    ///
    /// // Query domains for all layers
    /// let all_domains = service.query_domains(vec![]).await?;
    /// # Ok(())
    /// # }
    /// ```
    #[instrument(skip(self), fields(layer_count = layers.len()))]
    pub async fn query_domains(
        &self,
        layers: Vec<crate::LayerId>,
    ) -> Result<crate::QueryDomainsResponse> {
        tracing::debug!("Querying map service domains");

        let url = format!("{}/queryDomains", self.base_url);
        let token = self.client.auth().get_token().await?;

        let layers_str = layers
            .iter()
            .map(|id| id.to_string())
            .collect::<Vec<_>>()
            .join(",");

        tracing::debug!(url = %url, layers = %layers_str, "Sending queryDomains request");

        let mut form = vec![("f", "json"), ("token", token.as_str())];

        if !layers_str.is_empty() {
            form.push(("layers", &layers_str));
        }

        let response = self.client.http().post(&url).form(&form).send().await?;

        let status = response.status();
        if !status.is_success() {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|e| format!("Failed to read error: {}", e));
            tracing::error!(status = %status, error = %error_text, "queryDomains request failed");
            return Err(crate::Error::from(crate::ErrorKind::Api {
                code: status.as_u16() as i32,
                message: format!("HTTP {}: {}", status, error_text),
            }));
        }

        let result: crate::QueryDomainsResponse = response.json().await?;

        tracing::info!(
            layer_count = result.layers().len(),
            "Map service queryDomains completed"
        );

        Ok(result)
    }
}
