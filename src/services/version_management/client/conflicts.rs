//! Conflict detection and resolution operations for the Version Management client.

use super::super::{
    ConflictsResponse, InspectConflictLayer, InspectConflictsResponse, RestoreRowsLayer,
    RestoreRowsResponse, SessionId, VersionGuid,
};
use super::VersionManagementClient;
use crate::Result;
use tracing::instrument;

impl<'a> VersionManagementClient<'a> {
    /// Retrieves conflicts detected during the last reconcile operation.
    ///
    /// Returns all conflicts organized by layer and type (update-update, update-delete,
    /// delete-update). Conflicts must be reviewed and resolved before posting can succeed.
    ///
    /// # Arguments
    ///
    /// * `version_guid` - The GUID of the version to query conflicts for
    /// * `session_id` - The session ID from the active edit session
    ///
    /// # Returns
    ///
    /// Returns a [`ConflictsResponse`] with all conflicts organized by layer.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The version doesn't exist
    /// - No active edit session with matching session ID
    /// - Reconcile has not been performed
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
    /// // Start edit session and reconcile
    /// vm_client.start_editing(version_guid.into(), session_id).await?;
    /// let reconcile_response = vm_client.reconcile(
    ///     version_guid.into(),
    ///     session_id,
    ///     false,
    ///     ConflictDetection::ByObject,
    ///     false,
    /// ).await?;
    ///
    /// if reconcile_response.has_conflicts().as_ref().map_or(false, |x| *x) {
    ///     // Retrieve detailed conflict information
    ///     let conflicts = vm_client.conflicts(version_guid.into(), session_id).await?;
    ///
    ///     if let Some(conflict_layers) = conflicts.conflicts() {
    ///         for layer in conflict_layers {
    ///             println!("Layer {} has conflicts", layer.layer_id());
    ///         }
    ///     }
    /// }
    /// # Ok(())
    /// # }
    /// ```
    #[instrument(skip(self), fields(base_url = %self.base_url, version_guid = %version_guid, session_id = %session_id))]
    pub async fn conflicts(
        &self,
        version_guid: VersionGuid,
        session_id: SessionId,
    ) -> Result<ConflictsResponse> {
        tracing::debug!(
            version_guid = %version_guid,
            session_id = %session_id,
            "Retrieving conflicts"
        );

        let url = format!("{}/versions/{}/conflicts", self.base_url, version_guid);

        tracing::debug!(url = %url, "Sending conflicts request");

            let session_id_str = session_id.to_string();
            let mut form = vec![
                ("sessionId", session_id_str.as_str()),
                ("f", "json")
            ];

            // Add token if required by auth provider
            let token_opt = self.client.get_token_if_required().await?;
            let token_str;
            if let Some(token) = token_opt {
                token_str = token;
                form.push(("token", token_str.as_str()));
            }

        let response = self
            .client
            .http()
            .post(&url)
            .form(&form)
            .send()
            .await?;

        let status = response.status();
        if !status.is_success() {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|e| format!("Failed to read error: {}", e));
            tracing::error!(status = %status, error = %error_text, "conflicts failed");
            return Err(crate::Error::from(crate::ErrorKind::Api {
                code: status.as_u16() as i32,
                message: format!("HTTP {}: {}", status, error_text),
            }));
        }

        let conflicts_response: ConflictsResponse = response.json().await?;

        if *conflicts_response.success() {
            let conflict_count = conflicts_response
                .conflicts()
                .as_ref()
                .map(|c| c.len())
                .unwrap_or(0);
            tracing::info!(
                version_guid = %version_guid,
                conflict_count = conflict_count,
                "Conflicts retrieved"
            );
        } else {
            tracing::warn!(
                version_guid = %version_guid,
                error = ?conflicts_response.error(),
                "conflicts reported failure"
            );
        }

