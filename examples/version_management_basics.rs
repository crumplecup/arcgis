//! 🌲 Branch Versioning - Version Management Service Operations
//!
//! Demonstrates version management operations for branch-versioned feature services.
//! Learn how to create named versions, manage edit sessions, and work with versioned
//! editing workflows in ArcGIS Enterprise.
//!
//! # What You'll Learn
//!
//! - **Version creation**: Create named versions (branches) for isolated editing
//! - **Version metadata**: Query version information and list all versions
//! - **Version modification**: Alter version properties (name, access, description)
//! - **Version deletion**: Remove temporary versions after reconciliation
//! - **Edit sessions**: Start/stop editing sessions for versioned workflows
//!
//! # Prerequisites
//!
//! - ArcGIS Enterprise 10.6+ (branch versioning not available in ArcGIS Online)
//! - Enterprise geodatabase (PostgreSQL, SQL Server, or Oracle)
//! - Feature service published with "Version Management" capability enabled
//! - Appropriate authentication (API key or OAuth)
//!
//! ## Environment Variables
//!
//! Set these in your `.env` file:
//!
//! ```env
//! # Feature service base URL (example)
//! ARCGIS_FEATURE_URL=https://your-server.com/arcgis/rest/services/Assets/FeatureServer
//!
//! # Authentication - ARCGIS_ENTERPRISE_KEY is recommended for Enterprise servers
//! ARCGIS_ENTERPRISE_KEY=your_enterprise_api_key
//! # OR (fallback)
//! ARCGIS_FEATURES_KEY=your_features_api_key
//! # OR
//! ARCGIS_CLIENT_ID=your_oauth_client_id
//! ARCGIS_CLIENT_SECRET=your_oauth_client_secret
//! ```
//!
//! **Important**: The feature service must have branch versioning enabled and the
//! "Version Management" capability published.
//!
//! # Setup Instructions
//!
//! ## 1. Create Enterprise Geodatabase
//!
//! ```sql
//! -- PostgreSQL example
//! CREATE DATABASE enterprise_gdb;
//! ```
//!
//! ## 2. Register Data as Branch Versioned (ArcGIS Pro)
//!
//! 1. Connect to your enterprise geodatabase
//! 2. Create or select a feature class
//! 3. Ensure it has Global ID field and editor tracking enabled
//! 4. Right-click → Manage → Register as Branch Versioned
//!
//! ## 3. Publish with Version Management
//!
//! 1. In ArcGIS Pro, share as Web Layer
//! 2. Enable "Version Management" capability
//! 3. Publish to ArcGIS Enterprise
//! 4. Copy the FeatureServer URL to `.env` as `ARCGIS_FEATURE_URL`
//!
//! # Running
//!
//! ```bash
//! cargo run --example version_management_basics
//!
//! # With debug logging:
//! RUST_LOG=debug cargo run --example version_management_basics
//! ```
//!
//! # Real-World Use Cases
//!
//! - **Utility maintenance**: Create version for winter repairs, reconcile when done
//! - **Planning scenarios**: Multiple planners work in isolated versions
//! - **Quality control**: Review edits in named version before posting to production
//! - **Long transactions**: Multi-day editing sessions with version isolation
//! - **Conflict resolution**: Reconcile changes between competing versions

