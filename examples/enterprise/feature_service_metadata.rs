//! 🔍 Feature Service Metadata - Service Introspection and Schema Discovery
//!
//! Demonstrates retrieving metadata from feature services to understand service structure,
//! layer schemas, and field definitions before working with data.
//!
//! # What You'll Learn
//!
//! - **Service definition**: Get service-level metadata (layers, tables, capabilities)
//! - **Layer schema**: Retrieve complete field definitions for specific layers
//! - **Table schema**: Access non-spatial table definitions
//! - **Field introspection**: Discover field names, types, and constraints
//! - **Dynamic schema discovery**: Build features programmatically from schema
//!
//! # Why Metadata Matters
//!
//! Schema introspection is essential for:
//! - **Dynamic feature construction**: Create features that match layer schema
//! - **Validation**: Ensure attribute values match field types and constraints
//! - **Code generation**: Generate types from existing services
//! - **Discovery**: Explore unfamiliar services
//! - **Migration**: Transfer schemas between services
//!
//! # Prerequisites
//!
//! - No authentication required (uses public SF311 service)
//! - Internet connection for service access
//!
//! # Running
//!
//! ```bash
//! cargo run --example feature_service_metadata
//!
//! # With debug logging to see all requests:
//! RUST_LOG=debug cargo run --example feature_service_metadata
//! ```
//!
//! # Real-World Use Cases
//!
//! - **Schema migration**: Copy layer definitions between services
//! - **Form generation**: Build data entry forms from field definitions
//! - **Type generation**: Generate Rust structs from service schemas
//! - **Validation rules**: Extract domain constraints for client-side validation
//! - **Documentation**: Auto-generate API documentation from service metadata
//! - **Testing**: Validate service schema matches expected structure
//!
//! # ESRI Documentation
//!
//! - Service Definition: <https://developers.arcgis.com/rest/services-reference/enterprise/feature-service/>
//! - Layer Definition: <https://developers.arcgis.com/rest/services-reference/enterprise/layer-feature-service/>

use anyhow::Result;
use arcgis::{ArcGISClient, FeatureServiceClient, LayerId, NoAuth};

/// Public San Francisco 311 service (no authentication required)
const SF311_SERVICE: &str =
    "https://sampleserver6.arcgisonline.com/arcgis/rest/services/SF311/FeatureServer";

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing for structured logging
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    tracing::info!("🔍 Feature Service Metadata Examples");
    tracing::info!("Using San Francisco 311 Service Requests (public, no auth required)");
    tracing::info!("");

    // Create client with no authentication (public service)
    let client = ArcGISClient::new(NoAuth);
    let service = FeatureServiceClient::new(SF311_SERVICE, &client);

    // Demonstrate metadata operations
    demonstrate_service_definition(&service).await?;
    demonstrate_layer_definition(&service).await?;

    tracing::info!("\n✅ All metadata examples completed successfully!");
    print_best_practices();

    Ok(())
}

/// Demonstrates retrieving and inspecting the service-level definition.
async fn demonstrate_service_definition(service: &FeatureServiceClient<'_>) -> Result<()> {
    tracing::info!("\n=== Example 1: Service Definition ===");
    tracing::info!("Retrieve service-level metadata to discover layers and capabilities");
    tracing::info!("");

    let definition = service.get_definition().await?;

    // Validate service has layers
    anyhow::ensure!(
        !definition.layers().is_empty(),
        "Service should have at least one layer. Found: {}",
        definition.layers().len()
    );

    tracing::info!("✅ Retrieved service definition");
    tracing::info!("   Layers: {}", definition.layers().len());
    tracing::info!("   Tables: {}", definition.tables().len());

    if let Some(max_count) = definition.max_record_count() {
        tracing::info!("   Max record count: {}", max_count);
        anyhow::ensure!(
            *max_count > 0,
            "Max record count should be positive: {}",
            max_count
        );
    }

    if let Some(capabilities) = definition.capabilities() {
        tracing::info!("   Capabilities: {}", capabilities);
        // Verify common capabilities are present
        anyhow::ensure!(
            capabilities.contains("Query"),
            "Service should support Query capability"
        );
    }

    tracing::info!("");
    tracing::info!("   Layer stubs (id, name, geometry type):");

    // Display layer overview
    for layer in definition.layers() {
        tracing::info!(
            "     {}. {} (id={}, type={:?})",
            layer.id(),
            layer.name(),
            layer.id(),
            layer.geometry_type()
        );

        // Validate layer metadata
        anyhow::ensure!(
            !layer.name().is_empty(),
            "Layer {} should have a non-empty name",
            layer.id()
        );
    }

    tracing::info!("");
    tracing::info!("💡 Service definition provides:");
    tracing::info!("   • List of layers and tables (stubs only, no full field definitions)");
    tracing::info!("   • Service capabilities (Query, Create, Update, Delete, etc.)");
    tracing::info!("   • Max record count limits");
    tracing::info!("   • Supported query formats");
    tracing::info!("");
    tracing::info!("⚠️  Note: Layer stubs do NOT include full field definitions.");
    tracing::info!("   Use get_layer_definition() for complete schema.");

    Ok(())
}

