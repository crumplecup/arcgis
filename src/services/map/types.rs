//! Types for map service operations.

use crate::{ArcGISGeometry, GeometryType};
use derive_getters::Getters;
use derive_setters::Setters;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use tokio::io::AsyncWrite;

use super::{ImageFormat, LayerSelection, ResponseFormat, TimeRelation};

/// Target for map export output.
///
/// Similar to `DownloadTarget` from feature service attachments.
pub enum ExportTarget {
    /// Export to file path.
    Path(PathBuf),

    /// Export to in-memory bytes.
    Bytes,

    /// Stream to async writer.
    Writer(Box<dyn AsyncWrite + Send + Sync + Unpin>),
}

impl ExportTarget {
    /// Creates an export target for a file path.
    pub fn to_path(path: impl Into<PathBuf>) -> Self {
        Self::Path(path.into())
    }

    /// Creates an export target for in-memory bytes.
    pub fn to_bytes() -> Self {
        Self::Bytes
    }

    /// Creates an export target for an async writer.
    pub fn to_writer<W>(writer: W) -> Self
    where
        W: AsyncWrite + Send + Sync + Unpin + 'static,
    {
        Self::Writer(Box::new(writer))
    }
}

/// Result of a map export operation.
pub enum ExportResult {
    /// Image written to file path.
    Path(PathBuf),

    /// Image loaded into memory.
    Bytes(Vec<u8>),

    /// Bytes written to writer.
    Written(u64),
}

impl ExportResult {
    /// Returns the path if this is a Path result.
    pub fn path(&self) -> Option<&PathBuf> {
        match self {
            Self::Path(p) => Some(p),
            _ => None,
        }
    }

    /// Returns the bytes if this is a Bytes result.
    pub fn bytes(&self) -> Option<&[u8]> {
        match self {
            Self::Bytes(b) => Some(b),
            _ => None,
        }
    }

    /// Returns the bytes count if this is a Written result.
    pub fn written(&self) -> Option<u64> {
        match self {
            Self::Written(n) => Some(*n),
            _ => None,
        }
    }
}

/// Builder for layer definition expressions.
///
/// Provides a type-safe way to construct layer definition expressions (SQL WHERE
/// clauses) for filtering map layers. Each layer can have its own WHERE clause.
///
/// # Example
///
/// ```
/// use arcgis::LayerDefinitions;
///
/// let defs = LayerDefinitions::new()
///     .add_layer_def(0, "POPULATION > 100000")
///     .add_layer_def(1, "STATE = 'CA'")
///     .build();
/// ```
#[derive(Debug, Clone, Default)]
pub struct LayerDefinitions {
    definitions: HashMap<i32, String>,
}

impl LayerDefinitions {
    /// Creates a new empty layer definitions builder.
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds a definition expression for a specific layer.
    ///
    /// # Arguments
    ///
    /// * `layer_id` - The layer ID to apply the filter to
    /// * `where_clause` - SQL WHERE clause (without the "WHERE" keyword)
    pub fn add_layer_def(mut self, layer_id: i32, where_clause: impl Into<String>) -> Self {
        self.definitions.insert(layer_id, where_clause.into());
        self
    }

    /// Adds a definition expression for a specific layer (mutable version).
    pub fn add_layer_def_mut(&mut self, layer_id: i32, where_clause: impl Into<String>) {
        self.definitions.insert(layer_id, where_clause.into());
    }

    /// Builds the layer definitions into the JSON format expected by ArcGIS.
    ///
    /// Returns a JSON string in the format: `{"0": "expression1", "1": "expression2"}`
    pub fn build(&self) -> String {
        serde_json::to_string(&self.definitions).unwrap_or_else(|_| "{}".to_string())
    }

    /// Returns true if no layer definitions have been added.
    pub fn is_empty(&self) -> bool {
        self.definitions.is_empty()
    }

    /// Returns the number of layer definitions.
    pub fn len(&self) -> usize {
        self.definitions.len()
    }

    /// Gets the definition for a specific layer.
    pub fn get(&self, layer_id: i32) -> Option<&String> {
        self.definitions.get(&layer_id)
    }
}

impl From<LayerDefinitions> for String {
    fn from(defs: LayerDefinitions) -> Self {
        defs.build()
    }
}