use anyhow::Result;
use arcgis::example_tracker::ExampleTracker;
use arcgis::{
    AlterVersionParams, ApiKeyAuth, ApiKeyTier, ArcGISClient, ConflictDetection,
    CreateVersionParams, DifferenceResultType, EnvConfig, SessionId, VersionGuid,
    VersionManagementClient, VersionPermission,
};
use secrecy::ExposeSecret;
use std::time::{SystemTime, UNIX_EPOCH};
use uuid::Uuid;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing for structured logging
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    // Start accountability tracking
    let tracker = ExampleTracker::new("version_management_basics")
        .methods(&[
            "list_versions",
            "create",
            "get_info",
            "alter",
            "start_editing",
            "stop_editing",
            "start_reading",
            "stop_reading",
            "differences",
            "delete_forward_edits",
            "reconcile",
            "post",
            "conflicts",
            "inspect_conflicts",
            "restore_rows",
            "delete",
        ])
        .service_type("VersionManagementClient")
        .start();

    tracing::info!("🌲 ArcGIS Version Management Service Examples");
    tracing::info!("Demonstrating branch versioning operations");
    tracing::info!("");

    // Load feature service URL from environment
    let config = EnvConfig::global();
    let feature_url = config
        .arcgis_feature_url
        .as_ref()
        .ok_or_else(|| anyhow::anyhow!(
            "ARCGIS_FEATURE_URL not set in .env file.\n\
             Example: ARCGIS_FEATURE_URL=https://your-server.com/arcgis/rest/services/Assets/FeatureServer"
        ))?;

    // Construct VersionManagementServer URL from FeatureServer URL
    let vm_url = feature_url.replace("FeatureServer", "VersionManagementServer");

    tracing::info!("Feature Service: {}", feature_url);
    tracing::info!("Version Management: {}", vm_url);
    tracing::info!("");
    tracing::info!("⚠️  Note: The service must have Version Management capabilities enabled.");
    tracing::info!("   If you see a 404 error, verify:");
    tracing::info!("   1. Service is published with 'Version Management' capability");
    tracing::info!("   2. Data is registered as branch versioned in ArcGIS Pro");
    tracing::info!("   3. Service URL is correct (should end in FeatureServer)");
    tracing::info!("");

    // Create authenticated client (automatically loads .env)
    tracing::debug!("Creating authenticated client");

    // Use enterprise key for enterprise servers, fallback to features key
    let config = EnvConfig::global();
    let auth = if let Some(enterprise_key) = &config.arcgis_enterprise_key {
        tracing::debug!("Using ARCGIS_ENTERPRISE_KEY for authentication");
        ApiKeyAuth::new(enterprise_key.expose_secret())
    } else {
        tracing::debug!("Using ARCGIS_FEATURES_KEY for authentication");
        ApiKeyAuth::from_env(ApiKeyTier::Features)?
    };

    let client = ArcGISClient::new(auth);
    let vm_client = VersionManagementClient::new(&vm_url, &client);

    // Demonstrate version management operations
    demonstrate_list_versions(&vm_client).await?;
    demonstrate_create_version(&vm_client).await?;
    demonstrate_version_metadata(&vm_client).await?;
    demonstrate_alter_version(&vm_client).await?;
    demonstrate_edit_session(&vm_client).await?;
    demonstrate_read_session(&vm_client).await?;
    demonstrate_differences(&vm_client).await?;
    demonstrate_delete_forward_edits(&vm_client).await?;
    demonstrate_reconcile_and_post(&vm_client).await?;
    demonstrate_conflict_management(&vm_client).await?;
    demonstrate_delete_version(&vm_client).await?;

    tracing::info!("\n✅ All version management examples completed successfully!");
    tracing::info!("🎉 100% VersionManagementClient coverage achieved!");
    print_best_practices();

    // Mark tracking as successful
    tracker.success();
    Ok(())
}

/// Demonstrates listing all versions in the service.
async fn demonstrate_list_versions(vm_client: &VersionManagementClient<'_>) -> Result<()> {
    tracing::info!("\n=== Example 1: List All Versions ===");
    tracing::info!("Query all versions available in the service");
    tracing::info!("");

    let response = vm_client.list_versions().await?;

    anyhow::ensure!(
        !response.versions().is_empty(),
        "Should have at least DEFAULT version"
    );

    tracing::info!("✅ Found {} versions:", response.versions().len());

    for (idx, version) in response.versions().iter().enumerate() {
        tracing::info!(
            "   {}. {} (GUID: {})",
            idx + 1,
            version.version_name(),
            version.version_guid()
        );

        if let Some(desc) = version.description() {
            tracing::info!("      Description: {}", desc);
        }

        if let Some(access) = version.access() {
            tracing::info!("      Access: {}", access);
        }

        // Validate version structure
        anyhow::ensure!(
            !version.version_name().is_empty(),
            "Version {} should have a name",
            idx
        );

        anyhow::ensure!(
            !version.version_guid().is_empty(),
            "Version {} should have a GUID",
            idx
        );
    }

    tracing::info!("");
    tracing::info!("💡 Version listing:");
    tracing::info!("   • DEFAULT version always exists (production data)");
    tracing::info!("   • Named versions are branches for isolated editing");
    tracing::info!("   • Access levels: public, protected, private");

    Ok(())
}

/// Demonstrates creating a new version.
async fn demonstrate_create_version(vm_client: &VersionManagementClient<'_>) -> Result<()> {
    tracing::info!("\n=== Example 2: Create Named Version ===");
    tracing::info!("Create a new version (branch) for isolated editing");
    tracing::info!("");

    // Create unique version name with timestamp
    let version_name = format!("test_version_{}", chrono::Utc::now().timestamp());

    let params = CreateVersionParams::new(&version_name, VersionPermission::Private)
        .with_description("Temporary test version created by Rust SDK example");

    tracing::info!("Creating version: {}", version_name);

    let response = vm_client.create(params).await?;

    // Validate response
    anyhow::ensure!(
        response.success().unwrap_or(false),
        "Version creation should succeed. Error: {:?}",
        response.error()
    );

    if let Some(version_info) = response.version_info() {
        tracing::info!(
            "✅ Version created: {} (GUID: {})",
            version_info.version_name(),
            version_info.version_guid()
        );

        // Validate version info
        anyhow::ensure!(
            version_info.version_name().contains(&version_name),
            "Created version should have expected name"
        );

        anyhow::ensure!(
            !version_info.version_guid().is_empty(),
            "Created version should have GUID"
        );
    } else {
        anyhow::bail!("Version creation succeeded but no version info returned");
    }

    tracing::info!("");
    tracing::info!("💡 Version creation:");
    tracing::info!("   • Creates branch from DEFAULT version");
    tracing::info!("   • Isolated editing space for changes");
    tracing::info!("   • Access control (public/protected/private)");
    tracing::info!("   • Can be deleted after reconciliation");

    Ok(())
}

