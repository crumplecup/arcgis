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

/// Parameters for summarizing elevation statistics.
///
/// Computes elevation, slope, and aspect statistics for input features (points, lines, or polygons).
/// This operation runs asynchronously via the Elevation GPServer.
///
/// # Example
///
/// ```no_run
/// use arcgis::{SummarizeElevationParametersBuilder, DemResolution};
///
/// let polygon_featureset = r#"{"geometryType":"esriGeometryPolygon","spatialReference":{"wkid":4326},"features":[{"geometry":{"rings":[[[-119.5,37.8],[-119.4,37.8],[-119.4,37.9],[-119.5,37.9],[-119.5,37.8]]]},"attributes":{"OID":1}}]}"#;
///
/// let params = SummarizeElevationParametersBuilder::default()
///     .input_features(polygon_featureset)
///     .dem_resolution(DemResolution::ThirtyMeter.as_str())
///     .include_slope_aspect(true)
///     .build()
///     .expect("Valid parameters");
/// ```
#[derive(Debug, Clone, Serialize, derive_builder::Builder, Getters)]
#[builder(setter(into, strip_option))]
#[serde(rename_all = "camelCase")]
pub struct SummarizeElevationParameters {
    /// Input features as FeatureSet JSON string.
    ///
    /// Must be a valid FeatureSet with geometryType, features array, and spatialReference.
    /// Accepts point, line, or polygon geometries.
    ///
    /// Example: `{"geometryType":"esriGeometryPolygon","spatialReference":{"wkid":4326},"features":[...]}`
    #[serde(rename = "InputFeatures")]
    input_features: String,

    /// Field name for feature IDs (optional).
    ///
    /// Used to match input features with output statistics.
    #[builder(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "FeatureIDField")]
    feature_id_field: Option<String>,

    /// DEM resolution.
    ///
    /// Use `DemResolution` enum and call `.as_str()` to get the string value.
    /// Default: 90m
    #[builder(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "DEMResolution")]
    dem_resolution: Option<String>,

    /// Include slope and aspect statistics in output.
    ///
    /// When true, output includes MinSlope, MeanSlope, MaxSlope, and MeanAspect fields.
    /// Default: false
    #[builder(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "IncludeSlopeAspect")]
    include_slope_aspect: Option<bool>,
}

/// Result from summarize elevation operation.
///
/// Contains elevation statistics extracted from the FeatureSet returned by the GP service.
/// Statistics are stored in feature attributes.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Getters)]
#[serde(rename_all = "PascalCase")]
pub struct SummarizeElevationResult {
    /// Minimum elevation in meters.
    #[serde(skip_serializing_if = "Option::is_none")]
    min_elevation: Option<f64>,

    /// Mean elevation in meters.
    #[serde(skip_serializing_if = "Option::is_none")]
    mean_elevation: Option<f64>,

    /// Maximum elevation in meters.
    #[serde(skip_serializing_if = "Option::is_none")]
    max_elevation: Option<f64>,

    /// Minimum slope in degrees (when IncludeSlopeAspect=true).
    #[serde(skip_serializing_if = "Option::is_none")]
    min_slope: Option<f64>,

    /// Mean slope in degrees (when IncludeSlopeAspect=true).
    #[serde(skip_serializing_if = "Option::is_none")]
    mean_slope: Option<f64>,

    /// Maximum slope in degrees (when IncludeSlopeAspect=true).
    #[serde(skip_serializing_if = "Option::is_none")]
    max_slope: Option<f64>,

    /// Mean aspect in degrees (when IncludeSlopeAspect=true).
    ///
    /// Aspect is the direction the slope faces, measured clockwise from north (0-360 degrees).
    #[serde(skip_serializing_if = "Option::is_none")]
    mean_aspect: Option<f64>,
}

impl SummarizeElevationResult {
    /// Creates a result by extracting statistics from a GP FeatureSet.
    ///
    /// The FeatureSet should contain a single feature with elevation statistics in its attributes.
    ///
    /// # Errors
    ///
    /// Returns an error if the FeatureSet is empty or missing required attributes.
    #[instrument(skip(feature_set))]
    pub fn from_feature_set(feature_set: &FeatureSet) -> Result<Self, ArcGISGeometryError> {
        tracing::debug!("Parsing SummarizeElevationResult from FeatureSet");

        let features = feature_set.features();
        if features.is_empty() {
            let err = ArcGISGeometryError::new(ArcGISGeometryErrorKind::InvalidGeometry(
                "SummarizeElevation result has no features".to_string(),
            ));
            tracing::error!(error = %err, "No features in result");
            return Err(err);
        }

        let feature = &features[0];
        let attrs = feature.attributes();

        // Extract elevation statistics from attributes
        let result = Self {
            min_elevation: attrs
                .get("MinElevation")
                .and_then(|v: &serde_json::Value| v.as_f64()),
            mean_elevation: attrs
                .get("MeanElevation")
                .and_then(|v: &serde_json::Value| v.as_f64()),
            max_elevation: attrs
                .get("MaxElevation")
                .and_then(|v: &serde_json::Value| v.as_f64()),
            min_slope: attrs
                .get("MinSlope")
                .and_then(|v: &serde_json::Value| v.as_f64()),
            mean_slope: attrs
                .get("MeanSlope")
                .and_then(|v: &serde_json::Value| v.as_f64()),
            max_slope: attrs
                .get("MaxSlope")
                .and_then(|v: &serde_json::Value| v.as_f64()),
            mean_aspect: attrs
                .get("MeanAspect")
                .and_then(|v: &serde_json::Value| v.as_f64()),
        };

        tracing::debug!(
            min_elevation = ?result.min_elevation,
            mean_elevation = ?result.mean_elevation,
            max_elevation = ?result.max_elevation,
            "Parsed elevation statistics"
        );

        Ok(result)
    }
}

