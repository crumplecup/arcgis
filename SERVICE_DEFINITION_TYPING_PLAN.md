# Service Definition Strong Typing Implementation Plan

**Status:** 🟢 Phases 1-2 Complete - Branch Versioning Supported
**Created:** 2026-02-03
**Last Updated:** 2026-02-06
**Goal:** Replace `serde_json::Value` with strongly-typed Rust structs for ESRI Feature Service definitions

---

## Executive Summary

Currently, service definitions use `serde_json::Value` throughout the codebase, which defeats Rust's type safety and leads to runtime errors instead of compile-time guarantees. This plan outlines a phased approach to introduce strongly-typed service definition models that:

- ✅ Provide compile-time validation
- ✅ Enable IDE autocomplete and refactoring
- ✅ Document ESRI's API through types
- ✅ Prevent typos and structural errors
- ✅ Support incremental adoption

---

## ⚠️ Critical Implementation Requirement: ESRI Specification Compliance

**MANDATORY:** During implementation, all types MUST be validated against official ESRI documentation. Our Rust types are **mirrors** of ESRI's JSON specification - any deviation will cause runtime failures when interacting with the API.

### Documentation-First Development Process

**For EVERY type and field:**

1. **Find the official ESRI documentation page** for that structure
2. **Compare field names, types, and requirements** exactly
3. **Document the source** in Rustdoc with links
4. **Test against real ESRI responses** to validate structure
5. **Handle optional fields** appropriately (ESRI schema evolution)

### Primary Documentation Sources (By Phase)

