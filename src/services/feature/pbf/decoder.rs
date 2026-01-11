//! Decoder for converting ArcGIS Protocol Buffer responses to Rust types.

use super::feature_collection_p_buffer::*;
use super::FeatureCollectionPBuffer;
use crate::{Feature, FeatureSet, GeometryType, Result};
use prost::Message;
use std::collections::HashMap;

/// Decode a PBF FeatureCollection into a FeatureSet.
///
/// This function handles the conversion from Esri's protocol buffer format
/// to our standard FeatureSet type, including geometry reconstruction and
/// attribute mapping.
pub fn decode_feature_collection(bytes: &[u8]) -> Result<FeatureSet> {
    // Parse the protocol buffer message
    let pbf = FeatureCollectionPBuffer::decode(bytes)
        .map_err(|e| crate::Error::from(crate::ErrorKind::Other(format!("PBF decode error: {}", e))))?;

    // Extract the query result
    let query_result = pbf
        .query_result
        .ok_or_else(|| crate::Error::from(crate::ErrorKind::Other("Missing query result in PBF".to_string())))?;

    // Handle different result types
    match query_result.results {
        Some(query_result::Results::FeatureResult(feature_result)) => {
            decode_feature_result(feature_result)
        }
        Some(query_result::Results::CountResult(count_result)) => {
            // Count-only query result
            Ok(FeatureSet::new(
                None,
                vec![],
                Some(count_result.count as u32),
                false,
            ))
        }
        Some(query_result::Results::IdsResult(object_ids_result)) => {
            // Object IDs only result - convert to features with just OBJECTID attribute
            let features = object_ids_result
                .object_ids
                .iter()
                .map(|&oid| {
                    let mut attributes = HashMap::new();
                    attributes.insert(
                        object_ids_result.object_id_field_name.clone(),
                        serde_json::Value::Number(serde_json::Number::from(oid)),
                    );
                    Feature::new(attributes, None)
                })
                .collect();

            Ok(FeatureSet::new(None, features, None, false))
        }
        None => Err(crate::Error::from(crate::ErrorKind::Other(
            "No result data in query result".to_string(),
        ))),
    }
}

/// Decode a FeatureResult into a FeatureSet.
fn decode_feature_result(feature_result: FeatureResult) -> Result<FeatureSet> {
    // Convert PBF geometry type to our GeometryType
    let geometry_type = convert_geometry_type(feature_result.geometry_type)?;

    // Build a lookup for reusable values (PBF uses indexed values to save space)
    // TODO: Use this lookup when attributes reference indexed values
    let _values_lookup: Vec<serde_json::Value> = feature_result
        .values
        .iter()
        .map(convert_pbf_value)
        .collect();

    // Convert PBF features to our Feature type
    let features: Vec<Feature> = feature_result
        .features
        .iter()
        .map(|pbf_feature| {
            // Map attributes from field names to values
            let mut attributes = HashMap::new();
            for (idx, field) in feature_result.fields.iter().enumerate() {
                if let Some(attr) = pbf_feature.attributes.get(idx) {
                    // Attributes can be inline values or indices into the values array
                    let value = if let Some(ref value_type) = attr.value_type {
                        match value_type {
                            value::ValueType::StringValue(ref s) => serde_json::Value::String(s.clone()),
                            value::ValueType::FloatValue(f) => {
                                serde_json::Value::Number(serde_json::Number::from_f64(*f as f64).unwrap())
                            }
                            value::ValueType::DoubleValue(d) => {
                                serde_json::Value::Number(serde_json::Number::from_f64(*d).unwrap())
                            }
                            value::ValueType::SintValue(i) => serde_json::Value::Number((*i).into()),
                            value::ValueType::UintValue(u) => serde_json::Value::Number((*u).into()),
                            value::ValueType::Int64Value(i) => serde_json::Value::Number((*i).into()),
                            value::ValueType::Uint64Value(u) => serde_json::Value::Number((*u).into()),
                            value::ValueType::Sint64Value(i) => serde_json::Value::Number((*i).into()),
                            value::ValueType::BoolValue(b) => serde_json::Value::Bool(*b),
                        }
                    } else {
                        serde_json::Value::Null
                    };
                    attributes.insert(field.name.clone(), value);
                }
            }

            // TODO: Decode geometry from delta-encoded coordinates
            // For now, we'll leave geometry as None and implement this next
            let geometry = None;

            Feature::new(attributes, geometry)
        })
        .collect();

    Ok(FeatureSet::new(
        Some(geometry_type),
        features,
        None,
        feature_result.exceeded_transfer_limit,
    ))
}

/// Convert PBF GeometryType enum to our GeometryType.
fn convert_geometry_type(pbf_type: i32) -> Result<GeometryType> {
    use GeometryType as GT;

    // PBF geometry type values from the proto enum
    match pbf_type {
        0 => Ok(GT::Point),
        1 => Ok(GT::Multipoint),
        2 => Ok(GT::Polyline),
        3 => Ok(GT::Polygon),
        4 => Err(crate::Error::from(crate::ErrorKind::Other(
            "Multipatch geometry not supported yet".to_string(),
        ))), // Multipatch not in our enum yet
        127 => Err(crate::Error::from(crate::ErrorKind::Other(
            "Geometry type is None".to_string(),
        ))),
        _ => Err(crate::Error::from(crate::ErrorKind::Other(format!(
            "Unknown geometry type: {}",
            pbf_type
        )))),
    }
}

/// Convert a PBF Value to a serde_json::Value.
fn convert_pbf_value(pbf_value: &Value) -> serde_json::Value {
    if let Some(ref value_type) = pbf_value.value_type {
        match value_type {
            value::ValueType::StringValue(ref s) => serde_json::Value::String(s.clone()),
            value::ValueType::FloatValue(f) => {
                serde_json::Value::Number(serde_json::Number::from_f64(*f as f64).unwrap())
            }
            value::ValueType::DoubleValue(d) => {
                serde_json::Value::Number(serde_json::Number::from_f64(*d).unwrap())
            }
            value::ValueType::SintValue(i) => serde_json::Value::Number((*i).into()),
            value::ValueType::UintValue(u) => serde_json::Value::Number((*u).into()),
            value::ValueType::Int64Value(i) => serde_json::Value::Number((*i).into()),
            value::ValueType::Uint64Value(u) => serde_json::Value::Number((*u).into()),
            value::ValueType::Sint64Value(i) => serde_json::Value::Number((*i).into()),
            value::ValueType::BoolValue(b) => serde_json::Value::Bool(*b),
        }
    } else {
        serde_json::Value::Null
    }
}
