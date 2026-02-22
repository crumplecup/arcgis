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
//! # Authentication (choose one)
//! ARCGIS_ENTERPRISE_KEY=your_enterprise_api_key
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
use arcgis::{
    ApiKeyAuth, ApiKeyTier, ArcGISClient, CreateVersionParams, EnvConfig, SessionId, VersionGuid,
    VersionManagementClient, VersionPermission,
};
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

    // Create authenticated client (automatically loads .env)
    tracing::debug!("Creating authenticated client");
    let auth = ApiKeyAuth::from_env(ApiKeyTier::Features)?;
    let client = ArcGISClient::new(auth);
    let vm_client = VersionManagementClient::new(&vm_url, &client);

    // Demonstrate version management operations
    demonstrate_list_versions(&vm_client).await?;
    demonstrate_create_version(&vm_client).await?;
    demonstrate_version_metadata(&vm_client).await?;
    demonstrate_edit_session(&vm_client).await?;

    tracing::info!("\n✅ All version management examples completed successfully!");
    print_best_practices();

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

    if let Some(created) = version_info.created_date() {
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

    // Get DEFAULT version GUID
    let versions_response = vm_client.list_versions().await?;

    anyhow::ensure!(
        !versions_response.versions().is_empty(),
        "Need at least one version for edit session"
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

    // Start editing session
    tracing::info!("Starting edit session...");
    let start_response = vm_client.start_editing(version_guid, session_id).await?;

    anyhow::ensure!(
        *start_response.success(),
        "Start editing should succeed. Error: {:?}",
        start_response.error()
    );

    tracing::info!("✅ Edit session started");

    if let Some(moment) = start_response.moment() {
        tracing::info!("   Moment: {}", moment);
    }

    // In real workflow, you would make edits here via FeatureService API

    tracing::info!("");
    tracing::info!("(In real workflow: make edits to features here)");
    tracing::info!("");

    // Stop editing session (save changes)
    tracing::info!("Stopping edit session (saving changes)...");
    let stop_response = vm_client
        .stop_editing(version_guid, session_id, true)
        .await?;

    anyhow::ensure!(
        *stop_response.success(),
        "Stop editing should succeed. Error: {:?}",
        stop_response.error()
    );

    tracing::info!("✅ Edit session stopped (changes saved)");

    if let Some(moment) = stop_response.moment() {
        tracing::info!("   Moment: {}", moment);
    }

    tracing::info!("");
    tracing::info!("💡 Edit sessions:");
    tracing::info!("   • Required for branch versioning edits");
    tracing::info!("   • Provide write locks and transactions");
    tracing::info!("   • Save with stop_editing(version, session, true)");
    tracing::info!("   • Discard with stop_editing(version, session, false)");

    Ok(())
}

/// Prints best practices for version management.
fn print_best_practices() {
    tracing::info!("\n💡 Branch Versioning Best Practices:");
    tracing::info!("   - Create named versions for isolated editing workflows");
    tracing::info!("   - Use edit sessions to ensure transaction integrity");
    tracing::info!("   - Reconcile versions regularly to avoid conflicts");
    tracing::info!("   - Post changes to DEFAULT when editing is complete");
    tracing::info!("   - Delete temporary versions after reconciliation");
    tracing::info!("");
    tracing::info!("📊 Version Lifecycle:");
    tracing::info!("   1. create() - Create named version from DEFAULT");
    tracing::info!("   2. start_editing() - Begin edit session");
    tracing::info!("   3. <Make edits via FeatureService API>");
    tracing::info!("   4. stop_editing() - Save or discard changes");
    tracing::info!("   5. reconcile() - Merge changes from parent");
    tracing::info!("   6. post() - Push changes to DEFAULT");
    tracing::info!("   7. delete() - Remove temporary version");
    tracing::info!("");
    tracing::info!("⚠️  Requirements:");
    tracing::info!("   - Enterprise geodatabase (PostgreSQL/SQL Server/Oracle)");
    tracing::info!("   - Data registered as branch versioned in ArcGIS Pro");
    tracing::info!("   - Service published with Version Management capability");
    tracing::info!("   - ArcGIS Enterprise 10.6+ (not supported in ArcGIS Online)");
}
