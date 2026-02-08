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
//! use arcgis::{
//!     ServiceDefinitionBuilder, LayerDefinitionBuilder, FieldDefinitionBuilder,
//!     GeometryTypeDefinition, FieldType,
//! };
//!
//! let field = FieldDefinitionBuilder::default()
//!     .name("OBJECTID")
//!     .field_type(FieldType::Oid)
//!     .nullable(false)
//!     .editable(false)
//!     .build()?;
//!
//! let mut layer_builder = LayerDefinitionBuilder::default();
//! layer_builder.id(0u32).name("Points").geometry_type(GeometryTypeDefinition::Point);
//! let layer = layer_builder.add_field(field).build()?;
//!
//! let mut svc_builder = ServiceDefinitionBuilder::default();
//! svc_builder.name("MyVersionedService");
//! let service_def = svc_builder.add_layer(layer).build()?;
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
    /// Service name (REQUIRED for creation).
    ///
    /// Must be unique within the user's content.
    ///
    /// Note: When deserializing from an existing service endpoint (`GET {serviceUrl}?f=json`),
    /// the name is not included in the JSON response (it is encoded in the URL path).
    /// In that case this field defaults to an empty string.
    #[serde(default)]
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
    ///
    /// Note: When deserializing from the service root endpoint (`GET {serviceUrl}?f=json`),
    /// ESRI returns only layer stubs without field definitions. Use
    /// `FeatureServiceClient::get_layer_definition()` to retrieve full field definitions.
    #[serde(default)]
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

    /// Indexes on layer fields.
    ///
    /// ESRI automatically creates indexes on ObjectID and geometry fields.
    /// Additional indexes can improve query performance on frequently-queried fields.
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    #[builder(default)]
    indexes: Vec<Index>,

    /// Editor tracking field configuration.
    ///
    /// Specifies which fields track creation and edit information.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default)]
    edit_fields_info: Option<EditFieldsInfo>,

    /// Relationship classes this layer participates in.
    ///
    /// Each entry describes a relationship with another layer or table.
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    #[builder(default)]
    relationships: Vec<LayerRelationship>,

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

    /// Adds an index to the layer.
    pub fn add_index(mut self, index: Index) -> Self {
        self.indexes.get_or_insert_with(Vec::new).push(index);
        self
    }

    /// Adds a relationship to the layer.
    pub fn add_relationship(mut self, relationship: LayerRelationship) -> Self {
        self.relationships
            .get_or_insert_with(Vec::new)
            .push(relationship);
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
    Debug,
    Clone,
    PartialEq,
    Serialize,
    Deserialize,
    derive_builder::Builder,
    derive_getters::Getters,
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

// ==================== Phase 4: Table Definition ====================

/// Table definition within a Feature Service.
///
/// Tables are non-spatial layers — they have fields and relationships
/// but no geometry. Common uses include related records, lookup tables,
/// and attachments tables.
///
/// # ESRI Documentation
///
/// Source: <https://developers.arcgis.com/rest/services-reference/enterprise/layer-feature-service/>
///
/// Tables share most properties with layers, but do not have:
/// - `geometryType` (no spatial data)
/// - `defaultVisibility` (not rendered on maps)
///
/// The `type` field is always `"Table"` for table definitions.
///
/// # Example
///
/// ```rust
/// use arcgis::{TableDefinitionBuilder, FieldDefinitionBuilder, FieldType};
///
/// let field = FieldDefinitionBuilder::default()
///     .name("OBJECTID")
///     .field_type(FieldType::Oid)
///     .nullable(false)
///     .editable(false)
///     .build()
///     .expect("Valid field");
///
/// let table = TableDefinitionBuilder::default()
///     .id(1u32)
///     .name("Permits")
///     .build()
///     .expect("Valid table");
///
/// assert_eq!(table.id(), &1u32);
/// assert_eq!(table.name(), "Permits");
/// ```
#[derive(
    Debug, Clone, Default, Serialize, Deserialize, derive_builder::Builder, derive_getters::Getters,
)]
#[builder(setter(into, strip_option), default)]
#[serde(rename_all = "camelCase")]
pub struct TableDefinition {
    /// Table ID (must be unique within service).
    #[builder(setter(into))]
    id: u32,

    /// Table name.
    #[builder(setter(into))]
    name: String,

    /// Table type.
    ///
    /// Always `"Table"` for table definitions.
    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    #[builder(default)]
    table_type: Option<String>,

    /// Field definitions.
    ///
    /// Must include at least an ObjectID field.
    /// For versioning, must also include GlobalID field.
    #[serde(default)]
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

    /// Table description.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default)]
    description: Option<String>,

    /// Copyright text.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default)]
    copyright_text: Option<String>,

    /// Templates for record creation.
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    #[builder(default)]
    templates: Vec<FeatureTemplate>,

    /// Indexes on table fields.
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    #[builder(default)]
    indexes: Vec<Index>,

    /// Editor tracking field configuration.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default)]
    edit_fields_info: Option<EditFieldsInfo>,

