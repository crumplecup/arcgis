//! Geoprocessing parameter types.

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// A geoprocessing parameter value.
///
/// GP parameters can be various types (string, number, geometry, etc.).
/// This enum provides type-safe construction of parameter values.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum GPParameter {
    /// String parameter.
    String(GPString),

    /// Long (integer) parameter.
    Long(GPLong),

    /// Double (floating point) parameter.
    Double(GPDouble),

    /// Boolean parameter.
    Boolean(GPBoolean),

    /// Date parameter.
    Date(GPDate),

    /// Linear unit parameter (value + unit).
    LinearUnit(GPLinearUnit),

    /// Feature record set layer.
    FeatureRecordSetLayer(GPFeatureRecordSetLayer),

    /// Raster data layer.
    RasterDataLayer(GPRasterDataLayer),

    /// Data file.
    DataFile(GPDataFile),

    /// Raw JSON value (for other types).
    Raw(Value),
}

/// A string parameter value.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct GPString(pub String);

impl From<String> for GPString {
    fn from(s: String) -> Self {
        GPString(s)
    }
}

impl From<&str> for GPString {
    fn from(s: &str) -> Self {
        GPString(s.to_string())
    }
}

/// A long (integer) parameter value.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct GPLong(pub i64);

impl From<i64> for GPLong {
    fn from(i: i64) -> Self {
        GPLong(i)
    }
}

impl From<i32> for GPLong {
    fn from(i: i32) -> Self {
        GPLong(i as i64)
    }
}

/// A double (floating point) parameter value.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct GPDouble(pub f64);

impl From<f64> for GPDouble {
    fn from(f: f64) -> Self {
        GPDouble(f)
    }
}

impl From<f32> for GPDouble {
    fn from(f: f32) -> Self {
        GPDouble(f as f64)
    }
}

/// A boolean parameter value.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct GPBoolean(pub bool);

impl From<bool> for GPBoolean {
    fn from(b: bool) -> Self {
        GPBoolean(b)
    }
}

/// A date parameter value (milliseconds since epoch).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct GPDate(pub i64);

impl GPDate {
    /// Create a date from milliseconds since Unix epoch.
    pub fn from_millis(millis: i64) -> Self {
        GPDate(millis)
    }
}

/// A linear unit parameter (distance with unit).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GPLinearUnit {
    /// Distance value.
    pub distance: f64,

    /// Unit of measurement.
    pub units: String,
}

impl GPLinearUnit {
    /// Create a linear unit with distance and unit.
    pub fn new(distance: f64, units: impl Into<String>) -> Self {
        GPLinearUnit {
            distance,
            units: units.into(),
        }
    }
}

/// A feature record set layer parameter (GeoJSON-like features).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GPFeatureRecordSetLayer {
    /// Geometry type.
    pub geometry_type: String,

    /// Features array.
    pub features: Vec<Value>,

    /// Spatial reference (optional).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub spatial_reference: Option<Value>,
}

impl GPFeatureRecordSetLayer {
    /// Create a feature record set with geometry type and features.
    pub fn new(geometry_type: impl Into<String>, features: Vec<Value>) -> Self {
        GPFeatureRecordSetLayer {
            geometry_type: geometry_type.into(),
            features,
            spatial_reference: None,
        }
    }

    /// Set the spatial reference.
    pub fn with_spatial_reference(mut self, sr: Value) -> Self {
        self.spatial_reference = Some(sr);
        self
    }
}

/// A raster data layer parameter.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GPRasterDataLayer {
    /// URL to the raster data.
    pub url: String,

    /// Format (optional).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub format: Option<String>,
}

impl GPRasterDataLayer {
    /// Create a raster data layer with URL.
    pub fn new(url: impl Into<String>) -> Self {
        GPRasterDataLayer {
            url: url.into(),
            format: None,
        }
    }

    /// Set the raster format.
    pub fn with_format(mut self, format: impl Into<String>) -> Self {
        self.format = Some(format.into());
        self
    }
}

/// A data file parameter.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GPDataFile {
    /// URL to the data file.
    pub url: String,
}

impl GPDataFile {
    /// Create a data file with URL.
    pub fn new(url: impl Into<String>) -> Self {
        GPDataFile { url: url.into() }
    }
}
