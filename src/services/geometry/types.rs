//! Types for geometry service operations.

use crate::ArcGISGeometry;
use derive_getters::Getters;
use serde::{Deserialize, Serialize};

/// Parameters for the project operation.
///
/// Use [`ProjectParameters::builder()`] to construct instances.
#[derive(Debug, Clone, Serialize, derive_builder::Builder, Getters)]
#[builder(setter(into, strip_option))]
#[serde(rename_all = "camelCase")]
pub struct ProjectParameters {
    /// Geometries to project (REQUIRED).
    #[serde(serialize_with = "serialize_geometries")]
    geometries: Vec<ArcGISGeometry>,

    /// Input spatial reference WKID (REQUIRED).
    in_sr: i32,

    /// Output spatial reference WKID (REQUIRED).
    out_sr: i32,

    /// Datum transformation WKID (optional).
    #[serde(skip_serializing_if = "Option::is_none")]
    transformation: Option<i32>,

    /// Whether to transform forward or reverse.
    #[serde(skip_serializing_if = "Option::is_none")]
    transform_forward: Option<bool>,
}

impl ProjectParameters {
    /// Creates a builder for ProjectParameters.
    pub fn builder() -> ProjectParametersBuilder {
        ProjectParametersBuilder::default()
    }
}

/// Helper to serialize geometries as JSON array.
fn serialize_geometries<S>(geoms: &[ArcGISGeometry], serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    use serde::ser::SerializeSeq;
    let mut seq = serializer.serialize_seq(Some(geoms.len()))?;
    for geom in geoms {
        seq.serialize_element(geom)?;
    }
    seq.end()
}

/// Response from project operation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Getters)]
pub struct ProjectResult {
    /// Projected geometries.
    geometries: Vec<ArcGISGeometry>,
}

/// Parameters for the buffer operation.
///
/// Use [`BufferParameters::builder()`] to construct instances.
#[derive(Debug, Clone, Serialize, derive_builder::Builder, Getters)]
#[builder(setter(into, strip_option))]
#[serde(rename_all = "camelCase")]
pub struct BufferParameters {
    /// Geometries to buffer (REQUIRED).
    #[serde(serialize_with = "serialize_geometries")]
    geometries: Vec<ArcGISGeometry>,

    /// Spatial reference of input geometries (REQUIRED).
    in_sr: i32,

    /// Buffer distances (REQUIRED).
    /// One distance per geometry, or a single distance for all.
    distances: Vec<f64>,

    /// Distance unit (REQUIRED).
    unit: LinearUnit,

    /// Whether to union results.
    #[serde(skip_serializing_if = "Option::is_none")]
    union_results: Option<bool>,

    /// Whether to use geodesic buffers.
    #[serde(skip_serializing_if = "Option::is_none")]
    geodesic: Option<bool>,

    /// Output spatial reference WKID.
    #[serde(skip_serializing_if = "Option::is_none")]
    out_sr: Option<i32>,
}

impl BufferParameters {
    /// Creates a builder for BufferParameters.
    pub fn builder() -> BufferParametersBuilder {
        BufferParametersBuilder::default()
    }
}

/// Response from buffer operation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Getters)]
pub struct BufferResult {
    /// Buffer polygon geometries.
    geometries: Vec<ArcGISGeometry>,
}

/// Linear units for distance measurements and buffers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum LinearUnit {
    /// Meters
    #[serde(rename = "esriMeters")]
    Meters,
    /// Kilometers
    #[serde(rename = "esriKilometers")]
    Kilometers,
    /// Feet
    #[serde(rename = "esriFeet")]
    Feet,
    /// Miles
    #[serde(rename = "esriMiles")]
    Miles,
    /// Nautical miles
    #[serde(rename = "esriNauticalMiles")]
    NauticalMiles,
    /// Yards
    #[serde(rename = "esriYards")]
    Yards,
}

/// Datum transformation information.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Getters)]
#[serde(rename_all = "camelCase")]
pub struct Transformation {
    /// Well-Known ID of the transformation.
    wkid: i32,

    /// Well-Known Text representation.
    #[serde(skip_serializing_if = "Option::is_none")]
    wkt: Option<String>,

    /// Name of the transformation.
    #[serde(skip_serializing_if = "Option::is_none")]
    name: Option<String>,
}

/// Parameters for the simplify operation.
///
/// Use [`SimplifyParameters::builder()`] to construct instances.
#[derive(Debug, Clone, Serialize, derive_builder::Builder, Getters)]
#[builder(setter(into, strip_option))]
#[serde(rename_all = "camelCase")]
pub struct SimplifyParameters {
    /// Geometries to simplify (REQUIRED).
    #[serde(serialize_with = "serialize_geometries")]
    geometries: Vec<ArcGISGeometry>,

    /// Spatial reference of input geometries (REQUIRED).
    sr: i32,
}