    /// Relationship classes this table participates in.
    ///
    /// Each entry describes a relationship with another layer or table.
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    #[builder(default)]
    relationships: Vec<LayerRelationship>,

    /// Whether data is branch versioned.
    ///
    /// ESRI sets this automatically when GlobalID field is present.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default)]
    is_data_branch_versioned: Option<bool>,
}

impl TableDefinitionBuilder {
    /// Adds a field to the table.
    pub fn add_field(mut self, field: FieldDefinition) -> Self {
        self.fields.get_or_insert_with(Vec::new).push(field);
        self
    }

    /// Adds a template to the table.
    pub fn add_template(mut self, template: FeatureTemplate) -> Self {
        self.templates.get_or_insert_with(Vec::new).push(template);
        self
    }

    /// Adds an index to the table.
    pub fn add_index(mut self, index: Index) -> Self {
        self.indexes.get_or_insert_with(Vec::new).push(index);
        self
    }

    /// Adds a relationship to the table.
    pub fn add_relationship(mut self, relationship: LayerRelationship) -> Self {
        self.relationships
            .get_or_insert_with(Vec::new)
            .push(relationship);
        self
    }
}

// ==================== Phase 3: Advanced Layer Features ====================

/// Domain for a field (coded values, numeric range, or inherited from subtype).
///
/// # ESRI Documentation
///
/// Source: <https://developers.arcgis.com/rest/services-reference/enterprise/layer-feature-service/>
///
/// Domains constrain the allowed values for a field. ESRI supports three domain types:
/// - `CodedValue`: Discrete list of allowed values (like enum)
/// - `Range`: Numeric min/max boundaries
/// - `Inherited`: Use domain from subtype definition
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum Domain {
    /// Coded value domain with discrete allowed values.
    ///
    /// Example: Status field with values ["Active", "Inactive", "Pending"]
    #[serde(rename = "codedValue")]
    CodedValue(CodedValueDomain),

    /// Range domain with numeric min/max boundaries.
    ///
    /// Example: Temperature field with range [-40, 120]
    #[serde(rename = "range")]
    Range(RangeDomain),

    /// Inherited domain from subtype.
    ///
    /// Used in subtype definitions to inherit the parent field's domain.
    #[serde(rename = "inherited")]
    Inherited,
}

/// Coded value domain with discrete allowed values.
///
/// # ESRI Documentation
///
/// Maps field values to human-readable names. Commonly used for:
/// - Status fields (Active/Inactive)
/// - Category fields (Residential/Commercial/Industrial)
/// - Priority fields (Low/Medium/High)
///
/// # Example from ESRI
///
/// ```json
/// {
///   "type": "codedValue",
///   "name": "Priority",
///   "codedValues": [
///     {"name": "Low", "code": 1},
///     {"name": "Medium", "code": 2},
///     {"name": "High", "code": 3}
///   ],
///   "mergePolicy": "esriMPTDefaultValue",
///   "splitPolicy": "esriSPTDuplicate"
/// }
/// ```
#[derive(
    Debug,
    Clone,
    Default,
    PartialEq,
    Serialize,
    Deserialize,
    derive_getters::Getters,
    derive_builder::Builder,
)]
#[builder(setter(into, strip_option), default)]
#[serde(rename_all = "camelCase")]
pub struct CodedValueDomain {
    /// Domain name.
    #[builder(setter(into))]
    name: String,

    /// Domain description (optional).
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default)]
    description: Option<String>,

    /// List of coded values (code + name pairs).
    #[builder(default)]
    coded_values: Vec<CodedValue>,

    /// Merge policy (how values combine when features merge).
    ///
    /// Default: `esriMPTDefaultValue` (use default value on merge).
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default)]
    merge_policy: Option<MergePolicy>,

    /// Split policy (how values divide when features split).
    ///
    /// Default: `esriSPTDuplicate` (duplicate value to both features).
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default)]
    split_policy: Option<SplitPolicy>,
}

impl CodedValueDomainBuilder {
    /// Adds a coded value to the domain.
    pub fn add_coded_value(mut self, coded_value: CodedValue) -> Self {
        self.coded_values
            .get_or_insert_with(Vec::new)
            .push(coded_value);
        self
    }
}

/// A single coded value (code + name pair).
///
/// Maps a code value (stored in database) to a human-readable name (shown in UI).
///
/// # Examples
///
/// ```rust
/// use arcgis::{DomainCodedValue, CodedValueCode};
///
/// // Numeric code
/// let status = DomainCodedValue::new("Active".to_string(), CodedValueCode::Number(1.0));
///
/// // String code
/// let reliability = DomainCodedValue::new("Completely Reliable".to_string(), CodedValueCode::String("A".to_string()));
/// ```
#[derive(
    Debug, Clone, PartialEq, Serialize, Deserialize, derive_getters::Getters, derive_new::new,
)]
pub struct CodedValue {
    /// Display name (shown to users).
    name: String,

    /// Code value (stored in database).
    ///
    /// Can be either a number or string depending on field type.
    code: CodedValueCode,
}