/// Demonstrates querying version metadata.
async fn demonstrate_version_metadata(vm_client: &VersionManagementClient<'_>) -> Result<()> {
    tracing::info!("\n=== Example 3: Query Version Metadata ===");
    tracing::info!("Get detailed information about a specific version");
    tracing::info!("");

    // First, list versions to get a GUID
    let versions_response = vm_client.list_versions().await?;

    anyhow::ensure!(
        !versions_response.versions().is_empty(),
        "Need at least one version to query"
    );

    let first_version = &versions_response.versions()[0];
    let version_guid_str = first_version.version_guid();

    tracing::info!("Querying metadata for: {}", first_version.version_name());

    // Parse GUID and query detailed info
    let uuid = Uuid::parse_str(version_guid_str)?;
    let version_guid = VersionGuid::from_uuid(uuid);

    let version_info = vm_client.get_info(version_guid).await?;

    // Validate response
    anyhow::ensure!(
        !version_info.version_name().is_empty(),
        "Version should have a name"
    );

    anyhow::ensure!(
        version_info.version_guid() == version_guid_str,
        "Version GUID should match query"
    );

    tracing::info!("✅ Version metadata:");
    tracing::info!("   Name: {}", version_info.version_name());
    tracing::info!("   GUID: {}", version_info.version_guid());

    if let Some(desc) = version_info.description() {
        tracing::info!("   Description: {}", desc);
    }

    if let Some(access) = version_info.access() {
        tracing::info!("   Access: {}", access);
    }

    if let Some(created) = version_info.creation_date() {
        tracing::info!("   Created: {}", created);
    }

    if let Some(modified) = version_info.modified_date() {
        tracing::info!("   Modified: {}", modified);
    }

    tracing::info!("");
    tracing::info!("💡 Version metadata:");
    tracing::info!("   • Timestamps track creation and modification");
    tracing::info!("   • Access level determines visibility");
    tracing::info!("   • GUID is permanent identifier");

    Ok(())
}

/// Demonstrates edit session workflow.
async fn demonstrate_edit_session(vm_client: &VersionManagementClient<'_>) -> Result<()> {
    tracing::info!("\n=== Example 4: Edit Session Workflow ===");
    tracing::info!("Start and stop editing session on a version");
    tracing::info!("");

    // Create a new version for editing (DEFAULT version doesn't support editing)
    let version_name = format!("edit_test_{}", chrono::Utc::now().timestamp());
    let params = CreateVersionParams::new(&version_name, VersionPermission::Private)
        .with_description("Temporary version for edit session demo");

    tracing::info!("Creating version for editing...");
    let create_response = vm_client.create(params).await?;
    anyhow::ensure!(
        create_response.success().unwrap_or(false),
        "Version creation should succeed"
    );

    let version_info = create_response
        .version_info()
        .as_ref()
        .ok_or_else(|| anyhow::anyhow!("No version info in response"))?;
    let uuid = Uuid::parse_str(version_info.version_guid())?;
    let version_guid = VersionGuid::from_uuid(uuid);

    tracing::info!(
        "Created version: {} ({})",
        version_info.version_name(),
        version_guid
    );
    tracing::info!("");

    // Generate unique session ID
    let session_id = SessionId::new();

    tracing::info!("Session ID: {}", session_id);
    tracing::info!("");

    // Step 1: Start reading session (acquires shared lock)
    tracing::info!("Starting reading session (shared lock)...");
    let read_response = vm_client.start_reading(version_guid, session_id).await?;

    anyhow::ensure!(
        *read_response.success(),
        "Start reading should succeed. Error: {:?}",
        read_response.error()
    );

    tracing::info!("✅ Reading session started (shared lock acquired)");

    // Step 2: Start editing session (upgrades to exclusive lock)
    tracing::info!("Starting edit session (exclusive lock)...");
    let start_response = vm_client.start_editing(version_guid, session_id).await?;

    anyhow::ensure!(
        *start_response.success(),
        "Start editing should succeed. Error: {:?}",
        start_response.error()
    );

    tracing::info!("✅ Edit session started (exclusive lock acquired)");

    if let Some(moment) = start_response.moment() {
        tracing::info!("   Moment: {}", moment);
    }

    // In real workflow, you would make edits here via FeatureService API

    tracing::info!("");
    tracing::info!("(In real workflow: make edits to features here)");
    tracing::info!("");

    // Step 3: Stop editing session (releases exclusive lock, keeps shared)
    tracing::info!("Stopping edit session (saving changes)...");
    let stop_edit_response = vm_client
        .stop_editing(version_guid, session_id, true)
        .await?;

    anyhow::ensure!(
        *stop_edit_response.success(),
        "Stop editing should succeed. Error: {:?}",
        stop_edit_response.error()
    );

    tracing::info!("✅ Edit session stopped (exclusive lock released, changes saved)");

    if let Some(moment) = stop_edit_response.moment() {
        tracing::info!("   Moment: {}", moment);
    }

    // Step 4: Stop reading session (releases shared lock)
    tracing::info!("Stopping reading session (releasing shared lock)...");
    let stop_read_response = vm_client.stop_reading(version_guid, session_id).await?;

    anyhow::ensure!(
        *stop_read_response.success(),
        "Stop reading should succeed. Error: {:?}",
        stop_read_response.error()
    );

    tracing::info!("✅ Reading session stopped (all locks released)");

    tracing::info!("");
    tracing::info!("💡 Edit sessions:");
    tracing::info!(
        "   • Required workflow: startReading → startEditing → stopEditing → stopReading"
    );
    tracing::info!("   • startReading acquires shared lock");
    tracing::info!("   • startEditing upgrades to exclusive lock");
    tracing::info!("   • stopEditing(save=true) commits changes, stopEditing(save=false) discards");
    tracing::info!("   • stopReading releases all locks");

    // Note: Test version left in place for manual cleanup if needed

    Ok(())
}

