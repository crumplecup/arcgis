//! 🌳 Branch Versioning Workflow - Complete Lifecycle
//!
//! Demonstrates the complete, self-contained branch versioning workflow from service
//! creation to cleanup. This example creates its own Feature Service with branch
//! versioning enabled, performs edits in isolated versions, reconciles with DEFAULT,
//! and cleans up afterward.
//!
//! # What You'll Learn
//!
//! - **Service creation**: Create branch-versioned Feature Service via Portal API
//! - **URL discovery**: Derive VersionManagementServer URL from FeatureServer URL
//! - **Version lifecycle**: Create named versions for isolated editing
//! - **Edit sessions**: Start/stop editing with transaction semantics
//! - **Feature editing**: Add features within versioned edit sessions
//! - **Reconcile & Post**: Merge changes back to DEFAULT version
//! - **Complete cleanup**: Delete versions and services (idempotent)
//!
//! # Prerequisites
//!
//! - **ArcGIS Online** or **ArcGIS Enterprise** account
//! - **API keys** with appropriate privileges:
//!   - Content Management tier (create/delete services)
//!   - Features tier (edit features)
//!
//! ## Environment Variables
//!
//! Set these in your `.env` file:
//!
//! ```env
//! # Content management operations
//! ARCGIS_CONTENT_KEY=your_content_key
//!
//! # Feature editing operations
//! ARCGIS_FEATURES_KEY=your_features_key
//! ```
//!
//! **That's it!** No manual service setup, no URLs to configure.
//! The example creates and destroys everything it needs.
//!
//! # Running
//!
//! ```bash
//! cargo run --example branch_versioning_workflow
//!
//! # With debug logging:
//! RUST_LOG=debug cargo run --example branch_versioning_workflow
//! ```
//!
//! # Real-World Use Cases
//!
//! - **Utility management**: Planner creates design version for infrastructure updates
//! - **GIS workflows**: Isolated editing for field data collection before publishing
//! - **Multi-user editing**: Team members work in separate versions, reconcile later
//! - **Quality assurance**: Review changes in version before merging to production
//! - **Rollback capability**: Discard entire version if issues found
//!
//! # Branch Versioning Concepts
//!
//! **Branch versioning** is ArcGIS Enterprise's modern versioning system:
//! - Service-based architecture (no direct database access needed)
//! - Flat version hierarchy (all versions branch from DEFAULT)
//! - No compression required
//! - Temporal model with editor tracking
//! - Supports concurrent reconcile/post operations
//!
//! # Workflow Phases
//!
//! 1. **Service Creation** (ARCGIS_CONTENT_KEY)
//!    - Create empty Feature Service via Portal API
//!    - Enable branch versioning on creation
//!    - Extract service URLs from response
//!
//! 2. **Version Management** (ARCGIS_FEATURES_KEY)
//!    - Create named version from DEFAULT
//!    - Start edit session with write lock
//!    - Add/update/delete features in version
//!    - Save or discard changes
//!
//! 3. **Reconcile & Post**
//!    - Reconcile version with DEFAULT (detect conflicts)
//!    - Resolve conflicts if any
//!    - Post changes to DEFAULT version
//!
//! 4. **Cleanup**
//!    - Delete named version
//!    - Delete Feature Service (ARCGIS_CONTENT_KEY)
//!    - No trace left - fully idempotent
//!
//! # Why This Example Is Special
//!
//! Unlike other examples that require pre-configured services:
//! - ✅ **Zero manual setup** - just API keys
//! - ✅ **Idempotent** - run repeatedly without conflicts
//! - ✅ **Complete lifecycle** - create, use, destroy
//! - ✅ **Production patterns** - shows real-world workflows
//! - ✅ **Self-documenting** - demonstrates all key operations