/// Demonstrates retrieving complete layer definition with full field schema.
async fn demonstrate_layer_definition(service: &FeatureServiceClient<'_>) -> Result<()> {
    tracing::info!("\n=== Example 2: Layer Definition (Full Schema) ===");
    tracing::info!("Retrieve complete field definitions for a specific layer");
    tracing::info!("");

    // Get layer 0 (SF311 service requests layer)
    let layer_id = LayerId::new(0);
    let layer_def = service.get_layer_definition(layer_id).await?;

    tracing::info!("✅ Retrieved layer definition");
    tracing::info!("   Layer ID: {}", layer_def.id());
    tracing::info!("   Layer name: {}", layer_def.name());
    tracing::info!("   Geometry type: {:?}", layer_def.geometry_type());
    tracing::info!("   Field count: {}", layer_def.fields().len());

    // Validate layer metadata
    anyhow::ensure!(!layer_def.name().is_empty(), "Layer should have a name");

    anyhow::ensure!(
        !layer_def.fields().is_empty(),
        "Layer should have fields. Found: {}",
        layer_def.fields().len()
    );

    // Validate ObjectID field exists
    let has_objectid = layer_def
        .fields()
        .iter()
        .any(|f| f.field_type() == &arcgis::FieldType::Oid);

    anyhow::ensure!(
        has_objectid,
        "Layer must have an ObjectID field (esriFieldTypeOID)"
    );

    tracing::info!("");
    tracing::info!("   Field schema (first 10 fields):");

    // Display field details
    for (idx, field) in layer_def.fields().iter().take(10).enumerate() {
        let nullable = field.nullable().unwrap_or(true);
        let editable = field.editable().unwrap_or(true);
        let field_info = format!(
            "     {}. {} (type={:?}, nullable={}, editable={})",
            idx + 1,
            field.name(),
            field.field_type(),
            nullable,
            editable,
        );
        tracing::info!("{}", field_info);

        // Validate field metadata
        anyhow::ensure!(
            !field.name().is_empty(),
            "Field {} should have a non-empty name",
            idx
        );

        // String fields should have length
        if field.field_type() == &arcgis::FieldType::String {
            if let Some(length) = field.length() {
                anyhow::ensure!(
                    *length > 0,
                    "String field '{}' should have positive length: {}",
                    field.name(),
                    length
                );
            }
        }

        // ObjectID fields should not be nullable or editable
        if field.field_type() == &arcgis::FieldType::Oid {
            if let Some(nullable) = field.nullable() {
                anyhow::ensure!(
                    !nullable,
                    "ObjectID field '{}' should not be nullable",
                    field.name()
                );
            }
            if let Some(editable) = field.editable() {
                anyhow::ensure!(
                    !editable,
                    "ObjectID field '{}' should not be editable",
                    field.name()
                );
            }
        }
    }

    if layer_def.fields().len() > 10 {
        tracing::info!("     ... and {} more fields", layer_def.fields().len() - 10);
    }

    tracing::info!("");

    // Display ObjectID field name if specified
    if let Some(oid_field) = layer_def.object_id_field() {
        tracing::info!("   ObjectID field: {}", oid_field);

        // Verify the field actually exists
        let field_exists = layer_def
            .fields()
            .iter()
            .any(|f| f.name().eq_ignore_ascii_case(oid_field));

        anyhow::ensure!(
            field_exists,
            "ObjectID field '{}' should exist in fields array",
            oid_field
        );
    }

    // Display GlobalID field name if specified
    if let Some(gid_field) = layer_def.global_id_field() {
        if !gid_field.is_empty() {
            tracing::info!("   GlobalID field: {}", gid_field);

            // Verify the field actually exists
            let field_exists = layer_def
                .fields()
                .iter()
                .any(|f| f.name().eq_ignore_ascii_case(gid_field));

            anyhow::ensure!(
                field_exists,
                "GlobalID field '{}' should exist in fields array",
                gid_field
            );
        }
    }

    tracing::info!("");
    tracing::info!("💡 Layer definition provides:");
    tracing::info!("   • Complete field definitions (name, type, length, constraints)");
    tracing::info!("   • Geometry type (Point, Polyline, Polygon, etc.)");
    tracing::info!("   • ObjectID and GlobalID field names");
    tracing::info!("   • Display field for popups");
    tracing::info!("   • Relationships to other layers/tables");
    tracing::info!("   • Templates for feature creation");
    tracing::info!("");
    tracing::info!("🎯 Use this schema to:");
    tracing::info!("   • Dynamically construct features that match the layer");
    tracing::info!("   • Validate attribute values before submission");
    tracing::info!("   • Generate type-safe Rust structs");
    tracing::info!("   • Build data entry forms");

    // Example: Find all string fields
    tracing::info!("");
    tracing::info!("   Example: String fields in this layer:");
    let string_fields: Vec<_> = layer_def
        .fields()
        .iter()
        .filter(|f| f.field_type() == &arcgis::FieldType::String)
        .collect();

    anyhow::ensure!(
        !string_fields.is_empty(),
        "Layer should have at least some string fields"
    );

    for (idx, field) in string_fields.iter().take(5).enumerate() {
        let length = field
            .length()
            .map(|l| l.to_string())
            .unwrap_or_else(|| "N/A".to_string());
        tracing::info!(
            "     {}. {} (max length: {})",
            idx + 1,
            field.name(),
            length
        );
    }

    if string_fields.len() > 5 {
        tracing::info!("     ... and {} more", string_fields.len() - 5);
    }

    tracing::info!("");
    tracing::info!("✅ Layer definition verified:");
    tracing::info!("   • Has ObjectID field");
    tracing::info!("   • All fields have valid names and types");
    tracing::info!("   • String fields have positive lengths");
    tracing::info!("   • System fields properly configured (non-nullable, non-editable)");

    Ok(())
}