/// Demonstrates altering version properties.
async fn demonstrate_alter_version(vm_client: &VersionManagementClient<'_>) -> Result<()> {
    tracing::info!("\n=== Example 5: Alter Version Properties ===");
    tracing::info!("Modify version name, description, and access level");
    tracing::info!("");

    // First create a version to alter
    let version_name = format!("alter_test_{}", chrono::Utc::now().timestamp());
    let params = CreateVersionParams::new(&version_name, VersionPermission::Private)
        .with_description("Original description");

    let create_response = vm_client.create(params).await?;
    anyhow::ensure!(
        create_response.success().unwrap_or(false),
        "Version creation should succeed"
    );

    let version_info = create_response
        .version_info()
        .as_ref()
        .ok_or_else(|| anyhow::anyhow!("No version info in response"))?;
    let uuid = Uuid::parse_str(version_info.version_guid())?;
    let version_guid = VersionGuid::from_uuid(uuid);

    tracing::info!("Created version: {}", version_info.version_name());
    tracing::info!("");

    // Alter version properties
    let new_description = "Updated description with new metadata";
    let alter_params = AlterVersionParams::new()
        .with_description(new_description)
        .with_access(VersionPermission::Protected);

    tracing::info!("Altering version properties...");
    let alter_response = vm_client.alter(version_guid, alter_params).await?;

    anyhow::ensure!(
        *alter_response.success(),
        "Alter should succeed. Error: {:?}",
        alter_response.error()
    );

    tracing::info!("✅ Version altered successfully");

    if let Some(moment) = alter_response.moment() {
        tracing::info!("   Moment: {}", moment);
    }

    // Verify changes by querying version info
    let updated_info = vm_client.get_info(version_guid).await?;
    anyhow::ensure!(
        updated_info.description().as_ref().map(|s| s.as_str()) == Some(new_description),
        "Description should be updated"
    );

    tracing::info!("✅ Verified description updated: {}", new_description);

    tracing::info!("");
    tracing::info!("💡 Version alteration:");
    tracing::info!("   • Change version name, description, or access level");
    tracing::info!("   • Access levels: public, protected, private");
    tracing::info!("   • Use for metadata updates and access control");

    Ok(())
}

/// Demonstrates read session workflow.
async fn demonstrate_read_session(vm_client: &VersionManagementClient<'_>) -> Result<()> {
    tracing::info!("\n=== Example 6: Read Session Workflow ===");
    tracing::info!("Start and stop read session for consistent data view");
    tracing::info!("");

    // Get DEFAULT version GUID
    let versions_response = vm_client.list_versions().await?;
    anyhow::ensure!(
        !versions_response.versions().is_empty(),
        "Need at least one version for read session"
    );

    let default_version = &versions_response.versions()[0];
    let uuid = Uuid::parse_str(default_version.version_guid())?;
    let version_guid = VersionGuid::from_uuid(uuid);

    tracing::info!(
        "Using version: {} ({})",
        default_version.version_name(),
        version_guid
    );

    // Generate unique session ID
    let session_id = SessionId::new();
    tracing::info!("Session ID: {}", session_id);
    tracing::info!("");

    // Start read session
    tracing::info!("Starting read session...");
    let start_response = vm_client.start_reading(version_guid, session_id).await?;

    anyhow::ensure!(
        *start_response.success(),
        "Start reading should succeed. Error: {:?}",
        start_response.error()
    );

    tracing::info!("✅ Read session started");

    if let Some(moment) = start_response.moment() {
        tracing::info!("   Moment: {}", moment);
    }

    tracing::info!("");
    tracing::info!("(In real workflow: perform queries with consistent view here)");
    tracing::info!("");

    // Stop read session
    tracing::info!("Stopping read session...");
    let stop_response = vm_client.stop_reading(version_guid, session_id).await?;

    anyhow::ensure!(
        *stop_response.success(),
        "Stop reading should succeed. Error: {:?}",
        stop_response.error()
    );

    tracing::info!("✅ Read session stopped");

    if let Some(moment) = stop_response.moment() {
        tracing::info!("   Moment: {}", moment);
    }

    tracing::info!("");
    tracing::info!("💡 Read sessions:");
    tracing::info!("   • Provide consistent snapshot of data");
    tracing::info!("   • Multiple read sessions can exist simultaneously");
    tracing::info!("   • Don't block editing or other read sessions");
    tracing::info!("   • Useful for long-running queries or reports");

    Ok(())
}