/// Tile coordinate for tile requests.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Getters)]
pub struct TileCoordinate {
    /// Zoom level
    level: u32,
    /// Row index
    row: u32,
    /// Column index
    col: u32,
}

impl TileCoordinate {
    /// Creates a new tile coordinate.
    pub fn new(level: u32, row: u32, col: u32) -> Self {
        Self { level, row, col }
    }
}

/// Spatial reference information.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Getters)]
#[serde(rename_all = "camelCase")]
pub struct SpatialReference {
    /// Well-known ID (WKID).
    wkid: i32,

    /// Latest WKID (updated/deprecated WKIDs).
    #[serde(skip_serializing_if = "Option::is_none")]
    latest_wkid: Option<i32>,
}

/// Parameters for exporting a map image.
///
/// Use [`ExportMapParams::builder()`] to construct instances.
///
/// # Example
/// ```no_run
/// use arcgis::{ExportMapParams, ImageFormat};
///
/// let params = ExportMapParams::builder()
///     .bbox("-130,20,-65,50")
///     .size("800,600")
///     .format(ImageFormat::Png32)
///     .transparent(true)
///     .build()
///     .expect("Valid params");
/// ```
#[derive(Debug, Clone, Serialize, derive_builder::Builder, derive_getters::Getters, Setters)]
#[builder(setter(into, strip_option), default)]
#[setters(prefix = "set_", borrow_self)]
pub struct ExportMapParams {
    /// Bounding box of the exported image (xmin,ymin,xmax,ymax).
    /// REQUIRED parameter.
    bbox: String,

    /// Spatial reference of the bbox coordinates.
    #[serde(rename = "bboxSR", skip_serializing_if = "Option::is_none")]
    bbox_sr: Option<i32>,

    /// Layer visibility control.
    /// Format: "show:0,2,4" or "hide:1,3" or "include:0,2" or "exclude:1"
    #[serde(skip_serializing_if = "Option::is_none")]
    layers: Option<String>,

    /// Layer definition expressions (filters).
    #[serde(rename = "layerDefs", skip_serializing_if = "Option::is_none")]
    layer_defs: Option<String>,

    /// Image size in pixels (width,height).
    #[serde(skip_serializing_if = "Option::is_none")]
    size: Option<String>,

    /// Spatial reference of the output image.
    #[serde(rename = "imageSR", skip_serializing_if = "Option::is_none")]
    image_sr: Option<i32>,

    /// Historic moment (epoch milliseconds) for archive-enabled layers.
    #[serde(rename = "historicMoment", skip_serializing_if = "Option::is_none")]
    historic_moment: Option<i64>,

    /// Image format.
    #[serde(skip_serializing_if = "Option::is_none")]
    format: Option<ImageFormat>,

    /// Whether to use transparency.
    #[serde(skip_serializing_if = "Option::is_none")]
    transparent: Option<bool>,

    /// Dots per inch (device resolution).
    #[serde(skip_serializing_if = "Option::is_none")]
    dpi: Option<i32>,

    /// Time instant or range (epoch milliseconds).
    /// Format: "timestamp" or "start,end"
    #[serde(skip_serializing_if = "Option::is_none")]
    time: Option<String>,

    /// Time relationship for temporal queries.
    #[serde(rename = "timeRelation", skip_serializing_if = "Option::is_none")]
    time_relation: Option<TimeRelation>,

    /// Per-layer time options (JSON).
    #[serde(rename = "layerTimeOptions", skip_serializing_if = "Option::is_none")]
    layer_time_options: Option<String>,

    /// Dynamic layer definitions (JSON).
    #[serde(rename = "dynamicLayers", skip_serializing_if = "Option::is_none")]
    dynamic_layers: Option<String>,

    /// Geodatabase version name.
    #[serde(rename = "gdbVersion", skip_serializing_if = "Option::is_none")]
    gdb_version: Option<String>,

    /// Map scale (1:x ratio).
    #[serde(rename = "mapScale", skip_serializing_if = "Option::is_none")]
    map_scale: Option<f64>,

    /// Rotation angle in degrees.
    #[serde(skip_serializing_if = "Option::is_none")]
    rotation: Option<f64>,