/// Code value (string or number).
///
/// # ESRI Specification
///
/// ESRI allows codes to be either strings or numbers:
/// - Integer fields use numeric codes: `{"name": "Active", "code": 1}`
/// - String fields use string codes: `{"name": "Grade A", "code": "A"}`
///
/// This enum provides type-safe representation of both cases.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum CodedValueCode {
    /// String code value.
    String(String),

    /// Numeric code value (JSON numbers deserialize as f64).
    Number(f64),
}

/// Range domain with numeric boundaries.
///
/// # ESRI Documentation
///
/// Constrains field values to a numeric range [min, max]. Commonly used for:
/// - Measurements (temperature, pressure, elevation)
/// - Percentages (0-100)
/// - Angles (-180 to 180, 0 to 360)
///
/// # Example from ESRI
///
/// ```json
/// {
///   "type": "range",
///   "name": "Direction",
///   "description": "Direction of Movement",
///   "range": [-1, 360],
///   "mergePolicy": "esriMPTDefaultValue",
///   "splitPolicy": "esriSPTDuplicate"
/// }
/// ```
#[derive(
    Debug,
    Clone,
    PartialEq,
    Serialize,
    Deserialize,
    derive_getters::Getters,
    derive_builder::Builder,
)]
#[builder(setter(into, strip_option), default)]
#[serde(rename_all = "camelCase")]
pub struct RangeDomain {
    /// Domain name.
    #[builder(setter(into))]
    name: String,

    /// Domain description (optional).
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default)]
    description: Option<String>,

    /// Range as [min, max].
    range: [f64; 2],

    /// Merge policy.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default)]
    merge_policy: Option<MergePolicy>,

    /// Split policy.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default)]
    split_policy: Option<SplitPolicy>,
}

impl Default for RangeDomain {
    fn default() -> Self {
        Self {
            name: String::new(),
            description: None,
            range: [0.0, 0.0],
            merge_policy: None,
            split_policy: None,
        }
    }
}

/// Merge policy for domain values.
///
/// # ESRI Documentation
///
/// Defines how attribute values are computed when features are merged:
/// - `DefaultValue`: Use the field's default value
/// - `SumValues`: Sum the values from merged features
/// - `AreaWeighted`: Weight values by feature area
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MergePolicy {
    /// Use field's default value.
    #[serde(rename = "esriMPTDefaultValue")]
    DefaultValue,

    /// Sum values from merged features.
    #[serde(rename = "esriMPTSumValues")]
    SumValues,

    /// Weight values by feature area.
    #[serde(rename = "esriMPTAreaWeighted")]
    AreaWeighted,
}

/// Cardinality of a relationship class.
///
/// # ESRI Documentation
///
/// Source: <https://developers.arcgis.com/rest/services-reference/enterprise/layer-feature-service/>
///
/// Defines how many features on each side of a relationship can be related:
/// - `OneToOne`: Each origin feature relates to at most one destination feature
/// - `OneToMany`: Each origin feature can relate to many destination features
/// - `ManyToMany`: Many origin features can relate to many destination features
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RelationshipCardinality {
    /// One origin feature to at most one destination feature.
    #[serde(rename = "esriRelCardinalityOneToOne")]
    OneToOne,

    /// One origin feature to many destination features.
    #[serde(rename = "esriRelCardinalityOneToMany")]
    OneToMany,

    /// Many origin features to many destination features.
    #[serde(rename = "esriRelCardinalityManyToMany")]
    ManyToMany,
}

/// Role of a layer in a relationship class.
///
/// # ESRI Documentation
///
/// Source: <https://developers.arcgis.com/rest/services-reference/enterprise/layer-feature-service/>
///
/// - `Origin`: The layer holds the primary key of the relationship
/// - `Destination`: The layer holds the foreign key of the relationship
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RelationshipRole {
    /// Layer holds the primary key (origin side).
    #[serde(rename = "esriRelRoleOrigin")]
    Origin,

    /// Layer holds the foreign key (destination side).
    #[serde(rename = "esriRelRoleDestination")]
    Destination,
}

/// Relationship to another layer or table in the service.
///
/// # ESRI Documentation
///
/// Source: <https://developers.arcgis.com/rest/services-reference/enterprise/layer-feature-service/>
///
/// Each layer or table can participate in one or more relationship classes.
/// The `relationships` array in a layer definition lists the relationships
/// that this layer participates in.
///
/// # Example from ESRI
///
/// ```json
/// {
///   "id": 2,
///   "role": "esriRelRoleOrigin",
///   "keyField": "GlobalID",
///   "cardinality": "esriRelCardinalityOneToMany",
///   "relatedTableId": 3,
///   "name": "Buildings_Permits"
/// }
/// ```
#[derive(
    Debug,
    Clone,
    PartialEq,
    Eq,
    Serialize,
    Deserialize,
    derive_getters::Getters,
    derive_builder::Builder,
)]
#[builder(setter(into, strip_option), default)]
#[serde(rename_all = "camelCase")]
pub struct LayerRelationship {
    /// Relationship class ID (unique within service).
    id: i32,

    /// Relationship class name.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default)]
    name: Option<String>,

    /// Role of this layer in the relationship.
    role: RelationshipRole,

