//! Elevation Service types and parameters.

use crate::{ArcGISGeometryError, ArcGISGeometryErrorKind, FeatureSet};
use derive_getters::Getters;
use serde::{Deserialize, Serialize};
use tracing::instrument;

/// Parameters for generating an elevation profile.
#[derive(Debug, Clone, Serialize, derive_builder::Builder, Getters)]
#[builder(setter(into, strip_option))]
#[serde(rename_all = "camelCase")]
pub struct ProfileParameters {
    /// Input line features as a FeatureSet JSON string.
    ///
    /// Must be a valid FeatureSet with geometryType, features array, and spatialReference.
    /// Example: `{"geometryType":"esriGeometryPolyline","features":[{"geometry":{"paths":[...]}}],"spatialReference":{"wkid":4326}}`
    #[serde(rename = "InputLineFeatures")]
    input_line_features: String,

    /// DEM resolution (FINEST, 10m, 30m, 90m).
    #[builder(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "DEMResolution")]
    dem_resolution: Option<String>,

    /// Profile ID field for grouping multiple profiles.
    #[builder(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "ProfileIDField")]
    profile_id_field: Option<String>,

    /// Maximum distance between sample points.
    #[builder(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "MaximumSampleDistance")]
    maximum_sample_distance: Option<f64>,

    /// Units for maximum sample distance (Meters, Kilometers, etc).
    #[builder(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "MaximumSampleDistanceUnits")]
    maximum_sample_distance_units: Option<String>,

    /// Return Z values (elevation) in output geometry.
    ///
    /// Must be true to get elevation data.
    #[builder(default = "true")]
    #[serde(rename = "returnZ")]
    return_z: bool,

    /// Return M values (distance along profile) in output geometry.
    ///
    /// Must be true to get distance data.
    #[builder(default = "true")]
    #[serde(rename = "returnM")]
    return_m: bool,

    /// Input spatial reference WKID.
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
///
/// Contains elevation profile data as a FeatureSet with Z (elevation) and M (distance) values
/// in the geometry coordinates.
#[derive(Debug, Clone, PartialEq, Getters)]
pub struct ProfileResult {
    /// Profile feature set with elevation data.
    ///
    /// The geometry contains Z values (elevation in meters) and M values (distance in meters)
    /// along the profile path.
    output_profile: FeatureSet,
}

impl ProfileResult {
    /// Creates a new profile result from a FeatureSet.
    pub fn new(output_profile: FeatureSet) -> Self {
        Self { output_profile }
    }

    /// Gets the first point elevation in meters.
    ///
    /// # Errors
    ///
    /// Returns an error if the geometry is missing or invalid.
    #[instrument(skip(self))]
    pub fn first_point_z(&self) -> Result<f64, ArcGISGeometryError> {
        let points = self.elevation_points()?;
        points
            .first()
            .map(|p| *p.elevation_meters())
            .ok_or_else(|| {
                ArcGISGeometryError::new(ArcGISGeometryErrorKind::InvalidGeometry(
                    "Profile has no points".to_string(),
                ))
            })
    }

    /// Gets the last point elevation in meters.
    ///
    /// # Errors
    ///
    /// Returns an error if the geometry is missing or invalid.
    #[instrument(skip(self))]
    pub fn last_point_z(&self) -> Result<f64, ArcGISGeometryError> {
        let points = self.elevation_points()?;
        points.last().map(|p| *p.elevation_meters()).ok_or_else(|| {
            ArcGISGeometryError::new(ArcGISGeometryErrorKind::InvalidGeometry(
                "Profile has no points".to_string(),
            ))
        })
    }

    /// Extracts elevation profile points from the feature set.
    ///
    /// Returns a vector of elevation points ordered by distance along the profile.
    /// The elevation data is stored in the geometry Z values and distance in M values.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Profile geometry is missing or invalid
    /// - Geometry is not a polyline
    /// - Coordinates don't have Z or M values
    #[instrument(skip(self))]
    pub fn elevation_points(&self) -> Result<Vec<ElevationPoint>, ArcGISGeometryError> {
        use crate::ArcGISGeometry;

        tracing::debug!("Extracting elevation points from profile result");

        let feature_count = self.output_profile.features().len();
        tracing::debug!(feature_count, "Processing profile features");

        if self.output_profile.features().is_empty() {
            let err = ArcGISGeometryError::new(ArcGISGeometryErrorKind::InvalidGeometry(
                "Profile result has no features".to_string(),
            ));
            tracing::error!(error = %err, "No features in profile");
            return Err(err);
        }

        // Get the first feature (profile is typically a single polyline)
        let feature = &self.output_profile.features()[0];

        let geometry = feature.geometry().as_ref().ok_or_else(|| {
            let err = ArcGISGeometryError::new(ArcGISGeometryErrorKind::InvalidGeometry(
                "Profile feature missing geometry".to_string(),
            ));
            tracing::error!(error = %err, "Missing geometry");
            err
        })?;

        // Extract polyline paths
        let polyline = match geometry {
            ArcGISGeometry::Polyline(polyline) => polyline,
            _ => {
                let err = ArcGISGeometryError::new(ArcGISGeometryErrorKind::InvalidGeometry(
                    format!("Expected polyline geometry, got {:?}", geometry),
                ));
                tracing::error!(error = %err, "Wrong geometry type");
                return Err(err);
            }
        };

        let paths = polyline.paths();
        if paths.is_empty() {
            let err = ArcGISGeometryError::new(ArcGISGeometryErrorKind::InvalidGeometry(
                "Polyline has no paths".to_string(),
            ));
            tracing::error!(error = %err, "No paths in polyline");
            return Err(err);
        }

        // Get the first path (profile is a single path)
        let path = &paths[0];
        tracing::debug!(coord_count = path.len(), "Processing path coordinates");

        let points: Result<Vec<ElevationPoint>, ArcGISGeometryError> = path
            .iter()
            .enumerate()
            .map(|(idx, coord)| {
                // Coordinates are [x, y, z, m] when hasZ and hasM are true
                if coord.len() < 4 {
                    let err = ArcGISGeometryError::new(ArcGISGeometryErrorKind::InvalidGeometry(
                        format!(
                            "Coordinate {} missing Z or M values (length: {}, expected 4)",
                            idx,
                            coord.len()
                        ),
                    ));
                    tracing::error!(
                        coord_index = idx,
                        coord_length = coord.len(),
                        error = %err,
                        "Invalid coordinate"
                    );
                    return Err(err);
                }

                let elevation = coord[2]; // Z value (elevation in meters)
                let distance = coord[3]; // M value (distance in meters)

                tracing::trace!(
                    coord_index = idx,
                    distance_m = distance,
                    elevation_m = elevation,
                    "Parsed elevation point"
                );

                Ok(ElevationPoint::new(distance, elevation))
            })
            .collect();

        let points = points?;
        tracing::debug!(
            point_count = points.len(),
            "Successfully extracted elevation points"
        );
        Ok(points)
    }
}

/// A single point along an elevation profile.
#[derive(Debug, Clone, PartialEq, Getters)]
pub struct ElevationPoint {
    /// Distance from start in meters.
    distance_meters: f64,

    /// Elevation in meters.
    elevation_meters: f64,
}

impl ElevationPoint {
    /// Creates a new elevation point.
    pub fn new(distance_meters: f64, elevation_meters: f64) -> Self {
        Self {
            distance_meters,
            elevation_meters,
        }
    }
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
