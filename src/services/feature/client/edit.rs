//! Edit operations for the Feature Service client.

use super::super::{EditOptions, EditResult, Feature};
use super::FeatureServiceClient;
use crate::{LayerId, ObjectId, Result, check_esri_error};
use tracing::instrument;

impl<'a> FeatureServiceClient<'a> {
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
    /// let new_feature = Feature::new(attributes, None);
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

        tracing::debug!(url = %url, feature_count = features.len(), "Sending addFeatures request");

        // Build form data
        let features_json = serde_json::to_string(&features)?;
        let mut form = vec![("features", features_json.as_str()), ("f", "json")];

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
            tracing::error!(status = %status, error = %error_text, "addFeatures request failed");
            return Err(crate::Error::from(crate::ErrorKind::Api {
                code: status.as_u16() as i32,
                message: format!("HTTP {}: {}", status, error_text),
            }));
        }

        let response_text = response.text().await?;
        check_esri_error(&response_text, "addFeatures")?;
        let result: EditResult = serde_json::from_str(&response_text)?;

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
    /// let updated_feature = Feature::new(attributes, None);
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

        tracing::debug!(url = %url, feature_count = features.len(), "Sending updateFeatures request");

        // Build form data
        let features_json = serde_json::to_string(&features)?;
        let mut form = vec![("features", features_json.as_str()), ("f", "json")];

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
            tracing::error!(status = %status, error = %error_text, "updateFeatures request failed");
            return Err(crate::Error::from(crate::ErrorKind::Api {
                code: status.as_u16() as i32,
                message: format!("HTTP {}: {}", status, error_text),
            }));
        }

        let response_text = response.text().await?;
        check_esri_error(&response_text, "updateFeatures")?;
        let result: EditResult = serde_json::from_str(&response_text)?;

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

        // Convert ObjectIds to comma-separated string
        let object_ids_str = object_ids
            .iter()
            .map(|id| id.to_string())
            .collect::<Vec<_>>()
            .join(",");

        tracing::debug!(url = %url, object_ids = %object_ids_str, "Sending deleteFeatures request");

        // Build form data
        let mut form = vec![("objectIds", object_ids_str.as_str()), ("f", "json")];

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
            tracing::error!(status = %status, error = %error_text, "deleteFeatures request failed");
            return Err(crate::Error::from(crate::ErrorKind::Api {
                code: status.as_u16() as i32,
                message: format!("HTTP {}: {}", status, error_text),
            }));
        }

        let response_text = response.text().await?;
        check_esri_error(&response_text, "deleteFeatures")?;
        let result: EditResult = serde_json::from_str(&response_text)?;

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
    /// let new_feature = Feature::new(new_attrs, None);
    ///
    /// // Prepare features to update
    /// let mut update_attrs = HashMap::new();
    /// update_attrs.insert("OBJECTID".to_string(), json!(123));
    /// update_attrs.insert("POPULATION".to_string(), json!(55000));
    /// let updated_feature = Feature::new(update_attrs, None);
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
        let mut form: Vec<(&str, &str)> = vec![("f", "json")];

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
            tracing::error!(status = %status, error = %error_text, "applyEdits request failed");
            return Err(crate::Error::from(crate::ErrorKind::Api {
                code: status.as_u16() as i32,
                message: format!("HTTP {}: {}", status, error_text),
            }));
        }

        let response_text = response.text().await?;
        check_esri_error(&response_text, "applyEdits")?;
        let result: EditResult = serde_json::from_str(&response_text)?;

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

    /// Calculates field values for features using SQL expressions.
    ///
    /// This operation performs field calculations on existing features, similar
    /// to a field calculator. Supports both simple field assignments and complex
    /// SQL expressions.
    ///
    /// # Arguments
    ///
    /// * `layer_id` - The layer containing features to update
    /// * `where_clause` - SQL WHERE clause to select features to update
    /// * `calc_expression` - Array of field calculation expressions
    /// * `options` - Edit options (session, versioning, etc.)
    ///
    /// # Example
    ///
    /// ```no_run
    /// use arcgis::{ArcGISClient, ApiKeyAuth, FeatureServiceClient, LayerId, EditOptions, FieldCalculation};
    ///
    /// # async fn example() -> arcgis::Result<()> {
    /// let auth = ApiKeyAuth::new("YOUR_API_KEY");
    /// let client = ArcGISClient::new(auth);
    /// let service = FeatureServiceClient::new("https://example.com/FeatureServer", &client);
    ///
    /// // Calculate field values
    /// let calculations = vec![
    ///     FieldCalculation::with_sql_expression("STATUS", "CASE WHEN POPULATION > 100000 THEN 'Large' ELSE 'Small' END"),
    /// ];
    ///
    /// let result = service
    ///     .calculate_records(
    ///         LayerId::new(0),
    ///         "STATE = 'CA'",
    ///         calculations,
    ///         EditOptions::default(),
    ///     )
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    #[instrument(skip(self, where_clause, calc_expression, options), fields(layer_id = %layer_id, calc_count = calc_expression.len()))]
    pub async fn calculate_records(
        &self,
        layer_id: LayerId,
        where_clause: impl Into<String>,
        calc_expression: Vec<crate::FieldCalculation>,
        options: EditOptions,
    ) -> Result<crate::CalculateResult> {
        tracing::debug!("Calculating field values");

        let url = format!("{}/{}/calculate", self.base_url, layer_id);

        let where_str = where_clause.into();
        let calc_json = serde_json::to_string(&calc_expression)?;

        tracing::debug!(
            url = %url,
            where_clause = %where_str,
            calc_expression_count = calc_expression.len(),
            rollback_on_failure = ?options.rollback_on_failure,
            "Sending calculate request"
        );

        let mut form = vec![
            ("where", where_str.as_str()),
            ("calcExpression", calc_json.as_str()),
            ("f", "json"),
        ];

        // Add optional parameters
        let session_id_str = options.session_id.as_ref().map(|s| s.to_string());
        if let Some(ref session_id) = session_id_str {
            tracing::debug!(session_id = %session_id, "Using edit session");
            form.push(("sessionId", session_id.as_str()));
        }
        if let Some(ref gdb_version) = options.gdb_version {
            tracing::debug!(gdb_version = %gdb_version, "Using geodatabase version");
            form.push(("gdbVersion", gdb_version.as_str()));
        }
        if let Some(rollback) = options.rollback_on_failure {
            form.push(("rollbackOnFailure", if rollback { "true" } else { "false" }));
        }

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
            tracing::error!(status = %status, error = %error_text, "calculate request failed");
            return Err(crate::Error::from(crate::ErrorKind::Api {
                code: status.as_u16() as i32,
                message: format!("HTTP {}: {}", status, error_text),
            }));
        }

        let response_text = response.text().await?;
        tracing::debug!(
            response_length = response_text.len(),
            "Received calculate response"
        );

        check_esri_error(&response_text, "calculate")?;

        let result: crate::CalculateResult = serde_json::from_str(&response_text).map_err(|e| {
            tracing::error!(
                error = %e,
                response_preview = %&response_text[..response_text.len().min(500)],
                "Failed to deserialize CalculateResult"
            );
            e
        })?;

        tracing::info!(
            success = result.success(),
            updated_count = ?result.updated_feature_count(),
            edit_moment = ?result.edit_moment(),
            "Calculate completed"
        );

        Ok(result)
    }

    /// Applies edits to a layer using global IDs instead of object IDs.
    ///
    /// This method is similar to [`apply_edits`](Self::apply_edits) but uses global IDs
    /// for identifying features. Global IDs are stable across replicas and are useful
    /// in disconnected editing scenarios.
    ///
    /// # Arguments
    ///
    /// * `layer_id` - The layer to apply edits to
    /// * `adds` - Optional vector of features to add
    /// * `updates` - Optional vector of features to update (must include globalId attribute)
    /// * `deletes` - Optional vector of global IDs to delete
    /// * `options` - Edit options (session, versioning, etc.)
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
    /// // Update using global IDs
    /// let mut update_attrs = HashMap::new();
    /// update_attrs.insert("globalId".to_string(), json!("{12345678-1234-1234-1234-123456789012}"));
    /// update_attrs.insert("STATUS".to_string(), json!("Updated"));
    /// let updated_feature = Feature::new(update_attrs, None);
    ///
    /// // Delete by global IDs
    /// let global_ids_to_delete = vec![
    ///     "{87654321-4321-4321-4321-210987654321}".to_string(),
    /// ];
    ///
    /// let result = service
    ///     .apply_edits_with_global_ids(
    ///         LayerId::new(0),
    ///         None,
    ///         Some(vec![updated_feature]),
    ///         Some(global_ids_to_delete),
    ///         EditOptions::default(),
    ///     )
    ///     .await?;
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
    pub async fn apply_edits_with_global_ids(
        &self,
        layer_id: LayerId,
        adds: Option<Vec<Feature>>,
        updates: Option<Vec<Feature>>,
        deletes: Option<Vec<String>>,
        options: EditOptions,
    ) -> Result<EditResult> {
        tracing::debug!("Applying batch edits to layer using global IDs");

        let url = format!("{}/{}/applyEdits", self.base_url, layer_id);

        tracing::debug!(url = %url, "Sending applyEdits (global IDs) request");

        // Pre-allocate owned strings that need to live for the duration of the request
        let adds_json = adds.as_ref().map(serde_json::to_string).transpose()?;
        let updates_json = updates.as_ref().map(serde_json::to_string).transpose()?;
        let deletes_json = deletes.as_ref().map(serde_json::to_string).transpose()?;

        // Build form data with references to owned strings
        let mut form: Vec<(&str, &str)> = vec![("f", "json"), ("useGlobalIds", "true")];

        if let Some(ref adds) = adds_json {
            form.push(("adds", adds.as_str()));
        }
        if let Some(ref updates) = updates_json {
            form.push(("updates", updates.as_str()));
        }
        if let Some(ref deletes) = deletes_json {
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
            tracing::error!(status = %status, error = %error_text, "applyEditsWithGlobalIds request failed");
            return Err(crate::Error::from(crate::ErrorKind::Api {
                code: status.as_u16() as i32,
                message: format!("HTTP {}: {}", status, error_text),
            }));
        }

        let response_text = response.text().await?;
        check_esri_error(&response_text, "applyEditsWithGlobalIds")?;
        let result: EditResult = serde_json::from_str(&response_text)?;

        tracing::info!(
            success_count = result.success_count(),
            failure_count = result.failure_count(),
            "applyEditsWithGlobalIds completed"
        );

        Ok(result)
    }
}
