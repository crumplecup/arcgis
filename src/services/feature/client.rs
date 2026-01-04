//! Feature Service client for querying and editing features.

use crate::{
    AddAttachmentResult, ArcGISClient, AttachmentId, AttachmentInfo, AttachmentInfosResponse,
    AttachmentSource, DeleteAttachmentsResponse, DownloadResult, DownloadTarget, EditOptions,
    EditResult, Feature, FeatureQueryParams, FeatureSet, LayerId, ObjectId, QueryBuilder, Result,
    UpdateAttachmentResult,
};
use futures::StreamExt;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tracing::instrument;

/// Client for interacting with an ArcGIS Feature Service.
///
/// # Example
/// ```no_run
/// use arcgis::{ApiKeyAuth, ArcGISClient, FeatureServiceClient};
///
/// # async fn example() -> arcgis::Result<()> {
/// let auth = ApiKeyAuth::new("YOUR_API_KEY");
/// let client = ArcGISClient::new(auth);
///
/// let feature_service = FeatureServiceClient::new(
///     "https://services.arcgis.com/org/arcgis/rest/services/Dataset/FeatureServer",
///     &client,
/// );
/// # Ok(())
/// # }
/// ```
pub struct FeatureServiceClient<'a> {
    /// Base URL of the feature service.
    base_url: String,
    /// Reference to the ArcGIS client for HTTP operations.
    client: &'a ArcGISClient,
}

impl<'a> FeatureServiceClient<'a> {
    /// Creates a new Feature Service client.
    ///
    /// # Arguments
    ///
    /// * `base_url` - The base URL of the feature service (e.g., `https://services.arcgis.com/.../FeatureServer`)
    /// * `client` - Reference to an authenticated ArcGIS client
    ///
    /// # Example
    /// ```no_run
    /// use arcgis::{ApiKeyAuth, ArcGISClient, FeatureServiceClient};
    ///
    /// let auth = ApiKeyAuth::new("YOUR_API_KEY");
    /// let client = ArcGISClient::new(auth);
    ///
    /// let feature_service = FeatureServiceClient::new(
    ///     "https://services.arcgis.com/org/arcgis/rest/services/Dataset/FeatureServer",
    ///     &client,
    /// );
    /// ```
    #[instrument(skip(base_url, client))]
    pub fn new(base_url: impl Into<String>, client: &'a ArcGISClient) -> Self {
        let base_url = base_url.into();
        tracing::debug!(base_url = %base_url, "Creating FeatureServiceClient");
        Self { base_url, client }
    }

    /// Creates a fluent query builder for the specified layer.
    ///
    /// This is the recommended way to query features. It provides a more
    /// ergonomic API than manually constructing [`FeatureQueryParams`].
    ///
    /// # Example
    /// ```no_run
    /// use arcgis::{ApiKeyAuth, ArcGISClient, FeatureServiceClient, LayerId};
    ///
    /// # async fn example() -> arcgis::Result<()> {
    /// let auth = ApiKeyAuth::new("YOUR_API_KEY");
    /// let client = ArcGISClient::new(auth);
    /// let service = FeatureServiceClient::new(
    ///     "https://services.arcgis.com/org/arcgis/rest/services/Dataset/FeatureServer",
    ///     &client,
    /// );
    ///
    /// // Use the fluent query builder
    /// let features = service
    ///     .query(LayerId::new(0))
    ///     .where_clause("POPULATION > 100000")
    ///     .out_fields(&["NAME", "POPULATION"])
    ///     .execute()
    ///     .await?;
    ///
    /// println!("Retrieved {} features", features.features.len());
    /// # Ok(())
    /// # }
    /// ```
    #[instrument(skip(self), fields(layer_id = %layer_id, base_url = %self.base_url))]
    pub fn query(&'a self, layer_id: LayerId) -> QueryBuilder<'a> {
        tracing::debug!(layer_id = %layer_id, "Creating query builder");
        QueryBuilder::new(self, layer_id)
    }

    /// Queries features from a specific layer with pre-built parameters.
    ///
    /// This is a lower-level method. For most use cases, prefer the
    /// [`query`](Self::query) builder method.
    ///
    /// # Example
    /// ```no_run
    /// use arcgis::{ApiKeyAuth, ArcGISClient, FeatureQueryParams, FeatureServiceClient, LayerId};
    ///
    /// # async fn example() -> arcgis::Result<()> {
    /// let auth = ApiKeyAuth::new("YOUR_API_KEY");
    /// let client = ArcGISClient::new(auth);
    /// let feature_service = FeatureServiceClient::new(
    ///     "https://services.arcgis.com/org/arcgis/rest/services/Dataset/FeatureServer",
    ///     &client,
    /// );
    ///
    /// let params = FeatureQueryParams::builder()
    ///     .where_clause("POPULATION > 100000")
    ///     .out_fields(vec!["NAME".to_string(), "POPULATION".to_string()])
    ///     .build()
    ///     .unwrap();
    ///
    /// let features = feature_service.query_with_params(LayerId::new(0), params).await?;
    /// println!("Retrieved {} features", features.features.len());
    /// # Ok(())
    /// # }
    /// ```
    #[instrument(skip(self, params), fields(layer_id = %layer_id, base_url = %self.base_url))]
    pub async fn query_with_params(
        &self,
        layer_id: LayerId,
        params: FeatureQueryParams,
    ) -> Result<FeatureSet> {
        tracing::debug!("Querying feature layer");

        // Construct the query URL
        let url = format!("{}/{}/query", self.base_url, layer_id);

        // Get authentication token
        let token = self.client.auth().get_token().await?;

        tracing::debug!(url = %url, "Sending query request");

        // Build request with query parameters and token
        let response = self
            .client
            .http()
            .get(&url)
            .query(&params)
            .query(&[("token", token)])
            .send()
            .await?;

        // Check for HTTP errors
        let status = response.status();
        if !status.is_success() {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|e| format!("Failed to read error response: {}", e));
            tracing::error!(status = %status, error = %error_text, "Query request failed");
            return Err(crate::Error::from(crate::ErrorKind::Api {
                code: status.as_u16() as i32,
                message: format!("HTTP {}: {}", status, error_text),
            }));
        }

        // Parse the response
        let feature_set: FeatureSet = response.json().await?;

        tracing::debug!(
            feature_count = feature_set.features.len(),
            exceeded_limit = feature_set.exceeded_transfer_limit,
            "Query completed successfully"
        );

        Ok(feature_set)
    }

