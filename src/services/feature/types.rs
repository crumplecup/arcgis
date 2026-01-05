//! Types for Feature Service operations.

use crate::{ArcGISGeometry, GeometryType, ObjectId, SpatialRel};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Serialization helpers for URL query parameters.
mod serde_helpers {
    use serde::Serializer;

    /// Serializes a Vec<String> as a comma-separated string for URL query parameters.
    pub fn serialize_string_vec<S>(
        vec: &Option<Vec<String>>,
        serializer: S,
    ) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match vec {
            Some(v) => serializer.serialize_str(&v.join(",")),
            None => serializer.serialize_none(),
        }
    }

    /// Serializes a Vec<ObjectId> as a comma-separated string for URL query parameters.
    pub fn serialize_object_ids<S>(
        vec: &Option<Vec<crate::ObjectId>>,
        serializer: S,
    ) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match vec {
            Some(v) => {
                let ids: Vec<String> = v.iter().map(|id| id.to_string()).collect();
                serializer.serialize_str(&ids.join(","))
            }
            None => serializer.serialize_none(),
        }
    }

    /// Serializes geometry as a JSON string for URL query parameters.
    pub fn serialize_geometry<S>(
        geom: &Option<crate::ArcGISGeometry>,
        serializer: S,
    ) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match geom {
            Some(g) => {
                let json = serde_json::to_string(g).map_err(serde::ser::Error::custom)?;
                serializer.serialize_str(&json)
            }
            None => serializer.serialize_none(),
        }
    }
}

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
    /// This field is optional because count-only queries don't return features.
    #[serde(default)]
    pub features: Vec<Feature>,

    /// Count of features (present when returnCountOnly=true).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub count: Option<u32>,

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
    #[serde(
        rename = "outFields",
        skip_serializing_if = "Option::is_none",
        serialize_with = "serde_helpers::serialize_string_vec"
    )]
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
    #[serde(
        skip_serializing_if = "Option::is_none",
        serialize_with = "serde_helpers::serialize_geometry"
    )]
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
    #[serde(
        rename = "objectIds",
        skip_serializing_if = "Option::is_none",
        serialize_with = "serde_helpers::serialize_object_ids"
    )]
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
    #[serde(
        rename = "orderByFields",
        skip_serializing_if = "Option::is_none",
        serialize_with = "serde_helpers::serialize_string_vec"
    )]
    pub order_by_fields: Option<Vec<String>>,

    /// GROUP BY clause.
    #[serde(
        rename = "groupByFieldsForStatistics",
        skip_serializing_if = "Option::is_none",
        serialize_with = "serde_helpers::serialize_string_vec"
    )]
    pub group_by_fields: Option<Vec<String>>,

    /// Statistics to calculate (aggregate queries).
    ///
    /// When specified, only these query parameters can be used:
    /// groupByFieldsForStatistics, orderByFields, time, returnDistinctValues, where.
    #[serde(rename = "outStatistics", skip_serializing_if = "Option::is_none")]
    pub out_statistics: Option<Vec<StatisticDefinition>>,

    /// HAVING clause for filtering aggregated results.
    ///
    /// Only valid when `out_statistics` is specified.
    /// Example: `"COUNT(*) > 1000"` or `"SUM(AREA) > 5000"`
    #[serde(rename = "having", skip_serializing_if = "Option::is_none")]
    pub having: Option<String>,

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
            out_statistics: None,
            having: None,
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

/// Statistical operation type for aggregate queries.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum StatisticType {
    /// Count of records
    Count,
    /// Sum of field values
    Sum,
    /// Minimum value
    Min,
    /// Maximum value
    Max,
    /// Average (mean) value
    Avg,
    /// Standard deviation
    Stddev,
    /// Variance
    Var,
    /// Continuous percentile
    #[serde(rename = "PERCENTILE_CONT")]
    PercentileCont,
    /// Discrete percentile
    #[serde(rename = "PERCENTILE_DISC")]
    PercentileDisc,
}

