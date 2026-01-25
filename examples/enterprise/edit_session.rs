//! 🔒 Edit Sessions - Versioned Editing Workflows
//!
//! Demonstrates using edit sessions for versioned editing in branch-versioned
//! geodatabases. Edit sessions provide write locks and transaction semantics
//! for multi-request editing operations.
//!
//! # What You'll Learn
//!
//! - **Start edit sessions**: Begin versioned editing with write locks
//! - **Perform edits**: Add features within a session
//! - **Save changes**: Commit edits to the version
//! - **Discard changes**: Roll back uncommitted edits
//! - **Session management**: Handle session IDs and lifecycle
//!
//! # Prerequisites
//!
//! **⚠️ Enterprise-Only Feature**: This example requires infrastructure that
//! cannot be auto-provisioned:
//!
//! - **ArcGIS Enterprise 11.2+** with Version Management Server
//! - **Branch-versioned feature service** (not available on ArcGIS Online)
//! - **ArcGIS Advanced Editing** license
//! - **OAuth 2.0 credentials** (Client ID and Secret)
//!
//! ## Setup Steps
//!
//! 1. **Create branch-versioned service** in ArcGIS Enterprise Portal
//! 2. **Enable versioning** on the feature service
//! 3. **Note the URLs and GUIDs** for your environment
//! 4. **Configure `.env`** with your service details
//!
//! ## Environment Variables
//!
//! Set these in your `.env` file:
//!
//! ```env
//! # OAuth credentials
//! ARCGIS_CLIENT_ID=your_client_id
//! ARCGIS_CLIENT_SECRET=your_client_secret
//!
//! # Enterprise service URLs (from your portal)
//! VERSION_MGMT_URL=https://your-server.com/.../VersionManagementServer
//! FEATURE_SERVICE_URL=https://your-server.com/.../FeatureServer
//!
//! # Version and layer identifiers
//! VERSION_GUID=550e8400-e29b-41d4-a716-446655440000
//! LAYER_ID=0
//! ```
//!
//! # Running
//!
//! ```bash
//! cargo run --example edit_session
//!
//! # With debug logging:
//! RUST_LOG=debug cargo run --example edit_session
//! ```
//!
//! # Real-World Use Case
//!
//! Utility company manages infrastructure in versioned geodatabase:
//! - Planner starts edit session on design version
//! - Adds multiple related assets (poles, transformers, cables)
//! - Reviews changes with team
//! - Either commits changes or discards draft work
//! - Version isolates work-in-progress from production data

use anyhow::{Context, Result};
use arcgis::{
    ArcGISClient, ClientCredentialsAuth, EditOptions, EnvConfig, Feature, FeatureServiceClient,
    LayerId, ObjectId, SessionId, VersionManagementClient,
};
use serde_json::json;
use std::collections::HashMap;
use std::env;
use uuid::Uuid;

/// Configuration loaded from environment variables.
struct Config {
    version_mgmt_url: String,
    feature_service_url: String,
    version_guid: Uuid,
    layer_id: LayerId,
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

    tracing::info!("🔒 ArcGIS Edit Sessions Example");
    tracing::info!("Demonstrating versioned editing workflows");

    // Load configuration
    let config = load_config()?;

    // Create authenticated client
    let client = create_client().await?;

    // Create service clients
    let vm_client = VersionManagementClient::new(&config.version_mgmt_url, &client);
    let feature_client = FeatureServiceClient::new(&config.feature_service_url, &client);

    // Demonstrate: Start session → Edit → Save
    demonstrate_save_workflow(&vm_client, &feature_client, &config).await?;

    // Demonstrate: Start session → Edit → Discard
    demonstrate_discard_workflow(&vm_client, &feature_client, &config).await?;

    tracing::info!("\n✅ Edit session examples completed successfully!");
    print_best_practices();

    Ok(())
}