    /// Cardinality of the relationship.
    cardinality: RelationshipCardinality,

    /// ID of the related layer or table.
    related_table_id: i32,

    /// Key field name on this layer (foreign or primary key).
    key_field: String,

    /// Composite key field name (for composite key relationships).
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default)]
    composite_key_field_name: Option<String>,
}

impl Default for LayerRelationship {
    fn default() -> Self {
        Self {
            id: 0,
            name: None,
            role: RelationshipRole::Origin,
            cardinality: RelationshipCardinality::OneToMany,
            related_table_id: 0,
            key_field: String::new(),
            composite_key_field_name: None,
        }
    }
}

/// Split policy for domain values.
///
/// # ESRI Documentation
///
/// Defines how attribute values are computed when features are split:
/// - `DefaultValue`: Use the field's default value for both parts
/// - `Duplicate`: Duplicate the value to both parts
/// - `GeometryRatio`: Ratio values based on geometry proportions
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SplitPolicy {
    /// Use field's default value for both parts.
    #[serde(rename = "esriSPTDefaultValue")]
    DefaultValue,

    /// Duplicate value to both parts.
    #[serde(rename = "esriSPTDuplicate")]
    Duplicate,

    /// Ratio values based on geometry proportions.
    #[serde(rename = "esriSPTGeometryRatio")]
    GeometryRatio,
}

/// Feature template for creating new features.
///
/// # ESRI Documentation
///
/// Source: <https://developers.arcgis.com/rest/services-reference/enterprise/layer-feature-service/>
///
/// Templates provide preset attribute values and drawing tools for feature creation.
/// Commonly used in:
/// - Categorized feature types (Residential/Commercial buildings)
/// - Workflow states (New/In Progress/Complete tasks)
/// - Standard feature configurations
///
/// # Example from ESRI
///
/// ```json
/// {
///   "name": "Residential Building",
///   "description": "Single-family residential structure",
///   "prototype": {
///     "attributes": {
///       "BuildingType": "Residential",
///       "Status": "Planned"
///     }
///   },
///   "drawingTool": "esriFeatureEditToolPolygon"
/// }
/// ```
#[derive(
    Debug,
    Clone,
    PartialEq,
    Serialize,
    Deserialize,
    derive_getters::Getters,
    derive_builder::Builder,
)]
#[builder(setter(into, strip_option), default)]
#[serde(rename_all = "camelCase")]
pub struct FeatureTemplate {
    /// Template name (required).
    #[builder(setter(into))]
    name: String,

    /// Template description.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default)]
    description: Option<String>,

    /// Prototype feature with default attribute values.
    prototype: TemplatePrototype,

    /// Drawing tool for this template.
    drawing_tool: DrawingTool,
}

impl Default for FeatureTemplate {
    fn default() -> Self {
        Self {
            name: String::new(),
            description: None,
            prototype: TemplatePrototype::default(),
            drawing_tool: DrawingTool::None,
        }
    }
}

/// Prototype feature with default attributes.
///
/// Defines default attribute values for features created from a template.
#[derive(
    Debug,
    Clone,
    Default,
    PartialEq,
    Serialize,
    Deserialize,
    derive_getters::Getters,
    derive_builder::Builder,
)]
#[builder(setter(into, strip_option), default)]
pub struct TemplatePrototype {
    /// Default attribute values (field name → value).
    ///
    /// Example: `{"BuildingType": "Residential", "Status": "Planned"}`
    #[builder(default)]
    attributes: std::collections::HashMap<String, serde_json::Value>,
}

/// Drawing tool for feature templates.
///
/// # ESRI Documentation
///
/// Specifies which editing tool to use when creating features from this template.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DrawingTool {
    /// No drawing tool.
    #[serde(rename = "esriFeatureEditToolNone")]
    None,

    /// Point drawing tool.
    #[serde(rename = "esriFeatureEditToolPoint")]
    Point,

    /// Line drawing tool.
    #[serde(rename = "esriFeatureEditToolLine")]
    Line,

    /// Polygon drawing tool.
    #[serde(rename = "esriFeatureEditToolPolygon")]
    Polygon,

    /// Auto-complete polygon tool.
    #[serde(rename = "esriFeatureEditToolAutoCompletePolygon")]
    AutoCompletePolygon,

    /// Circle drawing tool.
    #[serde(rename = "esriFeatureEditToolCircle")]
    Circle,

    /// Ellipse drawing tool.
    #[serde(rename = "esriFeatureEditToolEllipse")]
    Ellipse,

    /// Rectangle drawing tool.
    #[serde(rename = "esriFeatureEditToolRectangle")]
    Rectangle,

    /// Freehand drawing tool.
    #[serde(rename = "esriFeatureEditToolFreehand")]
    Freehand,
}