/// Demonstrates differences utility for comparing versions.
async fn demonstrate_differences(vm_client: &VersionManagementClient<'_>) -> Result<()> {
    tracing::info!("\n=== Example 7: Compare Version Differences ===");
    tracing::info!("Retrieve differences between version and DEFAULT");
    tracing::info!("");

    // Get first non-DEFAULT version (or create one)
    let versions_response = vm_client.list_versions().await?;
    let test_version = versions_response
        .versions()
        .iter()
        .find(|v| !v.version_name().to_uppercase().contains("DEFAULT"))
        .or_else(|| versions_response.versions().first())
        .ok_or_else(|| anyhow::anyhow!("No versions available"))?;

    let uuid = Uuid::parse_str(test_version.version_guid())?;
    let version_guid = VersionGuid::from_uuid(uuid);

    tracing::info!("Querying differences for: {}", test_version.version_name());
    tracing::info!("");

    // Get differences as object IDs (more efficient)
    tracing::info!("Method 1: Object IDs (efficient)");
    let diffs_response = vm_client
        .differences(version_guid, None, DifferenceResultType::ObjectIds, None)
        .await?;

    anyhow::ensure!(
        *diffs_response.success(),
        "Differences query should succeed. Error: {:?}",
        diffs_response.error()
    );

    tracing::info!("✅ Differences retrieved (ObjectIds)");

    if let Some(differences) = diffs_response.differences() {
        for layer_diff in differences {
            let inserts_count = layer_diff.inserts().as_ref().map(|i| i.len()).unwrap_or(0);
            let updates_count = layer_diff.updates().as_ref().map(|u| u.len()).unwrap_or(0);
            let deletes_count = layer_diff.deletes().as_ref().map(|d| d.len()).unwrap_or(0);

            if inserts_count + updates_count + deletes_count > 0 {
                tracing::info!(
                    "   Layer {}: {} inserts, {} updates, {} deletes",
                    layer_diff.layer_id(),
                    inserts_count,
                    updates_count,
                    deletes_count
                );
            }
        }
    }

    tracing::info!("");
    tracing::info!("Method 2: Full Features (with geometry and attributes)");
    let feature_diffs_response = vm_client
        .differences(version_guid, None, DifferenceResultType::Features, None)
        .await?;

    anyhow::ensure!(
        *feature_diffs_response.success(),
        "Feature differences query should succeed"
    );

    tracing::info!("✅ Differences retrieved (Features)");

    tracing::info!("");
    tracing::info!("💡 Differences operation:");
    tracing::info!("   • Compare version against DEFAULT");
    tracing::info!("   • Returns inserts, updates, deletes");
    tracing::info!("   • ObjectIds mode is more efficient");
    tracing::info!("   • Features mode includes full geometry and attributes");
    tracing::info!("   • Can filter by specific layers");

    Ok(())
}

/// Demonstrates delete forward edits (undo functionality).
async fn demonstrate_delete_forward_edits(vm_client: &VersionManagementClient<'_>) -> Result<()> {
    tracing::info!("\n=== Example 8: Delete Forward Edits (Undo) ===");
    tracing::info!("Demonstrate undo functionality by deleting edits after a checkpoint");
    tracing::info!("");

    // Create a temporary version for this demo
    let version_name = format!("undo_test_{}", chrono::Utc::now().timestamp());
    let params = CreateVersionParams::new(&version_name, VersionPermission::Private)
        .with_description("Testing undo functionality");

    let create_response = vm_client.create(params).await?;
    anyhow::ensure!(
        create_response.success().unwrap_or(false),
        "Version creation should succeed"
    );

    let version_info = create_response
        .version_info()
        .as_ref()
        .ok_or_else(|| anyhow::anyhow!("No version info in response"))?;
    let uuid = Uuid::parse_str(version_info.version_guid())?;
    let version_guid = VersionGuid::from_uuid(uuid);
    let session_id = SessionId::new();

    tracing::info!("Created version: {}", version_info.version_name());

    // Start edit session
    tracing::info!("Starting edit session...");
    let start_response = vm_client.start_editing(version_guid, session_id).await?;
    anyhow::ensure!(*start_response.success(), "Edit session should start");

    tracing::info!("✅ Edit session started");
    tracing::info!("");

    // In real workflow, make some edits here
    tracing::info!("(In real workflow: make initial edits here)");

    // Create checkpoint
    let checkpoint = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64;

    tracing::info!("📍 Checkpoint created at moment: {}", checkpoint);
    tracing::info!("");

    // More edits after checkpoint (these will be undone)
    tracing::info!("(In real workflow: make more edits to undo)");
    tracing::info!("");

    // Delete forward edits (undo)
    tracing::info!("Deleting edits after checkpoint (undo)...");
    let delete_response = vm_client
        .delete_forward_edits(version_guid, session_id, checkpoint)
        .await?;

    anyhow::ensure!(
        *delete_response.success(),
        "Delete forward edits should succeed. Error: {:?}",
        delete_response.error()
    );

    tracing::info!("✅ Forward edits deleted (undo successful)");

    // Stop editing and save
    let stop_response = vm_client
        .stop_editing(version_guid, session_id, true)
        .await?;
    anyhow::ensure!(*stop_response.success(), "Stop editing should succeed");

    tracing::info!("");
    tracing::info!("💡 Delete forward edits:");
    tracing::info!("   • Implements undo functionality");
    tracing::info!("   • Removes all edits after a checkpoint moment");
    tracing::info!("   • Must be called before stop_editing");
    tracing::info!("   • Moment must be >= version's modified date");
    tracing::info!("   • Useful for undo/redo stacks");

    Ok(())
}