    /// Datum transformations (JSON array).
    #[serde(
        rename = "datumTransformations",
        skip_serializing_if = "Option::is_none"
    )]
    datum_transformations: Option<String>,

    /// Layer parameter values for parameterized filters (JSON).
    #[serde(
        rename = "layerParameterValues",
        skip_serializing_if = "Option::is_none"
    )]
    layer_parameter_values: Option<String>,

    /// Map-wide range values (JSON).
    #[serde(rename = "mapRangeValues", skip_serializing_if = "Option::is_none")]
    map_range_values: Option<String>,

    /// Per-layer range values (JSON).
    #[serde(rename = "layerRangeValues", skip_serializing_if = "Option::is_none")]
    layer_range_values: Option<String>,

    /// Clipping geometry (JSON).
    #[serde(skip_serializing_if = "Option::is_none")]
    clipping: Option<String>,

    /// Spatial filter (JSON).
    #[serde(rename = "spatialFilter", skip_serializing_if = "Option::is_none")]
    spatial_filter: Option<String>,

    /// Selection definitions for highlighting features (JSON, v11.4+).
    #[serde(
        rename = "selectionDefinitions",
        skip_serializing_if = "Option::is_none"
    )]
    selection_definitions: Option<String>,

    /// Response format.
    #[serde(rename = "f")]
    #[builder(default = "ResponseFormat::Json")]
    format_response: ResponseFormat,
}

impl Default for ExportMapParams {
    fn default() -> Self {
        Self {
            bbox: String::new(),
            bbox_sr: None,
            layers: None,
            layer_defs: None,
            size: Some("400,400".to_string()),
            image_sr: None,
            historic_moment: None,
            format: Some(ImageFormat::Png),
            transparent: Some(false),
            dpi: Some(96),
            time: None,
            time_relation: None,
            layer_time_options: None,
            dynamic_layers: None,
            gdb_version: None,
            map_scale: None,
            rotation: None,
            datum_transformations: None,
            layer_parameter_values: None,
            map_range_values: None,
            layer_range_values: None,
            clipping: None,
            spatial_filter: None,
            selection_definitions: None,
            format_response: ResponseFormat::Json,
        }
    }
}

impl ExportMapParams {
    /// Creates a builder for ExportMapParams.
    pub fn builder() -> ExportMapParamsBuilder {
        ExportMapParamsBuilder::default()
    }
}

/// Extent returned in export response.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, Getters)]
pub struct ExportExtent {
    /// Minimum X coordinate
    xmin: f64,
    /// Minimum Y coordinate
    ymin: f64,
    /// Maximum X coordinate
    xmax: f64,
    /// Maximum Y coordinate
    ymax: f64,
    /// Spatial reference
    #[serde(rename = "spatialReference", skip_serializing_if = "Option::is_none")]
    spatial_reference: Option<SpatialReference>,
}

/// Response from export map operation (JSON format).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Getters)]
pub struct ExportMapResponse {
    /// URL to the exported image.
    href: String,

    /// Image width in pixels.
    width: i32,

    /// Image height in pixels.
    height: i32,

    /// Extent of the exported map (may differ from requested bbox).
    #[serde(skip_serializing_if = "Option::is_none")]
    extent: Option<ExportExtent>,

    /// Map scale.
    #[serde(skip_serializing_if = "Option::is_none")]
    scale: Option<f64>,
}

/// Parameters for identifying features on a map.
///
/// Use [`IdentifyParams::builder()`] to construct instances.
#[derive(Debug, Clone, Serialize, derive_builder::Builder, derive_getters::Getters)]
#[builder(setter(into, strip_option), default)]
pub struct IdentifyParams {
    /// Geometry to identify on (JSON string or simple format).
    geometry: String,

    /// Type of geometry.
    #[serde(rename = "geometryType")]
    geometry_type: GeometryType,

    /// Tolerance in screen pixels.
    tolerance: i32,

    /// Map extent being viewed (xmin,ymin,xmax,ymax).
    #[serde(rename = "mapExtent")]
    map_extent: String,

    /// Image display parameters (width,height,dpi).
    #[serde(rename = "imageDisplay")]
    image_display: String,

    /// Spatial reference WKID.
    #[serde(skip_serializing_if = "Option::is_none")]
    sr: Option<i32>,

