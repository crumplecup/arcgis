//! Version Management Service client.

use crate::{
    AlterResponse, AlterVersionParams, ArcGISClient, ConflictDetection, CreateVersionParams,
    CreateVersionResponse, DeleteResponse, PartialPostRow, PostResponse, ReconcileResponse, Result,
    SessionId, StartEditingResponse, StartReadingResponse, StopEditingResponse,
    StopReadingResponse, VersionGuid, VersionInfo, VersionInfosResponse,
};
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
    base_url: String,
    /// Reference to the ArcGIS client for HTTP operations
    client: &'a ArcGISClient,
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

    /// Starts an edit session on a version.
    ///
    /// Starting an edit session acquires a write lock on the version, preventing
    /// other users from editing until the session is stopped. This is required
    /// for editing branch-versioned geodatabases.
    ///
    /// # Arguments
    ///
    /// * `version_guid` - The GUID of the version to edit
    /// * `session_id` - A unique identifier for this edit session
    ///
    /// # Returns
    ///
    /// Returns a [`StartEditingResponse`] indicating success or failure.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The version doesn't exist
    /// - Another user has a write lock on the version
    /// - The user doesn't have edit permissions
    /// - Authentication fails
    /// - Network error occurs
    ///
    /// # Example
    ///
    /// ```no_run
    /// use arcgis::{ArcGISClient, ClientCredentialsAuth, VersionManagementClient, SessionId};
    /// use uuid::Uuid;
    ///
    /// # async fn example() -> arcgis::Result<()> {
    /// # let auth = ClientCredentialsAuth::new("id".to_string(), "secret".to_string()).expect("Valid");
    /// # let client = ArcGISClient::new(auth);
    /// # let vm_client = VersionManagementClient::new("url", &client);
    /// let version_guid = Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000")
    ///     .expect("Valid UUID");
    /// let session_id = SessionId::new();
    ///
    /// let response = vm_client
    ///     .start_editing(version_guid.into(), session_id)
    ///     .await?;
    ///
    /// if *response.success() {
    ///     println!("Edit session started at {:?}", response.moment());
    /// } else {
    ///     eprintln!("Failed to start session: {:?}", response.error());
    /// }
    /// # Ok(())
    /// # }
    /// ```
    #[instrument(skip(self), fields(base_url = %self.base_url, version_guid = %version_guid, session_id = %session_id))]
    pub async fn start_editing(
        &self,
        version_guid: VersionGuid,
        session_id: SessionId,
    ) -> Result<StartEditingResponse> {
        tracing::debug!(
            version_guid = %version_guid,
            session_id = %session_id,
            "Starting edit session"
        );

        let url = format!("{}/versions/{}/startEditing", self.base_url, version_guid);
        let token = self.client.auth().get_token().await?;

        tracing::debug!(url = %url, "Sending startEditing request");

        let response = self
            .client
            .http()
            .post(&url)
            .form(&[
                ("sessionId", session_id.to_string().as_str()),
                ("f", "json"),
                ("token", token.as_str()),
            ])
            .send()
            .await?;

        let status = response.status();
        if !status.is_success() {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|e| format!("Failed to read error: {}", e));
            tracing::error!(status = %status, error = %error_text, "startEditing failed");
            return Err(crate::Error::from(crate::ErrorKind::Api {
                code: status.as_u16() as i32,
                message: format!("HTTP {}: {}", status, error_text),
            }));
        }

        let start_response: StartEditingResponse = response.json().await?;

        if *start_response.success() {
            tracing::info!(
                session_id = %session_id,
                moment = ?start_response.moment(),
                "Edit session started successfully"
            );
        } else {
            tracing::warn!(
                session_id = %session_id,
                error = ?start_response.error(),
                "startEditing reported failure"
            );
        }

        Ok(start_response)
    }

    /// Stops an edit session on a version.
    ///
    /// Stopping an edit session releases the write lock on the version. You can
    /// choose to save or discard changes made during the session.
    ///
    /// # Arguments
    ///
    /// * `version_guid` - The GUID of the version being edited
    /// * `session_id` - The session identifier from [`start_editing`](Self::start_editing)
    /// * `save_edits` - `true` to save changes, `false` to discard them
    ///
    /// # Returns
    ///
    /// Returns a [`StopEditingResponse`] indicating success or failure.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The session doesn't exist
    /// - The session ID doesn't match
    /// - Authentication fails
    /// - Network error occurs
    ///
    /// # Example
    ///
    /// ```no_run
    /// use arcgis::{ArcGISClient, ClientCredentialsAuth, VersionManagementClient, SessionId};
    /// use uuid::Uuid;
    ///
    /// # async fn example() -> arcgis::Result<()> {
    /// # let auth = ClientCredentialsAuth::new("id".to_string(), "secret".to_string()).expect("Valid");
    /// # let client = ArcGISClient::new(auth);
    /// # let vm_client = VersionManagementClient::new("url", &client);
    /// # let version_guid = Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000")
    /// #     .expect("Valid UUID");
    /// # let session_id = SessionId::new();
    /// # vm_client.start_editing(version_guid.into(), session_id).await?;
    /// // Save changes
    /// let response = vm_client
    ///     .stop_editing(version_guid.into(), session_id, true)
    ///     .await?;
    ///
    /// if *response.success() {
    ///     println!("Changes saved successfully");
    /// }
    ///
    /// // Or discard changes
    /// let response = vm_client
    ///     .stop_editing(version_guid.into(), session_id, false)
    ///     .await?;
    ///
    /// if *response.success() {
    ///     println!("Changes discarded");
    /// }
    /// # Ok(())
    /// # }
    /// ```
    #[instrument(skip(self), fields(base_url = %self.base_url, version_guid = %version_guid, session_id = %session_id, save_edits))]
    pub async fn stop_editing(
        &self,
        version_guid: VersionGuid,
        session_id: SessionId,
        save_edits: bool,
    ) -> Result<StopEditingResponse> {
        tracing::debug!(
            version_guid = %version_guid,
            session_id = %session_id,
            save_edits = save_edits,
            "Stopping edit session"
        );

        let url = format!("{}/versions/{}/stopEditing", self.base_url, version_guid);
        let token = self.client.auth().get_token().await?;

        let save_edits_str = if save_edits { "true" } else { "false" };

        tracing::debug!(url = %url, save_edits = save_edits, "Sending stopEditing request");

        let response = self
            .client
            .http()
            .post(&url)
            .form(&[
                ("sessionId", session_id.to_string().as_str()),
                ("saveEdits", save_edits_str),
                ("f", "json"),
                ("token", token.as_str()),
            ])
            .send()
            .await?;

        let status = response.status();
        if !status.is_success() {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|e| format!("Failed to read error: {}", e));
            tracing::error!(status = %status, error = %error_text, "stopEditing failed");
            return Err(crate::Error::from(crate::ErrorKind::Api {
                code: status.as_u16() as i32,
                message: format!("HTTP {}: {}", status, error_text),
            }));
        }

        let stop_response: StopEditingResponse = response.json().await?;

        if *stop_response.success() {
            tracing::info!(
                session_id = %session_id,
                save_edits = save_edits,
                moment = ?stop_response.moment(),
                "Edit session stopped successfully"
            );
        } else {
            tracing::warn!(
                session_id = %session_id,
                error = ?stop_response.error(),
                "stopEditing reported failure"
            );
        }

        Ok(stop_response)
    }

    /// Starts a read session on a version.
    ///
    /// Starting a read session acquires a read lock on the version, ensuring a
    /// consistent view of the data even if other users are editing. This is useful
    /// for long-running queries or reports that need to see a snapshot of the data
    /// at a specific moment in time.
    ///
    /// Read sessions do not prevent others from editing or reading - multiple read
    /// sessions can exist simultaneously on the same version.
    ///
    /// # Arguments
    ///
    /// * `version_guid` - The GUID of the version to read
    /// * `session_id` - A unique identifier for this read session
    ///
    /// # Returns
    ///
    /// Returns a [`StartReadingResponse`] indicating success or failure.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The version doesn't exist
    /// - The user doesn't have read permissions
    /// - Authentication fails
    /// - Network error occurs
    ///
    /// # Example
    ///
    /// ```no_run
    /// use arcgis::{ArcGISClient, ClientCredentialsAuth, VersionManagementClient, SessionId};
    /// use uuid::Uuid;
    ///
    /// # async fn example() -> arcgis::Result<()> {
    /// # let auth = ClientCredentialsAuth::new("id".to_string(), "secret".to_string()).expect("Valid");
    /// # let client = ArcGISClient::new(auth);
    /// # let vm_client = VersionManagementClient::new("url", &client);
    /// let version_guid = Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000")
    ///     .expect("Valid UUID");
    /// let session_id = SessionId::new();
    ///
    /// let response = vm_client
    ///     .start_reading(version_guid.into(), session_id)
    ///     .await?;
    ///
    /// if *response.success() {
    ///     println!("Read session started at {:?}", response.moment());
    ///
    ///     // Perform queries with consistent view of data...
    ///
    ///     // Stop the read session when done
    ///     vm_client.stop_reading(version_guid.into(), session_id).await?;
    /// }
    /// # Ok(())
    /// # }
    /// ```
    #[instrument(skip(self), fields(base_url = %self.base_url, version_guid = %version_guid, session_id = %session_id))]
    pub async fn start_reading(
        &self,
        version_guid: VersionGuid,
        session_id: SessionId,
    ) -> Result<StartReadingResponse> {
        tracing::debug!(
            version_guid = %version_guid,
            session_id = %session_id,
            "Starting read session"
        );

        let url = format!("{}/versions/{}/startReading", self.base_url, version_guid);
        let token = self.client.auth().get_token().await?;

        tracing::debug!(url = %url, "Sending startReading request");

        let response = self
            .client
            .http()
            .post(&url)
            .form(&[
                ("sessionId", session_id.to_string().as_str()),
                ("f", "json"),
                ("token", token.as_str()),
            ])
            .send()
            .await?;

        let status = response.status();
        if !status.is_success() {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|e| format!("Failed to read error: {}", e));
            tracing::error!(status = %status, error = %error_text, "startReading failed");
            return Err(crate::Error::from(crate::ErrorKind::Api {
                code: status.as_u16() as i32,
                message: format!("HTTP {}: {}", status, error_text),
            }));
        }

        let start_response: StartReadingResponse = response.json().await?;

        if *start_response.success() {
            tracing::info!(
                session_id = %session_id,
                moment = ?start_response.moment(),
                "Read session started successfully"
            );
        } else {
            tracing::warn!(
                session_id = %session_id,
                error = ?start_response.error(),
                "startReading reported failure"
            );
        }

        Ok(start_response)
    }

    /// Stops a read session on a version.
    ///
    /// Stopping a read session releases the read lock on the version. This should
    /// be called when you're finished with queries that needed a consistent view.
    ///
    /// # Arguments
    ///
    /// * `version_guid` - The GUID of the version being read
    /// * `session_id` - The session identifier from [`start_reading`](Self::start_reading)
    ///
    /// # Returns
    ///
    /// Returns a [`StopReadingResponse`] indicating success or failure.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The session doesn't exist
    /// - The session ID doesn't match
    /// - Authentication fails
    /// - Network error occurs
    ///
    /// # Example
    ///
    /// ```no_run
    /// use arcgis::{ArcGISClient, ClientCredentialsAuth, VersionManagementClient, SessionId};
    /// use uuid::Uuid;
    ///
    /// # async fn example() -> arcgis::Result<()> {
    /// # let auth = ClientCredentialsAuth::new("id".to_string(), "secret".to_string()).expect("Valid");
    /// # let client = ArcGISClient::new(auth);
    /// # let vm_client = VersionManagementClient::new("url", &client);
    /// # let version_guid = Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000")
    /// #     .expect("Valid UUID");
    /// # let session_id = SessionId::new();
    /// # vm_client.start_reading(version_guid.into(), session_id).await?;
    /// // Stop the read session
    /// let response = vm_client
    ///     .stop_reading(version_guid.into(), session_id)
    ///     .await?;
    ///
    /// if *response.success() {
    ///     println!("Read session stopped successfully");
    /// }
    /// # Ok(())
    /// # }
    /// ```
    #[instrument(skip(self), fields(base_url = %self.base_url, version_guid = %version_guid, session_id = %session_id))]
    pub async fn stop_reading(
        &self,
        version_guid: VersionGuid,
        session_id: SessionId,
    ) -> Result<StopReadingResponse> {
        tracing::debug!(
            version_guid = %version_guid,
            session_id = %session_id,
            "Stopping read session"
        );

        let url = format!("{}/versions/{}/stopReading", self.base_url, version_guid);
        let token = self.client.auth().get_token().await?;

        tracing::debug!(url = %url, "Sending stopReading request");

        let response = self
            .client
            .http()
            .post(&url)
            .form(&[
                ("sessionId", session_id.to_string().as_str()),
                ("f", "json"),
                ("token", token.as_str()),
            ])
            .send()
            .await?;

        let status = response.status();
        if !status.is_success() {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|e| format!("Failed to read error: {}", e));
            tracing::error!(status = %status, error = %error_text, "stopReading failed");
            return Err(crate::Error::from(crate::ErrorKind::Api {
                code: status.as_u16() as i32,
                message: format!("HTTP {}: {}", status, error_text),
            }));
        }

        let stop_response: StopReadingResponse = response.json().await?;

        if *stop_response.success() {
            tracing::info!(
                session_id = %session_id,
                moment = ?stop_response.moment(),
                "Read session stopped successfully"
            );
        } else {
            tracing::warn!(
                session_id = %session_id,
                error = ?stop_response.error(),
                "stopReading reported failure"
            );
        }

        Ok(stop_response)
    }

    /// Creates a new version from the DEFAULT version.
    ///
    /// Creates a named version that branches from the DEFAULT version. This is the
    /// starting point for versioned editing workflows.
    ///
    /// # Arguments
    ///
    /// * `params` - Parameters for creating the version (name, access level, description)
    ///
    /// # Returns
    ///
    /// Returns a [`CreateVersionResponse`] with the newly created version's information.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - A version with the same name already exists
    /// - The user doesn't have permission to create versions
    /// - Authentication fails
    /// - Network error occurs
    ///
    /// # Example
    ///
    /// ```no_run
    /// use arcgis::{
    ///     ArcGISClient, ClientCredentialsAuth, VersionManagementClient,
    ///     CreateVersionParams, VersionPermission,
    /// };
    ///
    /// # async fn example() -> arcgis::Result<()> {
    /// # let auth = ClientCredentialsAuth::new("id".to_string(), "secret".to_string()).expect("Valid");
    /// # let client = ArcGISClient::new(auth);
    /// # let vm_client = VersionManagementClient::new("url", &client);
    /// let params = CreateVersionParams::new("workplan_2024", VersionPermission::Public)
    ///     .with_description("Work plan for 2024 projects");
    ///
    /// let response = vm_client.create(params).await?;
    ///
    /// if *response.success() {
    ///     if let Some(version_info) = response.version_info() {
    ///         println!("Created version: {}", version_info.version_name());
    ///     }
    /// }
    /// # Ok(())
    /// # }
    /// ```
    #[instrument(skip(self, params), fields(base_url = %self.base_url, version_name = %params.version_name))]
    pub async fn create(&self, params: CreateVersionParams) -> Result<CreateVersionResponse> {
        tracing::debug!(
            version_name = %params.version_name,
            access = %params.access,
            "Creating new version"
        );

        let url = format!("{}/create", self.base_url);
        let token = self.client.auth().get_token().await?;

        let access_str = params.access.to_string();
        let mut form = vec![
            ("versionName", params.version_name.as_str()),
            ("access", access_str.as_str()),
            ("f", "json"),
            ("token", token.as_str()),
        ];

        if let Some(ref description) = params.description {
            form.push(("description", description.as_str()));
        }

        tracing::debug!(url = %url, "Sending create request");

        let response = self.client.http().post(&url).form(&form).send().await?;

        let status = response.status();
        if !status.is_success() {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|e| format!("Failed to read error: {}", e));
            tracing::error!(status = %status, error = %error_text, "create failed");
            return Err(crate::Error::from(crate::ErrorKind::Api {
                code: status.as_u16() as i32,
                message: format!("HTTP {}: {}", status, error_text),
            }));
        }

        let create_response: CreateVersionResponse = response.json().await?;

        if *create_response.success() {
            tracing::info!(
                version_name = %params.version_name,
                "Version created successfully"
            );
        } else {
            tracing::warn!(
                version_name = %params.version_name,
                error = ?create_response.error(),
                "create reported failure"
            );
        }

        Ok(create_response)
    }

    /// Alters an existing version's properties.
    ///
    /// Allows changing a version's name, description, or access permission level.
    ///
    /// # Arguments
    ///
    /// * `version_guid` - The GUID of the version to alter
    /// * `params` - Parameters specifying which properties to change
    ///
    /// # Returns
    ///
    /// Returns an [`AlterResponse`] indicating success or failure.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The version doesn't exist
    /// - The user doesn't have permission to alter the version
    /// - Authentication fails
    /// - Network error occurs
    ///
    /// # Example
    ///
    /// ```no_run
    /// use arcgis::{
    ///     ArcGISClient, ClientCredentialsAuth, VersionManagementClient,
    ///     AlterVersionParams, VersionPermission,
    /// };
    /// use uuid::Uuid;
    ///
    /// # async fn example() -> arcgis::Result<()> {
    /// # let auth = ClientCredentialsAuth::new("id".to_string(), "secret".to_string()).expect("Valid");
    /// # let client = ArcGISClient::new(auth);
    /// # let vm_client = VersionManagementClient::new("url", &client);
    /// let version_guid = Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000")
    ///     .expect("Valid UUID");
    ///
    /// let params = AlterVersionParams::new()
    ///     .with_access(VersionPermission::Protected)
    ///     .with_description("Updated description");
    ///
    /// let response = vm_client.alter(version_guid.into(), params).await?;
    ///
    /// if *response.success() {
    ///     println!("Version altered successfully");
    /// }
    /// # Ok(())
    /// # }
    /// ```
    #[instrument(skip(self, params), fields(base_url = %self.base_url, version_guid = %version_guid))]
    pub async fn alter(
        &self,
        version_guid: VersionGuid,
        params: AlterVersionParams,
    ) -> Result<AlterResponse> {
        tracing::debug!(version_guid = %version_guid, "Altering version");

        let url = format!("{}/versions/{}/alter", self.base_url, version_guid);
        let token = self.client.auth().get_token().await?;

        let mut form = vec![("f", "json"), ("token", token.as_str())];

        let version_name_str;
        if let Some(ref version_name) = params.version_name {
            version_name_str = version_name.clone();
            form.push(("versionName", version_name_str.as_str()));
        }

        let access_str;
        if let Some(ref access) = params.access {
            access_str = access.to_string();
            form.push(("access", access_str.as_str()));
        }

        let description_str;
        if let Some(ref description) = params.description {
            description_str = description.clone();
            form.push(("description", description_str.as_str()));
        }

        tracing::debug!(url = %url, "Sending alter request");

        let response = self.client.http().post(&url).form(&form).send().await?;

        let status = response.status();
        if !status.is_success() {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|e| format!("Failed to read error: {}", e));
            tracing::error!(status = %status, error = %error_text, "alter failed");
            return Err(crate::Error::from(crate::ErrorKind::Api {
                code: status.as_u16() as i32,
                message: format!("HTTP {}: {}", status, error_text),
            }));
        }

        let alter_response: AlterResponse = response.json().await?;

        if *alter_response.success() {
            tracing::info!(
                version_guid = %version_guid,
                moment = ?alter_response.moment(),
                "Version altered successfully"
            );
        } else {
            tracing::warn!(
                version_guid = %version_guid,
                error = ?alter_response.error(),
                "alter reported failure"
            );
        }

        Ok(alter_response)
    }

    /// Deletes a version.
    ///
    /// Permanently removes a version from the geodatabase. This operation cannot be undone.
    ///
    /// **Warning**: This operation is irreversible. All edits in the version that have
    /// not been posted to the parent version will be lost.
    ///
    /// # Arguments
    ///
    /// * `version_guid` - The GUID of the version to delete
    ///
    /// # Returns
    ///
    /// Returns a [`DeleteResponse`] indicating success or failure.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The version doesn't exist
    /// - The version is the DEFAULT version (cannot be deleted)
    /// - The user doesn't have permission to delete the version
    /// - Authentication fails
    /// - Network error occurs
    ///
    /// # Example
    ///
    /// ```no_run
    /// use arcgis::{ArcGISClient, ClientCredentialsAuth, VersionManagementClient};
    /// use uuid::Uuid;
    ///
    /// # async fn example() -> arcgis::Result<()> {
    /// # let auth = ClientCredentialsAuth::new("id".to_string(), "secret".to_string()).expect("Valid");
    /// # let client = ArcGISClient::new(auth);
    /// # let vm_client = VersionManagementClient::new("url", &client);
    /// let version_guid = Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000")
    ///     .expect("Valid UUID");
    ///
    /// let response = vm_client.delete(version_guid.into()).await?;
    ///
    /// if *response.success() {
    ///     println!("Version deleted successfully");
    /// } else {
    ///     eprintln!("Failed to delete version: {:?}", response.error());
    /// }
    /// # Ok(())
    /// # }
    /// ```
    #[instrument(skip(self), fields(base_url = %self.base_url, version_guid = %version_guid))]
    pub async fn delete(&self, version_guid: VersionGuid) -> Result<DeleteResponse> {
        tracing::debug!(version_guid = %version_guid, "Deleting version");

        let url = format!("{}/versions/{}/delete", self.base_url, version_guid);
        let token = self.client.auth().get_token().await?;

        tracing::debug!(url = %url, "Sending delete request");

        let response = self
            .client
            .http()
            .post(&url)
            .form(&[("f", "json"), ("token", token.as_str())])
            .send()
            .await?;

        let status = response.status();
        if !status.is_success() {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|e| format!("Failed to read error: {}", e));
            tracing::error!(status = %status, error = %error_text, "delete failed");
            return Err(crate::Error::from(crate::ErrorKind::Api {
                code: status.as_u16() as i32,
                message: format!("HTTP {}: {}", status, error_text),
            }));
        }

        let delete_response: DeleteResponse = response.json().await?;

        if *delete_response.success() {
            tracing::info!(
                version_guid = %version_guid,
                moment = ?delete_response.moment(),
                "Version deleted successfully"
            );
        } else {
            tracing::warn!(
                version_guid = %version_guid,
                error = ?delete_response.error(),
                "delete reported failure"
            );
        }

        Ok(delete_response)
    }

    /// Gets information about a specific version.
    ///
    /// Retrieves metadata for a version including its name, description, access level,
    /// and creation/modification timestamps.
    ///
    /// # Arguments
    ///
    /// * `version_guid` - The GUID of the version to query
    ///
    /// # Returns
    ///
    /// Returns a [`VersionInfo`] struct with the version's metadata.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The version doesn't exist
    /// - Authentication fails
    /// - Network error occurs
    ///
    /// # Example
    ///
    /// ```no_run
    /// use arcgis::{ArcGISClient, ClientCredentialsAuth, VersionManagementClient};
    /// use uuid::Uuid;
    ///
    /// # async fn example() -> arcgis::Result<()> {
    /// # let auth = ClientCredentialsAuth::new("id".to_string(), "secret".to_string()).expect("Valid");
    /// # let client = ArcGISClient::new(auth);
    /// # let vm_client = VersionManagementClient::new("url", &client);
    /// let version_guid = Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000")
    ///     .expect("Valid UUID");
    ///
    /// let version_info = vm_client.get_info(version_guid.into()).await?;
    ///
    /// println!("Version: {}", version_info.version_name());
    /// println!("Description: {:?}", version_info.description());
    /// # Ok(())
    /// # }
    /// ```
    #[instrument(skip(self), fields(base_url = %self.base_url, version_guid = %version_guid))]
    pub async fn get_info(&self, version_guid: VersionGuid) -> Result<VersionInfo> {
        tracing::debug!(version_guid = %version_guid, "Getting version info");

        let url = format!("{}/versions/{}", self.base_url, version_guid);
        let token = self.client.auth().get_token().await?;

        tracing::debug!(url = %url, "Sending get version info request");

        let response = self
            .client
            .http()
            .get(&url)
            .query(&[("f", "json"), ("token", token.as_str())])
            .send()
            .await?;

        let status = response.status();
        if !status.is_success() {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|e| format!("Failed to read error: {}", e));
            tracing::error!(status = %status, error = %error_text, "get version info failed");
            return Err(crate::Error::from(crate::ErrorKind::Api {
                code: status.as_u16() as i32,
                message: format!("HTTP {}: {}", status, error_text),
            }));
        }

        let version_info: VersionInfo = response.json().await?;

        tracing::info!(
            version_guid = %version_guid,
            version_name = version_info.version_name(),
            "Version info retrieved"
        );

        Ok(version_info)
    }

    /// Lists all versions available in the Version Management Service.
    ///
    /// Returns a list of all versions that the authenticated user has access to view.
    ///
    /// # Returns
    ///
    /// Returns a [`VersionInfosResponse`] containing a vector of [`VersionInfo`] structs.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Authentication fails
    /// - Network error occurs
    ///
    /// # Example
    ///
    /// ```no_run
    /// use arcgis::{ArcGISClient, ClientCredentialsAuth, VersionManagementClient};
    ///
    /// # async fn example() -> arcgis::Result<()> {
    /// # let auth = ClientCredentialsAuth::new("id".to_string(), "secret".to_string()).expect("Valid");
    /// # let client = ArcGISClient::new(auth);
    /// # let vm_client = VersionManagementClient::new("url", &client);
    /// let response = vm_client.list_versions().await?;
    ///
    /// println!("Found {} versions", response.versions().len());
    ///
    /// for version in response.versions() {
    ///     println!("  - {}: {:?}", version.version_name(), version.description());
    /// }
    /// # Ok(())
    /// # }
    /// ```
    #[instrument(skip(self), fields(base_url = %self.base_url))]
    pub async fn list_versions(&self) -> Result<VersionInfosResponse> {
        tracing::debug!("Listing all versions");

        let url = format!("{}/versionInfos", self.base_url);
        let token = self.client.auth().get_token().await?;

        tracing::debug!(url = %url, "Sending list versions request");

        let response = self
            .client
            .http()
            .get(&url)
            .query(&[("f", "json"), ("token", token.as_str())])
            .send()
            .await?;

        let status = response.status();
        if !status.is_success() {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|e| format!("Failed to read error: {}", e));
            tracing::error!(status = %status, error = %error_text, "list versions failed");
            return Err(crate::Error::from(crate::ErrorKind::Api {
                code: status.as_u16() as i32,
                message: format!("HTTP {}: {}", status, error_text),
            }));
        }

        let versions_response: VersionInfosResponse = response.json().await?;

        tracing::info!(
            version_count = versions_response.versions().len(),
            "Versions list retrieved"
        );

        Ok(versions_response)
    }

    /// Reconciles a version against the DEFAULT version.
    ///
    /// Reconciliation compares the current version against the DEFAULT version,
    /// identifying differences and detecting conflicts based on the specified
    /// conflict detection type. This is a required step before posting changes.
    ///
    /// **Important**: Reconcile requires an exclusive write lock on the version.
    /// You must have started an edit session and no read sessions can be active.
    ///
    /// # Arguments
    ///
    /// * `version_guid` - The GUID of the version to reconcile
    /// * `session_id` - The session ID from the active edit session
    /// * `abort_if_conflicts` - If `true`, abort if conflicts are detected
    /// * `conflict_detection` - Type of conflict detection (ByObject or ByAttribute)
    /// * `with_post` - If `true`, automatically post after successful reconcile
    ///
    /// # Returns
    ///
    /// Returns a [`ReconcileResponse`] with conflict information and post status.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The version doesn't exist
    /// - No active edit session with matching session ID
    /// - Read locks exist on the version
    /// - The user doesn't have edit permissions
    /// - Conflicts are detected and `abort_if_conflicts` is true
    /// - Authentication fails
    /// - Network error occurs
    ///
    /// # Example
    ///
    /// ```no_run
    /// use arcgis::{
    ///     ArcGISClient, ClientCredentialsAuth, VersionManagementClient,
    ///     SessionId, ConflictDetection,
    /// };
    /// use uuid::Uuid;
    ///
    /// # async fn example() -> arcgis::Result<()> {
    /// # let auth = ClientCredentialsAuth::new("id".to_string(), "secret".to_string()).expect("Valid");
    /// # let client = ArcGISClient::new(auth);
    /// # let vm_client = VersionManagementClient::new("url", &client);
    /// let version_guid = Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000")
    ///     .expect("Valid UUID");
    /// let session_id = SessionId::new();
    ///
    /// // Start edit session first
    /// vm_client.start_editing(version_guid.into(), session_id).await?;
    ///
    /// // Perform edits...
    ///
    /// // Reconcile with DEFAULT version
    /// let response = vm_client.reconcile(
    ///     version_guid.into(),
    ///     session_id,
    ///     true, // abort if conflicts
    ///     ConflictDetection::ByObject,
    ///     false, // don't auto-post
    /// ).await?;
    ///
    /// if *response.success() {
    ///     if response.has_conflicts().as_ref().map_or(false, |x| *x) {
    ///         println!("Conflicts detected - resolve before posting");
    ///     } else {
    ///         println!("Reconcile successful, ready to post");
    ///     }
    /// }
    /// # Ok(())
    /// # }
    /// ```
    #[instrument(skip(self), fields(
        base_url = %self.base_url,
        version_guid = %version_guid,
        session_id = %session_id,
        abort_if_conflicts,
        conflict_detection = %conflict_detection,
        with_post
    ))]
    pub async fn reconcile(
        &self,
        version_guid: VersionGuid,
        session_id: SessionId,
        abort_if_conflicts: bool,
        conflict_detection: ConflictDetection,
        with_post: bool,
    ) -> Result<ReconcileResponse> {
        tracing::debug!(
            version_guid = %version_guid,
            session_id = %session_id,
            abort_if_conflicts = abort_if_conflicts,
            conflict_detection = %conflict_detection,
            with_post = with_post,
            "Reconciling version"
        );

        let url = format!("{}/versions/{}/reconcile", self.base_url, version_guid);
        let token = self.client.auth().get_token().await?;

        let abort_str = if abort_if_conflicts { "true" } else { "false" };
        let with_post_str = if with_post { "true" } else { "false" };
        let conflict_detection_str = conflict_detection.to_string();

        tracing::debug!(url = %url, "Sending reconcile request");

        let response = self
            .client
            .http()
            .post(&url)
            .form(&[
                ("sessionId", session_id.to_string().as_str()),
                ("abortIfConflicts", abort_str),
                ("conflictDetection", conflict_detection_str.as_str()),
                ("withPost", with_post_str),
                ("f", "json"),
                ("token", token.as_str()),
            ])
            .send()
            .await?;

        let status = response.status();
        if !status.is_success() {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|e| format!("Failed to read error: {}", e));
            tracing::error!(status = %status, error = %error_text, "reconcile failed");
            return Err(crate::Error::from(crate::ErrorKind::Api {
                code: status.as_u16() as i32,
                message: format!("HTTP {}: {}", status, error_text),
            }));
        }

        let reconcile_response: ReconcileResponse = response.json().await?;

        if *reconcile_response.success() {
            tracing::info!(
                version_guid = %version_guid,
                has_conflicts = ?reconcile_response.has_conflicts(),
                did_post = ?reconcile_response.did_post(),
                moment = ?reconcile_response.moment(),
                "Reconcile completed successfully"
            );
        } else {
            tracing::warn!(
                version_guid = %version_guid,
                error = ?reconcile_response.error(),
                "reconcile reported failure"
            );
        }

        Ok(reconcile_response)
    }

    /// Posts changes from a version to the DEFAULT version.
    ///
    /// Posting applies the edits made in the current version to the DEFAULT version.
    /// This operation must be preceded by a successful reconcile operation with no
    /// unresolved conflicts.
    ///
    /// **Important**: The session ID must match the one used for reconcile, and the
    /// DEFAULT version must not have been modified since the reconcile.
    ///
    /// # Arguments
    ///
    /// * `version_guid` - The GUID of the version to post
    /// * `session_id` - The session ID from the active edit session (must match reconcile)
    /// * `partial_rows` - Optional subset of edits to post (for partial post)
    ///
    /// # Returns
    ///
    /// Returns a [`PostResponse`] indicating success or failure.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The version doesn't exist
    /// - No active edit session with matching session ID
    /// - Reconcile was not performed first
    /// - Session ID doesn't match the reconcile session
    /// - DEFAULT version was modified since reconcile
    /// - Unresolved conflicts exist
    /// - Authentication fails
    /// - Network error occurs
    ///
    /// # Example
    ///
    /// ```no_run
    /// use arcgis::{
    ///     ArcGISClient, ClientCredentialsAuth, VersionManagementClient,
    ///     SessionId, ConflictDetection,
    /// };
    /// use uuid::Uuid;
    ///
    /// # async fn example() -> arcgis::Result<()> {
    /// # let auth = ClientCredentialsAuth::new("id".to_string(), "secret".to_string()).expect("Valid");
    /// # let client = ArcGISClient::new(auth);
    /// # let vm_client = VersionManagementClient::new("url", &client);
    /// let version_guid = Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000")
    ///     .expect("Valid UUID");
    /// let session_id = SessionId::new();
    ///
    /// // Start edit session
    /// vm_client.start_editing(version_guid.into(), session_id).await?;
    ///
    /// // Perform edits...
    ///
    /// // Reconcile first
    /// let reconcile_response = vm_client.reconcile(
    ///     version_guid.into(),
    ///     session_id,
    ///     true,
    ///     ConflictDetection::ByObject,
    ///     false,
    /// ).await?;
    ///
    /// if !reconcile_response.has_conflicts().as_ref().map_or(false, |x| *x) {
    ///     // No conflicts - post changes
    ///     let post_response = vm_client.post(
    ///         version_guid.into(),
    ///         session_id,
    ///         None, // post all edits
    ///     ).await?;
    ///
    ///     if *post_response.success() {
    ///         println!("Changes posted to DEFAULT successfully");
    ///     }
    /// }
    ///
    /// // Stop editing and save
    /// vm_client.stop_editing(version_guid.into(), session_id, true).await?;
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Partial Post Example
    ///
    /// ```no_run
    /// use arcgis::{ArcGISClient, ClientCredentialsAuth, VersionManagementClient, PartialPostRow};
    /// # use arcgis::SessionId;
    /// # use uuid::Uuid;
    ///
    /// # async fn example() -> arcgis::Result<()> {
    /// # let auth = ClientCredentialsAuth::new("id".to_string(), "secret".to_string()).expect("Valid");
    /// # let client = ArcGISClient::new(auth);
    /// # let vm_client = VersionManagementClient::new("url", &client);
    /// # let version_guid = Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000")
    /// #     .expect("Valid UUID");
    /// # let session_id = SessionId::new();
    /// // Post only specific features from specific layers
    /// let partial_rows = vec![
    ///     PartialPostRow::new(0, vec![1, 2, 3]), // Layer 0, objects 1-3
    ///     PartialPostRow::new(1, vec![10, 20]),  // Layer 1, objects 10, 20
    /// ];
    ///
    /// let response = vm_client.post(
    ///     version_guid.into(),
    ///     session_id,
    ///     Some(partial_rows),
    /// ).await?;
    /// # Ok(())
    /// # }
    /// ```
    #[instrument(skip(self, partial_rows), fields(
        base_url = %self.base_url,
        version_guid = %version_guid,
        session_id = %session_id,
        partial_post = partial_rows.is_some()
    ))]
    pub async fn post(
        &self,
        version_guid: VersionGuid,
        session_id: SessionId,
        partial_rows: Option<Vec<PartialPostRow>>,
    ) -> Result<PostResponse> {
        tracing::debug!(
            version_guid = %version_guid,
            session_id = %session_id,
            partial_post = partial_rows.is_some(),
            "Posting changes to DEFAULT version"
        );

        let url = format!("{}/versions/{}/post", self.base_url, version_guid);
        let token = self.client.auth().get_token().await?;

        let mut form = vec![
            ("sessionId", session_id.to_string()),
            ("f", "json".to_string()),
            ("token", token.to_string()),
        ];

        // Serialize partial_rows if provided
        let rows_json;
        if let Some(rows) = partial_rows {
            rows_json = serde_json::to_string(&rows)?;
            form.push(("rows", rows_json));
        }

        tracing::debug!(url = %url, "Sending post request");

        let form_refs: Vec<(&str, &str)> = form.iter().map(|(k, v)| (*k, v.as_str())).collect();

        let response = self
            .client
            .http()
            .post(&url)
            .form(&form_refs)
            .send()
            .await?;

        let status = response.status();
        if !status.is_success() {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|e| format!("Failed to read error: {}", e));
            tracing::error!(status = %status, error = %error_text, "post failed");
            return Err(crate::Error::from(crate::ErrorKind::Api {
                code: status.as_u16() as i32,
                message: format!("HTTP {}: {}", status, error_text),
            }));
        }

        let post_response: PostResponse = response.json().await?;

        if *post_response.success() {
            tracing::info!(
                version_guid = %version_guid,
                moment = ?post_response.moment(),
                "Changes posted successfully"
            );
        } else {
            tracing::warn!(
                version_guid = %version_guid,
                error = ?post_response.error(),
                "post reported failure"
            );
        }

        Ok(post_response)
    }
}
