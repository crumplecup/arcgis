//! Session management operations for the Version Management client.

use super::super::{
    SessionId, StartEditingResponse, StartReadingResponse, StopEditingResponse,
    StopReadingResponse, VersionGuid,
};
use super::VersionManagementClient;
use crate::Result;
use tracing::instrument;

impl<'a> VersionManagementClient<'a> {
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
}