/// Index on layer fields.
///
/// # ESRI Documentation
///
/// Source: <https://developers.arcgis.com/rest/services-reference/enterprise/layer-feature-service/>
///
/// Indexes improve query performance on frequently-queried fields.
/// ESRI automatically creates indexes on ObjectID and geometry fields.
///
/// # Example from ESRI
///
/// ```json
/// {
///   "name": "shape_index",
///   "fields": "shape",
///   "isAscending": true,
///   "isUnique": true,
///   "description": "Spatial index"
/// }
/// ```
#[derive(
    Debug,
    Clone,
    Default,
    PartialEq,
    Eq,
    Serialize,
    Deserialize,
    derive_getters::Getters,
    derive_builder::Builder,
)]
#[builder(setter(into, strip_option), default)]
#[serde(rename_all = "camelCase")]
pub struct Index {
    /// Index name (required).
    #[builder(setter(into))]
    name: String,

    /// Field names in the index.
    ///
    /// ESRI serializes as comma-separated string ("field1,field2,field3")
    /// but we model as Vec<String> for better ergonomics.
    #[serde(
        serialize_with = "serialize_index_fields",
        deserialize_with = "deserialize_index_fields"
    )]
    fields: Vec<String>,

    /// Whether index is ascending.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default)]
    is_ascending: Option<bool>,

    /// Whether index enforces uniqueness.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default)]
    is_unique: Option<bool>,

    /// Index description.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default)]
    description: Option<String>,
}

/// Custom serialization for index fields (Vec<String> → "field1,field2,field3").
fn serialize_index_fields<S>(fields: &[String], serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    serializer.serialize_str(&fields.join(","))
}

/// Custom deserialization for index fields ("field1,field2,field3" → Vec<String>).
fn deserialize_index_fields<'de, D>(deserializer: D) -> Result<Vec<String>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    Ok(s.split(',').map(|s| s.trim().to_string()).collect())
}

/// Editor tracking field configuration.
///
/// # ESRI Documentation
///
/// Source: <https://developers.arcgis.com/rest/services-reference/enterprise/layer-feature-service/>
///
/// Configures which fields track creation and edit information.
/// ESRI automatically populates these fields with:
/// - Username of creator/editor
/// - Timestamp of creation/edit
///
/// # Example from ESRI
///
/// ```json
/// {
///   "creationDateField": "created_date",
///   "creatorField": "created_user",
///   "editDateField": "last_edited_date",
///   "editorField": "last_edited_user"
/// }
/// ```
#[derive(
    Debug,
    Clone,
    Default,
    PartialEq,
    Eq,
    Serialize,
    Deserialize,
    derive_getters::Getters,
    derive_builder::Builder,
)]
#[builder(setter(into, strip_option), default)]
#[serde(rename_all = "camelCase")]
pub struct EditFieldsInfo {
    /// Field storing creation date.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default)]
    creation_date_field: Option<String>,

    /// Field storing creator username.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default)]
    creator_field: Option<String>,

    /// Field storing last edit date.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default)]
    edit_date_field: Option<String>,