/// Loads configuration from environment variables.
fn load_config() -> Result<Config> {
    tracing::debug!("Loading configuration from environment");

    // Get EnvConfig for OAuth credentials (validates they exist)
    let env_config = EnvConfig::global();
    let _ = env_config.arcgis_client_id.as_ref().context(
        "ARCGIS_CLIENT_ID not set. Add to .env:\n\
         ARCGIS_CLIENT_ID=your_client_id",
    )?;
    let _ = env_config.arcgis_client_secret.as_ref().context(
        "ARCGIS_CLIENT_SECRET not set. Add to .env:\n\
         ARCGIS_CLIENT_SECRET=your_client_secret",
    )?;

    // Load Enterprise-specific configuration
    let version_mgmt_url = env::var("VERSION_MGMT_URL").context(
        "VERSION_MGMT_URL not set. Add to .env:\n\
         VERSION_MGMT_URL=https://your-server.com/.../VersionManagementServer\n\
         \n\
         This URL comes from your ArcGIS Enterprise portal.",
    )?;

    let feature_service_url = env::var("FEATURE_SERVICE_URL").context(
        "FEATURE_SERVICE_URL not set. Add to .env:\n\
         FEATURE_SERVICE_URL=https://your-server.com/.../FeatureServer\n\
         \n\
         This is your branch-versioned feature service URL.",
    )?;

    let version_guid_str = env::var("VERSION_GUID").context(
        "VERSION_GUID not set. Add to .env:\n\
         VERSION_GUID=550e8400-e29b-41d4-a716-446655440000\n\
         \n\
         This is the GUID of the version you want to edit.",
    )?;

    let layer_id_val: u32 = env::var("LAYER_ID")
        .context(
            "LAYER_ID not set. Add to .env:\n\
             LAYER_ID=0\n\
             \n\
             This is the layer index in your feature service.",
        )?
        .parse()
        .context("LAYER_ID must be a number")?;

    // Parse version GUID
    let version_guid = Uuid::parse_str(&version_guid_str)
        .context("VERSION_GUID must be a valid UUID")?;
    tracing::debug!(version_guid = %version_guid, "Parsed version GUID");

    Ok(Config {
        version_mgmt_url,
        feature_service_url,
        version_guid,
        layer_id: LayerId::new(layer_id_val),
    })
}

/// Creates an authenticated ArcGIS client using OAuth credentials.
async fn create_client() -> Result<ArcGISClient> {
    tracing::info!("Creating authenticated client");

    // Use from_env() which automatically loads .env and reads credentials
    let auth = ClientCredentialsAuth::from_env()
        .context("Failed to load OAuth credentials from environment")?;

    Ok(ArcGISClient::new(auth))
}

/// Demonstrates starting a session, making edits, and saving changes.
async fn demonstrate_save_workflow(
    vm_client: &VersionManagementClient<'_>,
    feature_client: &FeatureServiceClient<'_>,
    config: &Config,
) -> Result<()> {
    tracing::info!("\n=== Example 1: Start → Edit → Save ===");
    tracing::info!("Demonstrating committed edits workflow");

    // Step 1: Start edit session
    let session_id = start_edit_session(vm_client, config.version_guid).await?;

    // Step 2: Perform edits
    let object_id = add_feature_in_session(
        feature_client,
        config.layer_id,
        session_id,
        "Permanent Feature",
        "This feature will be saved",
    )
    .await?;

    // Step 3: Save changes
    save_edit_session(vm_client, config.version_guid, session_id).await?;

    tracing::info!(
        object_id = object_id.0,
        "✅ Feature saved to version (committed)"
    );

    Ok(())
}

/// Demonstrates starting a session, making edits, and discarding changes.
async fn demonstrate_discard_workflow(
    vm_client: &VersionManagementClient<'_>,
    feature_client: &FeatureServiceClient<'_>,
    config: &Config,
) -> Result<()> {
    tracing::info!("\n=== Example 2: Start → Edit → Discard ===");
    tracing::info!("Demonstrating rollback workflow");

    // Step 1: Start edit session
    let session_id = start_edit_session(vm_client, config.version_guid).await?;

    // Step 2: Perform edits
    let object_id = add_feature_in_session(
        feature_client,
        config.layer_id,
        session_id,
        "Temporary Feature",
        "This feature will be discarded",
    )
    .await?;

    // Step 3: Discard changes
    discard_edit_session(vm_client, config.version_guid, session_id).await?;

    tracing::info!(
        object_id = object_id.0,
        "✅ Feature discarded (rolled back)"
    );

    Ok(())
}