use anyhow::{Context, Result};
use arcgis::{
    ApiKeyAuth, ApiKeyTier, ArcGISClient, ArcGISGeometry, ArcGISPoint, ConflictDetection,
    CreateServiceParams, CreateVersionParams, EditOptions, Feature, FeatureServiceClient,
    FieldDefinitionBuilder, FieldType, GeometryTypeDefinition, LayerDefinitionBuilder, LayerId,
    ObjectId, PortalClient, ServiceDefinitionBuilder, SessionId, VersionGuid,
    VersionManagementClient, VersionPermission,
};
use serde_json::json;
use std::collections::HashMap;
use uuid::Uuid;

/// Service configuration extracted from creation response.
struct ServiceInfo {
    item_id: String,
    feature_service_url: String,
    version_mgmt_url: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing for structured logging
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    tracing::info!("🌳 Branch Versioning Workflow Example");
    tracing::info!("Complete lifecycle: Create → Edit → Reconcile → Post → Cleanup");
    tracing::info!("");

    // Execute complete workflow with automatic cleanup
    match execute_workflow().await {
        Ok(_) => {
            tracing::info!("\n✅ Branch versioning workflow completed successfully!");
            print_summary();
        }
        Err(e) => {
            tracing::error!("❌ Workflow failed: {}", e);
            return Err(e);
        }
    }

    Ok(())
}

/// Executes the complete branch versioning workflow.
async fn execute_workflow() -> Result<()> {
    // Phase 1: Create Feature Service
    let service = create_branch_versioned_service().await?;

    // Ensure cleanup happens even if later phases fail
    let cleanup_result = perform_versioning_operations(&service).await;

    // Phase 5: Cleanup - always delete service
    delete_service(&service.item_id).await?;

    cleanup_result
}

/// Phase 1: Creates a branch-versioned Feature Service.
async fn create_branch_versioned_service() -> Result<ServiceInfo> {
    tracing::info!("=== Phase 1: Service Creation ===");
    tracing::info!("Creating temporary Feature Service with branch versioning");
    tracing::info!("");

    // Create client with ARCGIS_CONTENT_KEY
    let content_auth = ApiKeyAuth::from_env(ApiKeyTier::Content)
        .context("Missing ARCGIS_CONTENT_KEY - add to .env for content management")?;
    let content_client = ArcGISClient::new(content_auth);
    let portal = PortalClient::new("https://www.arcgis.com/sharing/rest", &content_client);

    // Generate unique service name
    let unique_name = format!("demo_versioning_{}", Uuid::new_v4().simple());
    tracing::info!("   Service name: {}", unique_name);

    // Build service definition with branch versioning requirements
    tracing::info!("   Creating service definition with branch versioning");

    // ObjectID field (required for all feature services)
    let oid_field = FieldDefinitionBuilder::default()
        .name("OBJECTID")
        .field_type(FieldType::Oid)
        .alias("Object ID")
        .nullable(false)
        .editable(false)
        .build()
        .context("Failed to build ObjectID field")?;

    // GlobalID field (required for branch versioning)
    let globalid_field = FieldDefinitionBuilder::default()
        .name("GlobalID")
        .field_type(FieldType::GlobalId)
        .alias("Global ID")
        .nullable(false)
        .editable(false)
        .length(38)
        .build()
        .context("Failed to build GlobalID field")?;

    // User fields for the demo
    let name_field = FieldDefinitionBuilder::default()
        .name("NAME")
        .field_type(FieldType::String)
        .alias("Name")
        .length(255)
        .nullable(true)
        .editable(true)
        .build()
        .context("Failed to build NAME field")?;

    let description_field = FieldDefinitionBuilder::default()
        .name("DESCRIPTION")
        .field_type(FieldType::String)
        .alias("Description")
        .length(1024)
        .nullable(true)
        .editable(true)
        .build()
        .context("Failed to build DESCRIPTION field")?;

    let value_field = FieldDefinitionBuilder::default()
        .name("VALUE")
        .field_type(FieldType::Integer)
        .alias("Value")
        .nullable(true)
        .editable(true)
        .build()
        .context("Failed to build VALUE field")?;

    // Create branch-versioned layer
    let layer = LayerDefinitionBuilder::default()
        .id(0u32)
        .name("DemoPoints")
        .geometry_type(GeometryTypeDefinition::Point)
        .object_id_field("OBJECTID")
        .global_id_field("GlobalID")
        .display_field("NAME")
        .fields(vec![
            oid_field,
            globalid_field,
            name_field,
            description_field,
            value_field,
        ])
        .is_data_branch_versioned(true)
        .build()
        .context("Failed to build layer definition")?;

    // Create service definition
    let service_def = ServiceDefinitionBuilder::default()
        .name(&unique_name)
        .service_description("Temporary branch versioning demo - will be deleted")
        .capabilities("Query,Create,Update,Delete,Editing")
        .max_record_count(2000)
        .allow_geometry_updates(true)
        .layers(vec![layer])
        .build()
        .context("Failed to build service definition")?;

    tracing::info!("   ✅ Service definition built with branch versioning enabled");

    // Create service with strongly-typed definition
    let service_params = CreateServiceParams::new(&unique_name)
        .with_description("Temporary branch versioning demo - will be deleted")
        .with_capabilities("Query,Create,Update,Delete,Editing")
        .with_service_definition(service_def);

    let create_result = portal
        .create_service(service_params)
        .await
        .context("Failed to create Feature Service")?;

    if !create_result.success() {
        anyhow::bail!("Service creation reported failure");
    }

    // Extract service URLs
    let item_id = create_result
        .service_item_id()
        .as_ref()
        .context("No service item ID returned")?
        .clone();

    let feature_service_url = create_result
        .service_url()
        .as_ref()
        .context("No service URL returned")?
        .clone();

    // Derive VersionManagementServer URL
    let version_mgmt_url = feature_service_url.replace("FeatureServer", "VersionManagementServer");

    tracing::info!("✅ Feature Service created:");
    tracing::info!("   Item ID: {}", item_id);
    tracing::info!("   FeatureServer: {}", feature_service_url);
    tracing::info!("   VersionManagementServer: {}", version_mgmt_url);
    tracing::info!("");

    Ok(ServiceInfo {
        item_id,
        feature_service_url,
        version_mgmt_url,
    })
}

