//! Image service types and parameters.

use derive_getters::Getters;
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Parameters for exporting an image.
#[derive(Debug, Clone, Serialize, derive_builder::Builder, Getters)]
#[builder(setter(into, strip_option))]
#[serde(rename_all = "camelCase")]
pub struct ExportImageParameters {
    /// Bounding box for the image (xmin, ymin, xmax, ymax).
    bbox: String,

    /// Image width in pixels.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default)]
    size: Option<String>,

    /// Image format (png, jpg, tiff, etc.).
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default)]
    format: Option<String>,

    /// Pixel type for the output.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default)]
    pixel_type: Option<PixelType>,

    /// No data value.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default)]
    no_data: Option<f64>,

    /// Interpolation method.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default)]
    interpolation: Option<InterpolationType>,

    /// JPEG compression quality (0-100).
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default)]
    compression_quality: Option<u8>,

    /// Band IDs to include (comma-separated).
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default)]
    band_ids: Option<String>,

    /// Mosaic rule for multi-raster datasets.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default)]
    mosaic_rule: Option<MosaicRule>,

    /// Rendering rule for dynamic visualization.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default)]
    rendering_rule: Option<RenderingRule>,

    /// Spatial reference WKID.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "bboxSR")]
    #[builder(default)]
    bbox_sr: Option<u32>,

    /// Output spatial reference WKID.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "imageSR")]
    #[builder(default)]
    image_sr: Option<u32>,
}

/// Result from exporting an image.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Getters)]
#[serde(rename_all = "camelCase")]
pub struct ExportImageResult {
    /// URL to the exported image.
    href: String,

    /// Image width in pixels.
    #[serde(skip_serializing_if = "Option::is_none")]
    width: Option<u32>,

    /// Image height in pixels.
    #[serde(skip_serializing_if = "Option::is_none")]
    height: Option<u32>,

    /// Extent of the image.
    #[serde(skip_serializing_if = "Option::is_none")]
    extent: Option<Value>,
}

/// Pixel type for raster data.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum PixelType {
    /// 8-bit unsigned integer.
    U8,
    /// 8-bit signed integer.
    S8,
    /// 16-bit unsigned integer.
    U16,
    /// 16-bit signed integer.
    S16,
    /// 32-bit unsigned integer.
    U32,
    /// 32-bit signed integer.
    S32,
    /// 32-bit floating point.
    F32,
    /// 64-bit floating point.
    F64,
    /// Unknown pixel type.
    Unknown,
}

/// Interpolation type for resampling.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum InterpolationType {
    /// Nearest neighbor (fastest).
    #[serde(rename = "RSP_NearestNeighbor")]
    NearestNeighbor,
    /// Bilinear interpolation.
    #[serde(rename = "RSP_BilinearInterpolation")]
    BilinearInterpolation,
    /// Cubic convolution.
    #[serde(rename = "RSP_CubicConvolution")]
    CubicConvolution,
    /// Majority resampling.
    #[serde(rename = "RSP_Majority")]
    Majority,
}

/// Mosaic rule for combining multiple rasters.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Getters, derive_new::new)]
#[serde(rename_all = "camelCase")]
pub struct MosaicRule {
    /// Mosaic method.
    mosaic_method: String,

    /// Mosaic operation.
    #[serde(skip_serializing_if = "Option::is_none")]
    mosaic_operation: Option<String>,

    /// Lock raster IDs.
    #[serde(skip_serializing_if = "Option::is_none")]
    lock_raster_ids: Option<Vec<i64>>,

    /// Order by field.
    #[serde(skip_serializing_if = "Option::is_none")]
    order_by_field: Option<String>,

    /// Ascending order.
    #[serde(skip_serializing_if = "Option::is_none")]
    ascending: Option<bool>,
}

/// Rendering rule for dynamic visualization.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Getters, derive_new::new)]
#[serde(rename_all = "camelCase")]
pub struct RenderingRule {
    /// Raster function name.
    raster_function: String,

    /// Raster function arguments.
    #[serde(skip_serializing_if = "Option::is_none")]
    raster_function_arguments: Option<Value>,
}

/// Parameters for identify operation.
#[derive(Debug, Clone, Serialize, derive_builder::Builder, Getters)]
#[builder(setter(into, strip_option))]
#[serde(rename_all = "camelCase")]
pub struct IdentifyParameters {
    /// Geometry to identify at (point or polygon).
    geometry: String,

    /// Geometry type.
    geometry_type: String,

    /// Mosaic rule.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default)]
    mosaic_rule: Option<MosaicRule>,