    /// Layer definition expressions (JSON).
    #[serde(rename = "layerDefs", skip_serializing_if = "Option::is_none")]
    layer_defs: Option<String>,

    /// Time instant or range.
    #[serde(skip_serializing_if = "Option::is_none")]
    time: Option<String>,

    /// Time relationship.
    #[serde(rename = "timeRelation", skip_serializing_if = "Option::is_none")]
    time_relation: Option<TimeRelation>,

    /// Which layers to identify.
    #[serde(skip_serializing_if = "Option::is_none")]
    layers: Option<LayerSelection>,

    /// Return geometries in results.
    #[serde(rename = "returnGeometry", skip_serializing_if = "Option::is_none")]
    return_geometry: Option<bool>,

    /// Maximum offset for geometry generalization.
    #[serde(rename = "maxAllowableOffset", skip_serializing_if = "Option::is_none")]
    max_allowable_offset: Option<f64>,

    /// Decimal places for geometry coordinates.
    #[serde(rename = "geometryPrecision", skip_serializing_if = "Option::is_none")]
    geometry_precision: Option<i32>,

    /// Return z-values.
    #[serde(rename = "returnZ", skip_serializing_if = "Option::is_none")]
    return_z: Option<bool>,

    /// Return m-values.
    #[serde(rename = "returnM", skip_serializing_if = "Option::is_none")]
    return_m: Option<bool>,

    /// Geodatabase version.
    #[serde(rename = "gdbVersion", skip_serializing_if = "Option::is_none")]
    gdb_version: Option<String>,

    /// Return unformatted values.
    #[serde(
        rename = "returnUnformattedValues",
        skip_serializing_if = "Option::is_none"
    )]
    return_unformatted_values: Option<bool>,

    /// Return field names instead of aliases.
    #[serde(rename = "returnFieldName", skip_serializing_if = "Option::is_none")]
    return_field_name: Option<bool>,

    /// Response format.
    #[serde(rename = "f")]
    #[builder(default = "ResponseFormat::Json")]
    format: ResponseFormat,

    /// Dynamic layers (JSON).
    #[serde(rename = "dynamicLayers", skip_serializing_if = "Option::is_none")]
    dynamic_layers: Option<String>,

    /// Layer time options (JSON).
    #[serde(rename = "layerTimeOptions", skip_serializing_if = "Option::is_none")]
    layer_time_options: Option<String>,

    /// Datum transformations (JSON).
    #[serde(
        rename = "datumTransformations",
        skip_serializing_if = "Option::is_none"
    )]
    datum_transformations: Option<String>,

    /// Map range values (JSON).
    #[serde(rename = "mapRangeValues", skip_serializing_if = "Option::is_none")]
    map_range_values: Option<String>,

    /// Layer range values (JSON).
    #[serde(rename = "layerRangeValues", skip_serializing_if = "Option::is_none")]
    layer_range_values: Option<String>,

    /// Layer parameter values (JSON).
    #[serde(
        rename = "layerParameterValues",
        skip_serializing_if = "Option::is_none"
    )]
    layer_parameter_values: Option<String>,

    /// Historic moment (epoch milliseconds).
    #[serde(rename = "historicMoment", skip_serializing_if = "Option::is_none")]
    historic_moment: Option<i64>,

    /// Clipping geometry (JSON).
    #[serde(skip_serializing_if = "Option::is_none")]
    clipping: Option<String>,

    /// Spatial filter (JSON).
    #[serde(rename = "spatialFilter", skip_serializing_if = "Option::is_none")]
    spatial_filter: Option<String>,
}

impl Default for IdentifyParams {
    fn default() -> Self {
        Self {
            geometry: String::new(),
            geometry_type: GeometryType::Point,
            tolerance: 3,
            map_extent: String::new(),
            image_display: String::new(),
            sr: None,
            layer_defs: None,
            time: None,
            time_relation: None,
            layers: None,
            return_geometry: None,
            max_allowable_offset: None,
            geometry_precision: None,
            return_z: None,
            return_m: None,
            gdb_version: None,
            return_unformatted_values: None,
            return_field_name: None,
            format: ResponseFormat::Json,
            dynamic_layers: None,
            layer_time_options: None,
            datum_transformations: None,
            map_range_values: None,
            layer_range_values: None,
            layer_parameter_values: None,
            historic_moment: None,
            clipping: None,
            spatial_filter: None,
        }
    }
}