    /// Field storing last editor username.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default)]
    editor_field: Option<String>,
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
            indexes: Vec::new(),
            edit_fields_info: None,
            relationships: Vec::new(),
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

// ==================== Phase 5: Validation ====================

/// Validation error for service definition constraints.
///
/// Each variant carries enough context — entity type, name, ID, and field names —
/// for an agent or user to locate and fix the issue without re-reading the
/// entire service definition.
///
/// Validation rules enforce ESRI's requirements for Feature Services:
/// - ObjectID field is required and must be non-nullable and non-editable
/// - GlobalID field is required when branch versioning is enabled
/// - Field names must be unique within a layer or table
/// - Layer and table IDs must be unique within the service
/// - Named field references (object_id_field, global_id_field, display_field)
///   must point to existing fields
///
/// # Example
///
/// ```rust
/// use arcgis::{LayerDefinitionBuilder, FieldDefinitionBuilder, FieldType, GeometryTypeDefinition};
///
/// let layer = LayerDefinitionBuilder::default()
///     .id(0u32)
///     .name("Buildings")
///     .geometry_type(GeometryTypeDefinition::Polygon)
///     .build()
///     .expect("Valid layer");
///
/// // Layer has no fields yet — validation will catch the missing ObjectID
/// let errors = layer.validate();
/// assert!(errors.is_err());
/// let errs = errors.unwrap_err();
/// assert_eq!(errs.len(), 1);
/// println!("{}", errs[0]);
/// ```
#[derive(Debug, Clone, PartialEq, Eq, derive_more::Display, derive_more::Error)]
pub enum ServiceDefinitionValidationError {
    /// No ObjectID field found in the fields array.
    ///
    /// # Fix
    ///
    /// Add a field with `FieldType::Oid`, `nullable: false`, `editable: false`.
    ///
    /// See <https://developers.arcgis.com/rest/services-reference/enterprise/layer-feature-service/>
    #[display(
        "{entity_type} '{}' (id={}): no ObjectID field found. \
         Add a field with FieldType::Oid, nullable: false, editable: false. \
         See https://developers.arcgis.com/rest/services-reference/enterprise/layer-feature-service/",
        name,
        id
    )]
    MissingObjectId {
        /// Whether this is a "Layer" or "Table".
        entity_type: &'static str,
        /// Name of the layer or table.
        name: String,
        /// ID of the layer or table.
        id: u32,
    },

    /// More than one ObjectID field found; only one is allowed.
    #[display(
        "{entity_type} '{}' (id={}): {} ObjectID fields found; only one is allowed.",
        name,
        id,
        count
    )]
    MultipleObjectIds {
        /// Whether this is a "Layer" or "Table".
        entity_type: &'static str,
        /// Name of the layer or table.
        name: String,
        /// ID of the layer or table.
        id: u32,
        /// Number of ObjectID fields found.
        count: usize,
    },

    /// ObjectID field is nullable or editable, which ESRI prohibits.
    ///
    /// # Fix
    ///
    /// Set `nullable: false` and `editable: false` on the ObjectID field.
    #[display(
        "{entity_type} '{}' (id={}), field '{}': \
         ObjectID field must have nullable=false and editable=false.",
        name,
        id,
        field_name
    )]
    OidFieldInvalidConfig {
        /// Whether this is a "Layer" or "Table".
        entity_type: &'static str,
        /// Name of the layer or table.
        name: String,
        /// ID of the layer or table.
        id: u32,
        /// Name of the misconfigured ObjectID field.
        field_name: String,
    },

    /// Branch versioning is enabled but no GlobalID field exists.
    ///
    /// # Fix
    ///
    /// Add a field with `FieldType::GlobalId`, `nullable: false`, `editable: false`,
    /// `length: 38`.
    ///
    /// See <https://pro.arcgis.com/en/pro-app/latest/help/data/geodatabases/overview/branch-version-scenarios.htm>
    #[display(
        "{entity_type} '{}' (id={}): is_data_branch_versioned is true but no GlobalID field found. \
         Add a field with FieldType::GlobalId, nullable: false, editable: false, length: 38. \
         See https://pro.arcgis.com/en/pro-app/latest/help/data/geodatabases/overview/branch-version-scenarios.htm",
        name,
        id
    )]
    MissingGlobalIdForVersioning {
        /// Whether this is a "Layer" or "Table".
        entity_type: &'static str,
        /// Name of the layer or table.
        name: String,
        /// ID of the layer or table.
        id: u32,
    },

    /// GlobalID field is nullable or editable, which ESRI prohibits.
    ///
    /// # Fix
    ///
    /// Set `nullable: false` and `editable: false` on the GlobalID field.
    #[display(
        "{entity_type} '{}' (id={}), field '{}': \
         GlobalID field must have nullable=false and editable=false.",
        name,
        id,
        field_name
    )]
    GlobalIdFieldInvalidConfig {
        /// Whether this is a "Layer" or "Table".
        entity_type: &'static str,
        /// Name of the layer or table.
        name: String,
        /// ID of the layer or table.
        id: u32,
        /// Name of the misconfigured GlobalID field.
        field_name: String,
    },

    /// Two or more fields share the same name (case-insensitive).
    ///
    /// # Fix
    ///
    /// Rename one of the fields so all names are unique.
    #[display(
        "{entity_type} '{}' (id={}): duplicate field name '{}'. \
         Field names must be unique within a layer or table.",
        name,
        id,
        field_name
    )]
    DuplicateFieldName {
        /// Whether this is a "Layer" or "Table".
        entity_type: &'static str,
        /// Name of the layer or table.
        name: String,
        /// ID of the layer or table.
        id: u32,
        /// The duplicated field name.
        field_name: String,
    },

    /// A named field reference points to a field that does not exist.
    ///
    /// This applies to `object_id_field`, `global_id_field`, and `display_field`.
    ///
    /// # Fix
    ///
    /// Either add a field with the referenced name or correct the reference.
    #[display(
        "{entity_type} '{}' (id={}): {ref_type}_field references '{}' \
         but no field with that name exists.",
        name,
        id,
        field_name
    )]
    FieldRefNotFound {
        /// Whether this is a "Layer" or "Table".
        entity_type: &'static str,
        /// Name of the layer or table.
        name: String,
        /// ID of the layer or table.
        id: u32,
        /// Which reference field is broken: "object_id", "global_id", or "display".
        ref_type: &'static str,
        /// The referenced field name that was not found.
        field_name: String,
    },

    /// Two layers, two tables, or a layer and a table share the same ID.
    ///
    /// # Fix
    ///
    /// Assign unique IDs to each layer and table within the service.
    #[display(
        "Duplicate ID {} used by '{}' and '{}'. \
         Layer and table IDs must be unique within the service.",
        id,
        first_name,
        second_name
    )]
    DuplicateId {
        /// The duplicated ID value.
        id: u32,
        /// Name of the first entity with this ID.
        first_name: String,
        /// Name of the second entity with this ID.
        second_name: String,
    },
}

/// Context passed to the shared field-validation helper.
struct FieldValidationCtx<'a> {
    entity_type: &'static str,
    name: &'a str,
    id: u32,
    fields: &'a [FieldDefinition],
    object_id_field: &'a Option<String>,
    global_id_field: &'a Option<String>,
    display_field: &'a Option<String>,
    is_data_branch_versioned: &'a Option<bool>,
}