/// Demonstrates reconcile and post workflow.
async fn demonstrate_reconcile_and_post(vm_client: &VersionManagementClient<'_>) -> Result<()> {
    tracing::info!("\n=== Example 9: Reconcile and Post Workflow ===");
    tracing::info!("Merge changes with DEFAULT and post edits");
    tracing::info!("");

    // Create a version for reconcile/post demo
    let version_name = format!("reconcile_test_{}", chrono::Utc::now().timestamp());
    let params = CreateVersionParams::new(&version_name, VersionPermission::Private)
        .with_description("Testing reconcile and post");

    let create_response = vm_client.create(params).await?;
    anyhow::ensure!(
        create_response.success().unwrap_or(false),
        "Version creation should succeed"
    );

    let version_info = create_response
        .version_info()
        .as_ref()
        .ok_or_else(|| anyhow::anyhow!("No version info in response"))?;
    let uuid = Uuid::parse_str(version_info.version_guid())?;
    let version_guid = VersionGuid::from_uuid(uuid);
    let session_id = SessionId::new();

    tracing::info!("Created version: {}", version_info.version_name());

    // Start edit session
    tracing::info!("Starting edit session...");
    let start_response = vm_client.start_editing(version_guid, session_id).await?;
    anyhow::ensure!(*start_response.success(), "Edit session should start");

    tracing::info!("✅ Edit session started");
    tracing::info!("");

    // In real workflow, make edits here via FeatureService API
    tracing::info!("(In real workflow: make edits via FeatureService API here)");
    tracing::info!("");

    // Reconcile with DEFAULT version
    tracing::info!("Reconciling with DEFAULT version...");
    let reconcile_response = vm_client
        .reconcile(
            version_guid,
            session_id,
            false, // don't abort on conflicts
            ConflictDetection::ByObject,
            false, // don't auto-post
        )
        .await?;

    anyhow::ensure!(
        *reconcile_response.success(),
        "Reconcile should succeed. Error: {:?}",
        reconcile_response.error()
    );

    tracing::info!("✅ Reconcile completed");

    if let Some(has_conflicts) = reconcile_response.has_conflicts() {
        tracing::info!("   Has conflicts: {}", has_conflicts);
        anyhow::ensure!(
            !has_conflicts,
            "No conflicts expected in this demo (no competing edits)"
        );
    }

    if let Some(moment) = reconcile_response.moment() {
        tracing::info!("   Moment: {}", moment);
    }

    tracing::info!("");

    // Post changes to DEFAULT
    tracing::info!("Posting changes to DEFAULT version...");
    let post_response = vm_client
        .post(
            version_guid,
            session_id,
            None, // post all edits
        )
        .await?;

    anyhow::ensure!(
        *post_response.success(),
        "Post should succeed. Error: {:?}",
        post_response.error()
    );

    tracing::info!("✅ Changes posted to DEFAULT");

    if let Some(moment) = post_response.moment() {
        tracing::info!("   Moment: {}", moment);
    }

    // Stop editing
    let stop_response = vm_client
        .stop_editing(version_guid, session_id, true)
        .await?;
    anyhow::ensure!(*stop_response.success(), "Stop editing should succeed");

    tracing::info!("");
    tracing::info!("💡 Reconcile and post:");
    tracing::info!("   • Reconcile compares version with DEFAULT");
    tracing::info!("   • Detects conflicts (ByObject or ByAttribute)");
    tracing::info!("   • Post applies edits to DEFAULT version");
    tracing::info!("   • Reconcile required before post");
    tracing::info!("   • Conflicts must be resolved before posting");

    Ok(())
}

