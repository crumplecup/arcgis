//! Example: Versioned editing with edit sessions
//!
//! This example demonstrates using edit sessions for versioned editing workflows
//! in branch-versioned geodatabases. Edit sessions provide write locks and
//! transaction semantics for multi-request editing operations.
//!
//! # Prerequisites
//!
//! - ArcGIS Enterprise 11.2+ with Version Management Server
//! - Branch-versioned feature service
//! - ArcGIS Advanced Editing license
//! - OAuth 2.0 Client Credentials (client_id and client_secret)
//!
//! # Environment Variables
//!
//! Set these in a `.env` file:
//! ```text
//! ARCGIS_CLIENT_ID=your_client_id
//! ARCGIS_CLIENT_SECRET=your_client_secret
//! VERSION_MGMT_URL=https://services.arcgis.com/.../VersionManagementServer
//! FEATURE_SERVICE_URL=https://services.arcgis.com/.../FeatureServer
//! VERSION_GUID=550e8400-e29b-41d4-a716-446655440000
//! LAYER_ID=0
//! ```
//!
//! # Usage
//!
//! ```bash
//! cargo run --example edit_session
//! ```

use arcgis::{
    ArcGISClient, ClientCredentialsAuth, EditOptions, Feature, FeatureServiceClient, LayerId,
    SessionId, VersionManagementClient,
};
use serde_json::json;
use std::collections::HashMap;
use std::env;
use uuid::Uuid;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing for structured logging
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    tracing::info!("Starting edit_session example");

    // Load environment variables (.env automatically loaded by library)
    tracing::debug!("Loading configuration from environment");
    let client_id = env::var("ARCGIS_CLIENT_ID")
        .map_err(|_| anyhow::anyhow!("ARCGIS_CLIENT_ID not set in .env"))?;
    let client_secret = env::var("ARCGIS_CLIENT_SECRET")
        .map_err(|_| anyhow::anyhow!("ARCGIS_CLIENT_SECRET not set in .env"))?;
    let version_mgmt_url = env::var("VERSION_MGMT_URL")
        .map_err(|_| anyhow::anyhow!("VERSION_MGMT_URL not set in .env"))?;
    let feature_service_url = env::var("FEATURE_SERVICE_URL")
        .map_err(|_| anyhow::anyhow!("FEATURE_SERVICE_URL not set in .env"))?;
    let version_guid_str =
        env::var("VERSION_GUID").map_err(|_| anyhow::anyhow!("VERSION_GUID not set in .env"))?;
    let layer_id_val: u32 = env::var("LAYER_ID")
        .map_err(|_| anyhow::anyhow!("LAYER_ID not set in .env"))?
        .parse()
        .map_err(|_| anyhow::anyhow!("LAYER_ID must be a number"))?;

    // Parse version GUID
    let version_guid = Uuid::parse_str(&version_guid_str)?;
    tracing::debug!(version_guid = %version_guid, "Parsed version GUID");

    // Create authenticated client
    tracing::info!("Creating authenticated client");
    let auth = ClientCredentialsAuth::new(client_id, client_secret)?;
    let client = ArcGISClient::new(auth);

    // Create service clients
    tracing::debug!(
        version_mgmt_url = %version_mgmt_url,
        feature_service_url = %feature_service_url,
        "Creating service clients"
    );
    let vm_client = VersionManagementClient::new(&version_mgmt_url, &client);
    let feature_client = FeatureServiceClient::new(&feature_service_url, &client);

    // Step 1: Start an edit session
    tracing::info!("Starting edit session");
    let session_id = SessionId::new();
    tracing::info!(session_id = %session_id, "Generated session ID");

    let start_response = vm_client
        .start_editing(version_guid.into(), session_id)
        .await?;

    if !start_response.success() {
        tracing::error!(
            error = ?start_response.error(),
            "Failed to start editing session"
        );
        anyhow::bail!("Edit session failed to start");
    }

    tracing::info!(
        moment = ?start_response.moment(),
        "Edit session started successfully"
    );

    // Step 2: Perform edits within the session
    tracing::info!("Performing edits within session");

    // Create a new feature
    let mut attributes = HashMap::new();
    attributes.insert("NAME".to_string(), json!("Example Feature"));
    attributes.insert("DESCRIPTION".to_string(), json!("Created via edit session"));
    attributes.insert("VALUE".to_string(), json!(42));

    let new_feature = Feature::new(attributes, None);

    // Add feature with session ID
    let edit_options = EditOptions::new()
        .with_session_id(session_id)
        .with_rollback_on_failure(true)
        .with_return_edit_results(true);

    let layer_id = LayerId::new(layer_id_val);
    tracing::debug!(layer_id = layer_id_val, "Adding features to layer");
    let add_result = feature_client
        .add_features(layer_id, vec![new_feature], edit_options.clone())
        .await?;

    tracing::info!("Add feature results:");
    for item in add_result.add_results() {
        if *item.success() {
            let object_id = item.object_id().expect("Has ID");
            tracing::info!(object_id = %object_id, "✓ Added feature successfully");
        } else {
            tracing::warn!(error = ?item.error(), "✗ Failed to add feature");
        }
    }

    // Step 3: Stop the edit session (save changes)
    tracing::info!("Saving changes");
    let stop_response = vm_client
        .stop_editing(version_guid.into(), session_id, true)
        .await?;

    if *stop_response.success() {
        tracing::info!(
            moment = ?stop_response.moment(),
            "Changes saved successfully"
        );
    } else {
        tracing::error!(
            error = ?stop_response.error(),
            "Failed to save changes"
        );
    }

    // Example: Discard changes instead
    tracing::info!("--- Alternative: Discarding changes ---");

    let session_id_2 = SessionId::new();
    tracing::info!(session_id = %session_id_2, "Starting new session");

    vm_client
        .start_editing(version_guid.into(), session_id_2)
        .await?;

    // Perform some edits...
    let mut attributes2 = HashMap::new();
    attributes2.insert("NAME".to_string(), json!("Temporary Feature"));
    let temp_feature = Feature::new(attributes2, None);

    let edit_options_2 = EditOptions::new().with_session_id(session_id_2);

    feature_client
        .add_features(layer_id, vec![temp_feature], edit_options_2)
        .await?;

    // Discard changes (saveEdits = false)
    tracing::info!("Discarding changes");
    let discard_response = vm_client
        .stop_editing(version_guid.into(), session_id_2, false)
        .await?;

    if *discard_response.success() {
        tracing::info!(
            moment = ?discard_response.moment(),
            "Changes discarded successfully"
        );
    }

    tracing::info!("Edit session example completed successfully");

    Ok(())
}