/// Defines a field-based statistic to calculate.
///
/// Used with [`FeatureQueryParams::out_statistics`] to perform aggregate queries.
///
/// # Example
///
/// ```
/// use arcgis::{StatisticDefinition, StatisticType};
///
/// let stat = StatisticDefinition::new(
///     StatisticType::Avg,
///     "POPULATION".to_string(),
///     "avg_population".to_string()
/// );
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, derive_new::new)]
#[serde(rename_all = "camelCase")]
pub struct StatisticDefinition {
    /// The type of statistic to calculate.
    pub statistic_type: StatisticType,

    /// The field name to calculate statistics on.
    pub on_statistic_field: String,

    /// The output field name for the result.
    pub out_statistic_field_name: String,
}

/// Response from a feature statistics query.
///
/// Contains aggregate results when querying with `outStatistics`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, derive_getters::Getters)]
#[serde(rename_all = "camelCase")]
pub struct FeatureStatisticsResponse {
    /// The features containing statistical results.
    features: Vec<Feature>,

    /// Field aliases mapping output field names to descriptions.
    #[serde(default)]
    field_aliases: HashMap<String, String>,
}

/// Parameters for querying related records.
///
/// Use [`RelatedRecordsParams::builder()`] to construct instances.
///
/// # Example
///
/// ```
/// use arcgis::{RelatedRecordsParams, ObjectId};
///
/// let params = RelatedRecordsParams::builder()
///     .object_ids(vec![ObjectId::new(1), ObjectId::new(2)])
///     .relationship_id(3u32)
///     .out_fields(vec!["NAME".to_string(), "STATUS".to_string()])
///     .build()
///     .expect("Valid params");
/// ```
#[derive(Debug, Clone, Serialize, derive_builder::Builder)]
#[builder(setter(into, strip_option), default)]
pub struct RelatedRecordsParams {
    /// Object IDs of features to query related records for (REQUIRED).
    #[serde(
        rename = "objectIds",
        serialize_with = "serde_helpers::serialize_object_ids"
    )]
    pub object_ids: Option<Vec<ObjectId>>,

    /// ID of the relationship to query (REQUIRED).
    #[serde(rename = "relationshipId")]
    pub relationship_id: Option<u32>,

    /// Fields to include in the response.
    #[serde(
        rename = "outFields",
        skip_serializing_if = "Option::is_none",
        serialize_with = "serde_helpers::serialize_string_vec"
    )]
    pub out_fields: Option<Vec<String>>,

    /// WHERE clause to filter related records.
    #[serde(rename = "definitionExpression", skip_serializing_if = "Option::is_none")]
    pub definition_expression: Option<String>,

    /// Whether to return geometry with features.
    #[serde(rename = "returnGeometry", skip_serializing_if = "Option::is_none")]
    pub return_geometry: Option<bool>,

    /// Output spatial reference WKID.
    #[serde(rename = "outSR", skip_serializing_if = "Option::is_none")]
    pub out_sr: Option<i32>,

    /// Maximum offset for geometry generalization.
    #[serde(rename = "maxAllowableOffset", skip_serializing_if = "Option::is_none")]
    pub max_allowable_offset: Option<f64>,

    /// Decimal places for geometry coordinates.
    #[serde(rename = "geometryPrecision", skip_serializing_if = "Option::is_none")]
    pub geometry_precision: Option<i32>,

    /// Return z-values.
    #[serde(rename = "returnZ", skip_serializing_if = "Option::is_none")]
    pub return_z: Option<bool>,

    /// Return m-values.
    #[serde(rename = "returnM", skip_serializing_if = "Option::is_none")]
    pub return_m: Option<bool>,

    /// Geodatabase version.
    #[serde(rename = "gdbVersion", skip_serializing_if = "Option::is_none")]
    pub gdb_version: Option<String>,

    /// Historic moment (epoch milliseconds).
    #[serde(rename = "historicMoment", skip_serializing_if = "Option::is_none")]
    pub historic_moment: Option<i64>,

    /// Return only counts per object ID.
    #[serde(rename = "returnCountOnly", skip_serializing_if = "Option::is_none")]
    pub return_count_only: Option<bool>,

    /// ORDER BY clause.
    #[serde(
        rename = "orderByFields",
        skip_serializing_if = "Option::is_none",
        serialize_with = "serde_helpers::serialize_string_vec"
    )]
    pub order_by_fields: Option<Vec<String>>,

    /// Offset for pagination.
    #[serde(rename = "resultOffset", skip_serializing_if = "Option::is_none")]
    pub result_offset: Option<u32>,

    /// Maximum number of features to return per object ID.
    #[serde(rename = "resultRecordCount", skip_serializing_if = "Option::is_none")]
    pub result_record_count: Option<u32>,

    /// Response format.
    #[serde(rename = "f")]
    #[builder(default = "ResponseFormat::Json")]
    pub format: ResponseFormat,
}

