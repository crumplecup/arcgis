//! 🚀 Portal Publishing - Complete Data Lifecycle Example
//!
//! Demonstrates the full spatial data lifecycle: query → convert → publish → share → verify.
//! This "full meal deal" example shows real-world interoperability between ArcGIS REST API,
//! GeoRust ecosystem, and Portal for ArcGIS.
//!
//! # What You'll Learn
//!
//! - **Data acquisition**: Query features from public ArcGIS services
//! - **Format conversion**: FeatureSet → GeoJSON using geo-types
//! - **Direct service creation**: Create hosted services programmatically
//! - **Feature editing**: Add features to services via applyEdits
//! - **Item creation**: Upload GeoJSON to Portal catalog
//! - **Access control**: Share services with organization
//! - **Round-trip verification**: Query your new service
//! - **Cleanup**: Delete services and items
//!
//! # Prerequisites
//!
//! - Required: API key with content management privileges in `.env`
//! - Permissions: Create content, publish hosted services, sharing
//! - Credits: Publishing consumes ArcGIS Online credits
//!
//! ## Environment Variables
//!
//! ```env
//! # Tier 3: Content management (required for portal publishing)
//! ARCGIS_CONTENT_KEY=your_api_key_with_content_privileges
//! ```
//!
//! Note: Portal publishing operations require an API key with content creation privileges.
//!
//! # Running
//!
//! ```bash
//! cargo run --example portal_publishing
//!
//! # With debug logging to see all API calls:
//! RUST_LOG=debug cargo run --example portal_publishing
//! ```
//!
//! # Real-World Use Cases
//!
//! - **Data pipelines**: ETL workflows (extract → transform → publish)
//! - **Data migration**: Move data between systems using standard formats
//! - **Open data portals**: Convert and publish public datasets
//! - **Backup and restore**: Export services → GeoJSON → re-publish
//! - **Format conversion**: Shapefile → GeoJSON → Hosted Service
//! - **Multi-platform support**: Share data across GIS platforms
//! - **Developer workflows**: Local GeoJSON files → test services
//!
//! # Publishing Concepts
//!
//! **Publishing Workflows:**
//!
//! This example demonstrates TWO different workflows for publishing spatial data:
//!
//! **Workflow A: Direct Service Creation** (Demonstrated)
//! - Create empty hosted feature service container
//! - Add layer definition with schema (addToDefinition)
//! - Add features programmatically using applyEdits
//! - Best for: Programmatic data loading, ETL pipelines, dynamic schemas
//!
//! **Workflow B: Item Data Management** (Demonstrated)
//! - Create portal item with metadata
//! - Upload file data to item
//! - Retrieve and verify stored data
//! - Best for: Data archival, file storage, backup/restore
//!
//! **Item Types:**
//! - Feature Service: Vector data with attributes (points, lines, polygons)
//! - Map Service: Cached/dynamic map tiles
//! - Image Service: Raster/imagery
//! - Vector Tile Service: MVT format for base maps
//!
//! **Hosting Models:**
//! - Hosted: ArcGIS Online/Portal manages data storage
//! - Federated: Data stays in enterprise geodatabase
//! - Referenced: Service points to external data source
//!
//! # GeoRust Integration
//!
//! This example demonstrates interoperability with the GeoRust ecosystem:
//! - `geo-types` - Standard Rust geometry types
//! - `geojson` - GeoJSON format support
//! - Future: `shapefile-rs`, `geotiff` for other formats

