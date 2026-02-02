//! Elevation Service types and parameters.

use crate::FeatureSet;
use derive_getters::Getters;
use serde::{Deserialize, Serialize};

/// Parameters for generating an elevation profile.
#[derive(Debug, Clone, Serialize, derive_builder::Builder, Getters)]
#[builder(setter(into, strip_option))]
#[serde(rename_all = "camelCase")]
pub struct ProfileParameters {
    /// Input geometry (polyline or multipoint).
    #[serde(rename = "InputLineOfSight")]
    input_geometry: String,

    /// Geometry type.
    #[builder(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    geometry_type: Option<String>,

    /// DEM resolution (FINEST, 10m, 30m, 90m).
    #[builder(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "DEMResolution")]
    dem_resolution: Option<String>,

    /// Profile ID field for grouping.
    #[builder(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "ProfileIDField")]
    profile_id_field: Option<String>,

    /// Return first point elevation.
    #[builder(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    return_first_point: Option<bool>,

    /// Return last point elevation.
    #[builder(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    return_last_point: Option<bool>,

    /// Spatial reference WKID.
    #[builder(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "inSR")]
    in_sr: Option<u32>,

    /// Output spatial reference WKID.
    #[builder(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "outSR")]
    out_sr: Option<u32>,
}

/// Result from elevation profile operation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Getters)]
#[serde(rename_all = "camelCase")]
pub struct ProfileResult {
    /// Profile feature set with elevation data.
    #[serde(skip_serializing_if = "Option::is_none")]
    output_profile: Option<FeatureSet>,

    /// First point elevation.
    #[serde(skip_serializing_if = "Option::is_none")]
    first_point_z: Option<f64>,

    /// Last point elevation.
    #[serde(skip_serializing_if = "Option::is_none")]
    last_point_z: Option<f64>,
}

/// Parameters for summarizing elevation within a polygon.
#[derive(Debug, Clone, Serialize, derive_builder::Builder, Getters)]
#[builder(setter(into, strip_option))]
#[serde(rename_all = "camelCase")]
pub struct SummarizeElevationParameters {
    /// Input polygon geometry.
    #[serde(rename = "InputPolygon")]
    input_geometry: String,

    /// Geometry type.
    #[builder(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    geometry_type: Option<String>,

    /// DEM resolution (FINEST, 10m, 30m, 90m).
    #[builder(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "DEMResolution")]
    dem_resolution: Option<String>,

    /// Include slope statistics.
    #[builder(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    include_slope: Option<bool>,

    /// Include aspect statistics.
    #[builder(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    include_aspect: Option<bool>,

    /// Spatial reference WKID.
    #[builder(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "inSR")]
    in_sr: Option<u32>,

    /// Output spatial reference WKID.
    #[builder(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "outSR")]
    out_sr: Option<u32>,
}

/// Result from summarize elevation operation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Getters)]
#[serde(rename_all = "camelCase")]
pub struct SummarizeElevationResult {
    /// Summary feature set with statistics.
    #[serde(skip_serializing_if = "Option::is_none")]
    output_summary: Option<FeatureSet>,

    /// Minimum elevation.
    #[serde(skip_serializing_if = "Option::is_none")]
    min_elevation: Option<f64>,

    /// Maximum elevation.
    #[serde(skip_serializing_if = "Option::is_none")]
    max_elevation: Option<f64>,

    /// Mean elevation.
    #[serde(skip_serializing_if = "Option::is_none")]
    mean_elevation: Option<f64>,

    /// Area in square meters.
    #[serde(skip_serializing_if = "Option::is_none")]
    area: Option<f64>,
}

/// Parameters for viewshed analysis.
#[derive(Debug, Clone, Serialize, derive_builder::Builder, Getters)]
#[builder(setter(into, strip_option))]
#[serde(rename_all = "camelCase")]
pub struct ViewshedParameters {
    /// Observer point(s) geometry.
    #[serde(rename = "InputPoints")]
    input_points: String,

    /// Geometry type.
    #[builder(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    geometry_type: Option<String>,

    /// Maximum viewing distance in meters.
    #[builder(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    maximum_distance: Option<f64>,

    /// Maximum horizontal viewing angle (degrees).
    #[builder(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    maximum_horizontal_angle: Option<f64>,

    /// Maximum vertical angle (degrees).
    #[builder(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    maximum_vertical_angle: Option<f64>,

    /// Observer height above ground (meters).
    #[builder(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    observer_height: Option<f64>,

    /// Observer offset (additional height).
    #[builder(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    observer_offset: Option<f64>,

    /// Surface offset (target height above ground).
    #[builder(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    surface_offset: Option<f64>,

    /// DEM resolution (FINEST, 10m, 30m, 90m).
    #[builder(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "DEMResolution")]
    dem_resolution: Option<String>,

    /// Generalize viewshed polygons.
    #[builder(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    generalize: Option<bool>,

    /// Spatial reference WKID.
    #[builder(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "inSR")]
    in_sr: Option<u32>,

    /// Output spatial reference WKID.
    #[builder(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "outSR")]
    out_sr: Option<u32>,
}

/// Result from viewshed analysis.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Getters)]
#[serde(rename_all = "camelCase")]
pub struct ViewshedResult {
    /// Viewshed polygon feature set.
    #[serde(skip_serializing_if = "Option::is_none")]
    output_viewshed: Option<FeatureSet>,

    /// Visible area in square meters.
    #[serde(skip_serializing_if = "Option::is_none")]
    visible_area: Option<f64>,

    /// Total area analyzed in square meters.
    #[serde(skip_serializing_if = "Option::is_none")]
    total_area: Option<f64>,

    /// Percentage visible (0-100).
    #[serde(skip_serializing_if = "Option::is_none")]
    percent_visible: Option<f64>,
}

/// DEM resolution options.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum DemResolution {
    /// Finest available resolution.
    Finest,
    /// 10 meter resolution.
    #[serde(rename = "10m")]
    TenMeter,
    /// 30 meter resolution.
    #[serde(rename = "30m")]
    ThirtyMeter,
    /// 90 meter resolution.
    #[serde(rename = "90m")]
    NinetyMeter,
}

impl DemResolution {
    /// Convert to API string representation.
    pub fn as_str(&self) -> &'static str {
        match self {
            DemResolution::Finest => "FINEST",
            DemResolution::TenMeter => "10m",
            DemResolution::ThirtyMeter => "30m",
            DemResolution::NinetyMeter => "90m",
        }
    }
}