    /// Rendering rule.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default)]
    rendering_rule: Option<RenderingRule>,

    /// Spatial reference of input geometry.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "geometrySR")]
    #[builder(default)]
    geometry_sr: Option<u32>,

    /// Return geometry.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default)]
    return_geometry: Option<bool>,

    /// Return catalog items.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default)]
    return_catalog_items: Option<bool>,
}

/// Result from identify operation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Getters)]
#[serde(rename_all = "camelCase")]
pub struct ImageIdentifyResult {
    /// Pixel value(s).
    #[serde(skip_serializing_if = "Option::is_none")]
    value: Option<String>,

    /// Location of the identified point.
    #[serde(skip_serializing_if = "Option::is_none")]
    location: Option<Value>,

    /// Properties/attributes.
    #[serde(skip_serializing_if = "Option::is_none")]
    properties: Option<Value>,

    /// Catalog items (if requested).
    #[serde(skip_serializing_if = "Option::is_none")]
    catalog_items: Option<Value>,
}

/// Parameters for sampling operation.
#[derive(Debug, Clone, Serialize, derive_builder::Builder, Getters)]
#[builder(setter(into, strip_option))]
#[serde(rename_all = "camelCase")]
pub struct SampleParameters {
    /// Geometry to sample along (polyline or polygon).
    geometry: String,

    /// Geometry type.
    geometry_type: String,

    /// Spatial reference of input geometry.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "geometrySR")]
    #[builder(default)]
    geometry_sr: Option<u32>,

    /// Sample count (for polylines).
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default)]
    sample_count: Option<u32>,

    /// Sample distance (for polylines).
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default)]
    sample_distance: Option<f64>,

    /// Output fields.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default)]
    out_fields: Option<String>,

    /// Return geometry.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default)]
    return_geometry: Option<bool>,

    /// Mosaic rule.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default)]
    mosaic_rule: Option<MosaicRule>,

    /// Rendering rule.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default)]
    rendering_rule: Option<RenderingRule>,
}

/// Result from sampling operation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Getters)]
#[serde(rename_all = "camelCase")]
pub struct SampleResult {
    /// Sample points/locations.
    samples: Vec<Value>,
}

/// Parameters for histogram computation.
#[derive(Debug, Clone, Serialize, derive_builder::Builder, Getters)]
#[builder(setter(into, strip_option))]
#[serde(rename_all = "camelCase")]
pub struct HistogramParameters {
    /// Geometry (area of interest).
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default)]
    geometry: Option<String>,

    /// Geometry type.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default)]
    geometry_type: Option<String>,

    /// Spatial reference.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "geometrySR")]
    #[builder(default)]
    geometry_sr: Option<u32>,

    /// Mosaic rule.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default)]
    mosaic_rule: Option<MosaicRule>,

    /// Rendering rule.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default)]
    rendering_rule: Option<RenderingRule>,
}

/// Result from histogram computation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Getters)]
#[serde(rename_all = "camelCase")]
pub struct HistogramResult {
    /// Histograms per band.
    histograms: Vec<Histogram>,
}

/// Histogram for a single band.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Getters)]
#[serde(rename_all = "camelCase")]
pub struct Histogram {
    /// Bin counts.
    counts: Vec<u64>,

    /// Minimum value.
    #[serde(skip_serializing_if = "Option::is_none")]
    min: Option<f64>,

    /// Maximum value.
    #[serde(skip_serializing_if = "Option::is_none")]
    max: Option<f64>,

    /// Mean value.
    #[serde(skip_serializing_if = "Option::is_none")]
    mean: Option<f64>,

    /// Standard deviation.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "stddev")]
    std_dev: Option<f64>,
}

/// Raster information metadata.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Getters)]
#[serde(rename_all = "camelCase")]
pub struct RasterInfo {
    /// Number of bands.
    #[serde(skip_serializing_if = "Option::is_none")]
    band_count: Option<u32>,

    /// Pixel type.
    #[serde(skip_serializing_if = "Option::is_none")]
    pixel_type: Option<String>,

    /// Extent.
    #[serde(skip_serializing_if = "Option::is_none")]
    extent: Option<Value>,

    /// Spatial reference.
    #[serde(skip_serializing_if = "Option::is_none")]
    spatial_reference: Option<Value>,

    /// Pixel size (x, y).
    #[serde(skip_serializing_if = "Option::is_none")]
    pixel_size_x: Option<f64>,

    /// Pixel size Y.
    #[serde(skip_serializing_if = "Option::is_none")]
    pixel_size_y: Option<f64>,
}