/// Starts an edit session on the specified version.
async fn start_edit_session(
    vm_client: &VersionManagementClient<'_>,
    version_guid: Uuid,
) -> Result<SessionId> {
    let session_id = SessionId::new();
    tracing::info!(session_id = %session_id, "Starting edit session");

    let start_response = vm_client
        .start_editing(version_guid.into(), session_id)
        .await?;

    if !start_response.success() {
        anyhow::bail!(
            "Failed to start editing session: {:?}",
            start_response.error()
        );
    }

    tracing::info!(
        moment = ?start_response.moment(),
        "✅ Edit session started successfully"
    );

    Ok(session_id)
}

/// Adds a feature within an edit session.
async fn add_feature_in_session(
    feature_client: &FeatureServiceClient<'_>,
    layer_id: LayerId,
    session_id: SessionId,
    name: &str,
    description: &str,
) -> Result<ObjectId> {
    tracing::info!(name = %name, "Adding feature within session");

    // Create feature attributes
    let mut attributes = HashMap::new();
    attributes.insert("NAME".to_string(), json!(name));
    attributes.insert("DESCRIPTION".to_string(), json!(description));
    attributes.insert("VALUE".to_string(), json!(42));

    let new_feature = Feature::new(attributes, None);

    // Configure edit options with session ID
    let edit_options = EditOptions::new()
        .with_session_id(session_id)
        .with_rollback_on_failure(true)
        .with_return_edit_results(true);

    let add_result = feature_client
        .add_features(layer_id, vec![new_feature], edit_options)
        .await?;

    // Extract ObjectID from result
    let object_id = add_result
        .add_results()
        .first()
        .context("No add results returned")?
        .object_id()
        .as_ref()
        .copied()
        .context("Added feature has no ObjectID")?;

    tracing::info!(object_id = object_id.0, "✅ Feature added to session");

    Ok(object_id)
}

/// Saves changes and stops the edit session (commits edits).
async fn save_edit_session(
    vm_client: &VersionManagementClient<'_>,
    version_guid: Uuid,
    session_id: SessionId,
) -> Result<()> {
    tracing::info!("Saving changes and stopping session");

    let stop_response = vm_client
        .stop_editing(version_guid.into(), session_id, true)
        .await?;

    if !stop_response.success() {
        anyhow::bail!(
            "Failed to save changes: {:?}",
            stop_response.error()
        );
    }

    tracing::info!(
        moment = ?stop_response.moment(),
        "✅ Changes saved successfully"
    );

    Ok(())
}

/// Discards changes and stops the edit session (rolls back edits).
async fn discard_edit_session(
    vm_client: &VersionManagementClient<'_>,
    version_guid: Uuid,
    session_id: SessionId,
) -> Result<()> {
    tracing::info!("Discarding changes and stopping session");

    let stop_response = vm_client
        .stop_editing(version_guid.into(), session_id, false)
        .await?;

    if !stop_response.success() {
        anyhow::bail!(
            "Failed to discard changes: {:?}",
            stop_response.error()
        );
    }

    tracing::info!(
        moment = ?stop_response.moment(),
        "✅ Changes discarded successfully"
    );

    Ok(())
}

/// Prints best practices for working with edit sessions.
fn print_best_practices() {
    tracing::info!("\n💡 Edit Session Best Practices:");
    tracing::info!("   - Always stop sessions (save or discard) to release locks");
    tracing::info!("   - Use rollbackOnFailure for transactional consistency");
    tracing::info!("   - Session IDs must be unique (use SessionId::new())");
    tracing::info!("   - Sessions timeout after inactivity (server-configured)");
    tracing::info!("   - Don't share session IDs between concurrent operations");
    tracing::info!("   - Test rollback scenarios in development");
    tracing::info!("");
    tracing::info!("🏗️  Version Management Workflow:");
    tracing::info!("   1. Create child version for isolation");
    tracing::info!("   2. Start edit session on child version");
    tracing::info!("   3. Perform all related edits");
    tracing::info!("   4. Review changes (query features)");
    tracing::info!("   5. Save (commit) or discard (rollback)");
    tracing::info!("   6. Reconcile and post to parent version");
    tracing::info!("");
    tracing::info!("⚠️  Common Issues:");
    tracing::info!("   - Session already started: Each session ID can only be used once");
    tracing::info!("   - Lock conflicts: Another user has write lock on version");
    tracing::info!("   - Timeout: Session expired due to inactivity");
    tracing::info!("   - Permission denied: User lacks edit privileges on version");
}