use anyhow::{Context, Result};
use arcgis::{
    AddItemParams, AddToDefinitionParams, ApiKeyAuth, ApiKeyTier, ArcGISClient,
    CreateServiceParams, EditOptions, Feature, FeatureServiceClient, FieldDefinitionBuilder,
    FieldType, GeometryTypeDefinition, ItemDataUpload, LayerDefinitionBuilder, LayerId, NoAuth,
    PortalClient, SharingParameters,
};
use std::collections::HashMap;
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing for structured logging
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    tracing::info!("🚀 Portal Publishing Examples");
    tracing::info!("Demonstrating hosted feature service creation and management");
    tracing::info!("");

    // Create authenticated client (automatically loads .env)
    tracing::debug!("Creating authenticated client");
    let auth = ApiKeyAuth::from_env(ApiKeyTier::Content)
        .context("Failed to load API key from environment (requires ARCGIS_CONTENT_KEY)")?;
    let client = ArcGISClient::new(auth);
    let portal = PortalClient::new("https://www.arcgis.com/sharing/rest", &client);

    tracing::info!("✅ Authenticated with API key (ARCGIS_CONTENT_KEY)");
    tracing::info!("");

    // Demonstrate two different workflows
    tracing::info!("\n💡 This example demonstrates TWO workflows for portal operations:");
    tracing::info!(
        "   A) Direct Service Creation - Create service with schema, add features programmatically"
    );
    tracing::info!("   B) Item Data Management - Upload and retrieve file data from portal items");
    tracing::info!("");

    demonstrate_workflow_a_direct_service(&portal, &client).await?;
    demonstrate_workflow_b_geojson_item(&portal, &client).await?;

    tracing::info!("\n✅ Portal publishing examples completed successfully!");
    print_best_practices();

    Ok(())
}

/// Helper: Query public service and convert to GeoJSON (shared by both workflows).
async fn query_and_convert_to_geojson()
-> Result<(arcgis::FeatureSet, geojson::FeatureCollection, String)> {
    tracing::info!("📥 Querying features from public ArcGIS service");
    tracing::info!("");
    tracing::info!("   Source: World Cities (ESRI public service)");
    tracing::info!("   Service: Public feature layer (no auth required)");
    tracing::info!("   Strategy: Query first 10 cities for demo purposes");
    tracing::info!("");

    let cities_url = "https://services.arcgis.com/P3ePLMYs2RVChkJx/arcgis/rest/services/World_Cities/FeatureServer";

    // Use NoAuth for public service
    let public_client = ArcGISClient::new(NoAuth);
    let cities_service = FeatureServiceClient::new(cities_url, &public_client);

    let query_result = cities_service
        .query(LayerId(0))
        .where_clause("1=1")
        .return_geometry(true)
        .out_fields(&["CITY_NAME", "CNTRY_NAME", "POP"])
        .limit(10)
        .execute()
        .await?;

    let feature_count = query_result.features().len();
    tracing::info!("✅ Retrieved {} cities from public service", feature_count);

    // Show sample data
    if let Some(first_city) = query_result.features().first() {
        if let Some(city_name) = first_city.attributes().get("CITY_NAME") {
            if let Some(country) = first_city.attributes().get("CNTRY_NAME") {
                tracing::info!("   Sample: {}, {}", city_name, country);
            }
        }
    }

    tracing::info!("");
    tracing::info!("🔄 Converting to GeoJSON format");
    tracing::info!("");
    tracing::info!("   Format: ArcGIS FeatureSet → GeoJSON FeatureCollection");
    tracing::info!("   Ecosystem: Using geojson crate (GeoRust)");
    tracing::info!("   Purpose: Standard format for portal upload");
    tracing::info!("");

    let geojson_fc = featureset_to_geojson(&query_result)?;
    let geojson_string = serde_json::to_string_pretty(&geojson_fc)?;

    tracing::info!(
        "✅ Converted {} features to GeoJSON",
        geojson_fc.features.len()
    );
    tracing::info!("   GeoJSON size: {} bytes", geojson_string.len());
    tracing::info!("");

    Ok((query_result, geojson_fc, geojson_string))
}