/// Phase 2-4: Performs version management and editing operations.
async fn perform_versioning_operations(service: &ServiceInfo) -> Result<()> {
    tracing::info!("=== Phase 2: Client Setup ===");
    tracing::info!("Initializing clients for feature editing");
    tracing::info!("");

    // Create client with ARCGIS_FEATURES_KEY
    let features_auth = ApiKeyAuth::from_env(ApiKeyTier::Features)
        .context("Missing ARCGIS_FEATURES_KEY - add to .env for feature editing")?;
    let features_client = ArcGISClient::new(features_auth);

    let feature_client = FeatureServiceClient::new(&service.feature_service_url, &features_client);
    let vm_client = VersionManagementClient::new(&service.version_mgmt_url, &features_client);

    tracing::info!("✅ Clients initialized");
    tracing::info!("");

    // Phase 3: Version Creation and Editing
    tracing::info!("=== Phase 3: Version Creation & Editing ===");
    tracing::info!("Creating named version and performing edits");
    tracing::info!("");

    let version_guid = create_and_edit_version(&vm_client, &feature_client).await?;

    // Phase 4: Reconcile and Post
    tracing::info!("=== Phase 4: Reconcile & Post ===");
    tracing::info!("Merging changes back to DEFAULT version");
    tracing::info!("");

    reconcile_and_post(&vm_client, version_guid).await?;

    // Cleanup: Delete the version
    tracing::info!("🗑️  Deleting named version: {}", version_guid);
    vm_client.delete(version_guid).await?;
    tracing::info!("✅ Version deleted");
    tracing::info!("");

    Ok(())
}