impl Default for RelatedRecordsParams {
    fn default() -> Self {
        Self {
            object_ids: None,
            relationship_id: None,
            out_fields: None,
            definition_expression: None,
            return_geometry: Some(true),
            out_sr: None,
            max_allowable_offset: None,
            geometry_precision: None,
            return_z: None,
            return_m: None,
            gdb_version: None,
            historic_moment: None,
            return_count_only: None,
            order_by_fields: None,
            result_offset: None,
            result_record_count: None,
            format: ResponseFormat::Json,
        }
    }
}

impl RelatedRecordsParams {
    /// Creates a builder for RelatedRecordsParams.
    pub fn builder() -> RelatedRecordsParamsBuilder {
        RelatedRecordsParamsBuilder::default()
    }
}

/// A group of related records for a specific source object ID.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, derive_getters::Getters)]
#[serde(rename_all = "camelCase")]
pub struct RelatedRecordGroup {
    /// The source object ID.
    object_id: ObjectId,

    /// Related records for this object ID.
    #[serde(default)]
    related_records: Vec<Feature>,
}

/// Response from a query related records operation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, derive_getters::Getters)]
#[serde(rename_all = "camelCase")]
pub struct RelatedRecordsResponse {
    /// Groups of related records, one per source object ID.
    #[serde(default)]
    related_record_groups: Vec<RelatedRecordGroup>,

    /// Geometry type of the related records (if applicable).
    #[serde(skip_serializing_if = "Option::is_none")]
    geometry_type: Option<GeometryType>,

    /// Spatial reference of the geometries.
    #[serde(skip_serializing_if = "Option::is_none")]
    spatial_reference: Option<serde_json::Value>,

    /// Field definitions.
    #[serde(default)]
    fields: Vec<serde_json::Value>,
}

/// Top filter specification for queryTopFeatures operations.
///
/// Specifies how to group results, how many features to return from each group,
/// and how to order results within each group.
///
/// # Example
///
/// ```
/// use arcgis::TopFilter;
///
/// // Get top 3 most populous cities from each state
/// let filter = TopFilter::new(
///     vec!["State".to_string()],
///     3,
///     vec!["Population DESC".to_string()],
/// );
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, derive_new::new)]
#[serde(rename_all = "camelCase")]
pub struct TopFilter {
    /// Fields to group results by.
    pub group_by_fields: Vec<String>,

    /// Number of top features to return from each group.
    pub top_count: u32,

    /// Fields to order results by (format: "FieldName ASC" or "FieldName DESC").
    pub order_by_fields: Vec<String>,
}

/// Parameters for querying top features from a feature service layer.
///
/// The queryTopFeatures operation returns features based on ranking within groups.
/// For example, you can query the top 3 most populous cities from each state.
///
/// # Example
///
/// ```
/// use arcgis::{TopFeaturesParams, TopFilter};
///
/// let filter = TopFilter::new(
///     vec!["State".to_string()],
///     3,
///     vec!["Population DESC".to_string()],
/// );
///
/// let params = TopFeaturesParams::builder()
///     .top_filter(filter)
///     .where_("Population > 100000")
///     .out_fields(vec!["Name".to_string(), "Population".to_string()])
///     .build()
///     .expect("Valid params");
/// ```
#[derive(Debug, Clone, Serialize, derive_builder::Builder)]
#[builder(setter(into, strip_option), default)]
pub struct TopFeaturesParams {
    /// Required: Top filter specification (group by, count, order by).
    #[serde(rename = "topFilter")]
    pub top_filter: Option<TopFilter>,

    /// WHERE clause for the query filter.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub where_: Option<String>,

