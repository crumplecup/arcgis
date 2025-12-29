//! Version Management Service.
//!
//! This module provides support for ArcGIS Version Management Services, which
//! handle versioned geodatabase operations including:
//!
//! - Edit sessions (startEditing/stopEditing)
//! - Version creation and management
//! - Reconciliation and posting
//! - Branch vs Traditional versioning workflows
//!
//! # Edit Sessions
//!
//! Edit sessions are required when working with branch-versioned geodatabases.
//! They provide write locks and transaction semantics:
//!
//! ```no_run
//! use arcgis::{
//!     ArcGISClient, ClientCredentialsAuth, VersionManagementClient, SessionId
//! };
//! use uuid::Uuid;
//!
//! # async fn example() -> arcgis::Result<()> {
//! let auth = ClientCredentialsAuth::new(
//!     "client_id".to_string(),
//!     "client_secret".to_string(),
//! ).expect("Valid credentials");
//! let client = ArcGISClient::new(auth);
//!
//! let vm_client = VersionManagementClient::new(
//!     "https://services.arcgis.com/org/arcgis/rest/services/MyService/VersionManagementServer",
//!     &client,
//! );
//!
//! let version_guid = Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000")
//!     .expect("Valid UUID");
//! let session_id = SessionId::new();
//!
//! // Start editing
//! vm_client.start_editing(version_guid.into(), session_id).await?;
//!
//! // Perform edits...
//!
//! // Save changes
//! vm_client.stop_editing(version_guid.into(), session_id, true).await?;
//! # Ok(())
//! # }
//! ```

mod client;
mod types;

pub use client::VersionManagementClient;
pub use types::{
    AlterResponse, AlterVersionParams, ConflictDetection, ConflictEntry, ConflictFeature,
    ConflictsResponse, CreateVersionParams, CreateVersionResponse, DeleteForwardEditsResponse,
    DeleteResponse, DifferenceFeature, DifferenceResultType, DifferencesResponse, EditSessionError,
    InspectConflictFeature, InspectConflictLayer, InspectConflictsResponse, LayerConflicts,
    LayerFeatureDifferences, LayerObjectIdDifferences, PartialPostRow, PostResponse,
    ReconcileResponse, RestoreRowsLayer, RestoreRowsResponse, SessionId, StartEditingResponse,
    StartReadingResponse, StopEditingResponse, StopReadingResponse, VersionGuid, VersionInfo,
    VersionInfosResponse, VersionPermission, VersioningType,
};