impl IdentifyParams {
    /// Creates a builder for IdentifyParams.
    pub fn builder() -> IdentifyParamsBuilder {
        IdentifyParamsBuilder::default()
    }
}

/// A single identified feature.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Getters)]
#[serde(rename_all = "camelCase")]
pub struct IdentifyResult {
    /// Layer ID.
    layer_id: i32,

    /// Layer name.
    layer_name: String,

    /// Display field name.
    #[serde(skip_serializing_if = "Option::is_none")]
    display_field_name: Option<String>,

    /// Feature attributes.
    #[serde(default)]
    attributes: HashMap<String, serde_json::Value>,

    /// Feature geometry.
    #[serde(skip_serializing_if = "Option::is_none")]
    geometry: Option<ArcGISGeometry>,

    /// Geometry type.
    #[serde(skip_serializing_if = "Option::is_none")]
    geometry_type: Option<GeometryType>,
}

/// Response from identify operation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Getters)]
pub struct IdentifyResponse {
    /// Identified features.
    results: Vec<IdentifyResult>,
}

/// Symbol in a layer legend.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Getters)]
#[serde(rename_all = "camelCase")]
pub struct LegendSymbol {
    /// Label for the symbol.
    label: String,

    /// URL to symbol image (may be relative or absolute).
    #[serde(skip_serializing_if = "Option::is_none")]
    url: Option<String>,

    /// Base64-encoded image data.
    #[serde(skip_serializing_if = "Option::is_none")]
    image_data: Option<String>,

    /// MIME type (e.g., "image/png").
    content_type: String,

    /// Symbol height in pixels.
    height: i32,

    /// Symbol width in pixels.
    width: i32,

    /// Values for class breaks/unique values.
    #[serde(skip_serializing_if = "Option::is_none")]
    values: Option<Vec<String>>,
}

/// Legend information for a single layer.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Getters)]
#[serde(rename_all = "camelCase")]
pub struct LayerLegend {
    /// Layer ID.
    layer_id: i32,

    /// Layer name.
    layer_name: String,

    /// Layer type (e.g., "Feature Layer").
    layer_type: String,

    /// Minimum scale for visibility.
    #[serde(skip_serializing_if = "Option::is_none")]
    min_scale: Option<f64>,

    /// Maximum scale for visibility.
    #[serde(skip_serializing_if = "Option::is_none")]
    max_scale: Option<f64>,

    /// Legend symbols.
    legend: Vec<LegendSymbol>,
}

/// Response from legend operation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Getters)]
pub struct LegendResponse {
    /// Layers with legend information.
    layers: Vec<LayerLegend>,
}

/// A level of detail in a tile cache.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Getters)]
pub struct LevelOfDetail {
    /// Level number.
    level: i32,

    /// Resolution at this level.
    resolution: f64,

    /// Scale at this level.
    scale: f64,
}

/// Tile cache information.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Getters)]
#[serde(rename_all = "camelCase")]
pub struct TileInfo {
    /// Number of rows per tile.
    rows: i32,

    /// Number of columns per tile.
    cols: i32,

    /// DPI of tiles.
    dpi: i32,

    /// Image format.
    format: String,

    /// Levels of detail.
    #[serde(default)]
    lods: Vec<LevelOfDetail>,
}

/// Layer information in service metadata.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Getters)]
#[serde(rename_all = "camelCase")]
pub struct ServiceLayer {
    /// Layer ID.
    id: i32,

    /// Layer name.
    name: String,

    /// Parent layer ID.
    #[serde(skip_serializing_if = "Option::is_none")]
    parent_layer_id: Option<i32>,

    /// Default visibility.
    #[serde(skip_serializing_if = "Option::is_none")]
    default_visibility: Option<bool>,

    /// Minimum scale.
    #[serde(skip_serializing_if = "Option::is_none")]
    min_scale: Option<f64>,

    /// Maximum scale.
    #[serde(skip_serializing_if = "Option::is_none")]
    max_scale: Option<f64>,
}

/// Map service metadata.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Getters)]
#[serde(rename_all = "camelCase")]
pub struct MapServiceMetadata {
    /// Service name.
    #[serde(skip_serializing_if = "Option::is_none")]
    service_name: Option<String>,

