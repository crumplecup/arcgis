//! Reconciliation and post workflow operations for the Version Management client.

use super::super::{
    ConflictDetection, PartialPostRow, PostResponse, ReconcileResponse, SessionId, VersionGuid,
};
use super::VersionManagementClient;
use crate::Result;
use tracing::instrument;

impl<'a> VersionManagementClient<'a> {
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

        let abort_str = if abort_if_conflicts { "true" } else { "false" };
        let with_post_str = if with_post { "true" } else { "false" };
        let conflict_detection_str = conflict_detection.to_string();

        tracing::debug!(url = %url, "Sending reconcile request");

        let session_id_str = session_id.to_string();
        let mut form = vec![
            ("sessionId", session_id_str.as_str()),
            ("abortIfConflicts", abort_str),
            ("conflictDetection", conflict_detection_str.as_str()),
            ("withPost", with_post_str),
            ("f", "json"),
        ];

        // Add token if required by auth provider
        let token_opt = self.client.get_token_if_required().await?;
        let token_str;
        if let Some(token) = token_opt {
            token_str = token;
            form.push(("token", token_str.as_str()));
        }

        let response = self.client.http().post(&url).form(&form).send().await?;

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

        let mut form = vec![
            ("sessionId", session_id.to_string()),
            ("f", "json".to_string()),
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