/// Validate fields common to both layers and tables.
fn validate_fields(
    ctx: &FieldValidationCtx<'_>,
    errors: &mut Vec<ServiceDefinitionValidationError>,
) {
    let entity_type = ctx.entity_type;
    let name = ctx.name;
    let id = ctx.id;
    let fields = ctx.fields;
    let object_id_field = ctx.object_id_field;
    let global_id_field = ctx.global_id_field;
    let display_field = ctx.display_field;
    let is_data_branch_versioned = ctx.is_data_branch_versioned;
    // Duplicate field names (case-insensitive)
    let mut seen = std::collections::HashSet::new();
    for field in fields {
        let lower = field.name.to_lowercase();
        if !seen.insert(lower) {
            errors.push(ServiceDefinitionValidationError::DuplicateFieldName {
                entity_type,
                name: name.to_string(),
                id,
                field_name: field.name.clone(),
            });
        }
    }

    // ObjectID field validation
    let oid_fields: Vec<&FieldDefinition> = fields
        .iter()
        .filter(|f| f.field_type == FieldType::Oid)
        .collect();

    match oid_fields.len() {
        0 => errors.push(ServiceDefinitionValidationError::MissingObjectId {
            entity_type,
            name: name.to_string(),
            id,
        }),
        1 => {
            let oid = oid_fields[0];
            if oid.nullable == Some(true) || oid.editable == Some(true) {
                errors.push(ServiceDefinitionValidationError::OidFieldInvalidConfig {
                    entity_type,
                    name: name.to_string(),
                    id,
                    field_name: oid.name.clone(),
                });
            }
        }
        count => errors.push(ServiceDefinitionValidationError::MultipleObjectIds {
            entity_type,
            name: name.to_string(),
            id,
            count,
        }),
    }

    // GlobalID validation for branch versioning
    if is_data_branch_versioned == &Some(true) {
        let gid_fields: Vec<&FieldDefinition> = fields
            .iter()
            .filter(|f| f.field_type == FieldType::GlobalId)
            .collect();

        if gid_fields.is_empty() {
            errors.push(
                ServiceDefinitionValidationError::MissingGlobalIdForVersioning {
                    entity_type,
                    name: name.to_string(),
                    id,
                },
            );
        } else {
            for gid in &gid_fields {
                if gid.nullable == Some(true) || gid.editable == Some(true) {
                    errors.push(
                        ServiceDefinitionValidationError::GlobalIdFieldInvalidConfig {
                            entity_type,
                            name: name.to_string(),
                            id,
                            field_name: gid.name.clone(),
                        },
                    );
                }
            }
        }
    }

    // Named field references
    let field_names: std::collections::HashSet<String> =
        fields.iter().map(|f| f.name.to_lowercase()).collect();

    if let Some(oid_ref) = object_id_field {
        if !field_names.contains(&oid_ref.to_lowercase()) {
            errors.push(ServiceDefinitionValidationError::FieldRefNotFound {
                entity_type,
                name: name.to_string(),
                id,
                ref_type: "object_id",
                field_name: oid_ref.clone(),
            });
        }
    }

    if let Some(gid_ref) = global_id_field {
        if !field_names.contains(&gid_ref.to_lowercase()) {
            errors.push(ServiceDefinitionValidationError::FieldRefNotFound {
                entity_type,
                name: name.to_string(),
                id,
                ref_type: "global_id",
                field_name: gid_ref.clone(),
            });
        }
    }

    if let Some(disp_ref) = display_field {
        if !field_names.contains(&disp_ref.to_lowercase()) {
            errors.push(ServiceDefinitionValidationError::FieldRefNotFound {
                entity_type,
                name: name.to_string(),
                id,
                ref_type: "display",
                field_name: disp_ref.clone(),
            });
        }
    }
}