    /// Service description.
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<String>,

    /// Map name.
    #[serde(skip_serializing_if = "Option::is_none")]
    map_name: Option<String>,

    /// Service capabilities.
    #[serde(skip_serializing_if = "Option::is_none")]
    capabilities: Option<String>,

    /// Supported image formats.
    #[serde(skip_serializing_if = "Option::is_none")]
    supported_image_format_types: Option<String>,

    /// Layers in the service.
    #[serde(default)]
    layers: Vec<ServiceLayer>,

    /// Spatial reference.
    #[serde(skip_serializing_if = "Option::is_none")]
    spatial_reference: Option<SpatialReference>,

    /// Initial extent.
    #[serde(skip_serializing_if = "Option::is_none")]
    initial_extent: Option<ExportExtent>,

    /// Full extent.
    #[serde(skip_serializing_if = "Option::is_none")]
    full_extent: Option<ExportExtent>,

    /// Tile info (for cached services).
    #[serde(skip_serializing_if = "Option::is_none")]
    tile_info: Option<TileInfo>,
}

/// Parameters for the find operation.
///
/// Use [`FindParams::builder()`] to construct instances.
#[derive(Debug, Clone, Serialize, derive_builder::Builder, Getters)]
#[builder(setter(into, strip_option))]
#[serde(rename_all = "camelCase")]
pub struct FindParams {
    /// Text to search for (REQUIRED).
    search_text: String,

    /// Comma-separated list of field names to search (REQUIRED).
    #[serde(serialize_with = "serialize_vec_as_comma")]
    search_fields: Vec<String>,

    /// Comma-separated list of layer IDs to search.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(serialize_with = "serialize_opt_vec_as_comma")]
    #[builder(default)]
    layers: Option<Vec<i32>>,

    /// Whether the search text should match the exact whole value.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default)]
    contains: Option<bool>,

    /// Whether to return geometry with results.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default)]
    return_geometry: Option<bool>,

    /// Output spatial reference WKID.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default)]
    sr: Option<i32>,

    /// Layer definitions (WHERE clauses for layers).
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default)]
    layer_defs: Option<String>,

    /// Whether to return Z-values.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default)]
    return_z: Option<bool>,

    /// Whether to return M-values.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default)]
    return_m: Option<bool>,
}

impl FindParams {
    /// Creates a builder for FindParams.
    pub fn builder() -> FindParamsBuilder {
        FindParamsBuilder::default()
    }
}

/// Helper to serialize Vec<String> as comma-separated.
fn serialize_vec_as_comma<S>(vec: &[String], serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    serializer.serialize_str(&vec.join(","))
}

/// Helper to serialize Option<Vec<i32>> as comma-separated.
fn serialize_opt_vec_as_comma<S>(vec: &Option<Vec<i32>>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    match vec {
        Some(v) => {
            let s = v
                .iter()
                .map(|i| i.to_string())
                .collect::<Vec<_>>()
                .join(",");
            serializer.serialize_str(&s)
        }
        None => serializer.serialize_none(),
    }
}

/// Response from find operation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Getters)]
pub struct FindResponse {
    /// Array of find results.
    results: Vec<FindResult>,
}

/// A single find result.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Getters)]
#[serde(rename_all = "camelCase")]
pub struct FindResult {
    /// Layer ID where the feature was found.
    layer_id: i32,

    /// Layer name.
    layer_name: String,

    /// Display field name.
    #[serde(skip_serializing_if = "Option::is_none")]
    display_field_name: Option<String>,

    /// Field name where the value was found.
    found_field_name: String,

    /// Value that was found.
    value: serde_json::Value,

    /// Feature attributes.
    attributes: HashMap<String, serde_json::Value>,

    /// Feature geometry.
    #[serde(skip_serializing_if = "Option::is_none")]
    geometry: Option<ArcGISGeometry>,

    /// Geometry type.
    #[serde(skip_serializing_if = "Option::is_none")]
    geometry_type: Option<GeometryType>,
}