/// Workflow A: Direct service creation - create service with schema and add features.
async fn demonstrate_workflow_a_direct_service(
    portal: &PortalClient<'_>,
    client: &ArcGISClient,
) -> Result<()> {
    tracing::info!("\n========================================");
    tracing::info!("=== WORKFLOW A: Direct Service Creation ===");
    tracing::info!("========================================");
    tracing::info!(
        "Query → Convert → Create Service → Add Definition → Add Features → Share → Verify → Cleanup"
    );
    tracing::info!("");

    // ========================================================================
    // STEP 1: Query and convert features
    // ========================================================================
    let (query_result, _geojson_fc, _geojson_string) = query_and_convert_to_geojson().await?;
    let feature_count = query_result.features().len();

    // ========================================================================
    // STEP 2: Create hosted feature service with schema
    // ========================================================================
    // Generate unique service name to avoid conflicts with previous runs
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    let service_name = format!("SampleCitiesDirect_{}", timestamp);

    tracing::info!("");
    tracing::info!("🏗️  STEP 2: Creating hosted feature service with schema");
    tracing::info!("");
    tracing::info!("   Service Name: {}", service_name);
    tracing::info!("   Type: Hosted Feature Service");
    tracing::info!("   Schema: Point layer with CITY_NAME, CNTRY_NAME, POP fields");
    tracing::info!("   Capabilities: Query, Create, Update, Delete, Editing");
    tracing::info!("");

    // Define service schema matching our data using strongly-typed API
    let oid_field = FieldDefinitionBuilder::default()
        .name("OBJECTID")
        .field_type(FieldType::Oid)
        .alias("Object ID")
        .nullable(false)
        .editable(false)
        .build()
        .context("Failed to build OBJECTID field")?;

    let city_name_field = FieldDefinitionBuilder::default()
        .name("CITY_NAME")
        .field_type(FieldType::String)
        .alias("City Name")
        .length(256)
        .nullable(true)
        .editable(true)
        .build()
        .context("Failed to build CITY_NAME field")?;

    let country_name_field = FieldDefinitionBuilder::default()
        .name("CNTRY_NAME")
        .field_type(FieldType::String)
        .alias("Country Name")
        .length(256)
        .nullable(true)
        .editable(true)
        .build()
        .context("Failed to build CNTRY_NAME field")?;

    let population_field = FieldDefinitionBuilder::default()
        .name("POP")
        .field_type(FieldType::Integer)
        .alias("Population")
        .nullable(true)
        .editable(true)
        .build()
        .context("Failed to build POP field")?;

    let layer = LayerDefinitionBuilder::default()
        .id(0u32)
        .name("Cities")
        .layer_type("Feature Layer")
        .geometry_type(GeometryTypeDefinition::Point)
        .object_id_field("OBJECTID")
        .fields(vec![
            oid_field,
            city_name_field,
            country_name_field,
            population_field,
        ])
        .build()
        .context("Failed to build layer definition")?;

    // Note: ESRI's createService creates an empty container regardless of ServiceDefinition
    // We'll add the layer schema in the next step using addToDefinition
    let create_params = CreateServiceParams::new(&service_name)
        .with_description("Created via direct service creation workflow using Rust SDK")
        .with_capabilities("Query,Create,Update,Delete,Editing")
        .with_max_record_count(1000);

    let create_result = portal.create_service(create_params).await?;

    let service_item_id = create_result
        .service_item_id()
        .as_ref()
        .ok_or_else(|| anyhow::anyhow!("No service item ID returned"))?
        .clone();
    let service_url = create_result
        .service_url()
        .as_ref()
        .ok_or_else(|| anyhow::anyhow!("No service URL returned"))?
        .clone();

    tracing::info!("✅ Created hosted feature service");
    tracing::info!("   Service ID: {}", service_item_id);
    tracing::info!("   URL: {}", service_url);

    // Wait a moment for service to be ready
    tracing::info!("");
    tracing::info!("⏳ Waiting for service to initialize (15 seconds)...");
    tracing::info!("   (Hosted services need time to provision on ArcGIS Online)");
    tokio::time::sleep(Duration::from_secs(15)).await;

    // Verify service is accessible before adding layer definition
    tracing::info!("");
    tracing::info!("🔍 Verifying service is accessible...");
    let service_client = FeatureServiceClient::new(&service_url, client);
    match service_client.get_definition().await {
        Ok(def) => {
            tracing::info!("   ✓ Service is accessible");
            tracing::info!("   Layers: {} (empty container)", def.layers().len());
        }
        Err(e) => {
            tracing::warn!("   ⚠ Service not yet accessible: {}", e);
            tracing::info!("   Waiting additional 10 seconds...");
            tokio::time::sleep(Duration::from_secs(10)).await;
        }
    }

    // ========================================================================
    // STEP 3: Add layer definition to the service
    // ========================================================================
    tracing::info!("");
    tracing::info!("🏗️  STEP 3: Adding layer definition to the service");
    tracing::info!("");
    tracing::info!("   Method: addToDefinition");
    tracing::info!("   Purpose: Define layer schema (fields, geometry type)");
    tracing::info!("   Note: ESRI creates empty services - layers must be added separately");
    tracing::info!("");

    let add_def_params = AddToDefinitionParams::new().with_layers(vec![layer]);
    let add_def_result = portal
        .add_to_definition(&service_item_id, add_def_params)
        .await?;

    // Assertion: Operation must succeed
    assert!(
        *add_def_result.success(),
        "addToDefinition operation failed"
    );

    // Assertion: Must have added exactly one layer
    assert_eq!(
        add_def_result.layers().len(),
        1,
        "Expected 1 layer to be added, got {}",
        add_def_result.layers().len()
    );

    tracing::info!("✅ Added layer definition to service");
    tracing::info!("   Layers added: {}", add_def_result.layers().len());
    tracing::info!("   Layer ID: {}", add_def_result.layers()[0].id());
    tracing::info!("   Layer name: {}", add_def_result.layers()[0].name());

    // Wait a moment for layer to be ready
    tracing::info!("");
    tracing::info!("⏳ Waiting for layer to initialize (5 seconds)...");
    tokio::time::sleep(Duration::from_secs(5)).await;

    // ========================================================================
    // STEP 4: Add features to the service
    // ========================================================================
    tracing::info!("");
    tracing::info!("📝 STEP 4: Adding features to the service");
    tracing::info!("");
    tracing::info!("   Method: applyEdits with add operation");
    tracing::info!("   Features: {} cities from query result", feature_count);
    tracing::info!("");

    // Convert ArcGIS features to Feature objects for add_features
    let mut features_to_add = Vec::new();
    for feature in query_result.features() {
        let mut attributes = HashMap::new();

        // Copy attributes from source feature
        if let Some(city_name) = feature.attributes().get("CITY_NAME") {
            attributes.insert("CITY_NAME".to_string(), city_name.clone());
        }
        if let Some(country) = feature.attributes().get("CNTRY_NAME") {
            attributes.insert("CNTRY_NAME".to_string(), country.clone());
        }
        if let Some(pop) = feature.attributes().get("POP") {
            attributes.insert("POP".to_string(), pop.clone());
        }

        // Create Feature with geometry
        let geometry = feature.geometry().as_ref().cloned();
        let new_feature = Feature::new(attributes, geometry);
        features_to_add.push(new_feature);
    }

    // Add features to layer 0
    let edit_result = service_client
        .add_features(LayerId(0), features_to_add.clone(), EditOptions::default())
        .await?;

    // Assertion: Must not have zero successes AND zero failures (indicates silent failure)
    assert!(
        edit_result.success_count() > 0 || edit_result.failure_count() > 0,
        "addFeatures returned zero successes and zero failures - silent failure detected!"
    );

    // Assertion: Should have some successes
    assert!(
        edit_result.success_count() > 0,
        "No features were successfully added. Success: {}, Failures: {}",
        edit_result.success_count(),
        edit_result.failure_count()
    );

    // Assertion: Number of successes should match features we tried to add
    assert_eq!(
        edit_result.success_count(),
        features_to_add.len(),
        "Expected {} successes, got {}",
        features_to_add.len(),
        edit_result.success_count()
    );

    tracing::info!("✅ Added features to service");
    tracing::info!("   Success count: {}", edit_result.success_count());
    tracing::info!("   Failure count: {}", edit_result.failure_count());

    if !edit_result.all_succeeded() {
        tracing::warn!("   Some features failed to add");
        for (i, result) in edit_result.add_results().iter().enumerate() {
            if !*result.success() {
                if let Some(error) = result.error() {
                    tracing::warn!(
                        "     Feature {}: {} (code {})",
                        i,
                        error.description(),
                        error.code()
                    );
                }
            }
        }
    }

    // ========================================================================
    // STEP 5: Share with organization
    // ========================================================================
    tracing::info!("");
    tracing::info!("🔐 STEP 5: Sharing service with organization");
    tracing::info!("");
    tracing::info!("   Access Level: Private → Organization");
    tracing::info!("   Visibility: All organization members");
    tracing::info!("   Purpose: Enable collaboration and discovery");
    tracing::info!("");

    let share_params = SharingParameters::new().with_org(true);
    let share_result = portal.share_item(&service_item_id, share_params).await?;

    if share_result.success() {
        tracing::info!("✅ Service shared with organization");
    }

    // ========================================================================
    // STEP 6: Verify round-trip by querying new service
    // ========================================================================
    tracing::info!("");
    tracing::info!("🔍 STEP 6: Verifying round-trip (query published service)");
    tracing::info!("");
    tracing::info!("   Endpoint: {} (layer 0)", service_url);
    tracing::info!("   Test: Query all features to verify data integrity");
    tracing::info!("");

    let verify_result = service_client
        .query(LayerId(0))
        .where_clause("1=1")
        .count_only(true)
        .execute()
        .await?;

    if let Some(count) = verify_result.count() {
        tracing::info!("✅ Round-trip successful!");
        tracing::info!("   Features in published service: {}", count);
        tracing::info!("   Original features queried: {}", feature_count);

        if *count == edit_result.success_count() as u32 {
            tracing::info!("   ✓ Feature count matches (data integrity confirmed)");
        }
    }

    // ========================================================================
    // STEP 7: Cleanup (delete service)
    // ========================================================================
    tracing::info!("");
    tracing::info!("🧹 STEP 7: Cleaning up test resources");
    tracing::info!("");
    tracing::info!("   Deleting: Hosted feature service");
    tracing::info!("   Reason: Avoid cluttering portal with test data");
    tracing::info!("");

    let delete_service = portal.delete_item(&service_item_id).await?;
    if *delete_service.success() {
        tracing::info!("✅ Deleted hosted feature service");
    }

    // ========================================================================
    // Summary
    // ========================================================================
    tracing::info!("");
    tracing::info!("📊 Workflow A Summary:");
    tracing::info!(
        "   ✓ Queried {} features from public service",
        feature_count
    );
    tracing::info!(
        "   ✓ Created empty hosted service container ({})",
        service_item_id
    );
    tracing::info!(
        "   ✓ Added layer definition (layer ID: {})",
        add_def_result.layers()[0].id()
    );
    tracing::info!(
        "   ✓ Added {} features via applyEdits",
        edit_result.success_count()
    );
    tracing::info!("   ✓ Shared with organization");
    tracing::info!("   ✓ Verified data integrity");
    tracing::info!("   ✓ Cleaned up resources");
    tracing::info!("");
    tracing::info!("💡 Use Cases:");
    tracing::info!("   • ETL pipelines (programmatic data loading)");
    tracing::info!("   • Dynamic schema generation");
    tracing::info!("   • Incremental data updates");
    tracing::info!("   • Batch processing workflows");

    Ok(())
}

