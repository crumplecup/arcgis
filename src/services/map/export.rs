//! Export map builder for fluent API.

use super::ResponseFormat;
use crate::{
    ExportMapParams, ExportResult, ExportTarget, ImageFormat, LayerOperation, MapServiceClient,
    Result, TimeRelation,
};
use tracing::instrument;

/// A fluent builder for constructing and executing map export operations.
///
/// This provides an ergonomic API for exporting maps without
/// manually constructing [`ExportMapParams`].
///
/// # Example
///
/// ```no_run
/// use arcgis::{ArcGISClient, ApiKeyAuth, MapServiceClient, ExportTarget, ImageFormat};
///
/// # async fn example() -> arcgis::Result<()> {
/// let auth = ApiKeyAuth::new("YOUR_API_KEY");
/// let client = ArcGISClient::new(auth);
/// let service = MapServiceClient::new("https://example.com/MapServer", &client);
///
/// // Export with fluent builder
/// let result = service
///     .export()
///     .bbox("-118.0,34.0,-117.0,35.0")
///     .size(800, 600)
///     .format(ImageFormat::Png32)
///     .transparent(true)
///     .dpi(96)
///     .execute(ExportTarget::to_path("map.png"))
///     .await?;
/// # Ok(())
/// # }
/// ```
pub struct ExportMapBuilder<'a> {
    client: &'a MapServiceClient<'a>,
    params: ExportMapParams,
}

impl<'a> ExportMapBuilder<'a> {
    /// Creates a new export builder.
    ///
    /// Typically you don't call this directly - use [`MapServiceClient::export`] instead.
    #[instrument(skip(client))]
    pub(crate) fn new(client: &'a MapServiceClient<'a>) -> Self {
        tracing::debug!("Creating ExportMapBuilder");
        Self {
            client,
            params: ExportMapParams::default(),
        }
    }

    /// Sets the bounding box extent to export (REQUIRED).
    ///
    /// Format: "xmin,ymin,xmax,ymax"
    ///
    /// # Example
    /// ```no_run
    /// # use arcgis::{MapServiceClient, ExportTarget};
    /// # async fn example(service: &MapServiceClient<'_>) -> arcgis::Result<()> {
    /// service
    ///     .export()
    ///     .bbox("-180,-90,180,90")
    ///     .execute(ExportTarget::to_bytes())
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn bbox(mut self, bbox: impl Into<String>) -> Self {
        self.params.bbox = bbox.into();
        self
    }

    /// Sets the spatial reference WKID for the bounding box.
    ///
    /// If not specified, defaults to the map's spatial reference.
    pub fn bbox_sr(mut self, wkid: i32) -> Self {
        self.params.bbox_sr = Some(wkid);
        self
    }

    /// Sets the layer visibility string.
    ///
    /// Controls which layers are visible in the exported map.
    /// Format depends on operation: "show:1,2,3" or "hide:4,5"
    ///
    /// # Example
    /// ```no_run
    /// # use arcgis::{MapServiceClient, ExportTarget};
    /// # async fn example(service: &MapServiceClient<'_>) -> arcgis::Result<()> {
    /// service
    ///     .export()
    ///     .bbox("-118,34,-117,35")
    ///     .layers("show:0,1,2")  // Only show layers 0, 1, 2
    ///     .execute(ExportTarget::to_bytes())
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn layers(mut self, layers: impl Into<String>) -> Self {
        self.params.layers = Some(layers.into());
        self
    }

    /// Sets layer visibility using an operation and layer IDs.
    ///
    /// Convenience method that constructs the layers string.
    ///
    /// # Example
    /// ```no_run
    /// # use arcgis::{MapServiceClient, ExportTarget, LayerOperation};
    /// # async fn example(service: &MapServiceClient<'_>) -> arcgis::Result<()> {
    /// service
    ///     .export()
    ///     .bbox("-118,34,-117,35")
    ///     .layer_visibility(LayerOperation::Show, &[0, 1, 2])
    ///     .execute(ExportTarget::to_bytes())
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn layer_visibility(mut self, operation: LayerOperation, layer_ids: &[i32]) -> Self {
        let ids = layer_ids
            .iter()
            .map(|id| id.to_string())
            .collect::<Vec<_>>()
            .join(",");
        self.params.layers = Some(format!("{}:{}", operation.as_str(), ids));
        self
    }

    /// Sets layer definition expressions (filters).
    ///
    /// Format: JSON array or semicolon-separated expressions.
    /// Example: `"0:POPULATION > 100000;1:STATE = 'CA'"`
    pub fn layer_defs(mut self, defs: impl Into<String>) -> Self {
        self.params.layer_defs = Some(defs.into());
        self
    }

    /// Sets the output image size.
    ///
    /// # Example
    /// ```no_run
    /// # use arcgis::{MapServiceClient, ExportTarget};
    /// # async fn example(service: &MapServiceClient<'_>) -> arcgis::Result<()> {
    /// service
    ///     .export()
    ///     .bbox("-118,34,-117,35")
    ///     .size(1024, 768)
    ///     .execute(ExportTarget::to_bytes())
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn size(mut self, width: u32, height: u32) -> Self {
        self.params.size = Some(format!("{},{}", width, height));
        self
    }

    /// Sets the dots-per-inch (resolution) of the exported map.
    ///
    /// Common values: 96 (screen), 300 (print).
    pub fn dpi(mut self, dpi: i32) -> Self {
        self.params.dpi = Some(dpi);
        self
    }

    /// Sets the spatial reference WKID for the output image.
    ///
    /// If not specified, uses the map's spatial reference.
    pub fn image_sr(mut self, wkid: i32) -> Self {
        self.params.image_sr = Some(wkid);
        self
    }