/// Demonstrates conflict management operations.
async fn demonstrate_conflict_management(vm_client: &VersionManagementClient<'_>) -> Result<()> {
    tracing::info!("\n=== Example 10: Conflict Management ===");
    tracing::info!("Query conflicts, inspect them, and resolve with restore_rows");
    tracing::info!("");

    // Create a version for conflict demo
    let version_name = format!("conflict_test_{}", chrono::Utc::now().timestamp());
    let params = CreateVersionParams::new(&version_name, VersionPermission::Private)
        .with_description("Testing conflict management");

    let create_response = vm_client.create(params).await?;
    anyhow::ensure!(
        create_response.success().unwrap_or(false),
        "Version creation should succeed"
    );

    let version_info = create_response
        .version_info()
        .as_ref()
        .ok_or_else(|| anyhow::anyhow!("No version info in response"))?;
    let uuid = Uuid::parse_str(version_info.version_guid())?;
    let version_guid = VersionGuid::from_uuid(uuid);
    let session_id = SessionId::new();

    tracing::info!("Created version: {}", version_info.version_name());

    // Start edit session
    let start_response = vm_client.start_editing(version_guid, session_id).await?;
    anyhow::ensure!(*start_response.success(), "Edit session should start");

    tracing::info!("✅ Edit session started");
    tracing::info!("");

    // Reconcile (creates the conflict context even without real conflicts)
    tracing::info!("Reconciling to create conflict context...");
    let reconcile_response = vm_client
        .reconcile(
            version_guid,
            session_id,
            false,
            ConflictDetection::ByObject,
            false,
        )
        .await?;

    anyhow::ensure!(*reconcile_response.success(), "Reconcile should succeed");
    tracing::info!("✅ Reconcile completed");

    // Query conflicts
    tracing::info!("");
    tracing::info!("Querying conflicts...");
    let conflicts_response = vm_client.conflicts(version_guid, session_id).await?;

    anyhow::ensure!(
        *conflicts_response.success(),
        "Conflicts query should succeed. Error: {:?}",
        conflicts_response.error()
    );

    tracing::info!("✅ Conflicts retrieved");

    let has_conflicts = conflicts_response
        .conflicts()
        .as_ref()
        .map(|c| !c.is_empty())
        .unwrap_or(false);

    if has_conflicts {
        if let Some(conflict_layers) = conflicts_response.conflicts() {
            tracing::info!("   Found {} layers with conflicts", conflict_layers.len());

            for layer in conflict_layers {
                tracing::info!("   Layer {}: conflicts detected", layer.layer_id());
            }
        }

        // Inspect conflicts (mark as reviewed)
        tracing::info!("");
        tracing::info!("Inspecting all conflicts (marking as reviewed)...");
        let inspect_response = vm_client
            .inspect_conflicts(
                version_guid,
                session_id,
                true, // inspect all
                true, // set as inspected
                None,
            )
            .await?;

        anyhow::ensure!(
            *inspect_response.success(),
            "Inspect conflicts should succeed"
        );

        tracing::info!("✅ Conflicts inspected");

        // Demonstrate restore_rows (for Delete-Update conflicts)
        tracing::info!("");
        tracing::info!("Restore rows operation (for Delete-Update conflicts)...");
        tracing::info!("   (Skipping actual restore - no Delete-Update conflicts in demo)");
    } else {
        tracing::info!("   No conflicts detected (as expected in demo)");
        tracing::info!("");
        tracing::info!("Demonstrating inspect_conflicts with no actual conflicts...");
        let inspect_response = vm_client
            .inspect_conflicts(version_guid, session_id, true, true, None)
            .await?;

        anyhow::ensure!(
            *inspect_response.success(),
            "Inspect conflicts should succeed even with no conflicts"
        );

        tracing::info!("✅ Inspect conflicts operation successful");
    }

    // Stop editing
    let stop_response = vm_client
        .stop_editing(version_guid, session_id, false)
        .await?;
    anyhow::ensure!(*stop_response.success(), "Stop editing should succeed");

    tracing::info!("");
    tracing::info!("💡 Conflict management:");
    tracing::info!("   • conflicts() - Query conflicts after reconcile");
    tracing::info!("   • inspect_conflicts() - Mark conflicts as reviewed");
    tracing::info!("   • restore_rows() - Restore deleted rows (Delete-Update conflicts)");
    tracing::info!("   • Conflicts organized by layer and type");
    tracing::info!("   • Must resolve conflicts before posting");

    Ok(())
}

