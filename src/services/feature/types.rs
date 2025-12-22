//! Types for Feature Service operations.

use crate::{ArcGISGeometry, GeometryType, ObjectId, SpatialRel};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Response format for feature service queries.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum ResponseFormat {
    /// JSON format.
    #[default]
    Json,
    /// GeoJSON format.
    #[serde(rename = "geojson")]
    GeoJson,
    /// Protocol Buffer format.
    Pbf,
}

/// A single feature returned from a feature service.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Feature {
    /// Feature attributes as key-value pairs.
    pub attributes: HashMap<String, serde_json::Value>,

    /// Optional geometry.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub geometry: Option<ArcGISGeometry>,
}

/// A set of features returned from a query.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FeatureSet {
    /// Geometry type of features in this set.
    #[serde(rename = "geometryType", skip_serializing_if = "Option::is_none")]
    pub geometry_type: Option<GeometryType>,

    /// Array of features.
    pub features: Vec<Feature>,

    /// Whether the result set exceeded the transfer limit.
    #[serde(rename = "exceededTransferLimit", default)]
    pub exceeded_transfer_limit: bool,
}

/// Parameters for querying features from a feature service.
///
/// Use the builder pattern to construct query parameters.
///
/// # Example
/// ```no_run
/// use arcgis::FeatureQueryParams;
///
/// let params = FeatureQueryParams::builder()
///     .where_clause("POPULATION > 100000")
///     .out_fields(vec!["NAME".to_string(), "POPULATION".to_string()])
///     .return_geometry(true)
///     .build();
/// ```
#[derive(Debug, Clone, Serialize, derive_builder::Builder)]
#[builder(setter(into, strip_option), default)]
pub struct FeatureQueryParams {
    /// WHERE clause for the query.
    #[serde(rename = "where")]
    #[builder(default = "String::from(\"1=1\")")]
    pub where_clause: String,

    /// Fields to include in the response.
    /// If None, all fields are returned.
    #[serde(rename = "outFields", skip_serializing_if = "Option::is_none")]
    pub out_fields: Option<Vec<String>>,

    /// Whether to return geometry with features.
    #[serde(rename = "returnGeometry")]
    #[builder(default = "true")]
    pub return_geometry: bool,

    /// Response format.
    #[serde(rename = "f")]
    #[builder(default)]
    pub format: ResponseFormat,

    /// Geometry to use for spatial filter.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub geometry: Option<ArcGISGeometry>,

    /// Geometry type of the geometry parameter.
    #[serde(rename = "geometryType", skip_serializing_if = "Option::is_none")]
    pub geometry_type: Option<GeometryType>,

    /// Spatial relationship to use for spatial filter.
    #[serde(rename = "spatialRel", skip_serializing_if = "Option::is_none")]
    pub spatial_rel: Option<SpatialRel>,

    /// Maximum number of features to return.
    #[serde(rename = "resultRecordCount", skip_serializing_if = "Option::is_none")]
    pub result_record_count: Option<u32>,

    /// Offset for pagination.
    #[serde(rename = "resultOffset", skip_serializing_if = "Option::is_none")]
    pub result_offset: Option<u32>,

    /// Object IDs to query.
    #[serde(rename = "objectIds", skip_serializing_if = "Option::is_none")]
    pub object_ids: Option<Vec<ObjectId>>,

    /// Whether to return distinct values only.
    #[serde(
        rename = "returnDistinctValues",
        skip_serializing_if = "Option::is_none"
    )]
    pub return_distinct_values: Option<bool>,

    /// Whether to return object IDs only.
    #[serde(rename = "returnIdsOnly", skip_serializing_if = "Option::is_none")]
    pub return_ids_only: Option<bool>,

    /// Whether to return count only.
    #[serde(rename = "returnCountOnly", skip_serializing_if = "Option::is_none")]
    pub return_count_only: Option<bool>,

    /// ORDER BY clause.
    #[serde(rename = "orderByFields", skip_serializing_if = "Option::is_none")]
    pub order_by_fields: Option<Vec<String>>,

    /// GROUP BY clause.
    #[serde(
        rename = "groupByFieldsForStatistics",
        skip_serializing_if = "Option::is_none"
    )]
    pub group_by_fields: Option<Vec<String>>,

    /// Output spatial reference WKID.
    #[serde(rename = "outSR", skip_serializing_if = "Option::is_none")]
    pub out_sr: Option<i32>,
}

impl Default for FeatureQueryParams {
    fn default() -> Self {
        Self {
            where_clause: "1=1".to_string(),
            out_fields: None,
            return_geometry: true,
            format: ResponseFormat::default(),
            geometry: None,
            geometry_type: None,
            spatial_rel: None,
            result_record_count: None,
            result_offset: None,
            object_ids: None,
            return_distinct_values: None,
            return_ids_only: None,
            return_count_only: None,
            order_by_fields: None,
            group_by_fields: None,
            out_sr: None,
        }
    }
}

impl FeatureQueryParams {
    /// Creates a new builder for FeatureQueryParams.
    pub fn builder() -> FeatureQueryParamsBuilder {
        FeatureQueryParamsBuilder::default()
    }
}