/// Creates a version, performs edits, and returns the version GUID.
async fn create_and_edit_version(
    vm_client: &VersionManagementClient<'_>,
    feature_client: &FeatureServiceClient<'_>,
) -> Result<VersionGuid> {
    // Step 1: Create named version
    let version_name = format!(
        "edit_branch_{}",
        &Uuid::new_v4().simple().to_string()[..8]
    );
    tracing::info!("   Creating version: {}", version_name);

    let create_params = CreateVersionParams::new(&version_name, VersionPermission::Private)
        .with_description("Temporary editing version");

    let create_response = vm_client
        .create(create_params)
        .await
        .context("Failed to create version")?;

    if !create_response.success() {
        anyhow::bail!("Version creation failed: {:?}", create_response.error());
    }

    let version_info = create_response
        .version_info()
        .as_ref()
        .context("No version info in response")?;

    let version_uuid =
        Uuid::parse_str(version_info.version_guid()).context("Invalid version GUID format")?;
    let version_guid: VersionGuid = version_uuid.into();
    tracing::info!("   Version GUID: {}", version_guid);
    tracing::info!("");

    // Step 2: Start edit session
    tracing::info!("   Starting edit session");
    let session_id = SessionId::new();
    tracing::info!("   Session ID: {}", session_id);

    let start_response = vm_client
        .start_editing(version_guid, session_id)
        .await
        .context("Failed to start edit session")?;

    if !start_response.success() {
        anyhow::bail!("Start editing failed: {:?}", start_response.error());
    }
    tracing::info!("✅ Edit session started");
    tracing::info!("");

    // Step 3: Add features
    tracing::info!("   Adding features to version");
    let object_id = add_demo_feature(feature_client, session_id).await?;
    tracing::info!("   Added feature with ObjectID: {}", object_id.0);
    tracing::info!("");

    // Step 4: Save changes (stop editing with save=true)
    tracing::info!("   Saving changes and stopping edit session");
    let stop_response = vm_client
        .stop_editing(version_guid, session_id, true)
        .await
        .context("Failed to stop edit session")?;

    if !stop_response.success() {
        anyhow::bail!("Stop editing failed: {:?}", stop_response.error());
    }
    tracing::info!("✅ Changes saved to version");
    tracing::info!("");

    Ok(version_guid)
}

/// Adds a demonstration feature to the service.
async fn add_demo_feature(
    feature_client: &FeatureServiceClient<'_>,
    session_id: SessionId,
) -> Result<ObjectId> {
    // Create feature attributes
    let mut attributes = HashMap::new();
    attributes.insert("NAME".to_string(), json!("Demo Feature"));
    attributes.insert(
        "DESCRIPTION".to_string(),
        json!("Created by branch versioning workflow"),
    );
    attributes.insert("VALUE".to_string(), json!(42));

    // Create simple point geometry (Los Angeles)
    let point = ArcGISPoint::new(-118.2437, 34.0522);
    let geometry = ArcGISGeometry::Point(point);

    let feature = Feature::new(attributes, Some(geometry));

    // Add feature within edit session
    let edit_options = EditOptions::new()
        .with_session_id(session_id)
        .with_rollback_on_failure(true)
        .with_return_edit_results(true);

    let add_result = feature_client
        .add_features(LayerId::new(0), vec![feature], edit_options)
        .await
        .context("Failed to add feature")?;

    // Extract ObjectID
    let object_id = add_result
        .add_results()
        .first()
        .context("No add results returned")?
        .object_id()
        .as_ref()
        .copied()
        .context("Added feature has no ObjectID")?;

    Ok(object_id)
}