    /// Sets the output image format.
    ///
    /// # Example
    /// ```no_run
    /// # use arcgis::{MapServiceClient, ExportTarget, ImageFormat};
    /// # async fn example(service: &MapServiceClient<'_>) -> arcgis::Result<()> {
    /// service
    ///     .export()
    ///     .bbox("-118,34,-117,35")
    ///     .format(ImageFormat::Png32)
    ///     .execute(ExportTarget::to_bytes())
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn format(mut self, format: ImageFormat) -> Self {
        self.params.format = Some(format);
        self
    }

    /// Sets whether the background should be transparent.
    ///
    /// Only applicable for formats that support transparency (PNG, GIF).
    ///
    /// # Example
    /// ```no_run
    /// # use arcgis::{MapServiceClient, ExportTarget, ImageFormat};
    /// # async fn example(service: &MapServiceClient<'_>) -> arcgis::Result<()> {
    /// service
    ///     .export()
    ///     .bbox("-118,34,-117,35")
    ///     .format(ImageFormat::Png32)
    ///     .transparent(true)
    ///     .execute(ExportTarget::to_bytes())
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn transparent(mut self, transparent: bool) -> Self {
        self.params.transparent = Some(transparent);
        self
    }

    /// Sets the temporal extent for time-aware layers.
    ///
    /// Format: "startTime,endTime" or single instant "time"
    /// Times are Unix timestamps in milliseconds.
    pub fn time(mut self, time: impl Into<String>) -> Self {
        self.params.time = Some(time.into());
        self
    }

    /// Sets the time relationship for temporal queries.
    pub fn time_relation(mut self, relation: TimeRelation) -> Self {
        self.params.time_relation = Some(relation);
        self
    }

    /// Sets time options for individual layers.
    ///
    /// JSON format specifying time extent per layer.
    pub fn layer_time_options(mut self, options: impl Into<String>) -> Self {
        self.params.layer_time_options = Some(options.into());
        self
    }

    /// Sets dynamic layers definition.
    ///
    /// JSON format for defining dynamic layers at request time.
    pub fn dynamic_layers(mut self, layers: impl Into<String>) -> Self {
        self.params.dynamic_layers = Some(layers.into());
        self
    }

    /// Sets the GDB version to query.
    pub fn gdb_version(mut self, version: impl Into<String>) -> Self {
        self.params.gdb_version = Some(version.into());
        self
    }

    /// Sets the map scale for the export.
    ///
    /// When specified, overrides size-based scale calculation.
    pub fn map_scale(mut self, scale: f64) -> Self {
        self.params.map_scale = Some(scale);
        self
    }

    /// Sets the rotation angle for the map (in degrees).
    ///
    /// Positive values rotate clockwise.
    pub fn rotation(mut self, degrees: f64) -> Self {
        self.params.rotation = Some(degrees);
        self
    }

    /// Sets datum transformations to apply.
    ///
    /// JSON array specifying transformations between spatial references.
    pub fn datum_transformations(mut self, transformations: impl Into<String>) -> Self {
        self.params.datum_transformations = Some(transformations.into());
        self
    }

    /// Sets the map range values.
    pub fn map_range_values(mut self, values: impl Into<String>) -> Self {
        self.params.map_range_values = Some(values.into());
        self
    }

    /// Sets layer parameterized expressions.
    pub fn layer_parameter_values(mut self, values: impl Into<String>) -> Self {
        self.params.layer_parameter_values = Some(values.into());
        self
    }

    /// Sets whether to format the response as pretty JSON.
    pub fn pretty_json(mut self, pretty: bool) -> Self {
        self.params.format_response = if pretty {
            ResponseFormat::PJson
        } else {
            ResponseFormat::Json
        };
        self
    }

    /// Sets the response format.
    ///
    /// - `ResponseFormat::Image` - Direct binary image response (fastest)
    /// - `ResponseFormat::Json` - JSON with href to image
    /// - `ResponseFormat::PJson` - Pretty-printed JSON with href
    ///
    /// Default is `Json`.
    pub fn response_format(mut self, format: ResponseFormat) -> Self {
        self.params.format_response = format;
        self
    }

    /// Executes the export operation.
    ///
    /// # Arguments
    ///
    /// * `target` - Where to write the exported image (Path, Bytes, or Writer)
    ///
    /// # Example
    ///
    /// ```no_run
    /// use arcgis::{ArcGISClient, ApiKeyAuth, MapServiceClient, ExportTarget, ImageFormat};
    ///
    /// # async fn example() -> arcgis::Result<()> {
    /// let auth = ApiKeyAuth::new("YOUR_API_KEY");
    /// let client = ArcGISClient::new(auth);
    /// let service = MapServiceClient::new("https://example.com/MapServer", &client);
    ///
    /// // Export to file
    /// let result = service
    ///     .export()
    ///     .bbox("-118.0,34.0,-117.0,35.0")
    ///     .size(800, 600)
    ///     .format(ImageFormat::Png32)
    ///     .execute(ExportTarget::to_path("map.png"))
    ///     .await?;
    ///
    /// // Export to bytes
    /// let result = service
    ///     .export()
    ///     .bbox("-118.0,34.0,-117.0,35.0")
    ///     .size(400, 300)
    ///     .execute(ExportTarget::to_bytes())
    ///     .await?;
    ///
    /// if let Some(bytes) = result.bytes() {
    ///     println!("Exported {} bytes", bytes.len());
    /// }
    /// # Ok(())
    /// # }
    /// ```
    #[instrument(skip(self, target), fields(bbox = %self.params.bbox, size = ?self.params.size))]
    pub async fn execute(self, target: ExportTarget) -> Result<ExportResult> {
        tracing::debug!("Executing export");
        self.client.export_map(self.params, target).await
    }
}