    /// Object IDs to query.
    #[serde(
        rename = "objectIds",
        skip_serializing_if = "Option::is_none",
        serialize_with = "serde_helpers::serialize_object_ids"
    )]
    pub object_ids: Option<Vec<ObjectId>>,

    /// Time instant or extent to query.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub time: Option<String>,

    /// Geometry to apply as spatial filter.
    #[serde(
        skip_serializing_if = "Option::is_none",
        serialize_with = "serde_helpers::serialize_geometry"
    )]
    pub geometry: Option<ArcGISGeometry>,

    /// Type of geometry being used as spatial filter.
    #[serde(rename = "geometryType", skip_serializing_if = "Option::is_none")]
    pub geometry_type: Option<GeometryType>,

    /// Spatial reference of input geometry.
    #[serde(rename = "inSR", skip_serializing_if = "Option::is_none")]
    pub in_sr: Option<i32>,

    /// Spatial relationship for the query.
    #[serde(rename = "spatialRel", skip_serializing_if = "Option::is_none")]
    pub spatial_rel: Option<SpatialRel>,

    /// Buffer distance for input geometries.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub distance: Option<f64>,

    /// Units for distance parameter.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub units: Option<String>,

    /// Fields to include in the response.
    #[serde(
        rename = "outFields",
        skip_serializing_if = "Option::is_none",
        serialize_with = "serde_helpers::serialize_string_vec"
    )]
    pub out_fields: Option<Vec<String>>,

    /// Whether to return geometry with the results (default: true).
    #[serde(rename = "returnGeometry", skip_serializing_if = "Option::is_none")]
    pub return_geometry: Option<bool>,

    /// Maximum offset for geometry generalization.
    #[serde(
        rename = "maxAllowableOffset",
        skip_serializing_if = "Option::is_none"
    )]
    pub max_allowable_offset: Option<f64>,

    /// Number of decimal places for returned geometries.
    #[serde(
        rename = "geometryPrecision",
        skip_serializing_if = "Option::is_none"
    )]
    pub geometry_precision: Option<i32>,

    /// Spatial reference for returned geometry.
    #[serde(rename = "outSR", skip_serializing_if = "Option::is_none")]
    pub out_sr: Option<i32>,

    /// Whether to return only object IDs.
    #[serde(rename = "returnIdsOnly", skip_serializing_if = "Option::is_none")]
    pub return_ids_only: Option<bool>,

    /// Whether to return only the feature count.
    #[serde(rename = "returnCountOnly", skip_serializing_if = "Option::is_none")]
    pub return_count_only: Option<bool>,

    /// Whether to return only the extent.
    #[serde(rename = "returnExtentOnly", skip_serializing_if = "Option::is_none")]
    pub return_extent_only: Option<bool>,

    /// Whether to include z-values if available.
    #[serde(rename = "returnZ", skip_serializing_if = "Option::is_none")]
    pub return_z: Option<bool>,

    /// Whether to include m-values if available.
    #[serde(rename = "returnM", skip_serializing_if = "Option::is_none")]
    pub return_m: Option<bool>,

    /// Control on the number of features returned.
    #[serde(rename = "resultType", skip_serializing_if = "Option::is_none")]
    pub result_type: Option<String>,

    /// Output format (json, geojson, pbf).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub f: Option<String>,
}

impl Default for TopFeaturesParams {
    fn default() -> Self {
        Self {
            top_filter: None,
            where_: None,
            object_ids: None,
            time: None,
            geometry: None,
            geometry_type: None,
            in_sr: None,
            spatial_rel: None,
            distance: None,
            units: None,
            out_fields: None,
            return_geometry: Some(true),
            max_allowable_offset: None,
            geometry_precision: None,
            out_sr: None,
            return_ids_only: None,
            return_count_only: None,
            return_extent_only: None,
            return_z: None,
            return_m: None,
            result_type: None,
            f: Some("json".to_string()),
        }
    }
}

impl TopFeaturesParams {
    /// Creates a builder for TopFeaturesParams.
    pub fn builder() -> TopFeaturesParamsBuilder {
        TopFeaturesParamsBuilder::default()
    }
}
