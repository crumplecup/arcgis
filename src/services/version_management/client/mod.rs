//! Version Management Service client.

mod conflicts;
mod reconciliation;
mod sessions;
mod utilities;
mod versions;

use crate::ArcGISClient;
use tracing::instrument;

/// Client for interacting with an ArcGIS Version Management Service.
///
/// The Version Management Service provides operations for working with versioned
/// geodatabases, including edit sessions, version creation, and reconciliation.
///
/// # Edit Sessions
///
/// Edit sessions are required when working with branch-versioned geodatabases.
/// They provide write locks and transaction semantics for multi-request workflows:
///
/// 1. Start an edit session with [`start_editing`](Self::start_editing)
/// 2. Perform edits (add, update, delete features)
/// 3. Stop the session with [`stop_editing`](Self::stop_editing)
///
/// # Example
///
/// ```no_run
/// use arcgis::{ArcGISClient, ClientCredentialsAuth, VersionManagementClient, SessionId};
/// use uuid::Uuid;
///
/// # async fn example() -> arcgis::Result<()> {
/// let auth = ClientCredentialsAuth::new(
///     "client_id".to_string(),
///     "client_secret".to_string(),
/// ).expect("Valid credentials");
/// let client = ArcGISClient::new(auth);
///
/// let vm_client = VersionManagementClient::new(
///     "https://services.arcgis.com/org/arcgis/rest/services/MyService/VersionManagementServer",
///     &client,
/// );
///
/// // Start an edit session
/// let version_guid = Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000")
///     .expect("Valid UUID");
/// let session_id = SessionId::new();
///
/// let start_response = vm_client
///     .start_editing(version_guid.into(), session_id)
///     .await?;
///
/// if *start_response.success() {
///     println!("Edit session started");
///
///     // Perform edits here...
///
///     // Save changes
///     let stop_response = vm_client
///         .stop_editing(version_guid.into(), session_id, true)
///         .await?;
///
///     if *stop_response.success() {
///         println!("Changes saved");
///     }
/// }
/// # Ok(())
/// # }
/// ```
pub struct VersionManagementClient<'a> {
    /// Base URL of the Version Management Service
    pub(super) base_url: String,
    /// Reference to the ArcGIS client for HTTP operations
    pub(super) client: &'a ArcGISClient,
}

impl<'a> VersionManagementClient<'a> {
    /// Creates a new Version Management Service client.
    ///
    /// # Arguments
    ///
    /// * `base_url` - The base URL of the Version Management Service
    /// * `client` - Reference to an authenticated ArcGIS client
    ///
    /// # Example
    ///
    /// ```no_run
    /// use arcgis::{ApiKeyAuth, ArcGISClient, VersionManagementClient};
    ///
    /// let auth = ApiKeyAuth::new("YOUR_API_KEY");
    /// let client = ArcGISClient::new(auth);
    ///
    /// let vm_client = VersionManagementClient::new(
    ///     "https://services.arcgis.com/org/arcgis/rest/services/MyService/VersionManagementServer",
    ///     &client,
    /// );
    /// ```
    #[instrument(skip(base_url, client))]
    pub fn new(base_url: impl Into<String>, client: &'a ArcGISClient) -> Self {
        let base_url = base_url.into();
        tracing::debug!(base_url = %base_url, "Creating VersionManagementClient");
        Self { base_url, client }
    }
}