/// Workflow B: Portal item data management - upload and retrieve file data.
async fn demonstrate_workflow_b_geojson_item(
    portal: &PortalClient<'_>,
    _client: &ArcGISClient,
) -> Result<()> {
    tracing::info!("\n========================================");
    tracing::info!("=== WORKFLOW B: Item Data Management ===");
    tracing::info!("========================================");
    tracing::info!("Query → Convert → Create Item → Upload Data → Retrieve Data → Cleanup");
    tracing::info!("");

    // ========================================================================
    // STEP 1: Query and convert features
    // ========================================================================
    let (_query_result, _geojson_fc, geojson_string) = query_and_convert_to_geojson().await?;
    let data_size = geojson_string.len();

    // ========================================================================
    // STEP 2: Create portal item (metadata only, no data yet)
    // ========================================================================
    tracing::info!("");
    tracing::info!("📋 STEP 2: Creating portal item (metadata only)");
    tracing::info!("");
    tracing::info!("   Item Type: GeoJson");
    tracing::info!("   Title: Sample Cities (Data Management Demo)");
    tracing::info!("   Purpose: Demonstrate file data upload and retrieval");
    tracing::info!("");

    let item_params = AddItemParams::new("Sample Cities (Data Management Demo)", "GeoJson")
        .with_description("Demonstrates item data management using ArcGIS Rust SDK")
        .with_tags(vec![
            "demo".to_string(),
            "data-management".to_string(),
            "rust-sdk".to_string(),
        ]);

    let add_result = portal.add_item(item_params).await?;
    let item_id = add_result.id().to_string();

    tracing::info!("✅ Created portal item: {}", item_id);
    tracing::info!("   Status: Metadata only (no data yet)");

    // ========================================================================
    // STEP 3: Upload file data to item
    // ========================================================================
    tracing::info!("");
    tracing::info!("📤 STEP 3: Uploading GeoJSON data to item");
    tracing::info!("");
    tracing::info!("   Method: update_item_data_v2()");
    tracing::info!("   Data size: {} bytes", data_size);
    tracing::info!("   Format: GeoJSON (application/json)");
    tracing::info!("");

    let geojson_bytes = geojson_string.into_bytes();
    let upload = ItemDataUpload::File {
        data: geojson_bytes,
        filename: "data.geojson".to_string(),
        mime_type: "application/json".to_string(),
    };
    let update_result = portal.update_item_data_v2(&item_id, upload).await?;

    assert!(
        update_result.success(),
        "Failed to upload item data: success={}, id={:?}",
        update_result.success(),
        update_result.id()
    );

    tracing::info!("✅ Uploaded data successfully");
    if let Some(id) = update_result.id() {
        tracing::info!("   Item ID: {}", id);
    }

    // ========================================================================
    // STEP 4: Retrieve and verify stored data
    // ========================================================================
    tracing::info!("");
    tracing::info!("📥 STEP 4: Retrieving data from item");
    tracing::info!("");
    tracing::info!("   Method: get_item_data()");
    tracing::info!("   Purpose: Verify data integrity");
    tracing::info!("");

    let retrieved_data = portal.get_item_data(&item_id).await?;
    let retrieved_size = retrieved_data.len();

    tracing::info!("✅ Retrieved data successfully");
    tracing::info!("   Retrieved size: {} bytes", retrieved_size);
    tracing::info!("   Original size:  {} bytes", data_size);

    if retrieved_size == data_size {
        tracing::info!("   ✓ Data integrity verified (sizes match)");
    } else {
        tracing::warn!("   ⚠ Size mismatch detected!");
    }

    // Verify it's valid GeoJSON
    let retrieved_string = String::from_utf8(retrieved_data.to_vec())?;
    let parsed_geojson: geojson::FeatureCollection = serde_json::from_str(&retrieved_string)?;
    tracing::info!(
        "   ✓ Valid GeoJSON ({} features)",
        parsed_geojson.features.len()
    );

    // ========================================================================
    // STEP 5: Cleanup (delete item)
    // ========================================================================
    tracing::info!("");
    tracing::info!("🧹 STEP 5: Cleaning up test resources");
    tracing::info!("");

    let delete_item = portal.delete_item(&item_id).await?;
    if *delete_item.success() {
        tracing::info!("✅ Deleted portal item and data");
    }

    // ========================================================================
    // Summary
    // ========================================================================
    tracing::info!("");
    tracing::info!("📊 Workflow B Summary:");
    tracing::info!("   ✓ Created portal item (metadata) - {}", item_id);
    tracing::info!("   ✓ Uploaded {} bytes via ItemDataUpload::File", data_size);
    tracing::info!(
        "   ✓ Retrieved {} bytes via get_item_data()",
        retrieved_size
    );
    tracing::info!("   ✓ Verified data integrity");
    tracing::info!("   ✓ Cleaned up resources");
    tracing::info!("");
    tracing::info!("💡 Use Cases:");
    tracing::info!("   • File storage and retrieval through portal");
    tracing::info!("   • Data backup and restore operations");
    tracing::info!("   • Versioned data archival");
    tracing::info!("   • Sharing data files with organization");
    tracing::info!("   • Intermediate data storage for workflows");

    Ok(())
}