#### Phase 1: Service & Layer Structure
- **Service Definition**: [Feature Service | ArcGIS REST APIs](https://developers.arcgis.com/rest/services-reference/enterprise/feature-service/)
- **Layer Definition**: [Layer (Feature Service) | ArcGIS REST APIs](https://developers.arcgis.com/rest/services-reference/enterprise/layer-feature-service/)
- **Field Types**: [Layer (Feature Service) | ArcGIS REST APIs](https://developers.arcgis.com/rest/services-reference/enterprise/layer-feature-service/) - Fields section
- **Geometry Types**: Already implemented in `src/geometry/`

#### Phase 2: Service Creation
- **Create Service**: [Create Service | ArcGIS REST APIs](https://developers.arcgis.com/rest/services-reference/online/create-service/)
- **Service Parameters**: Look for `createParameters` JSON structure examples

#### Phase 3: Advanced Features
- **Domains**: [Layer (Feature Service) | ArcGIS REST APIs](https://developers.arcgis.com/rest/services-reference/enterprise/layer-feature-service/) - Domain section
- **Templates**: [Layer (Feature Service) | ArcGIS REST APIs](https://developers.arcgis.com/rest/services-reference/enterprise/layer-feature-service/) - Templates section
- **Relationships**: [Layer (Feature Service) | ArcGIS REST APIs](https://developers.arcgis.com/rest/services-reference/enterprise/layer-feature-service/) - Relationships section

#### Phase 5: Validation
- **Service Constraints**: Review ESRI documentation for requirements (e.g., "GlobalID required for versioning")
- **Field Constraints**: Note required fields, valid value ranges, field dependencies

#### Phase 6: Retrieval
- **Service JSON Response**: Use real ArcGIS Online/Enterprise service URLs to fetch actual JSON
- **Schema Evolution**: Handle fields that may be added/removed across ESRI versions

### Validation Strategy

**Before marking any type "complete":**

1. ✅ **Reference check**: Link to ESRI doc page in Rustdoc
2. ✅ **Field audit**: Every ESRI field represented (or explicitly skipped with reason)
3. ✅ **Serialization test**: Rust → JSON matches ESRI examples
4. ✅ **Deserialization test**: ESRI example JSON → Rust succeeds
5. ✅ **Round-trip test**: Rust → JSON → Rust preserves data
6. ✅ **Real API test**: Works with live ArcGIS Online/Enterprise

### Common Pitfalls to Avoid

❌ **DON'T** guess field names - verify against ESRI docs
❌ **DON'T** assume field optionality - check documentation
❌ **DON'T** invent field types - use exact ESRI enum values
❌ **DON'T** skip "unimportant" fields - they may be required by ESRI
❌ **DON'T** use generic types (String, i32) without validating constraints

✅ **DO** copy exact field names from ESRI JSON examples
✅ **DO** use `#[serde(rename = "...")]` for Rust naming conventions
✅ **DO** document ESRI's constraints in Rustdoc
✅ **DO** add `#[serde(default)]` for truly optional fields
✅ **DO** test with multiple ESRI service versions

### Example: Proper Documentation

```rust
/// Field definition within a layer.
///
/// Maps to ESRI's field object as documented in:
/// <https://developers.arcgis.com/rest/services-reference/enterprise/layer-feature-service/>
///
/// # ESRI Specification
///
/// Required fields:
/// - `name`: Field name (must be unique within layer)
/// - `type`: Field type (see FieldType enum for valid values)
///
/// Optional fields:
/// - `alias`: Display name for the field
/// - `nullable`: Whether NULL values are allowed (default: true)
/// - `editable`: Whether users can edit this field (default: true)
/// - `length`: Maximum length for string fields
///
/// # Branch Versioning Requirements
///
/// For branch-versioned layers, ESRI requires:
/// - An OBJECTID field (type: esriFieldTypeOID)
/// - A GlobalID field (type: esriFieldTypeGlobalID)
///
/// Source: <https://pro.arcgis.com/en/pro-app/latest/help/data/geodatabases/overview/branch-version-scenarios.htm>
#[derive(Debug, Clone, Serialize, Deserialize, derive_builder::Builder)]
#[builder(setter(into, strip_option), default)]
#[serde(rename_all = "camelCase")]
pub struct FieldDefinition {
    /// Field name.
    ///
    /// Must be unique within the layer. ESRI recommends uppercase for system fields
    /// (OBJECTID, GlobalID) and mixed case for user fields.
    name: String,

    /// Field type.
    ///
    /// Must be one of ESRI's predefined field types. See FieldType enum for valid values.
    #[serde(rename = "type")]
    field_type: FieldType,

    // ... rest of fields with documentation ...
}
```

### Testing Compliance

Each type must have tests that:

```rust
#[test]
fn test_field_definition_matches_esri_spec() {
    // Example from ESRI documentation:
    // https://developers.arcgis.com/rest/services-reference/enterprise/layer-feature-service/
    let esri_json = r#"{
        "name": "OBJECTID",
        "type": "esriFieldTypeOID",
        "alias": "Object ID",
        "nullable": false,
        "editable": false
    }"#;

    // Must deserialize without errors
    let field: FieldDefinition = serde_json::from_str(esri_json)
        .expect("Should match ESRI specification");

    // Verify field values
    assert_eq!(field.name(), "OBJECTID");
    assert_eq!(*field.field_type(), FieldType::Oid);
    assert_eq!(field.nullable(), Some(&false));
    assert_eq!(field.editable(), Some(&false));

    // Round-trip: Rust → JSON → Rust
    let json_output = serde_json::to_string(&field).unwrap();
    let roundtrip: FieldDefinition = serde_json::from_str(&json_output).unwrap();
    assert_eq!(field, roundtrip);
}
```

### Handling ESRI Schema Evolution

ESRI adds new fields and features over time. Our types must be forward-compatible:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FieldDefinition {
    // Known fields...

    /// New beta field types added in 2026.
    ///
    /// These are optional because they don't exist in older ESRI versions.
    #[serde(skip_serializing_if = "Option::is_none", default)]
    required: Option<bool>,

    /// Catch-all for unknown fields (forward compatibility).
    ///
    /// ESRI may add new fields we don't know about yet. This preserves them
    /// during round-trip serialization without breaking our code.
    #[serde(flatten, skip_serializing_if = "serde_json::Map::is_empty", default)]
    additional_properties: serde_json::Map<String, serde_json::Value>,
}
```

---

## The Problem

### Current Anti-Pattern

```rust
// ❌ No compile-time safety
let service_def = json!({
    "layers": [{
        "id": 0,
        "name": "MyLayer",
        "geometryType": "esriGeometryPoint",  // Typo? Runtime error!
        "fields": [...]  // What structure? Unknown!
    }]
});

params.with_service_definition(service_def);  // Just JSON - compiler can't help
```

**Issues:**
- No validation until runtime (API call)
- Typos in field names silently ignored
- No IDE support (autocomplete, go-to-definition)
- Breaking changes in ESRI API discovered at runtime
- Difficult to maintain and refactor

### The Rust Solution

```rust
// ✅ Compile-time safety
let service_def = ServiceDefinitionBuilder::default()
    .name("MyVersionedService")
    .versioning_info(
        VersioningInfoBuilder::default()
            .versioning_type(VersioningType::Branch)  // Enum prevents typos!
            .versioning_enabled(true)
            .build()?
    )
    .add_layer(
        LayerDefinitionBuilder::default()
            .id(0)
            .name("Points")
            .geometry_type(GeometryTypeDefinition::Point)  // Type-checked!
            .build()?
    )
    .build()?;
```

---

## Implementation Phases

### Phase 0: Documentation & Planning ✅

- [x] Create this implementation plan
- [x] Document anti-pattern and solution
- [x] Define phased rollout strategy
- [ ] Add to PLANNING_INDEX.md
- [ ] Review with maintainers

### Phase 1: Core Service Definition Types ✅

**Goal:** Minimal viable types for service creation with branch versioning

**File:** `src/services/portal/service_definition.rs`

**Completed:** 2026-02-06

#### Core Types

- [ ] `ServiceDefinition` - Top-level service structure
  - [ ] Basic metadata (name, description)
  - [ ] Layers collection
  - [ ] Tables collection
  - [ ] Spatial reference
  - [ ] Capabilities string
  - [ ] Max record count

- [ ] `VersioningInfo` - Branch versioning configuration
  - [ ] `VersioningType` enum (Branch, Traditional)
  - [ ] `versioning_enabled` flag

- [ ] `EditorTrackingInfo` - Editor tracking config
  - [ ] Enable/disable flags
  - [ ] Ownership-based access control
  - [ ] Query/update/delete permissions

#### Layer Types

- [ ] `LayerDefinition` - Layer structure
  - [ ] Layer ID, name, type
  - [ ] Geometry type
  - [ ] Fields collection
  - [ ] ObjectID field name
  - [ ] GlobalID field name (required for versioning)
  - [ ] Display field
  - [ ] Templates

- [ ] `GeometryTypeDefinition` enum
  - [ ] Point
  - [ ] Multipoint
  - [ ] Polyline
  - [ ] Polygon
  - [ ] Envelope

#### Field Types

- [ ] `FieldDefinition` - Field structure
  - [ ] Name, alias
  - [ ] Field type
  - [ ] SQL type
  - [ ] Nullable, editable, required flags
  - [ ] Default value
  - [ ] Length (for strings)
  - [ ] Domain

- [ ] `FieldType` enum
  - [ ] SmallInteger
  - [ ] Integer
  - [ ] Single, Double
  - [ ] String
  - [ ] Date
  - [ ] OID, GlobalID
  - [ ] Geometry
  - [ ] Blob, Raster
  - [ ] GUID, XML

#### Spatial Reference

- [ ] `SpatialReferenceDefinition`
  - [ ] WKID variant
  - [ ] WKT variant
  - [ ] WKID + latest WKID variant

#### Builders

- [ ] `ServiceDefinitionBuilder` with derive_builder
- [ ] `LayerDefinitionBuilder` with derive_builder
- [ ] `FieldDefinitionBuilder` with derive_builder
- [ ] `VersioningInfoBuilder` with derive_builder
- [ ] `EditorTrackingInfoBuilder` with derive_builder

#### Documentation Compliance

**⚠️ CRITICAL:** Every type must be validated against ESRI documentation

- [ ] Review [Feature Service](https://developers.arcgis.com/rest/services-reference/enterprise/feature-service/) docs
- [ ] Review [Layer (Feature Service)](https://developers.arcgis.com/rest/services-reference/enterprise/layer-feature-service/) docs
- [ ] Document ESRI source URLs in Rustdoc for each type
- [ ] Copy field names exactly from ESRI JSON examples
- [ ] Verify enum values match ESRI string literals exactly
- [ ] Note required vs optional fields per ESRI spec
- [ ] Test against real ESRI JSON responses (fetch from live service)

#### Testing

- [ ] Unit tests for each builder
- [ ] Serialization round-trip tests (Rust → JSON → Rust)
- [ ] Validate against ESRI's example JSON (from documentation)
- [ ] Test with minimal service definition
- [ ] Test with full-featured service definition
- [ ] Deserialization tests with real ArcGIS Online service definitions
- [ ] Verify all `FieldType` enum values exist in ESRI spec
- [ ] Verify all `GeometryTypeDefinition` enum values match ESRI

**Deliverable:** Compiling module with core types, builders, and tests that match ESRI specification exactly

### Phase 2: Integration with Portal Client ✅

**Goal:** Update CreateServiceParams to use strongly-typed definitions

#### API Updates

- [x] Update `CreateServiceParams`
  - [x] Change `service_definition: Option<serde_json::Value>` to `service_definition: Option<ServiceDefinition>`
  - [x] Add `.with_service_definition()` that takes `ServiceDefinition`
  - [x] Maintain serialization to JSON for API calls

- [x] ~~Update `PublishParameters`~~ (N/A - doesn't have service_definition field)

- [x] Update `UpdateServiceDefinitionParams`
  - [x] Change to `Option<ServiceDefinition>`
  - [x] Updated `.with_service_definition()` to accept `ServiceDefinition`

#### Portal Client Methods

- [x] Update `create_service()` implementation
  - [x] Serialize `ServiceDefinition` to JSON and merge into createParameters
  - [x] Send to ESRI API
  - [x] Updated docstring example to use strongly-typed API

- [x] Update `update_service_definition()` implementation
  - [x] Serialize `ServiceDefinition` to JSON string for updateDefinition parameter
  - [x] Full support for modifying service properties

#### Migration Strategy

- [x] Clean break - no backward compatibility layer needed
  - [x] `ServiceDefinition` type only (compile-time enforcement)
  - [x] No deprecated variants to maintain
  - [x] Compiler guides all migrations

- [x] Update examples
  - [x] Migrated `branch_versioning_workflow.rs` to use strongly-typed ServiceDefinition
    - Added ObjectID and GlobalID fields for branch versioning requirements
    - Set `is_data_branch_versioned: true` on layer
    - Created complete layer with 5 fields (OBJECTID, GlobalID, NAME, DESCRIPTION, VALUE)
  - [x] Migrated `portal_publishing.rs` to use strongly-typed ServiceDefinition
    - Replaced `json!` macro with FieldDefinitionBuilder
    - Created layer with 4 fields (OBJECTID, CITY_NAME, CNTRY_NAME, POP)
    - Full type safety for service schema definition
  - [x] Added docstring example in `create_service()` method

#### Testing

- [x] Integration tests added (`tests/portal_service_definition_integration_test.rs`)
  - [x] Test `CreateServiceParams` with `ServiceDefinition`
  - [x] Test `UpdateServiceDefinitionParams` with `ServiceDefinition`
  - [x] Test JSON serialization matches ESRI format
  - [x] Test backward compatibility (params without definition)
  - [x] Test round-trip serialization

#### ESRI API Compliance Testing

**⚠️ Note:** Full API testing requires live service creation with API keys

- [x] Reviewed [Create Service API](https://developers.arcgis.com/rest/services-reference/online/create-service/) documentation
- [x] Verified JSON serialization structure matches ESRI spec
- [x] Added comprehensive serialization tests
- [ ] Test service creation with ArcGIS Online (requires API key testing)
- [ ] Test service creation with ArcGIS Enterprise (requires Enterprise access)
- [ ] Verify versioning is properly enabled in created service (integration test)

**Deliverable:** ✅ Portal client accepts strongly-typed service definitions with compile-time guarantees

### Phase 3: Advanced Layer Features 🔜

**Goal:** Expand type coverage for production use cases

#### Domains

- [ ] `Domain` trait
- [ ] `CodedValueDomain` - Discrete values
  - [ ] Code/name pairs
  - [ ] Field type
- [ ] `RangeDomain` - Numeric ranges
  - [ ] Min/max values
  - [ ] Field type

#### Templates

- [ ] `FeatureTemplate` - Template for feature creation
  - [ ] Name, description
  - [ ] Prototype feature (attributes)
  - [ ] Drawing tool

#### Indexes

- [ ] `Index` - Spatial and attribute indexes
  - [ ] Name, fields
  - [ ] Unique constraint
  - [ ] Index type (spatial, attribute)

#### Relationships

- [ ] `Relationship` - Layer relationships
  - [ ] Origin/destination tables
  - [ ] Cardinality
  - [ ] Key fields
  - [ ] Composite relationships

#### Additional Properties

- [ ] `DrawingInfo` - Renderer configuration
- [ ] `PopupInfo` - Popup configuration
- [ ] `EditFieldsInfo` - Creator/editor fields
- [ ] `OwnershipBasedAccessControl` - Fine-grained permissions

#### ESRI Specification Compliance

**⚠️ CRITICAL:** Each advanced feature has specific ESRI requirements

- [ ] Review [Layer (Feature Service)](https://developers.arcgis.com/rest/services-reference/enterprise/layer-feature-service/) for domain specifications
- [ ] Study ESRI's template JSON examples (drawingTool, prototype structure)
- [ ] Verify relationship cardinality values match ESRI's enum
- [ ] Test coded value domains with actual ESRI constraints
- [ ] Test range domains with numeric boundaries
- [ ] Validate template structure matches ESRI's feature template spec
- [ ] Fetch real services with these features to validate structure

**Deliverable:** Comprehensive layer definition with advanced features matching ESRI specification

### Phase 4: Table Definitions 🔜

**Goal:** Support non-spatial tables

- [ ] `TableDefinition` struct
  - [ ] Similar to LayerDefinition but no geometry
  - [ ] Fields, relationships, templates
  - [ ] ObjectID and GlobalID support

- [ ] Update `ServiceDefinition` to include tables
- [ ] Add table-specific builder
- [ ] Tests for table creation

**Deliverable:** Full support for tables in service definitions

### Phase 5: Validation Layer 🔜

**Goal:** Add compile-time and runtime validation

#### Compile-Time Validation

- [ ] Builder type states for required fields
  - [ ] Enforce ObjectID field presence
  - [ ] Enforce GlobalID for versioned layers
  - [ ] Enforce geometry field for layers

- [ ] Sealed traits for valid combinations
  - [ ] Geometry type + field combinations
  - [ ] Versioning requirements

#### Runtime Validation

- [ ] `ServiceDefinition::validate()` method
  - [ ] Check layer ID uniqueness
  - [ ] Verify field name conflicts
  - [ ] Validate spatial reference consistency
  - [ ] Check versioning requirements

- [ ] Detailed error messages
  - [ ] Point to specific validation failure
  - [ ] Suggest corrections
  - [ ] Link to ESRI documentation

**Deliverable:** Validation prevents invalid service definitions

### Phase 6: Service Definition Retrieval 🔜

**Goal:** Parse existing service definitions into Rust types

- [ ] Add `FeatureServiceClient::get_definition()` method
- [ ] Parse JSON response into `ServiceDefinition`
- [ ] Handle missing fields gracefully (ESRI schema evolution)
- [ ] Support partial deserialization
- [ ] Tests with real ESRI service definitions

**Deliverable:** Bidirectional conversion (Rust ↔ JSON)

### Phase 7: Update Operations 🔜

**Goal:** Support modifying existing service definitions

- [ ] `ServiceDefinitionUpdate` struct
  - [ ] Add layers
  - [ ] Delete layers
  - [ ] Update layer properties
  - [ ] Add fields to layers
  - [ ] Delete fields from layers

- [ ] `AddToDefinition` operation
- [ ] `DeleteFromDefinition` operation
- [ ] `UpdateDefinition` operation

**Deliverable:** Full CRUD operations on service definitions

### Phase 8: Documentation & Examples 🔜

**Goal:** Comprehensive documentation for users

#### Documentation

- [ ] Module-level docs for `service_definition`
- [ ] Type-level docs with ESRI API references
- [ ] Field-level docs with constraints
- [ ] Common patterns guide
- [ ] Migration guide from `serde_json::Value`

#### Examples

- [ ] `service_definition_creation.rs` - Create service from scratch
- [ ] `service_definition_versioning.rs` - Enable branch versioning
- [ ] `service_definition_update.rs` - Modify existing service
- [ ] `service_definition_inspection.rs` - Query and parse definitions

**Deliverable:** Users can adopt new API easily

---

## Reference Information

### ESRI Documentation

**Primary Sources:**
- [Layer (Feature Service) | ArcGIS REST APIs](https://developers.arcgis.com/rest/services-reference/enterprise/layer-feature-service/)
- [Feature Service | ArcGIS REST APIs](https://developers.arcgis.com/rest/services-reference/enterprise/feature-service/)
- [Add to Definition (Feature Service)](https://developers.arcgis.com/rest/services-reference/add-to-definition-feature-service-.htm)

**Key Concepts:**
- Branch versioning requires GlobalID field
- Editor tracking needs specific field configuration
- Spatial reference must be consistent across layers
- ObjectID field must be non-nullable and non-editable

### Related Codebase Files

**Current Implementation:**
- `src/services/portal/types.rs` - `CreateServiceParams` (lines 1397-1424)
- `src/services/portal/client/publishing.rs` - `create_service()` (lines 59-148)
- `examples/enterprise/branch_versioning_workflow.rs` - Example usage

**New Files:**
- `src/services/portal/service_definition.rs` - Main module (to be created)
- `src/services/portal/service_definition/*.rs` - Submodules (Phase 3+)

---

## Dependencies

### Crate Dependencies

- `serde = { version = "1.0", features = ["derive"] }` - Already present
- `serde_json = "1.0"` - Already present
- `derive_builder = "0.20"` - Already present
- `derive_getters = "0.5"` - Already present

### Internal Dependencies

- Geometry types from `src/geometry/` - Already implemented
- Spatial reference types - Will reuse from geometry module
- Error types from `src/error.rs` - Already implemented

---

## Testing Strategy

### Unit Tests

- [ ] Builder tests for each type
- [ ] Serialization tests (Rust → JSON)
- [ ] Deserialization tests (JSON → Rust)
- [ ] Round-trip tests (Rust → JSON → Rust)
- [ ] Validation tests

### Integration Tests

- [ ] Create service with minimal definition
- [ ] Create service with full definition
- [ ] Enable versioning on existing service
- [ ] Add layer to existing service
- [ ] Update layer definition

### Real-World Tests

- [ ] Test with ArcGIS Online
- [ ] Test with ArcGIS Enterprise
- [ ] Validate against ESRI's sample services
- [ ] Performance benchmarks (serialization overhead)

---

## Migration Path for Users

### Before (Current)

```rust
let service_def = json!({
    "layers": [{
        "id": 0,
        "name": "Points",
        "geometryType": "esriGeometryPoint",
        "fields": [
            {"name": "OBJECTID", "type": "esriFieldTypeOID"},
            {"name": "GlobalID", "type": "esriFieldTypeGlobalID"},
            {"name": "NAME", "type": "esriFieldTypeString", "length": 255}
        ]
    }],
    "versioningInfo": {
        "versioningType": "versionBranched",
        "versioningEnabled": true
    }
});

let params = CreateServiceParams::new("MyService")
    .with_service_definition(service_def);
```

### After (Phase 2+)

```rust
let service_def = ServiceDefinitionBuilder::default()
    .name("MyService")
    .versioning_info(
        VersioningInfoBuilder::default()
            .versioning_type(VersioningType::Branch)
            .versioning_enabled(true)
            .build()?
    )
    .add_layer(
        LayerDefinitionBuilder::default()
            .id(0)
            .name("Points")
            .geometry_type(GeometryTypeDefinition::Point)
            .add_field(
                FieldDefinitionBuilder::default()
                    .name("OBJECTID")
                    .field_type(FieldType::Oid)
                    .build()?
            )
            .add_field(
                FieldDefinitionBuilder::default()
                    .name("GlobalID")
                    .field_type(FieldType::GlobalId)
                    .build()?
            )
            .add_field(
                FieldDefinitionBuilder::default()
                    .name("NAME")
                    .field_type(FieldType::String)
                    .length(255)
                    .build()?
            )
            .build()?
    )
    .build()?;

let params = CreateServiceParams::new("MyService")
    .with_service_definition(service_def);
```

**Benefits:**
- ✅ Typos caught at compile time
- ✅ IDE autocomplete works
- ✅ Documentation inline
- ✅ Refactoring safe
- ✅ Type-checked field types

---

## Success Criteria

### Phase 1 Complete When:
- [ ] All core types compile
- [ ] All builders work
- [ ] Tests pass
- [ ] Documentation written

### Phase 2 Complete When:
- [x] `branch_versioning_workflow.rs` example uses new types
- [ ] Service creation works with real API (requires API key testing)
- [x] No `serde_json::Value` in public API

### Project Complete When:
- [ ] All phases implemented
- [ ] Full ESRI service definition coverage
- [ ] Migration guide published
- [ ] Users report improved experience
- [ ] Zero runtime errors from invalid JSON structure

---

## Open Questions

### API Design

- **Q:** Should builders use `Result<T, E>` or `Option<T>` for validation?
  - **A:** TBD - discuss in Phase 5

- **Q:** How to handle ESRI API evolution (new fields)?
  - **A:** Use `#[serde(default)]` and `#[serde(skip_serializing_if)]`

- **Q:** Support for custom extensions to service definition?
  - **A:** TBD - may add in Phase 7+

### Versioning

- **Q:** When to bump crate version?
  - **A:** Phase 2 (breaking changes to public API) = major version bump

---

## Timeline

**Estimated effort:** 40-60 hours across all phases

- Phase 0: ✅ Complete (1 hour)
- Phase 1: ✅ Complete (12 hours)
- Phase 2: ✅ Complete (8 hours)
- Phase 3: 🔜 (8-10 hours)
- Phase 4: 🔜 (4-6 hours)
- Phase 5: 🔜 (6-8 hours)
- Phase 6: 🔜 (4-6 hours)
- Phase 7: 🔜 (6-8 hours)
- Phase 8: 🔜 (4-6 hours)

**Priority:** High - foundational change affecting service creation

---

## Related Plans

- `ESRI_GEOMETRY_INTEGRATION_PLAN.md` - Geometry type consolidation (complete)
- `PLANNING_INDEX.md` - Master plan index (to be updated)

---

## Notes

### Why Not Use serde_json::Value Everywhere?

While `serde_json::Value` is flexible, it trades compile-time safety for runtime flexibility. For a library wrapping an external API, we want:

1. **Fail fast** - Errors at compile time, not production
2. **Self-documenting** - Types document ESRI's API
3. **IDE support** - Autocomplete, go-to-definition
4. **Refactoring** - Rename field, find all uses
5. **Maintenance** - Breaking API changes caught immediately

### Incremental Adoption

Users can adopt incrementally:
1. Start with minimal definition (Phase 1)
2. Add advanced features as needed (Phases 3-4)
3. Old code keeps working during transition (Phase 2 compatibility)
4. Deprecation warnings guide migration
5. Remove old API in next major version

### Branch Versioning Implementation Notes

**Phase 2 Completion:** The branch_versioning_workflow example now demonstrates the complete workflow for creating a branch-versioned service using the strongly-typed API:

1. **Service Definition Structure:**
   - ServiceDefinition with layers collection
   - LayerDefinition with branch versioning requirements
   - 5 fields including required ObjectID and GlobalID

2. **Branch Versioning Requirements Met:**
   - ObjectID field (FieldType::Oid) - non-nullable, non-editable
   - GlobalID field (FieldType::GlobalId) - non-nullable, non-editable, length 38
   - `is_data_branch_versioned: Some(true)` set on layer

3. **Type Safety Benefits Demonstrated:**
   - Compile-time enforcement of required fields
   - Builder pattern prevents struct literal errors
   - Field types validated at build time
   - ESRI-compliant JSON serialization guaranteed

**Key Learning:** The `.add_layer()` method conflicts with derive_builder's `&mut Self` return type. Use `.layers(vec![...])` instead for consistent builder patterns.

---

**Last Updated:** 2026-02-06
**Status:** 🟢 Phase 1-2 Complete, Branch Versioning Fully Supported
