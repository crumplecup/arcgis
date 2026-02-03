//! Service definition types for ArcGIS Feature Services.
//!
//! This module provides strongly-typed Rust structures that mirror ESRI's Feature Service
//! JSON specification. All types are validated against official ESRI documentation to ensure
//! API compatibility.
//!
//! # ESRI Specification Sources
//!
//! - **Feature Service**: <https://developers.arcgis.com/rest/services-reference/enterprise/feature-service/>
//! - **Layer Definition**: <https://developers.arcgis.com/rest/services-reference/enterprise/layer-feature-service/>
//! - **Create Service**: <https://developers.arcgis.com/rest/users-groups-and-items/create-service/>
//!
//! # Example: Creating a Branch-Versioned Service
//!
//! ```no_run
//! use arcgis::service_definition::{
//!     ServiceDefinitionBuilder, LayerDefinitionBuilder, FieldDefinitionBuilder,
//!     GeometryTypeDefinition, FieldType, VersioningType,
//! };
//!
//! let service_def = ServiceDefinitionBuilder::default()
//!     .name("MyVersionedService")
//!     .add_layer(
//!         LayerDefinitionBuilder::default()
//!             .id(0)
//!             .name("Points")
//!             .geometry_type(GeometryTypeDefinition::Point)
//!             .add_field(
//!                 FieldDefinitionBuilder::default()
//!                     .name("OBJECTID")
//!                     .field_type(FieldType::Oid)
//!                     .nullable(false)
//!                     .editable(false)
//!                     .build()?
//!             )
//!             .build()?
//!     )
//!     .build()?;
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```

use serde::{Deserialize, Serialize};

/// Top-level service definition for Feature Services.
///
/// Maps to ESRI's `createParameters` JSON object used with the createService operation.
///
/// # ESRI Documentation
///
/// Source: <https://developers.arcgis.com/rest/users-groups-and-items/create-service/>
///
/// Required properties:
/// - `name`: Service name
///
/// All other properties are optional but recommended for production services.
///
/// # Branch Versioning
///
/// To enable branch versioning, include layers with ObjectID and GlobalID fields.
/// Branch versioning is automatically enabled when these requirements are met.
#[derive(
    Debug, Clone, Default, Serialize, Deserialize, derive_builder::Builder, derive_getters::Getters,
)]
#[builder(setter(into, strip_option), default)]
#[serde(rename_all = "camelCase")]
pub struct ServiceDefinition {
    /// Service name (REQUIRED).
    ///
    /// Must be unique within the user's content.
    #[builder(setter(into))]
    name: String,

    /// Service description.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default)]
    service_description: Option<String>,

    /// Whether the service contains static data (vs dynamic/editable).
    ///
    /// Default: `false` for editable services.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default)]
    has_static_data: Option<bool>,

    /// Maximum number of records returned by queries.
    ///
    /// ESRI default: 1000. Recommended: 2000 for better performance.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default)]
    max_record_count: Option<i32>,

    /// Supported query formats.
    ///
    /// Default: "JSON". Other options: "JSON, AMF, geoJSON, PBF".
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default)]
    supported_query_formats: Option<String>,

    /// Service capabilities.
    ///
    /// Comma-separated list. Common: "Create,Delete,Query,Update,Editing".
    /// For versioning: "Create,Delete,Query,Update,Editing,Sync".
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default)]
    capabilities: Option<String>,

    /// Layer definitions.
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    #[builder(default)]
    layers: Vec<LayerDefinition>,

    /// Table definitions (non-spatial).
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    #[builder(default)]
    tables: Vec<TableDefinition>,

    /// Spatial reference for the service.
    ///
    /// All layers must use the same spatial reference.
    /// Default: WGS84 (WKID 4326) if not specified.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default)]
    spatial_reference: Option<SpatialReferenceDefinition>,

    /// Initial extent as [[xmin, ymin], [xmax, ymax]].
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default)]
    initial_extent: Option<Vec<Vec<f64>>>,

    /// Whether geometry updates are allowed.
    ///
    /// Default: `true` for editable services.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default)]
    allow_geometry_updates: Option<bool>,

    /// Units for the service.
    ///
    /// Common values: "esriMeters", "esriFeet", "esriDecimalDegrees".
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default)]
    units: Option<String>,

    /// XSS prevention configuration.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default)]
    xss_prevention_info: Option<XssPreventionInfo>,

    /// Editor tracking configuration.
    ///
    /// Automatically tracks who created/edited features and when.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default)]
    editor_tracking_info: Option<EditorTrackingInfo>,
}

impl ServiceDefinitionBuilder {
    /// Adds a layer to the service.
    pub fn add_layer(mut self, layer: LayerDefinition) -> Self {
        self.layers.get_or_insert_with(Vec::new).push(layer);
        self
    }

    /// Adds a table to the service.
    pub fn add_table(mut self, table: TableDefinition) -> Self {
        self.tables.get_or_insert_with(Vec::new).push(table);
        self
    }
}

/// Layer definition within a Feature Service.
///
/// # ESRI Documentation
///
/// Source: <https://developers.arcgis.com/rest/services-reference/enterprise/layer-feature-service/>
///
/// Required properties:
/// - `id`: Layer ID (unique within service)
/// - `name`: Layer name
/// - `geometryType`: Type of geometry
/// - `fields`: Field definitions (must include ObjectID)
///
/// # Branch Versioning Requirements
///
/// For branch-versioned layers, ESRI requires:
/// - An ObjectID field (type: `esriFieldTypeOID`)
/// - A GlobalID field (type: `esriFieldTypeGlobalID`)
///
/// Source: <https://pro.arcgis.com/en/pro-app/latest/help/data/geodatabases/overview/branch-version-scenarios.htm>
#[derive(
    Debug, Clone, Serialize, Deserialize, derive_builder::Builder, derive_getters::Getters,
)]
#[builder(setter(into, strip_option), default)]
#[serde(rename_all = "camelCase")]
pub struct LayerDefinition {
    /// Layer ID (must be unique within service).
    #[builder(setter(into))]
    id: u32,

    /// Layer name.
    #[builder(setter(into))]
    name: String,

    /// Layer type.
    ///
    /// Common value: "Feature Layer". Other options: "Table".
    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    #[builder(default)]
    layer_type: Option<String>,

    /// Geometry type for this layer.
    #[serde(rename = "geometryType")]
    geometry_type: GeometryTypeDefinition,

    /// Field definitions.
    ///
    /// Must include at least an ObjectID field.
    /// For versioning, must also include GlobalID field.
    #[builder(default)]
    fields: Vec<FieldDefinition>,

    /// Name of the ObjectID field.
    ///
    /// Default: "OBJECTID". Must match a field in `fields` array.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default)]
    object_id_field: Option<String>,

    /// Name of the GlobalID field.
    ///
    /// Required for branch versioning. Default: "GlobalID".
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default)]
    global_id_field: Option<String>,

    /// Display field name (shown in popups).
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default)]
    display_field: Option<String>,

    /// Layer description.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default)]
    description: Option<String>,

    /// Copyright text.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default)]
    copyright_text: Option<String>,

    /// Default visibility.
    ///
    /// Default: `true`.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default)]
    default_visibility: Option<bool>,

    /// Templates for feature creation.
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    #[builder(default)]
    templates: Vec<FeatureTemplate>,

    /// Whether data is branch versioned.
    ///
    /// ESRI sets this automatically when GlobalID field is present.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default)]
    is_data_branch_versioned: Option<bool>,
}

impl LayerDefinitionBuilder {
    /// Adds a field to the layer.
    pub fn add_field(mut self, field: FieldDefinition) -> Self {
        self.fields.get_or_insert_with(Vec::new).push(field);
        self
    }

    /// Adds a template to the layer.
    pub fn add_template(mut self, template: FeatureTemplate) -> Self {
        self.templates.get_or_insert_with(Vec::new).push(template);
        self
    }
}

/// Field definition within a layer.
///
/// # ESRI Documentation
///
/// Source: <https://developers.arcgis.com/rest/services-reference/enterprise/layer-feature-service/>
///
/// Required properties:
/// - `name`: Field name (must be unique within layer)
/// - `type`: Field type (see `FieldType` enum)
///
/// # Special Fields
///
/// - **ObjectID**: Required for all layers. Must be non-nullable, non-editable.
/// - **GlobalID**: Required for branch versioning. Must be non-nullable, non-editable.
/// - **Geometry**: Implicitly defined by layer's `geometryType`.
///
/// # Field Naming
///
/// ESRI recommends:
/// - Uppercase for system fields (OBJECTID, GlobalID, Shape)
/// - Mixed case for user fields (CustomerName, BuildingType)
#[derive(
    Debug, Clone, Serialize, Deserialize, derive_builder::Builder, derive_getters::Getters,
)]
#[builder(setter(into, strip_option), default)]
#[serde(rename_all = "camelCase")]
pub struct FieldDefinition {
    /// Field name (must be unique within layer).
    #[builder(setter(into))]
    name: String,

    /// Field type.
    ///
    /// Must be one of ESRI's predefined field types. See `FieldType` enum.
    #[serde(rename = "type")]
    field_type: FieldType,

    /// Field alias (display name).
    ///
    /// Shown in UI instead of field name. Useful for human-readable labels.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default)]
    alias: Option<String>,

    /// Field length (for string fields).
    ///
    /// Maximum: 2147483647. Common: 255 for names, 1024 for descriptions.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default)]
    length: Option<i32>,

    /// Whether NULL values are allowed.
    ///
    /// Default: `true`. Set to `false` for required fields.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default)]
    nullable: Option<bool>,

    /// Whether users can edit this field.
    ///
    /// Default: `true`. Set to `false` for system fields (ObjectID, GlobalID).
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default)]
    editable: Option<bool>,

    /// Whether field is required (beta feature at 11.2+).
    ///
    /// When `true`, users can add/update content but can't delete the field.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default)]
    required: Option<bool>,

    /// Default value for the field.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default)]
    default_value: Option<serde_json::Value>,

    /// Domain for constrained values.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default)]
    domain: Option<Domain>,

    /// Model name (internal ESRI use).
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default)]
    model_name: Option<String>,
}

/// Field types as defined by ESRI.
///
/// # ESRI Documentation
///
/// Source: <https://developers.arcgis.com/rest/services-reference/enterprise/layer-feature-service/>
///
/// All values are exact string literals from ESRI's specification.
///
/// # New Field Types (11.2+)
///
/// The following types were added as beta in 11.2 and became standard in 11.3:
/// - `BigInteger`: 64-bit integer
/// - `TimeOnly`: Time without date
/// - `DateOnly`: Date without time
/// - `TimestampOffset`: Timestamp with timezone
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum FieldType {
    /// Small integer (-32,768 to 32,767).
    #[serde(rename = "esriFieldTypeSmallInteger")]
    SmallInteger,

    /// Integer (-2,147,483,648 to 2,147,483,647).
    #[serde(rename = "esriFieldTypeInteger")]
    Integer,

    /// Big integer (64-bit, 11.2+).
    #[serde(rename = "esriFieldTypeBigInteger")]
    BigInteger,

    /// Single-precision floating point.
    #[serde(rename = "esriFieldTypeSingle")]
    Single,

    /// Double-precision floating point.
    #[serde(rename = "esriFieldTypeDouble")]
    Double,

    /// String (text).
    #[serde(rename = "esriFieldTypeString")]
    String,

    /// Date and time.
    #[serde(rename = "esriFieldTypeDate")]
    Date,

    /// Date only without time (11.2+).
    #[serde(rename = "esriFieldTypeDateOnly")]
    DateOnly,

    /// Time only without date (11.2+).
    #[serde(rename = "esriFieldTypeTimeOnly")]
    TimeOnly,

    /// Timestamp with timezone offset (11.2+).
    #[serde(rename = "esriFieldTypeTimestampOffset")]
    TimestampOffset,

    /// Object ID (primary key).
    ///
    /// Must be non-nullable and non-editable. Required for all layers.
    #[serde(rename = "esriFieldTypeOID")]
    Oid,

    /// Geometry field.
    #[serde(rename = "esriFieldTypeGeometry")]
    Geometry,

    /// Binary large object.
    #[serde(rename = "esriFieldTypeBlob")]
    Blob,

    /// Raster image.
    #[serde(rename = "esriFieldTypeRaster")]
    Raster,

    /// GUID (globally unique identifier).
    #[serde(rename = "esriFieldTypeGUID")]
    Guid,

    /// Global ID (required for versioning).
    ///
    /// Must be non-nullable and non-editable. Required for branch versioning.
    #[serde(rename = "esriFieldTypeGlobalID")]
    GlobalId,

    /// XML document.
    #[serde(rename = "esriFieldTypeXML")]
    Xml,
}

/// Geometry type for a layer.
///
/// # ESRI Documentation
///
/// Source: <https://developers.arcgis.com/rest/services-reference/enterprise/layer-feature-service/>
///
/// All values are exact string literals from ESRI's specification.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum GeometryTypeDefinition {
    /// Point geometry.
    #[serde(rename = "esriGeometryPoint")]
    Point,

    /// Multipoint geometry.
    #[serde(rename = "esriGeometryMultipoint")]
    Multipoint,

    /// Polyline geometry (line/path).
    #[serde(rename = "esriGeometryPolyline")]
    Polyline,

    /// Polygon geometry (area).
    #[serde(rename = "esriGeometryPolygon")]
    Polygon,

    /// Envelope (bounding box).
    #[serde(rename = "esriGeometryEnvelope")]
    Envelope,
}

/// Spatial reference definition.
///
/// # ESRI Documentation
///
/// Source: <https://developers.arcgis.com/rest/services-reference/enterprise/layer-feature-service/>
///
/// Can be specified as:
/// - WKID only: `{"wkid": 4326}`
/// - WKID with latest: `{"wkid": 4326, "latestWkid": 4326}`
/// - WKT: `{"wkt": "GEOGCS[...]"}`
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum SpatialReferenceDefinition {
    /// Well-Known ID only.
    Wkid {
        /// Well-Known ID (e.g., 4326 for WGS84)
        wkid: i32,
    },

    /// Well-Known ID with latest WKID.
    WkidWithLatest {
        /// Well-Known ID
        wkid: i32,
        /// Latest Well-Known ID
        latest_wkid: i32,
    },

    /// Well-Known Text.
    Wkt {
        /// Well-Known Text string
        wkt: String,
    },
}

/// Editor tracking configuration.
///
/// # ESRI Documentation
///
/// Automatically tracks:
/// - Who created each feature (creator field)
/// - When it was created (creation date field)
/// - Who last edited it (editor field)
/// - When it was last edited (edit date field)
///
/// These fields are added automatically by ESRI when editor tracking is enabled.
#[derive(
    Debug, Clone, Default, Serialize, Deserialize, derive_builder::Builder, derive_getters::Getters,
)]
#[builder(setter(into, strip_option), default)]
#[serde(rename_all = "camelCase")]
pub struct EditorTrackingInfo {
    /// Whether editor tracking is enabled.
    enable_editor_tracking: bool,

    /// Whether ownership-based access control is enabled.
    ///
    /// When `true`, users can only edit features they created.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default)]
    enable_ownership_access_control: Option<bool>,

    /// Whether others can query features.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default)]
    allow_others_to_query: Option<bool>,

    /// Whether others can update features.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default)]
    allow_others_to_update: Option<bool>,

    /// Whether others can delete features.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default)]
    allow_others_to_delete: Option<bool>,
}

/// XSS prevention configuration.
///
/// Security settings to prevent cross-site scripting attacks.
#[derive(Debug, Clone, Serialize, Deserialize, derive_getters::Getters)]
#[serde(rename_all = "camelCase")]
pub struct XssPreventionInfo {
    /// Whether XSS prevention is enabled.
    xss_prevention_enabled: bool,

    /// Rule for XSS prevention.
    #[serde(skip_serializing_if = "Option::is_none")]
    xss_prevention_rule: Option<String>,

    /// Input fields to check for XSS.
    #[serde(skip_serializing_if = "Option::is_none")]
    xss_input_rule: Option<String>,
}

// Placeholder types for Phase 3+ implementation
/// Table definition (non-spatial).
///
/// Similar to LayerDefinition but without geometry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableDefinition {
    // TODO: Phase 4 - Implement table structure
}

/// Domain for constrained field values.
///
/// Can be either coded value domain or range domain.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Domain {
    // TODO: Phase 3 - Implement domain types
    /// Placeholder for future implementation.
    Placeholder {},
}

/// Template for feature creation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureTemplate {
    // TODO: Phase 3 - Implement template structure
}

// Default implementations
impl Default for LayerDefinition {
    fn default() -> Self {
        Self {
            id: 0,
            name: String::new(),
            layer_type: None,
            geometry_type: GeometryTypeDefinition::Point,
            fields: Vec::new(),
            object_id_field: None,
            global_id_field: None,
            display_field: None,
            description: None,
            copyright_text: None,
            default_visibility: None,
            templates: Vec::new(),
            is_data_branch_versioned: None,
        }
    }
}

impl Default for FieldDefinition {
    fn default() -> Self {
        Self {
            name: String::new(),
            field_type: FieldType::String,
            alias: None,
            length: None,
            nullable: None,
            editable: None,
            required: None,
            default_value: None,
            domain: None,
            model_name: None,
        }
    }
}