/// Parameters for viewshed analysis.
///
/// Determines visible areas from observer points based on terrain and viewing parameters.
/// This operation runs asynchronously via the Elevation GPServer.
///
/// # Example
///
/// ```no_run
/// use arcgis::{ViewshedParametersBuilder, DemResolution};
///
/// let observer_points = r#"{"geometryType":"esriGeometryMultipoint","spatialReference":{"wkid":4326},"points":[[-119.5,37.85]]}"#;
///
/// let params = ViewshedParametersBuilder::default()
///     .input_points(observer_points)
///     .maximum_distance(5000.0)  // 5 km
///     .maximum_distance_units("Meters")
///     .observer_height(1.75)  // Default human eye height
///     .dem_resolution(DemResolution::ThirtyMeter.as_str())
///     .build()
///     .expect("Valid parameters");
/// ```
#[derive(Debug, Clone, Serialize, derive_builder::Builder, Getters)]
#[builder(setter(into, strip_option))]
#[serde(rename_all = "camelCase")]
pub struct ViewshedParameters {
    /// Observer point(s) as FeatureSet JSON string.
    ///
    /// Must be a valid FeatureSet with geometryType (esriGeometryPoint or esriGeometryMultipoint),
    /// features array, and spatialReference.
    ///
    /// Example: `{"geometryType":"esriGeometryMultipoint","spatialReference":{"wkid":4326},"points":[[-119.5,37.85]]}`
    #[serde(rename = "InputPoints")]
    input_points: String,

    /// Maximum viewing distance (visibility cutoff).
    ///
    /// Up to 50 km maximum. Use with `maximum_distance_units` to specify units.
    #[builder(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "MaximumDistance")]
    maximum_distance: Option<f64>,

    /// Units for maximum distance.
    ///
    /// Valid values: "Meters", "Kilometers", "Feet", "Yards", "Miles"
    #[builder(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "MaximumDistanceUnits")]
    maximum_distance_units: Option<String>,

    /// DEM resolution.
    ///
    /// Use `DemResolution` enum and call `.as_str()` to get the string value.
    /// Default: 90m
    #[builder(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "DEMResolution")]
    dem_resolution: Option<String>,

    /// Observer height above ground.
    ///
    /// Default: 1.75 meters (average human eye height)
    /// Use with `observer_height_units` to specify units.
    #[builder(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "ObserverHeight")]
    observer_height: Option<f64>,

    /// Units for observer height.
    ///
    /// Valid values: "Meters", "Kilometers", "Feet", "Yards", "Miles"
    #[builder(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "ObserverHeightUnits")]
    observer_height_units: Option<String>,

    /// Surface offset (target object height above surface).
    ///
    /// Default: 0.0 meters (ground level)
    /// Use with `surface_offset_units` to specify units.
    #[builder(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "SurfaceOffset")]
    surface_offset: Option<f64>,

    /// Units for surface offset.
    ///
    /// Valid values: "Meters", "Kilometers", "Feet", "Yards", "Miles"
    #[builder(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "SurfaceOffsetUnits")]
    surface_offset_units: Option<String>,

    /// Generalize viewshed polygons for smoother output.
    ///
    /// Default: true
    #[builder(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "GeneralizeViewshedPolygons")]
    generalize_viewshed_polygons: Option<bool>,
}

/// Result from viewshed analysis.
///
/// Contains the viewshed polygon(s) showing visible areas from observer points.
#[derive(Debug, Clone, PartialEq, Getters)]
pub struct ViewshedResult {
    /// Viewshed polygon feature set.
    ///
    /// Contains polygon features representing areas visible from observer points.
    /// Attributes include: Frequency, DEMResolution, Product Name, Source, Source URL
    output_viewshed: FeatureSet,
}

impl ViewshedResult {
    /// Creates a viewshed result from a FeatureSet.
    pub fn new(output_viewshed: FeatureSet) -> Self {
        Self { output_viewshed }
    }

    /// Gets the number of viewshed polygons.
    pub fn viewshed_count(&self) -> usize {
        self.output_viewshed.features().len()
    }
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