impl SimplifyParameters {
    /// Creates a builder for SimplifyParameters.
    pub fn builder() -> SimplifyParametersBuilder {
        SimplifyParametersBuilder::default()
    }
}

/// Response from simplify operation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Getters)]
pub struct SimplifyResult {
    /// Simplified geometries.
    geometries: Vec<ArcGISGeometry>,
}

/// Parameters for the union operation.
///
/// Use [`UnionParameters::builder()`] to construct instances.
#[derive(Debug, Clone, Serialize, derive_builder::Builder, Getters)]
#[builder(setter(into, strip_option))]
#[serde(rename_all = "camelCase")]
pub struct UnionParameters {
    /// Geometries to union (REQUIRED).
    #[serde(serialize_with = "serialize_geometries")]
    geometries: Vec<ArcGISGeometry>,

    /// Spatial reference of input geometries (REQUIRED).
    sr: i32,
}

impl UnionParameters {
    /// Creates a builder for UnionParameters.
    pub fn builder() -> UnionParametersBuilder {
        UnionParametersBuilder::default()
    }
}

/// Response from union operation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Getters)]
pub struct UnionResult {
    /// Unioned geometry.
    geometry: ArcGISGeometry,
}

/// Parameters for calculating areas and lengths.
///
/// Use [`AreasAndLengthsParameters::builder()`] to construct instances.
#[derive(Debug, Clone, Serialize, derive_builder::Builder, Getters)]
#[builder(setter(into, strip_option))]
#[serde(rename_all = "camelCase")]
pub struct AreasAndLengthsParameters {
    /// Polygon geometries to calculate (REQUIRED).
    #[serde(serialize_with = "serialize_geometries")]
    polygons: Vec<ArcGISGeometry>,

    /// Spatial reference of input geometries (REQUIRED).
    sr: i32,

    /// Length unit for calculations.
    #[serde(skip_serializing_if = "Option::is_none")]
    length_unit: Option<LinearUnit>,

    /// Area unit for calculations.
    #[serde(skip_serializing_if = "Option::is_none")]
    area_unit: Option<AreaUnit>,

    /// Whether to use geodesic calculations.
    #[serde(skip_serializing_if = "Option::is_none")]
    calculation_type: Option<CalculationType>,
}

impl AreasAndLengthsParameters {
    /// Creates a builder for AreasAndLengthsParameters.
    pub fn builder() -> AreasAndLengthsParametersBuilder {
        AreasAndLengthsParametersBuilder::default()
    }
}

/// Response from areas and lengths calculation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Getters)]
pub struct AreasAndLengthsResult {
    /// Calculated areas for each polygon.
    areas: Vec<f64>,

    /// Calculated perimeter lengths for each polygon.
    lengths: Vec<f64>,
}

/// Parameters for distance calculation.
///
/// Use [`DistanceParameters::builder()`] to construct instances.
#[derive(Debug, Clone, Serialize, derive_builder::Builder, Getters)]
#[builder(setter(into, strip_option))]
#[serde(rename_all = "camelCase")]
pub struct DistanceParameters {
    /// First geometry (REQUIRED).
    geometry1: ArcGISGeometry,

    /// Second geometry (REQUIRED).
    geometry2: ArcGISGeometry,

    /// Spatial reference of input geometries (REQUIRED).
    sr: i32,

    /// Distance unit for result.
    #[serde(skip_serializing_if = "Option::is_none")]
    distance_unit: Option<LinearUnit>,

    /// Whether to use geodesic calculations.
    #[serde(skip_serializing_if = "Option::is_none")]
    geodesic: Option<bool>,
}

impl DistanceParameters {
    /// Creates a builder for DistanceParameters.
    pub fn builder() -> DistanceParametersBuilder {
        DistanceParametersBuilder::default()
    }
}

/// Response from distance calculation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Getters)]
pub struct DistanceResult {
    /// Calculated distance.
    distance: f64,
}

/// Area units for measurements.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum AreaUnit {
    /// Square meters
    #[serde(rename = "esriSquareMeters")]
    SquareMeters,
    /// Square kilometers
    #[serde(rename = "esriSquareKilometers")]
    SquareKilometers,
    /// Square feet
    #[serde(rename = "esriSquareFeet")]
    SquareFeet,
    /// Square miles
    #[serde(rename = "esriSquareMiles")]
    SquareMiles,
    /// Acres
    #[serde(rename = "esriAcres")]
    Acres,
    /// Hectares
    #[serde(rename = "esriHectares")]
    Hectares,
}

/// Calculation type for geometric operations.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum CalculationType {
    /// Planar (projected) calculations.
    #[serde(rename = "planar")]
    Planar,
    /// Geodesic (spherical) calculations.
    #[serde(rename = "geodesic")]
    Geodesic,
    /// Preserves shape calculations.
    #[serde(rename = "preserveShape")]
    PreserveShape,
}