/// Reconciles the version with DEFAULT and posts changes.
async fn reconcile_and_post(
    vm_client: &VersionManagementClient<'_>,
    version_guid: VersionGuid,
) -> Result<()> {
    // Need a new session for reconcile/post
    let session_id = SessionId::new();

    // Start editing session (required for reconcile/post)
    vm_client.start_editing(version_guid, session_id).await?;

    // Step 1: Reconcile with DEFAULT
    tracing::info!("   Reconciling with DEFAULT version");
    let reconcile_response = vm_client
        .reconcile(
            version_guid,
            session_id,
            true, // abort_if_conflicts
            ConflictDetection::ByObject,
            false, // don't auto-post
        )
        .await
        .context("Failed to reconcile")?;

    if !reconcile_response.success() {
        anyhow::bail!("Reconcile failed: {:?}", reconcile_response.error());
    }

    // Check for conflicts
    let has_conflicts = reconcile_response
        .has_conflicts()
        .as_ref()
        .is_some_and(|x| *x);

    if has_conflicts {
        tracing::warn!("⚠️  Conflicts detected during reconcile");
        tracing::warn!("   In production, you would resolve conflicts here");
        vm_client
            .stop_editing(version_guid, session_id, false)
            .await?;
        anyhow::bail!("Cannot post with unresolved conflicts");
    }

    tracing::info!("✅ Reconcile successful - no conflicts");
    tracing::info!("");

    // Step 2: Post changes to DEFAULT
    tracing::info!("   Posting changes to DEFAULT version");
    let post_response = vm_client
        .post(version_guid, session_id, None)
        .await
        .context("Failed to post changes")?;

    if !post_response.success() {
        anyhow::bail!("Post failed: {:?}", post_response.error());
    }

    tracing::info!("✅ Changes posted to DEFAULT");
    tracing::info!("");

    // Stop editing session
    vm_client
        .stop_editing(version_guid, session_id, true)
        .await?;

    Ok(())
}

/// Phase 5: Deletes the Feature Service.
async fn delete_service(item_id: &str) -> Result<()> {
    tracing::info!("=== Phase 5: Service Cleanup ===");
    tracing::info!("Deleting Feature Service");
    tracing::info!("");

    // Create client with ARCGIS_CONTENT_KEY
    let content_auth = ApiKeyAuth::from_env(ApiKeyTier::Content)
        .context("Missing ARCGIS_CONTENT_KEY for cleanup")?;
    let content_client = ArcGISClient::new(content_auth);
    let portal = PortalClient::new("https://www.arcgis.com/sharing/rest", &content_client);

    tracing::info!("   Deleting service item: {}", item_id);
    let delete_result = portal
        .delete_item(item_id)
        .await
        .context("Failed to delete service")?;

    if !delete_result.success() {
        anyhow::bail!("Service deletion failed");
    }

    tracing::info!("✅ Service deleted - no trace left");
    tracing::info!("");

    Ok(())
}

/// Prints workflow summary and best practices.
fn print_summary() {
    tracing::info!("📋 Workflow Summary:");
    tracing::info!("   1. Created Feature Service with branch versioning enabled");
    tracing::info!("   2. Derived VersionManagementServer URL from FeatureServer URL");
    tracing::info!("   3. Created named version from DEFAULT");
    tracing::info!("   4. Started edit session with write lock");
    tracing::info!("   5. Added features within the version");
    tracing::info!("   6. Saved changes to the named version");
    tracing::info!("   7. Reconciled version with DEFAULT (no conflicts)");
    tracing::info!("   8. Posted changes to DEFAULT version");
    tracing::info!("   9. Deleted named version");
    tracing::info!("   10. Deleted Feature Service (complete cleanup)");
    tracing::info!("");
    tracing::info!("💡 Key Concepts:");
    tracing::info!("   - Branch versioning uses service-based architecture");
    tracing::info!("   - All versions branch directly from DEFAULT (flat hierarchy)");
    tracing::info!("   - Edit sessions provide write locks and transaction semantics");
    tracing::info!("   - Reconcile detects conflicts before posting");
    tracing::info!("   - Post merges changes from named version to DEFAULT");
    tracing::info!("");
    tracing::info!("🎯 Production Patterns:");
    tracing::info!("   - Always use unique version names (prevent collisions)");
    tracing::info!("   - Handle conflicts gracefully (inspect and resolve)");
    tracing::info!("   - Clean up versions after posting (prevent clutter)");
    tracing::info!("   - Use rollbackOnFailure for transactional edits");
    tracing::info!("   - Monitor session timeouts (server-configured)");
    tracing::info!("");
    tracing::info!("🚀 Next Steps:");
    tracing::info!("   - Explore conflict resolution workflows");
    tracing::info!("   - Implement multi-user editing scenarios");
    tracing::info!("   - Add validation before posting");
    tracing::info!("   - Integrate with approval workflows");
}