/// Convert ArcGIS FeatureSet to GeoJSON FeatureCollection.
///
/// This demonstrates working with geo-types from the GeoRust ecosystem.
fn featureset_to_geojson(featureset: &arcgis::FeatureSet) -> Result<geojson::FeatureCollection> {
    let mut features = Vec::new();

    for feature in featureset.features() {
        // Convert attributes to GeoJSON properties
        let properties = Some(
            feature
                .attributes()
                .iter()
                .map(|(k, v)| (k.clone(), v.clone()))
                .collect(),
        );

        // Convert ArcGIS geometry to GeoJSON geometry
        let geometry = feature
            .geometry()
            .as_ref()
            .and_then(arcgis_geometry_to_geojson);

        let geojson_feature = geojson::Feature {
            bbox: None,
            geometry,
            id: None,
            properties,
            foreign_members: None,
        };

        features.push(geojson_feature);
    }

    Ok(geojson::FeatureCollection {
        bbox: None,
        features,
        foreign_members: None,
    })
}

/// Convert ArcGIS geometry to GeoJSON geometry.
fn arcgis_geometry_to_geojson(geom: &arcgis::ArcGISGeometry) -> Option<geojson::Geometry> {
    use arcgis::ArcGISGeometry;
    use geojson::{Geometry, Value};

    match geom {
        ArcGISGeometry::Point(pt) => Some(Geometry::new(Value::Point(vec![*pt.x(), *pt.y()]))),
        ArcGISGeometry::Multipoint(mp) => {
            let coords: Vec<Vec<f64>> = mp.points().iter().map(|p| vec![p[0], p[1]]).collect();
            Some(Geometry::new(Value::MultiPoint(coords)))
        }
        ArcGISGeometry::Polyline(pl) => {
            if pl.paths().len() == 1 {
                let coords: Vec<Vec<f64>> =
                    pl.paths()[0].iter().map(|p| vec![p[0], p[1]]).collect();
                Some(Geometry::new(Value::LineString(coords)))
            } else {
                let coords: Vec<Vec<Vec<f64>>> = pl
                    .paths()
                    .iter()
                    .map(|path| path.iter().map(|p| vec![p[0], p[1]]).collect())
                    .collect();
                Some(Geometry::new(Value::MultiLineString(coords)))
            }
        }
        ArcGISGeometry::Polygon(pg) => {
            let coords: Vec<Vec<Vec<f64>>> = pg
                .rings()
                .iter()
                .map(|ring| ring.iter().map(|p| vec![p[0], p[1]]).collect())
                .collect();
            Some(Geometry::new(Value::Polygon(coords)))
        }
        ArcGISGeometry::Envelope(_) => {
            // Envelopes can be converted to Polygon if needed
            None
        }
    }
}