    /// Adds new features to a layer.
    ///
    /// # Arguments
    ///
    /// * `layer_id` - The layer to add features to
    /// * `features` - Vector of features to add
    /// * `options` - Edit options (transaction control, etc.)
    ///
    /// # Example
    ///
    /// ```no_run
    /// use arcgis::{ArcGISClient, ApiKeyAuth, FeatureServiceClient, LayerId, Feature, EditOptions};
    /// use serde_json::json;
    /// use std::collections::HashMap;
    ///
    /// # async fn example() -> arcgis::Result<()> {
    /// let auth = ApiKeyAuth::new("YOUR_API_KEY");
    /// let client = ArcGISClient::new(auth);
    /// let service = FeatureServiceClient::new("https://example.com/FeatureServer", &client);
    ///
    /// let mut attributes = HashMap::new();
    /// attributes.insert("NAME".to_string(), json!("New City"));
    /// attributes.insert("POPULATION".to_string(), json!(50000));
    ///
    /// let new_feature = Feature {
    ///     attributes,
    ///     geometry: None,
    /// };
    ///
    /// let result = service
    ///     .add_features(LayerId::new(0), vec![new_feature], EditOptions::default())
    ///     .await?;
    ///
    /// if result.all_succeeded() {
    ///     println!("Added {} features", result.success_count());
    /// }
    /// # Ok(())
    /// # }
    /// ```
    #[instrument(skip(self, features, options), fields(layer_id = %layer_id, count = features.len()))]
    pub async fn add_features(
        &self,
        layer_id: LayerId,
        features: Vec<Feature>,
        options: EditOptions,
    ) -> Result<EditResult> {
        tracing::debug!("Adding features to layer");

        let url = format!("{}/{}/addFeatures", self.base_url, layer_id);
        let token = self.client.auth().get_token().await?;

        tracing::debug!(url = %url, feature_count = features.len(), "Sending addFeatures request");

        // Build form data
        let features_json = serde_json::to_string(&features)?;
        let mut form = vec![
            ("features", features_json.as_str()),
            ("f", "json"),
            ("token", token.as_str()),
        ];

        // Add optional parameters
        let session_id_str = options.session_id.as_ref().map(|s| s.to_string());
        if let Some(ref session_id) = session_id_str {
            form.push(("sessionId", session_id.as_str()));
        }
        if let Some(ref gdb_version) = options.gdb_version {
            form.push(("gdbVersion", gdb_version.as_str()));
        }
        if let Some(rollback) = options.rollback_on_failure {
            form.push(("rollbackOnFailure", if rollback { "true" } else { "false" }));
        }
        if let Some(use_global) = options.use_global_ids {
            form.push(("useGlobalIds", if use_global { "true" } else { "false" }));
        }
        if let Some(return_results) = options.return_edit_results {
            form.push((
                "returnEditResults",
                if return_results { "true" } else { "false" },
            ));
        }

        let response = self.client.http().post(&url).form(&form).send().await?;

        let status = response.status();
        if !status.is_success() {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|e| format!("Failed to read error: {}", e));
            tracing::error!(status = %status, error = %error_text, "addFeatures request failed");
            return Err(crate::Error::from(crate::ErrorKind::Api {
                code: status.as_u16() as i32,
                message: format!("HTTP {}: {}", status, error_text),
            }));
        }

        let result: EditResult = response.json().await?;

        tracing::info!(
            success_count = result.success_count(),
            failure_count = result.failure_count(),
            "addFeatures completed"
        );

