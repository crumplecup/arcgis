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
    #[instrument(skip(self, params, target), fields(bbox = %params.bbox, size = ?params.size))]
    pub async fn export_map(
        &self,
        params: ExportMapParams,
        target: ExportTarget,
    ) -> Result<ExportResult> {
        tracing::debug!("Exporting map");

        // Validate required parameters
        if params.bbox.is_empty() {
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
        match params.format_response {
            ResponseFormat::Image => {
                // Direct binary response - stream immediately
                self.stream_export(&url, &params, &token, target).await
            }
            ResponseFormat::Json | ResponseFormat::PJson => {
                // JSON response with href - fetch image from href
                self.export_via_json(&url, &params, &token, target).await
            }
            _ => {
                tracing::error!(format = ?params.format_response, "Unsupported response format");
                Err(crate::Error::from(crate::ErrorKind::Api {
                    code: 400,
                    message: format!("Unsupported response format: {:?}", params.format_response),
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

        tracing::info!(layer_count = legend.layers.len(), "Legend retrieved");

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
    #[instrument(skip(self, params), fields(geometry_type = ?params.geometry_type))]
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
}
