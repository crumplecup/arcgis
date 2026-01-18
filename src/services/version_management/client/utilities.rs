//! Utility operations for the Version Management client.

use super::super::{
    DeleteForwardEditsResponse, DifferenceResultType, DifferencesResponse, SessionId, VersionGuid,
};
use super::VersionManagementClient;
use crate::Result;
use tracing::instrument;

impl<'a> VersionManagementClient<'a> {
    /// Deletes forward edits after a specified moment.
    ///
    /// This operation supports undo functionality by removing all edits made after
    /// a specific timestamp. It must be called before `stop_editing` when implementing
    /// undo/redo stacks.
    ///
    /// # Arguments
    ///
    /// * `version_guid` - The GUID of the version
    /// * `session_id` - The session ID from the active edit session
    /// * `moment` - Epoch time in milliseconds; must be >= version's modified date
    ///
    /// # Returns
    ///
    /// Returns a [`DeleteForwardEditsResponse`] indicating success or failure.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The version doesn't exist
    /// - No active edit session with matching session ID
    /// - The moment is before the version's modified date
    /// - Authentication fails
    /// - Network error occurs
    ///
    /// # Example
    ///
    /// ```no_run
    /// use arcgis::{ArcGISClient, ClientCredentialsAuth, VersionManagementClient, SessionId};
    /// use uuid::Uuid;
    /// use std::time::{SystemTime, UNIX_EPOCH};
    ///
    /// # async fn example() -> arcgis::Result<()> {
    /// # let auth = ClientCredentialsAuth::new("id".to_string(), "secret".to_string()).expect("Valid");
    /// # let client = ArcGISClient::new(auth);
    /// # let vm_client = VersionManagementClient::new("url", &client);
    /// let version_guid = Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000")
    ///     .expect("Valid UUID");
    /// let session_id = SessionId::new();
    ///
    /// // Start editing
    /// vm_client.start_editing(version_guid.into(), session_id).await?;
    ///
    /// // Perform some edits...
    /// let checkpoint = SystemTime::now()
    ///     .duration_since(UNIX_EPOCH)
    ///     .unwrap()
    ///     .as_millis() as u64;
    ///
    /// // More edits...
    ///
    /// // Undo edits after checkpoint
    /// vm_client.delete_forward_edits(
    ///     version_guid.into(),
    ///     session_id,
    ///     checkpoint,
    /// ).await?;
    ///
    /// // Stop editing
    /// vm_client.stop_editing(version_guid.into(), session_id, true).await?;
    /// # Ok(())
    /// # }
    /// ```
    #[instrument(skip(self), fields(
        base_url = %self.base_url,
        version_guid = %version_guid,
        session_id = %session_id,
        moment
    ))]
    pub async fn delete_forward_edits(
        &self,
        version_guid: VersionGuid,
        session_id: SessionId,
        moment: u64,
    ) -> Result<DeleteForwardEditsResponse> {
        tracing::debug!(
            version_guid = %version_guid,
            session_id = %session_id,
            moment = moment,
            "Deleting forward edits"
        );

        let url = format!(
            "{}/versions/{}/deleteForwardEdits",
            self.base_url, version_guid
        );

        tracing::debug!(url = %url, "Sending delete forward edits request");

        let session_id_str = session_id.to_string();
        let moment_str = moment.to_string();
        let mut form = vec![
            ("sessionId", session_id_str.as_str()),
            ("moment", moment_str.as_str()),
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
            tracing::error!(status = %status, error = %error_text, "delete forward edits failed");
            return Err(crate::Error::from(crate::ErrorKind::Api {
                code: status.as_u16() as i32,
                message: format!("HTTP {}: {}", status, error_text),
            }));
        }

        let delete_response: DeleteForwardEditsResponse = response.json().await?;

        if *delete_response.success() {
            tracing::info!(
                version_guid = %version_guid,
                moment = moment,
                "Forward edits deleted successfully"
            );
        } else {
            tracing::warn!(
                version_guid = %version_guid,
                error = ?delete_response.error(),
                "delete forward edits reported failure"
            );
        }

        Ok(delete_response)
    }

    /// Retrieves differences between version states.
    ///
    /// Returns all edits categorized as inserts, updates, or deletes. Can return
    /// either object IDs (more efficient) or full features with attributes and geometry.
    ///
    /// # Arguments
    ///
    /// * `version_guid` - The GUID of the version to query
    /// * `session_id` - Optional session ID for the query
    /// * `result_type` - Whether to return object IDs or full features
    /// * `layers` - Optional list of layer IDs to include (defaults to all layers)
    ///
    /// # Returns
    ///
    /// Returns a [`DifferencesResponse`] with edits organized by layer and type.
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
    /// use arcgis::{
    ///     ArcGISClient, ClientCredentialsAuth, VersionManagementClient,
    ///     DifferenceResultType,
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
    /// // Get differences as object IDs (more efficient)
    /// let diffs = vm_client.differences(
    ///     version_guid.into(),
    ///     None,
    ///     DifferenceResultType::ObjectIds,
    ///     None, // all layers
    /// ).await?;
    ///
    /// if let Some(differences) = diffs.differences() {
    ///     for layer_diff in differences {
    ///         println!("Layer {}: {} inserts, {} updates, {} deletes",
    ///             layer_diff.layer_id(),
    ///             layer_diff.inserts().as_ref().map(|i| i.len()).unwrap_or(0),
    ///             layer_diff.updates().as_ref().map(|u| u.len()).unwrap_or(0),
    ///             layer_diff.deletes().as_ref().map(|d| d.len()).unwrap_or(0),
    ///         );
    ///     }
    /// }
    ///
    /// // Get differences as full features
    /// let feature_diffs = vm_client.differences(
    ///     version_guid.into(),
    ///     None,
    ///     DifferenceResultType::Features,
    ///     Some(vec![0, 1]), // only layers 0 and 1
    /// ).await?;
    /// # Ok(())
    /// # }
    /// ```
    #[instrument(skip(self, layers), fields(
        base_url = %self.base_url,
        version_guid = %version_guid,
        result_type = %result_type
    ))]
    pub async fn differences(
        &self,
        version_guid: VersionGuid,
        session_id: Option<SessionId>,
        result_type: DifferenceResultType,
        layers: Option<Vec<i64>>,
    ) -> Result<DifferencesResponse> {
        tracing::debug!(
            version_guid = %version_guid,
            result_type = %result_type,
            "Retrieving differences"
        );

        let url = format!("{}/versions/{}/differences", self.base_url, version_guid);

        let result_type_str = result_type.to_string();

        let mut form = vec![("resultType", result_type_str), ("f", "json".to_string())];

        // Add session ID if provided
        let session_id_str;
        if let Some(sid) = session_id {
            session_id_str = sid.to_string();
            form.push(("sessionId", session_id_str));
        }

        // Serialize layers if provided
        let layers_json;
        if let Some(layer_list) = layers {
            layers_json = serde_json::to_string(&layer_list)?;
            form.push(("layers", layers_json));
        }

        tracing::debug!(url = %url, "Sending differences request");

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
            tracing::error!(status = %status, error = %error_text, "differences failed");
            return Err(crate::Error::from(crate::ErrorKind::Api {
                code: status.as_u16() as i32,
                message: format!("HTTP {}: {}", status, error_text),
            }));
        }

        let diffs_response: DifferencesResponse = response.json().await?;

        if *diffs_response.success() {
            tracing::info!(
                version_guid = %version_guid,
                result_type = %result_type,
                "Differences retrieved successfully"
            );
        } else {
            tracing::warn!(
                version_guid = %version_guid,
                error = ?diffs_response.error(),
                "differences reported failure"
            );
        }

        Ok(diffs_response)
    }
}