        Ok(conflicts_response)
    }

    /// Marks conflicts as inspected.
    ///
    /// Sets the inspection status for specific conflicts or all conflicts. This is
    /// used to track which conflicts have been reviewed during the conflict resolution
    /// process.
    ///
    /// # Arguments
    ///
    /// * `version_guid` - The GUID of the version
    /// * `session_id` - The session ID from the active edit session
    /// * `inspect_all` - If `true`, inspect all conflicts; `conflicts` parameter is ignored
    /// * `set_inspected` - If `true`, mark as inspected; if `false`, mark as not inspected
    /// * `conflicts` - Specific conflicts to inspect (ignored if `inspect_all` is `true`)
    ///
    /// # Returns
    ///
    /// Returns an [`InspectConflictsResponse`] indicating success or failure.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The version doesn't exist
    /// - No active edit session with matching session ID
    /// - Authentication fails
    /// - Network error occurs
    ///
    /// # Example
    ///
    /// ```no_run
    /// use arcgis::{
    ///     ArcGISClient, ClientCredentialsAuth, VersionManagementClient,
    ///     SessionId, InspectConflictLayer, InspectConflictFeature,
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
    /// // Mark specific conflicts as inspected
    /// let conflicts_to_inspect = vec![
    ///     InspectConflictLayer::new(0, vec![
    ///         InspectConflictFeature::new(1)
    ///             .with_note("Accepting default version"),
    ///         InspectConflictFeature::new(2)
    ///             .with_note("Accepting branch version"),
    ///     ]),
    /// ];
    ///
    /// vm_client.inspect_conflicts(
    ///     version_guid.into(),
    ///     session_id,
    ///     false,
    ///     true,
    ///     Some(conflicts_to_inspect),
    /// ).await?;
    ///
    /// // Or mark all conflicts as inspected
    /// vm_client.inspect_conflicts(
    ///     version_guid.into(),
    ///     session_id,
    ///     true,
    ///     true,
    ///     None,
    /// ).await?;
    /// # Ok(())
    /// # }
    /// ```
    #[instrument(skip(self, conflicts), fields(
        base_url = %self.base_url,
        version_guid = %version_guid,
        session_id = %session_id,
        inspect_all,
        set_inspected
    ))]
    pub async fn inspect_conflicts(
        &self,
        version_guid: VersionGuid,
        session_id: SessionId,
        inspect_all: bool,
        set_inspected: bool,
        conflicts: Option<Vec<InspectConflictLayer>>,
    ) -> Result<InspectConflictsResponse> {
        tracing::debug!(
            version_guid = %version_guid,
            session_id = %session_id,
            inspect_all = inspect_all,
            set_inspected = set_inspected,
            "Inspecting conflicts"
        );

        let url = format!(
            "{}/versions/{}/inspectConflicts",
            self.base_url, version_guid
        );

        let inspect_all_str = if inspect_all { "true" } else { "false" };
        let set_inspected_str = if set_inspected { "true" } else { "false" };

        let mut form = vec![
            ("sessionId", session_id.to_string()),
            ("inspectAll", inspect_all_str.to_string()),
            ("setInspected", set_inspected_str.to_string()),
            ("f", "json".to_string()),
        ];

        // Serialize conflicts if provided and not inspecting all
        let conflicts_json;
        if !inspect_all {
            if let Some(c) = conflicts {
                conflicts_json = serde_json::to_string(&c)?;
                form.push(("conflicts", conflicts_json));
            }
        }

        tracing::debug!(url = %url, "Sending inspect conflicts request");

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
            tracing::error!(status = %status, error = %error_text, "inspect conflicts failed");
            return Err(crate::Error::from(crate::ErrorKind::Api {
                code: status.as_u16() as i32,
                message: format!("HTTP {}: {}", status, error_text),
            }));
        }

        let inspect_response: InspectConflictsResponse = response.json().await?;

        if *inspect_response.success() {
            tracing::info!(
                version_guid = %version_guid,
                inspect_all = inspect_all,
                "Conflicts inspection completed"
            );
        } else {
            tracing::warn!(
                version_guid = %version_guid,
                error = ?inspect_response.error(),
                "inspect conflicts reported failure"
            );
        }

        Ok(inspect_response)
    }

    /// Restores rows from the common ancestor version.
    ///
    /// This operation is used to resolve Delete-Update conflicts identified during
    /// reconciliation. It restores features that were deleted in the branch version
    /// but updated in the default version, bringing them back from the ancestor state.
    ///
    /// # Arguments
    ///
    /// * `version_guid` - The GUID of the version
    /// * `session_id` - The session ID from the active edit session
    /// * `rows` - Specifications of which features to restore in each layer
    ///
    /// # Returns
    ///
    /// Returns a [`RestoreRowsResponse`] indicating success or failure.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The version doesn't exist
    /// - No active edit session with matching session ID
    /// - The specified features don't exist in the ancestor version
    /// - Authentication fails
    /// - Network error occurs
    ///
    /// # Example
    ///
    /// ```no_run
    /// use arcgis::{
    ///     ArcGISClient, ClientCredentialsAuth, VersionManagementClient,
    ///     SessionId, RestoreRowsLayer,
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
    /// // After reconcile detects Delete-Update conflicts, restore specific features
    /// let rows_to_restore = vec![
    ///     RestoreRowsLayer::new(3, vec![1, 4, 5, 8]),
    ///     RestoreRowsLayer::new(5, vec![1, 4, 5, 9, 16, 35]),
    /// ];
    ///
    /// let response = vm_client.restore_rows(
    ///     version_guid.into(),
    ///     session_id,
    ///     rows_to_restore,
    /// ).await?;
    ///
    /// if *response.success() {
    ///     println!("Rows restored successfully at {:?}", response.moment());
    /// }
    ///
    /// // Continue with reconcile and post workflow...
    /// vm_client.stop_editing(version_guid.into(), session_id, true).await?;
    /// # Ok(())
    /// # }
    /// ```
    #[instrument(skip(self, rows), fields(
        base_url = %self.base_url,
        version_guid = %version_guid,
        session_id = %session_id
    ))]
    pub async fn restore_rows(
        &self,
        version_guid: VersionGuid,
        session_id: SessionId,
        rows: Vec<RestoreRowsLayer>,
    ) -> Result<RestoreRowsResponse> {
        tracing::debug!(
            version_guid = %version_guid,
            session_id = %session_id,
            "Restoring rows from ancestor"
        );

        let url = format!("{}/versions/{}/restoreRows", self.base_url, version_guid);

        // Serialize rows
        let rows_json = serde_json::to_string(&rows)?;

        tracing::debug!(url = %url, "Sending restore rows request");

        let form = [
            ("sessionId", session_id.to_string()),
            ("rows", rows_json),
            ("f", "json".to_string()),
        ];

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
            tracing::error!(status = %status, error = %error_text, "restore rows failed");
            return Err(crate::Error::from(crate::ErrorKind::Api {
                code: status.as_u16() as i32,
                message: format!("HTTP {}: {}", status, error_text),
            }));
        }

        let restore_response: RestoreRowsResponse = response.json().await?;

        if *restore_response.success() {
            tracing::info!(
                version_guid = %version_guid,
                moment = ?restore_response.moment(),
                "Rows restored successfully"
            );
        } else {
            tracing::warn!(
                version_guid = %version_guid,
                error = ?restore_response.error(),
                "restore rows reported failure"
            );
        }

        Ok(restore_response)
    }
}