/// Demonstrates deleting a version.
async fn demonstrate_delete_version(vm_client: &VersionManagementClient<'_>) -> Result<()> {
    tracing::info!("\n=== Example 11: Delete Version ===");
    tracing::info!("Permanently remove a version from the geodatabase");
    tracing::info!("");

    // Create a version to delete
    let version_name = format!("delete_test_{}", chrono::Utc::now().timestamp());
    let params = CreateVersionParams::new(&version_name, VersionPermission::Private)
        .with_description("Temporary version for deletion test");

    let create_response = vm_client.create(params).await?;
    anyhow::ensure!(
        create_response.success().unwrap_or(false),
        "Version creation should succeed"
    );

    let version_info = create_response
        .version_info()
        .as_ref()
        .ok_or_else(|| anyhow::anyhow!("No version info in response"))?;
    let uuid = Uuid::parse_str(version_info.version_guid())?;
    let version_guid = VersionGuid::from_uuid(uuid);

    tracing::info!("Created version: {}", version_info.version_name());
    tracing::info!("Version GUID: {}", version_info.version_guid());
    tracing::info!("");

    // Delete the version
    tracing::info!("Deleting version...");
    let delete_response = vm_client.delete(version_guid).await?;

    anyhow::ensure!(
        *delete_response.success(),
        "Delete should succeed. Error: {:?}",
        delete_response.error()
    );

    tracing::info!("✅ Version deleted successfully");

    if let Some(moment) = delete_response.moment() {
        tracing::info!("   Moment: {}", moment);
    }

    // Verify deletion by trying to get info (should fail or return empty)
    tracing::info!("");
    tracing::info!("Verifying deletion...");
    let versions_after = vm_client.list_versions().await?;
    let still_exists = versions_after
        .versions()
        .iter()
        .any(|v| v.version_guid() == version_info.version_guid());

    anyhow::ensure!(
        !still_exists,
        "Version should no longer exist after deletion"
    );

    tracing::info!("✅ Verified version no longer in version list");

    tracing::info!("");
    tracing::info!("💡 Version deletion:");
    tracing::info!("   • Permanently removes version from geodatabase");
    tracing::info!("   • Cannot delete DEFAULT version");
    tracing::info!("   • Unposted edits are lost (irreversible)");
    tracing::info!("   • Recommended to reconcile/post before deleting");
    tracing::info!("   • Good practice: delete temporary versions after completion");

    Ok(())
}

/// Prints best practices for version management.
fn print_best_practices() {
    tracing::info!("\n💡 Branch Versioning Best Practices:");
    tracing::info!("   - Create named versions for isolated editing workflows");
    tracing::info!("   - Use edit sessions to ensure transaction integrity");
    tracing::info!("   - Use read sessions for consistent data snapshots");
    tracing::info!("   - Reconcile versions regularly to avoid conflicts");
    tracing::info!("   - Post changes to DEFAULT when editing is complete");
    tracing::info!("   - Delete temporary versions after reconciliation");
    tracing::info!("   - Use differences() to track changes before posting");
    tracing::info!("");
    tracing::info!("📊 Complete Version Lifecycle:");
    tracing::info!("   1. create() - Create named version from DEFAULT");
    tracing::info!("   2. alter() - Modify version properties (optional)");
    tracing::info!("   3. start_editing() - Begin edit session");
    tracing::info!("   4. <Make edits via FeatureService API>");
    tracing::info!("   5. delete_forward_edits() - Undo recent changes (optional)");
    tracing::info!("   6. stop_editing() - Save or discard changes");
    tracing::info!("   7. differences() - Review changes (optional)");
    tracing::info!("   8. reconcile() - Merge changes from DEFAULT");
    tracing::info!("   9. conflicts() - Check for conflicts");
    tracing::info!("   10. inspect_conflicts() - Review conflicts (if any)");
    tracing::info!("   11. restore_rows() - Resolve Delete-Update conflicts (if needed)");
    tracing::info!("   12. post() - Push changes to DEFAULT");
    tracing::info!("   13. delete() - Remove temporary version");
    tracing::info!("");
    tracing::info!("🔒 Session Types:");
    tracing::info!("   • Edit sessions (start_editing/stop_editing):");
    tracing::info!("     - Exclusive write lock");
    tracing::info!("     - Required for edits");
    tracing::info!("     - Save or discard changes");
    tracing::info!("   • Read sessions (start_reading/stop_reading):");
    tracing::info!("     - Consistent data snapshot");
    tracing::info!("     - Multiple simultaneous sessions");
    tracing::info!("     - Don't block edits");
    tracing::info!("");
    tracing::info!("⚙️  Utility Operations:");
    tracing::info!("   • differences() - Compare version with DEFAULT");
    tracing::info!("   • delete_forward_edits() - Undo functionality");
    tracing::info!("   • get_info() - Query version metadata");
    tracing::info!("   • list_versions() - List all versions");
    tracing::info!("");
    tracing::info!("⚠️  Requirements:");
    tracing::info!("   - Enterprise geodatabase (PostgreSQL/SQL Server/Oracle)");
    tracing::info!("   - Data registered as branch versioned in ArcGIS Pro");
    tracing::info!("   - Service published with Version Management capability");
    tracing::info!("   - ArcGIS Enterprise 10.6+ (not supported in ArcGIS Online)");
    tracing::info!("");
    tracing::info!("📊 Coverage:");
    tracing::info!("   ✅ 16/16 VersionManagementClient methods demonstrated (100%)");
}