impl LayerDefinition {
    /// Validates the layer definition against ESRI's requirements.
    ///
    /// Returns all validation errors found. An empty `Ok(())` means the
    /// definition is valid and safe to submit to the ESRI API.
    ///
    /// # Validation Rules
    ///
    /// - At least one `FieldType::Oid` field must be present
    /// - Exactly one ObjectID field (not multiple)
    /// - ObjectID field must have `nullable: false` and `editable: false`
    /// - If `is_data_branch_versioned` is `true`, a `FieldType::GlobalId` field is required
    /// - GlobalID field must have `nullable: false` and `editable: false`
    /// - No duplicate field names (case-insensitive comparison)
    /// - `object_id_field`, `global_id_field`, and `display_field` references must resolve
    ///
    /// # Example
    ///
    /// ```rust
    /// use arcgis::{LayerDefinitionBuilder, FieldDefinitionBuilder, FieldType, GeometryTypeDefinition};
    ///
    /// let field = FieldDefinitionBuilder::default()
    ///     .name("OBJECTID")
    ///     .field_type(FieldType::Oid)
    ///     .nullable(false)
    ///     .editable(false)
    ///     .build()
    ///     .expect("Valid field");
    ///
    /// let layer = LayerDefinitionBuilder::default()
    ///     .id(0u32)
    ///     .name("Buildings")
    ///     .geometry_type(GeometryTypeDefinition::Polygon)
    ///     .build()
    ///     .expect("Valid layer");
    ///
    /// // Empty fields — will fail validation
    /// assert!(layer.validate().is_err());
    /// ```
    #[tracing::instrument(skip(self), fields(name = %self.name, id = self.id))]
    pub fn validate(&self) -> Result<(), Vec<ServiceDefinitionValidationError>> {
        let mut errors = Vec::new();
        validate_fields(
            &FieldValidationCtx {
                entity_type: "Layer",
                name: &self.name,
                id: self.id,
                fields: &self.fields,
                object_id_field: &self.object_id_field,
                global_id_field: &self.global_id_field,
                display_field: &self.display_field,
                is_data_branch_versioned: &self.is_data_branch_versioned,
            },
            &mut errors,
        );
        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

impl TableDefinition {
    /// Validates the table definition against ESRI's requirements.
    ///
    /// Returns all validation errors found. An empty `Ok(())` means the
    /// definition is valid and safe to submit to the ESRI API.
    ///
    /// Applies the same field-level rules as [`LayerDefinition::validate()`]
    /// since tables share the same field constraints.
    ///
    /// # Example
    ///
    /// ```rust
    /// use arcgis::{TableDefinitionBuilder, FieldDefinitionBuilder, FieldType};
    ///
    /// let table = TableDefinitionBuilder::default()
    ///     .id(1u32)
    ///     .name("Permits")
    ///     .build()
    ///     .expect("Valid table");
    ///
    /// // Empty fields — will fail validation
    /// assert!(table.validate().is_err());
    /// ```
    #[tracing::instrument(skip(self), fields(name = %self.name, id = self.id))]
    pub fn validate(&self) -> Result<(), Vec<ServiceDefinitionValidationError>> {
        let mut errors = Vec::new();
        validate_fields(
            &FieldValidationCtx {
                entity_type: "Table",
                name: &self.name,
                id: self.id,
                fields: &self.fields,
                object_id_field: &self.object_id_field,
                global_id_field: &self.global_id_field,
                display_field: &self.display_field,
                is_data_branch_versioned: &self.is_data_branch_versioned,
            },
            &mut errors,
        );
        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

impl ServiceDefinition {
    /// Validates the entire service definition against ESRI's requirements.
    ///
    /// Validates all layers and tables, then checks service-level constraints.
    /// Returns all validation errors found across the entire definition.
    ///
    /// # Validation Rules
    ///
    /// - All layers pass [`LayerDefinition::validate()`]
    /// - All tables pass [`TableDefinition::validate()`]
    /// - No duplicate IDs across all layers and tables
    ///
    /// # Example
    ///
    /// ```rust
    /// use arcgis::{
    ///     ServiceDefinitionBuilder, LayerDefinitionBuilder, FieldDefinitionBuilder,
    ///     FieldType, GeometryTypeDefinition,
    /// };
    ///
    /// let oid = FieldDefinitionBuilder::default()
    ///     .name("OBJECTID")
    ///     .field_type(FieldType::Oid)
    ///     .nullable(false)
    ///     .editable(false)
    ///     .build()
    ///     .expect("Valid OID field");
    ///
    /// let mut layer_builder = LayerDefinitionBuilder::default();
    /// layer_builder
    ///     .id(0u32)
    ///     .name("Points")
    ///     .geometry_type(GeometryTypeDefinition::Point);
    /// let layer = layer_builder.add_field(oid).build().expect("Valid layer");
    ///
    /// let mut svc_builder = ServiceDefinitionBuilder::default();
    /// svc_builder.name("MyService");
    /// let svc = svc_builder.add_layer(layer).build().expect("Valid service");
    ///
    /// assert!(svc.validate().is_ok());
    /// ```
    #[tracing::instrument(
        skip(self),
        fields(
            name = %self.name,
            layers = self.layers.len(),
            tables = self.tables.len()
        )
    )]
    pub fn validate(&self) -> Result<(), Vec<ServiceDefinitionValidationError>> {
        let mut errors = Vec::new();

        // Validate each layer
        for layer in &self.layers {
            if let Err(layer_errors) = layer.validate() {
                errors.extend(layer_errors);
            }
        }

        // Validate each table
        for table in &self.tables {
            if let Err(table_errors) = table.validate() {
                errors.extend(table_errors);
            }
        }

        // Check for duplicate IDs across layers and tables
        let mut id_registry: std::collections::HashMap<u32, String> =
            std::collections::HashMap::new();

        for layer in &self.layers {
            if let Some(existing) = id_registry.insert(layer.id, layer.name.clone()) {
                errors.push(ServiceDefinitionValidationError::DuplicateId {
                    id: layer.id,
                    first_name: existing,
                    second_name: layer.name.clone(),
                });
            }
        }

        for table in &self.tables {
            if let Some(existing) = id_registry.insert(table.id, table.name.clone()) {
                errors.push(ServiceDefinitionValidationError::DuplicateId {
                    id: table.id,
                    first_name: existing,
                    second_name: table.name.clone(),
                });
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}