/// Parameters for generating KML output.
///
/// Use [`GenerateKmlParams::builder()`] to construct instances.
#[derive(Debug, Clone, Serialize, derive_builder::Builder, Getters)]
#[builder(setter(into, strip_option))]
#[serde(rename_all = "camelCase")]
pub struct GenerateKmlParams {
    /// Name for the KML document (REQUIRED).
    doc_name: String,

    /// Comma-separated list of layer IDs to include.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(serialize_with = "serialize_opt_vec_as_comma")]
    layers: Option<Vec<i32>>,

    /// Layer definitions (WHERE clauses).
    #[serde(skip_serializing_if = "Option::is_none")]
    layer_defs: Option<String>,

    /// Image format for ground overlay.
    #[serde(skip_serializing_if = "Option::is_none")]
    image_format: Option<String>,

    /// DPI for image.
    #[serde(skip_serializing_if = "Option::is_none")]
    dpi: Option<i32>,

    /// Image size (width,height).
    #[serde(skip_serializing_if = "Option::is_none")]
    image_size: Option<String>,
}

impl GenerateKmlParams {
    /// Creates a builder for GenerateKmlParams.
    pub fn builder() -> GenerateKmlParamsBuilder {
        GenerateKmlParamsBuilder::default()
    }
}

/// Parameters for generating a classification renderer.
///
/// Use [`GenerateRendererParams::builder()`] to construct instances.
#[derive(Debug, Clone, Serialize, derive_builder::Builder, Getters)]
#[builder(setter(into, strip_option))]
#[serde(rename_all = "camelCase")]
pub struct GenerateRendererParams {
    /// Field to classify (REQUIRED).
    classification_field: String,

    /// Classification method: "equal-interval", "natural-breaks", "quantile", "standard-deviation".
    classification_method: String,

    /// Number of classes/breaks.
    #[serde(skip_serializing_if = "Option::is_none")]
    break_count: Option<i32>,

    /// Normalization field.
    #[serde(skip_serializing_if = "Option::is_none")]
    normalization_field: Option<String>,

    /// Normalization type: "field", "log", "percent-of-total".
    #[serde(skip_serializing_if = "Option::is_none")]
    normalization_type: Option<String>,

    /// Base symbol definition (as JSON string).
    #[serde(skip_serializing_if = "Option::is_none")]
    base_symbol: Option<String>,

    /// Color ramp definition.
    #[serde(skip_serializing_if = "Option::is_none")]
    color_ramp: Option<serde_json::Value>,
}

impl GenerateRendererParams {
    /// Creates a builder for GenerateRendererParams.
    pub fn builder() -> GenerateRendererParamsBuilder {
        GenerateRendererParamsBuilder::default()
    }
}

/// Response from generateRenderer operation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Getters)]
#[serde(rename_all = "camelCase")]
pub struct RendererResponse {
    /// Renderer type (e.g., "classBreaks", "uniqueValue").
    #[serde(rename = "type")]
    renderer_type: String,

    /// Field name used for classification.
    #[serde(skip_serializing_if = "Option::is_none")]
    field: Option<String>,

    /// Default symbol.
    #[serde(skip_serializing_if = "Option::is_none")]
    default_symbol: Option<serde_json::Value>,

    /// Default label.
    #[serde(skip_serializing_if = "Option::is_none")]
    default_label: Option<String>,

    /// Classification breaks (for classBreaks renderer).
    #[serde(skip_serializing_if = "Option::is_none")]
    class_break_infos: Option<Vec<ClassBreakInfo>>,

    /// Unique value infos (for uniqueValue renderer).
    #[serde(skip_serializing_if = "Option::is_none")]
    unique_value_infos: Option<Vec<UniqueValueInfo>>,
}

/// Information about a classification break.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Getters)]
#[serde(rename_all = "camelCase")]
pub struct ClassBreakInfo {
    /// Minimum value for this class.
    class_min_value: f64,

    /// Maximum value for this class.
    class_max_value: f64,

    /// Label for this class.
    label: String,

    /// Description.
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<String>,

    /// Symbol for this class.
    symbol: serde_json::Value,
}

/// Information about a unique value.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Getters)]
#[serde(rename_all = "camelCase")]
pub struct UniqueValueInfo {
    /// The unique value.
    value: serde_json::Value,

    /// Label for this value.
    label: String,

    /// Description.
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<String>,

    /// Symbol for this value.
    symbol: serde_json::Value,
}
