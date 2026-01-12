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
//! CLIENT_ID=your_client_id
//! CLIENT_SECRET=your_client_secret
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
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    // Load environment variables
    dotenvy::dotenv().ok();

    let client_id = env::var("CLIENT_ID").expect("CLIENT_ID not set");
    let client_secret = env::var("CLIENT_SECRET").expect("CLIENT_SECRET not set");
    let version_mgmt_url = env::var("VERSION_MGMT_URL").expect("VERSION_MGMT_URL not set");
    let feature_service_url = env::var("FEATURE_SERVICE_URL").expect("FEATURE_SERVICE_URL not set");
    let version_guid_str = env::var("VERSION_GUID").expect("VERSION_GUID not set");
    let layer_id_val: u32 = env::var("LAYER_ID")
        .expect("LAYER_ID not set")
        .parse()
        .expect("LAYER_ID must be a number");

    // Parse version GUID
    let version_guid = Uuid::parse_str(&version_guid_str)?;

    // Create authenticated client
    let auth = ClientCredentialsAuth::new(client_id, client_secret)?;
    let client = ArcGISClient::new(auth);

    // Create service clients
    let vm_client = VersionManagementClient::new(&version_mgmt_url, &client);
    let feature_client = FeatureServiceClient::new(&feature_service_url, &client);

    // Step 1: Start an edit session
    println!("Starting edit session...");
    let session_id = SessionId::new();
    println!("Session ID: {}", session_id);

    let start_response = vm_client
        .start_editing(version_guid.into(), session_id)
        .await?;

    if !start_response.success() {
        eprintln!(
            "Failed to start editing session: {:?}",
            start_response.error()
        );
        return Ok(());
    }

    println!("Edit session started at {:?}", start_response.moment());

    // Step 2: Perform edits within the session
    println!("\nPerforming edits...");

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
    let add_result = feature_client
        .add_features(layer_id, vec![new_feature], edit_options.clone())
        .await?;

    println!("Add result:");
    for item in add_result.add_results() {
        if *item.success() {
            println!(
                "  ✓ Added feature with ObjectID: {}",
                item.object_id().expect("Has ID")
            );
        } else {
            println!("  ✗ Failed: {:?}", item.error());
        }
    }

    // Step 3: Stop the edit session (save changes)
    println!("\nSaving changes...");
    let stop_response = vm_client
        .stop_editing(version_guid.into(), session_id, true)
        .await?;

    if *stop_response.success() {
        println!("Changes saved successfully at {:?}", stop_response.moment());
    } else {
        eprintln!("Failed to save changes: {:?}", stop_response.error());
    }

    // Example: Discard changes instead
    println!("\n--- Alternative: Discarding changes ---");

    let session_id_2 = SessionId::new();
    println!("Starting new session: {}", session_id_2);

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
    println!("Discarding changes...");
    let discard_response = vm_client
        .stop_editing(version_guid.into(), session_id_2, false)
        .await?;

    if *discard_response.success() {
        println!(
            "Changes discarded successfully at {:?}",
            discard_response.moment()
        );
    }

    Ok(())
}