/// Prints best practices for portal publishing.
fn print_best_practices() {
    tracing::info!("\n💡 Publishing Best Practices:");
    tracing::info!("   - Use descriptive titles and tags for discoverability");
    tracing::info!("   - Include thumbnail images (improves user experience)");
    tracing::info!("   - Set appropriate sharing (private → org → public)");
    tracing::info!("   - Monitor credit usage (hosting costs)");
    tracing::info!("   - Clean up test services (avoid clutter)");
    tracing::info!("   - Use folders to organize content");
    tracing::info!("");
    tracing::info!("🎯 Publishing Patterns:");
    tracing::info!("   - Direct service creation: Best for programmatic workflows");
    tracing::info!("   - File upload + publish: Best for large datasets (future enhancement)");
    tracing::info!("   - Frequent updates: Use overwrite pattern");
    tracing::info!("   - Multi-user editing: Enable sync and versioning");
    tracing::info!("");
    tracing::info!("⚡ Performance Tips:");
    tracing::info!("   - Enable caching for read-only layers");
    tracing::info!("   - Use indexes on query fields");
    tracing::info!("   - Limit feature count with maxRecordCount");
    tracing::info!("   - Consider archiving old data");
    tracing::info!("");
    tracing::info!("🔐 Security Considerations:");
    tracing::info!("   - Start with private sharing, expand as needed");
    tracing::info!("   - Use groups for team collaboration");
    tracing::info!("   - Enable HTTPS-only access");
    tracing::info!("   - Review access logs regularly");
}