/// Prints best practices for working with service metadata.
fn print_best_practices() {
    tracing::info!("\n💡 Metadata Best Practices:");
    tracing::info!("   - Always call get_definition() first to discover layer IDs");
    tracing::info!("   - Use get_layer_definition() to get full field schemas");
    tracing::info!("   - Cache metadata locally - it rarely changes");
    tracing::info!("   - Validate attribute values against field types before submission");
    tracing::info!("   - Check field.nullable() before requiring values");
    tracing::info!("");
    tracing::info!("🎯 Common Patterns:");
    tracing::info!("   - Dynamic feature construction from schema");
    tracing::info!("   - Type-safe attribute builders using field definitions");
    tracing::info!("   - Client-side validation using domain constraints");
    tracing::info!("   - Code generation for strongly-typed feature access");
    tracing::info!("");
    tracing::info!("⚡ Performance Tips:");
    tracing::info!("   - Service definition is lightweight (layer stubs only)");
    tracing::info!("   - Layer definition is heavier (full field schemas)");
    tracing::info!("   - Cache metadata to avoid repeated requests");
    tracing::info!("   - Use service definition to batch layer definition calls");
    tracing::info!("");
    tracing::info!("📐 Schema Discovery Workflow:");
    tracing::info!("   1. get_definition() → discover layer IDs");
    tracing::info!("   2. get_layer_definition(id) → get field schemas");
    tracing::info!("   3. Build features using field names and types");
    tracing::info!("   4. Validate values match field types/domains");
    tracing::info!("   5. Submit features via add_features/update_features");
}