        Ok(result)
    }

    /// Updates existing features in a layer.
    ///
    /// Features must include their ObjectID in the attributes to identify which feature to update.
    ///
    /// # Arguments
    ///
    /// * `layer_id` - The layer containing the features to update
    /// * `features` - Vector of features with updated attributes/geometry
    /// * `options` - Edit options (transaction control, etc.)
    ///
    /// # Example
    ///
    /// ```no_run
    /// use arcgis::{ArcGISClient, ApiKeyAuth, FeatureServiceClient, LayerId, Feature, EditOptions};
    /// use serde_json::json;
    /// use std::collections::HashMap;
    ///
    /// # async fn example() -> arcgis::Result<()> {
    /// let auth = ApiKeyAuth::new("YOUR_API_KEY");
    /// let client = ArcGISClient::new(auth);
    /// let service = FeatureServiceClient::new("https://example.com/FeatureServer", &client);
    ///
    /// let mut attributes = HashMap::new();
    /// attributes.insert("OBJECTID".to_string(), json!(123));
    /// attributes.insert("POPULATION".to_string(), json!(55000));
    ///
    /// let updated_feature = Feature {
    ///     attributes,
    ///     geometry: None,
    /// };
    ///
    /// let result = service
    ///     .update_features(LayerId::new(0), vec![updated_feature], EditOptions::default())
    ///     .await?;
    ///
    /// println!("Updated {} features", result.success_count());
    /// # Ok(())
    /// # }
    /// ```
    #[instrument(skip(self, features, options), fields(layer_id = %layer_id, count = features.len()))]
    pub async fn update_features(
        &self,
        layer_id: LayerId,
        features: Vec<Feature>,
        options: EditOptions,
    ) -> Result<EditResult> {
        tracing::debug!("Updating features in layer");

        let url = format!("{}/{}/updateFeatures", self.base_url, layer_id);
        let token = self.client.auth().get_token().await?;

        tracing::debug!(url = %url, feature_count = features.len(), "Sending updateFeatures request");

        // Build form data
        let features_json = serde_json::to_string(&features)?;
        let mut form = vec![
            ("features", features_json.as_str()),
            ("f", "json"),
            ("token", token.as_str()),
        ];

        // Add optional parameters
        let session_id_str = options.session_id.as_ref().map(|s| s.to_string());
        if let Some(ref session_id) = session_id_str {
            form.push(("sessionId", session_id.as_str()));
        }
        if let Some(ref gdb_version) = options.gdb_version {
            form.push(("gdbVersion", gdb_version.as_str()));
        }
        if let Some(rollback) = options.rollback_on_failure {
            form.push(("rollbackOnFailure", if rollback { "true" } else { "false" }));
        }
        if let Some(use_global) = options.use_global_ids {
            form.push(("useGlobalIds", if use_global { "true" } else { "false" }));
        }
        if let Some(return_results) = options.return_edit_results {
            form.push((
                "returnEditResults",
                if return_results { "true" } else { "false" },
            ));
        }

        let response = self.client.http().post(&url).form(&form).send().await?;

        let status = response.status();
        if !status.is_success() {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|e| format!("Failed to read error: {}", e));
            tracing::error!(status = %status, error = %error_text, "updateFeatures request failed");
            return Err(crate::Error::from(crate::ErrorKind::Api {
                code: status.as_u16() as i32,
                message: format!("HTTP {}: {}", status, error_text),
            }));
        }

        let result: EditResult = response.json().await?;

        tracing::info!(
            success_count = result.success_count(),
            failure_count = result.failure_count(),
            "updateFeatures completed"
        );

        Ok(result)
    }

    /// Deletes features from a layer by ObjectID.
    ///
    /// # Arguments
    ///
    /// * `layer_id` - The layer to delete features from
    /// * `object_ids` - Vector of ObjectIDs to delete
    /// * `options` - Edit options (transaction control, etc.)
    ///
    /// # Example
    ///
    /// ```no_run
    /// use arcgis::{ArcGISClient, ApiKeyAuth, FeatureServiceClient, LayerId, ObjectId, EditOptions};
    ///
    /// # async fn example() -> arcgis::Result<()> {
    /// let auth = ApiKeyAuth::new("YOUR_API_KEY");
    /// let client = ArcGISClient::new(auth);
    /// let service = FeatureServiceClient::new("https://example.com/FeatureServer", &client);
    ///
    /// let ids_to_delete = vec![ObjectId::new(1), ObjectId::new(2), ObjectId::new(3)];
    ///
    /// let result = service
    ///     .delete_features(LayerId::new(0), ids_to_delete, EditOptions::default())
    ///     .await?;
    ///
    /// println!("Deleted {} features", result.success_count());
    /// # Ok(())
    /// # }
    /// ```
    #[instrument(skip(self, object_ids, options), fields(layer_id = %layer_id, count = object_ids.len()))]
    pub async fn delete_features(
        &self,
        layer_id: LayerId,
        object_ids: Vec<ObjectId>,
        options: EditOptions,
    ) -> Result<EditResult> {
        tracing::debug!("Deleting features from layer");

        let url = format!("{}/{}/deleteFeatures", self.base_url, layer_id);
        let token = self.client.auth().get_token().await?;

        // Convert ObjectIds to comma-separated string
        let object_ids_str = object_ids
            .iter()
            .map(|id| id.to_string())
            .collect::<Vec<_>>()
            .join(",");

        tracing::debug!(url = %url, object_ids = %object_ids_str, "Sending deleteFeatures request");

        // Build form data
        let mut form = vec![
            ("objectIds", object_ids_str.as_str()),
            ("f", "json"),
            ("token", token.as_str()),
        ];

        // Add optional parameters
        if let Some(ref gdb_version) = options.gdb_version {
            form.push(("gdbVersion", gdb_version.as_str()));
        }
        if let Some(rollback) = options.rollback_on_failure {
            form.push(("rollbackOnFailure", if rollback { "true" } else { "false" }));
        }
        if let Some(return_results) = options.return_edit_results {
            form.push((
                "returnEditResults",
                if return_results { "true" } else { "false" },
            ));
        }

        let response = self.client.http().post(&url).form(&form).send().await?;

        let status = response.status();
        if !status.is_success() {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|e| format!("Failed to read error: {}", e));
            tracing::error!(status = %status, error = %error_text, "deleteFeatures request failed");
            return Err(crate::Error::from(crate::ErrorKind::Api {
                code: status.as_u16() as i32,
                message: format!("HTTP {}: {}", status, error_text),
            }));
        }

        let result: EditResult = response.json().await?;

        tracing::info!(
            success_count = result.success_count(),
            failure_count = result.failure_count(),
            "deleteFeatures completed"
        );

        Ok(result)
    }

    /// Applies batch edits (add, update, delete) in a single transaction.
    ///
    /// This is the most efficient way to perform multiple edit operations,
    /// as it allows adding, updating, and deleting features in a single request.
    ///
    /// # Arguments
    ///
    /// * `layer_id` - The layer to edit
    /// * `adds` - Features to add (optional)
    /// * `updates` - Features to update (optional)
    /// * `deletes` - ObjectIDs to delete (optional)
    /// * `options` - Edit options (transaction control, etc.)
    ///
    /// # Example
    ///
    /// ```no_run
    /// use arcgis::{ArcGISClient, ApiKeyAuth, FeatureServiceClient, LayerId, Feature, ObjectId, EditOptions};
    /// use serde_json::json;
    /// use std::collections::HashMap;
    ///
    /// # async fn example() -> arcgis::Result<()> {
    /// let auth = ApiKeyAuth::new("YOUR_API_KEY");
    /// let client = ArcGISClient::new(auth);
    /// let service = FeatureServiceClient::new("https://example.com/FeatureServer", &client);
    ///
    /// // Prepare features to add
    /// let mut new_attrs = HashMap::new();
    /// new_attrs.insert("NAME".to_string(), json!("New City"));
    /// new_attrs.insert("POPULATION".to_string(), json!(50000));
    /// let new_feature = Feature {
    ///     attributes: new_attrs,
    ///     geometry: None,
    /// };
    ///
    /// // Prepare features to update
    /// let mut update_attrs = HashMap::new();
    /// update_attrs.insert("OBJECTID".to_string(), json!(123));
    /// update_attrs.insert("POPULATION".to_string(), json!(55000));
    /// let updated_feature = Feature {
    ///     attributes: update_attrs,
    ///     geometry: None,
    /// };
    ///
    /// // Prepare IDs to delete
    /// let ids_to_delete = vec![ObjectId::new(456)];
    ///
    /// // Apply all edits in one transaction
    /// let result = service
    ///     .apply_edits(
    ///         LayerId::new(0),
    ///         Some(vec![new_feature]),
    ///         Some(vec![updated_feature]),
    ///         Some(ids_to_delete),
    ///         EditOptions::default(),
    ///     )
    ///     .await?;
    ///
    /// println!(
    ///     "Applied {} edits ({} added, {} updated, {} deleted)",
    ///     result.success_count(),
    ///     result.add_results().len(),
    ///     result.update_results().len(),
    ///     result.delete_results().len()
    /// );
    /// # Ok(())
    /// # }
    /// ```
    #[instrument(
        skip(self, adds, updates, deletes, options),
        fields(
            layer_id = %layer_id,
            add_count = adds.as_ref().map(|v| v.len()).unwrap_or(0),
            update_count = updates.as_ref().map(|v| v.len()).unwrap_or(0),
            delete_count = deletes.as_ref().map(|v| v.len()).unwrap_or(0)
        )
    )]
    pub async fn apply_edits(
        &self,
        layer_id: LayerId,
        adds: Option<Vec<Feature>>,
        updates: Option<Vec<Feature>>,
        deletes: Option<Vec<ObjectId>>,
        options: EditOptions,
    ) -> Result<EditResult> {
        tracing::debug!("Applying batch edits to layer");

        let url = format!("{}/{}/applyEdits", self.base_url, layer_id);
        let token = self.client.auth().get_token().await?;

        tracing::debug!(url = %url, "Sending applyEdits request");

        // Pre-allocate owned strings that need to live for the duration of the request
        let adds_json = adds.as_ref().map(serde_json::to_string).transpose()?;
        let updates_json = updates.as_ref().map(serde_json::to_string).transpose()?;
        let deletes_str = deletes.as_ref().map(|d| {
            d.iter()
                .map(|id| id.to_string())
                .collect::<Vec<_>>()
                .join(",")
        });

        // Build form data with references to owned strings
        let mut form: Vec<(&str, &str)> = vec![("f", "json"), ("token", token.as_str())];

        if let Some(ref adds) = adds_json {
            form.push(("adds", adds.as_str()));
        }
        if let Some(ref updates) = updates_json {
            form.push(("updates", updates.as_str()));
        }
        if let Some(ref deletes) = deletes_str {
            form.push(("deletes", deletes.as_str()));
        }

        // Add optional parameters
        let session_id_str = options.session_id.as_ref().map(|s| s.to_string());
        if let Some(ref session_id) = session_id_str {
            form.push(("sessionId", session_id.as_str()));
        }
        if let Some(ref gdb_version) = options.gdb_version {
            form.push(("gdbVersion", gdb_version.as_str()));
        }
        if let Some(rollback) = options.rollback_on_failure {
            form.push(("rollbackOnFailure", if rollback { "true" } else { "false" }));
        }
        if let Some(use_global) = options.use_global_ids {
            form.push(("useGlobalIds", if use_global { "true" } else { "false" }));
        }
        if let Some(return_results) = options.return_edit_results {
            form.push((
                "returnEditResults",
                if return_results { "true" } else { "false" },
            ));
        }

        let response = self.client.http().post(&url).form(&form).send().await?;

        let status = response.status();
        if !status.is_success() {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|e| format!("Failed to read error: {}", e));
            tracing::error!(status = %status, error = %error_text, "applyEdits request failed");
            return Err(crate::Error::from(crate::ErrorKind::Api {
                code: status.as_u16() as i32,
                message: format!("HTTP {}: {}", status, error_text),
            }));
        }

        let result: EditResult = response.json().await?;

        tracing::info!(
            success_count = result.success_count(),
            failure_count = result.failure_count(),
            add_results = result.add_results().len(),
            update_results = result.update_results().len(),
            delete_results = result.delete_results().len(),
            "applyEdits completed"
        );

        Ok(result)
    }

    /// Queries attachments for a specific feature.
    ///
    /// Returns metadata about all attachments associated with the feature.
    /// To download attachment data, use [`download_attachment`](Self::download_attachment).
    ///
    /// # Arguments
    ///
    /// * `layer_id` - The layer containing the feature
    /// * `object_id` - The feature to query attachments for
    ///
    /// # Example
    ///
    /// ```no_run
    /// use arcgis::{ArcGISClient, ApiKeyAuth, FeatureServiceClient, LayerId, ObjectId};
    ///
    /// # async fn example() -> arcgis::Result<()> {
    /// let auth = ApiKeyAuth::new("YOUR_API_KEY");
    /// let client = ArcGISClient::new(auth);
    /// let service = FeatureServiceClient::new("https://example.com/FeatureServer", &client);
    ///
    /// let attachments = service
    ///     .query_attachments(LayerId::new(0), ObjectId::new(123))
    ///     .await?;
    ///
    /// for attachment in &attachments {
    ///     println!("Attachment: {} ({} bytes)", attachment.name(), attachment.size());
    /// }
    /// # Ok(())
    /// # }
    /// ```
    #[instrument(skip(self), fields(layer_id = %layer_id, object_id = %object_id))]
    pub async fn query_attachments(
        &self,
        layer_id: LayerId,
        object_id: ObjectId,
    ) -> Result<Vec<AttachmentInfo>> {
        tracing::debug!("Querying attachments for feature");

        let url = format!("{}/{}/{}/attachments", self.base_url, layer_id, object_id);
        let token = self.client.auth().get_token().await?;

        tracing::debug!(url = %url, "Sending queryAttachments request");

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
            tracing::error!(status = %status, error = %error_text, "queryAttachments failed");
            return Err(crate::Error::from(crate::ErrorKind::Api {
                code: status.as_u16() as i32,
                message: format!("HTTP {}: {}", status, error_text),
            }));
        }

        let result: AttachmentInfosResponse = response.json().await?;

        tracing::info!(
            attachment_count = result.attachment_infos.len(),
            "queryAttachments completed"
        );

        Ok(result.attachment_infos)
    }

    /// Deletes one or more attachments from a feature.
    ///
    /// # Arguments
    ///
    /// * `layer_id` - The layer containing the feature
    /// * `object_id` - The feature that owns the attachments
    /// * `attachment_ids` - Vector of attachment IDs to delete
    ///
    /// # Example
    ///
    /// ```no_run
    /// use arcgis::{ArcGISClient, ApiKeyAuth, FeatureServiceClient, LayerId, ObjectId, AttachmentId};
    ///
    /// # async fn example() -> arcgis::Result<()> {
    /// let auth = ApiKeyAuth::new("YOUR_API_KEY");
    /// let client = ArcGISClient::new(auth);
    /// let service = FeatureServiceClient::new("https://example.com/FeatureServer", &client);
    ///
    /// let ids_to_delete = vec![AttachmentId::new(1), AttachmentId::new(2)];
    ///
    /// let result = service
    ///     .delete_attachments(LayerId::new(0), ObjectId::new(123), ids_to_delete)
    ///     .await?;
    ///
    /// for item in &result.delete_attachment_results {
    ///     if *item.success() {
    ///         println!("Deleted attachment from feature {}", item.object_id());
    ///     }
    /// }
    /// # Ok(())
    /// # }
    /// ```
    #[instrument(skip(self, attachment_ids), fields(layer_id = %layer_id, object_id = %object_id, count = attachment_ids.len()))]
    pub async fn delete_attachments(
        &self,
        layer_id: LayerId,
        object_id: ObjectId,
        attachment_ids: Vec<AttachmentId>,
    ) -> Result<DeleteAttachmentsResponse> {
        tracing::debug!("Deleting attachments from feature");

        let url = format!(
            "{}/{}/{}/deleteAttachments",
            self.base_url, layer_id, object_id
        );
        let token = self.client.auth().get_token().await?;

        // Convert AttachmentIds to comma-separated string
        let attachment_ids_str = attachment_ids
            .iter()
            .map(|id| id.to_string())
            .collect::<Vec<_>>()
            .join(",");

        tracing::debug!(
            url = %url,
            attachment_ids = %attachment_ids_str,
            "Sending deleteAttachments request"
        );

        let form = vec![
            ("attachmentIds", attachment_ids_str.as_str()),
            ("f", "json"),
            ("token", token.as_str()),
        ];

        let response = self.client.http().post(&url).form(&form).send().await?;

        let status = response.status();
        if !status.is_success() {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|e| format!("Failed to read error: {}", e));
            tracing::error!(status = %status, error = %error_text, "deleteAttachments failed");
            return Err(crate::Error::from(crate::ErrorKind::Api {
                code: status.as_u16() as i32,
                message: format!("HTTP {}: {}", status, error_text),
            }));
        }

        let result: DeleteAttachmentsResponse = response.json().await?;

        let success_count = result
            .delete_attachment_results
            .iter()
            .filter(|r| *r.success())
            .count();

        tracing::info!(
            success_count = success_count,
            total_count = result.delete_attachment_results.len(),
            "deleteAttachments completed"
        );

        Ok(result)
    }

    /// Adds an attachment to a feature.
    ///
    /// Uploads a file and associates it with the specified feature.
    /// Supports streaming for efficient handling of large files.
    ///
    /// # Arguments
    ///
    /// * `layer_id` - The layer containing the feature
    /// * `object_id` - The feature to attach the file to
    /// * `source` - The attachment source (file path, bytes, or stream)
    ///
    /// # Example
    ///
    /// ```no_run
    /// use arcgis::{ArcGISClient, ApiKeyAuth, FeatureServiceClient, LayerId, ObjectId, AttachmentSource};
    ///
    /// # async fn example() -> arcgis::Result<()> {
    /// let auth = ApiKeyAuth::new("YOUR_API_KEY");
    /// let client = ArcGISClient::new(auth);
    /// let service = FeatureServiceClient::new("https://example.com/FeatureServer", &client);
    ///
    /// // From file path (streaming)
    /// let result = service
    ///     .add_attachment(
    ///         LayerId::new(0),
    ///         ObjectId::new(123),
    ///         AttachmentSource::from_path("/path/to/photo.jpg"),
    ///     )
    ///     .await?;
    ///
    /// if *result.success() {
    ///     println!("Attachment added successfully");
    /// }
    ///
    /// // From bytes
    /// let image_data = vec![0xFF, 0xD8, 0xFF]; // JPEG header...
    /// let result = service
    ///     .add_attachment(
    ///         LayerId::new(0),
    ///         ObjectId::new(456),
    ///         AttachmentSource::from_bytes("photo.jpg", image_data),
    ///     )
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    #[instrument(skip(self, source), fields(layer_id = %layer_id, object_id = %object_id))]
    pub async fn add_attachment(
        &self,
        layer_id: LayerId,
        object_id: ObjectId,
        source: AttachmentSource,
    ) -> Result<AddAttachmentResult> {
        tracing::debug!("Adding attachment to feature");

        let url = format!("{}/{}/{}/addAttachment", self.base_url, layer_id, object_id);
        let token = self.client.auth().get_token().await?;

        // Build multipart form
        let mut form = reqwest::multipart::Form::new();

        // Add the file part based on source
        match source {
            AttachmentSource::Path(path) => {
                tracing::debug!(path = %path.display(), "Loading attachment from file");

                // Get filename
                let filename = path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("attachment")
                    .to_string();

                // Open file for streaming
                let file = tokio::fs::File::open(&path).await?;
                let metadata = file.metadata().await?;
                let file_size = metadata.len();

                // Detect content type
                let content_type = mime_guess::from_path(&path)
                    .first_or_octet_stream()
                    .to_string();

                tracing::debug!(
                    filename = %filename,
                    content_type = %content_type,
                    size = file_size,
                    "File opened for streaming"
                );

                // Create streaming body
                let part = reqwest::multipart::Part::stream(reqwest::Body::from(file))
                    .file_name(filename)
                    .mime_str(&content_type)?;

                form = form.part("attachment", part);
            }
            AttachmentSource::Bytes {
                filename,
                data,
                content_type,
            } => {
                tracing::debug!(
                    filename = %filename,
                    size = data.len(),
                    "Using in-memory attachment data"
                );

                let content_type = content_type.unwrap_or_else(|| {
                    mime_guess::from_path(&filename)
                        .first_or_octet_stream()
                        .to_string()
                });

                tracing::debug!(content_type = %content_type, "Content type determined");

                let part = reqwest::multipart::Part::bytes(data)
                    .file_name(filename)
                    .mime_str(&content_type)?;

                form = form.part("attachment", part);
            }
            AttachmentSource::Stream {
                filename,
                mut reader,
                content_type,
                size,
            } => {
                tracing::debug!(
                    filename = %filename,
                    size = ?size,
                    "Streaming from async reader"
                );

                // Read from the async reader into a buffer
                let mut buffer = Vec::new();
                reader.read_to_end(&mut buffer).await?;

                tracing::debug!(bytes_read = buffer.len(), "Data read from stream");

                let part = reqwest::multipart::Part::bytes(buffer)
                    .file_name(filename)
                    .mime_str(&content_type)?;

                form = form.part("attachment", part);
            }
        }

        // Add standard parameters
        form = form.text("f", "json");
        form = form.text("token", token);

        tracing::debug!(url = %url, "Sending addAttachment request");

        let response = self.client.http().post(&url).multipart(form).send().await?;

        let status = response.status();
        if !status.is_success() {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|e| format!("Failed to read error: {}", e));
            tracing::error!(status = %status, error = %error_text, "addAttachment failed");
            return Err(crate::Error::from(crate::ErrorKind::Api {
                code: status.as_u16() as i32,
                message: format!("HTTP {}: {}", status, error_text),
            }));
        }

        let result: AddAttachmentResult = response.json().await?;

        tracing::info!(
            success = result.success(),
            object_id = %result.object_id(),
            "addAttachment completed"
        );

        Ok(result)
    }

    /// Updates an existing attachment.
    ///
    /// Replaces the attachment file while keeping the same attachment ID.
    /// Supports streaming for efficient handling of large files.
    ///
    /// # Arguments
    ///
    /// * `layer_id` - The layer containing the feature
    /// * `object_id` - The feature that owns the attachment
    /// * `attachment_id` - The attachment to update
    /// * `source` - The new attachment source (file path, bytes, or stream)
    ///
    /// # Example
    ///
    /// ```no_run
    /// use arcgis::{ArcGISClient, ApiKeyAuth, FeatureServiceClient, LayerId, ObjectId, AttachmentId, AttachmentSource};
    ///
    /// # async fn example() -> arcgis::Result<()> {
    /// let auth = ApiKeyAuth::new("YOUR_API_KEY");
    /// let client = ArcGISClient::new(auth);
    /// let service = FeatureServiceClient::new("https://example.com/FeatureServer", &client);
    ///
    /// let result = service
    ///     .update_attachment(
    ///         LayerId::new(0),
    ///         ObjectId::new(123),
    ///         AttachmentId::new(5),
    ///         AttachmentSource::from_path("/path/to/updated_photo.jpg"),
    ///     )
    ///     .await?;
    ///
    /// if *result.success() {
    ///     println!("Attachment updated successfully");
    /// }
    /// # Ok(())
    /// # }
    /// ```
    #[instrument(skip(self, source), fields(layer_id = %layer_id, object_id = %object_id, attachment_id = %attachment_id))]
    pub async fn update_attachment(
        &self,
        layer_id: LayerId,
        object_id: ObjectId,
        attachment_id: AttachmentId,
        source: AttachmentSource,
    ) -> Result<UpdateAttachmentResult> {
        tracing::debug!("Updating attachment");

        let url = format!(
            "{}/{}/{}/updateAttachment",
            self.base_url, layer_id, object_id
        );
        let token = self.client.auth().get_token().await?;

        // Build multipart form
        let mut form = reqwest::multipart::Form::new();

        // Add attachment ID
        form = form.text("attachmentId", attachment_id.to_string());

        // Add the file part based on source
        match source {
            AttachmentSource::Path(path) => {
                tracing::debug!(path = %path.display(), "Loading attachment from file");

                let filename = path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("attachment")
                    .to_string();

                let file = tokio::fs::File::open(&path).await?;
                let metadata = file.metadata().await?;
                let file_size = metadata.len();

                let content_type = mime_guess::from_path(&path)
                    .first_or_octet_stream()
                    .to_string();

                tracing::debug!(
                    filename = %filename,
                    content_type = %content_type,
                    size = file_size,
                    "File opened for streaming"
                );

                let part = reqwest::multipart::Part::stream(reqwest::Body::from(file))
                    .file_name(filename)
                    .mime_str(&content_type)?;

                form = form.part("attachment", part);
            }
            AttachmentSource::Bytes {
                filename,
                data,
                content_type,
            } => {
                tracing::debug!(
                    filename = %filename,
                    size = data.len(),
                    "Using in-memory attachment data"
                );

                let content_type = content_type.unwrap_or_else(|| {
                    mime_guess::from_path(&filename)
                        .first_or_octet_stream()
                        .to_string()
                });

                tracing::debug!(content_type = %content_type, "Content type determined");

                let part = reqwest::multipart::Part::bytes(data)
                    .file_name(filename)
                    .mime_str(&content_type)?;

                form = form.part("attachment", part);
            }
            AttachmentSource::Stream {
                filename,
                mut reader,
                content_type,
                size,
            } => {
                tracing::debug!(
                    filename = %filename,
                    size = ?size,
                    "Streaming from async reader"
                );

                let mut buffer = Vec::new();
                reader.read_to_end(&mut buffer).await?;

                tracing::debug!(bytes_read = buffer.len(), "Data read from stream");

                let part = reqwest::multipart::Part::bytes(buffer)
                    .file_name(filename)
                    .mime_str(&content_type)?;

                form = form.part("attachment", part);
            }
        }

        // Add standard parameters
        form = form.text("f", "json");
        form = form.text("token", token);

        tracing::debug!(url = %url, "Sending updateAttachment request");

        let response = self.client.http().post(&url).multipart(form).send().await?;

        let status = response.status();
        if !status.is_success() {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|e| format!("Failed to read error: {}", e));
            tracing::error!(status = %status, error = %error_text, "updateAttachment failed");
            return Err(crate::Error::from(crate::ErrorKind::Api {
                code: status.as_u16() as i32,
                message: format!("HTTP {}: {}", status, error_text),
            }));
        }

        let result: UpdateAttachmentResult = response.json().await?;

        tracing::info!(
            success = result.success(),
            object_id = %result.object_id(),
            "updateAttachment completed"
        );

        Ok(result)
    }

    /// Downloads an attachment from a feature.
    ///
    /// Streams the attachment data to the specified target (file, bytes, or writer).
    /// Efficient for large files as it doesn't load the entire file into memory unless
    /// using the Bytes target.
    ///
    /// # Arguments
    ///
    /// * `layer_id` - The layer containing the feature
    /// * `object_id` - The feature that owns the attachment
    /// * `attachment_id` - The attachment to download
    /// * `target` - Where to write the downloaded data
    ///
    /// # Example
    ///
    /// ```no_run
    /// use arcgis::{ArcGISClient, ApiKeyAuth, FeatureServiceClient, LayerId, ObjectId, AttachmentId, DownloadTarget};
    /// use std::path::PathBuf;
    ///
    /// # async fn example() -> arcgis::Result<()> {
    /// let auth = ApiKeyAuth::new("YOUR_API_KEY");
    /// let client = ArcGISClient::new(auth);
    /// let service = FeatureServiceClient::new("https://example.com/FeatureServer", &client);
    ///
    /// // Download to file (streaming)
    /// let result = service
    ///     .download_attachment(
    ///         LayerId::new(0),
    ///         ObjectId::new(123),
    ///         AttachmentId::new(5),
    ///         DownloadTarget::to_path("/path/to/save/photo.jpg"),
    ///     )
    ///     .await?;
    ///
    /// if let Some(path) = result.path() {
    ///     println!("Downloaded to {:?}", path);
    /// }
    ///
    /// // Download to memory
    /// let result = service
    ///     .download_attachment(
    ///         LayerId::new(0),
    ///         ObjectId::new(123),
    ///         AttachmentId::new(6),
    ///         DownloadTarget::to_bytes(),
    ///     )
    ///     .await?;
    ///
    /// if let Some(bytes) = result.bytes() {
    ///     println!("Downloaded {} bytes", bytes.len());
    /// }
    /// # Ok(())
    /// # }
    /// ```
    #[instrument(skip(self, target), fields(layer_id = %layer_id, object_id = %object_id, attachment_id = %attachment_id))]
    pub async fn download_attachment(
        &self,
        layer_id: LayerId,
        object_id: ObjectId,
        attachment_id: AttachmentId,
        target: DownloadTarget,
    ) -> Result<DownloadResult> {
        tracing::debug!("Downloading attachment");

        let url = format!(
            "{}/{}/{}/attachments/{}",
            self.base_url, layer_id, object_id, attachment_id
        );
        let token = self.client.auth().get_token().await?;

        tracing::debug!(url = %url, "Sending download request");

        let response = self
            .client
            .http()
            .get(&url)
            .query(&[("token", token.as_str())])
            .send()
            .await?;

        let status = response.status();
        if !status.is_success() {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|e| format!("Failed to read error: {}", e));
            tracing::error!(status = %status, error = %error_text, "download attachment failed");
            return Err(crate::Error::from(crate::ErrorKind::Api {
                code: status.as_u16() as i32,
                message: format!("HTTP {}: {}", status, error_text),
            }));
        }

        // Stream the response based on target
        match target {
            DownloadTarget::Path(path) => {
                tracing::debug!(path = %path.display(), "Streaming to file");

                let mut file = tokio::fs::File::create(&path).await?;
                let mut stream = response.bytes_stream();
                let mut total_bytes = 0u64;

                while let Some(chunk_result) = stream.next().await {
                    let chunk = chunk_result?;
                    file.write_all(&chunk).await?;
                    total_bytes += chunk.len() as u64;
                }

                file.flush().await?;

                tracing::info!(
                    path = %path.display(),
                    bytes = total_bytes,
                    "Download to file completed"
                );

                Ok(DownloadResult::Path(path))
            }
            DownloadTarget::Bytes => {
                tracing::debug!("Collecting bytes to memory");

                let mut stream = response.bytes_stream();
                let mut buffer = Vec::new();

                while let Some(chunk_result) = stream.next().await {
                    let chunk = chunk_result?;
                    buffer.extend_from_slice(&chunk);
                }

                tracing::info!(bytes = buffer.len(), "Download to bytes completed");

                Ok(DownloadResult::Bytes(buffer))
            }
            DownloadTarget::Writer(mut writer) => {
                tracing::debug!("Streaming to writer");

                let mut stream = response.bytes_stream();
                let mut total_bytes = 0u64;

                while let Some(chunk_result) = stream.next().await {
                    let chunk = chunk_result?;
                    writer.write_all(&chunk).await?;
                    total_bytes += chunk.len() as u64;
                }

                writer.flush().await?;

                tracing::info!(bytes = total_bytes, "Download to writer completed");

                Ok(DownloadResult::Written(total_bytes))
            }
        }
    }
}
