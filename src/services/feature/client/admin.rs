//! Administrative operations for the Feature Service client.

use super::FeatureServiceClient;
use crate::{LayerId, Result};
use tracing::instrument;

impl<'a> FeatureServiceClient<'a> {
    /// Deletes all features from a layer.
    ///
    /// This operation removes all features from the specified layer while preserving
    /// the layer structure and schema. Use with caution as this operation cannot be undone.
    ///
    /// # Arguments
    ///
    /// * `layer_id` - The layer to truncate
    /// * `options` - Edit options (session ID for versioned editing)
    ///
    /// # Example
    ///
    /// ```no_run
    /// use arcgis::{ArcGISClient, ApiKeyAuth, FeatureServiceClient, LayerId, EditOptions};
    ///
    /// # async fn example() -> arcgis::Result<()> {
    /// let auth = ApiKeyAuth::new("YOUR_API_KEY");
    /// let client = ArcGISClient::new(auth);
    /// let service = FeatureServiceClient::new("https://example.com/FeatureServer", &client);
    ///
    /// // Delete all features from layer 0
    /// let result = service.truncate(LayerId::new(0), EditOptions::default()).await?;
    /// if result.success().unwrap_or(false) {
    ///     println!("Truncate successful");
    /// } else {
    ///     println!("Truncate failed: {:?}", result.error());
    /// }
    /// # Ok(())
    /// # }
    /// ```
    #[instrument(skip(self, options), fields(layer_id = %layer_id))]
    pub async fn truncate(&self, layer_id: LayerId, options: crate::EditOptions) -> Result<crate::TruncateResult> {
        tracing::debug!("Truncating layer");

        let url = format!("{}/{}/truncate", self.base_url, layer_id);

        tracing::debug!(url = %url, "Sending truncate request");

        // Build form data

        let mut form = vec![("f", "json")];

        // Add optional parameters (similar to add_features pattern)
        let session_id_str = options.session_id.as_ref().map(|s| s.to_string());
        if let Some(ref session_id) = session_id_str {
            tracing::debug!(session_id = %session_id, "Adding sessionId parameter");
            form.push(("sessionId", session_id.as_str()));
        }
        if let Some(ref gdb_version) = options.gdb_version {
            tracing::debug!(gdb_version = %gdb_version, "Adding gdbVersion parameter");
            form.push(("gdbVersion", gdb_version.as_str()));
        }

        // Add token if required by auth provider

        let token_value;

        if let Some(token) = self.client.get_token_if_required().await? {
            token_value = token;

            form.push(("token", token_value.as_str()));
        }

        let response = self.client.http().post(&url).form(&form).send().await?;

        let status = response.status();
        if !status.is_success() {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|e| format!("Failed to read error: {}", e));
            tracing::error!(status = %status, error = %error_text, "truncate request failed");
            return Err(crate::Error::from(crate::ErrorKind::Api {
                code: status.as_u16() as i32,
                message: format!("HTTP {}: {}", status, error_text),
            }));
        }

        // Get raw response text for debugging
        let response_text = response.text().await?;
        tracing::debug!(response = %response_text, "Raw truncate response");

        // Try to deserialize
        let result: crate::TruncateResult = serde_json::from_str(&response_text)?;

        if result.success().unwrap_or(false) {
            tracing::info!("Truncate completed successfully");
        } else {
            tracing::warn!(error = ?result.error(), "Truncate operation failed or not supported");
        }

        Ok(result)
    }

    /// Queries domains and subtypes for a layer.
    ///
    /// Returns coded value domains and subtype information for specified layers.
    /// This is useful for getting valid values for fields with domains.
    ///
    /// # Arguments
    ///
    /// * `layers` - Layer IDs to query domains for. If empty, queries all layers.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use arcgis::{ArcGISClient, ApiKeyAuth, FeatureServiceClient, LayerId};
    ///
    /// # async fn example() -> arcgis::Result<()> {
    /// let auth = ApiKeyAuth::new("YOUR_API_KEY");
    /// let client = ArcGISClient::new(auth);
    /// let service = FeatureServiceClient::new("https://example.com/FeatureServer", &client);
    ///
    /// // Query domains for specific layers
    /// let result = service.query_domains(vec![LayerId::new(0), LayerId::new(1)]).await?;
    /// # Ok(())
    /// # }
    /// ```
    #[instrument(skip(self), fields(layer_count = layers.len()))]
    pub async fn query_domains(&self, layers: Vec<LayerId>) -> Result<crate::QueryDomainsResponse> {
        tracing::debug!("Querying domains");

        let url = format!("{}/queryDomains", self.base_url);

        let layers_str = layers
            .iter()
            .map(|id| id.to_string())
            .collect::<Vec<_>>()
            .join(",");

        tracing::debug!(url = %url, layers = %layers_str, "Sending queryDomains request");

        let mut form = vec![("f", "json")];

        // Add token if required by auth provider
        let token_opt = self.client.get_token_if_required().await?;
        let token_str;
        if let Some(token) = token_opt {
            token_str = token;
            form.push(("token", token_str.as_str()));
        }

        if !layers_str.is_empty() {
            form.push(("layers", &layers_str));
        }

        let response = self.client.http().post(&url).form(&form).send().await?;

        let status = response.status();
        if !status.is_success() {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|e| format!("Failed to read error: {}", e));
            tracing::error!(status = %status, error = %error_text, "queryDomains request failed");
            return Err(crate::Error::from(crate::ErrorKind::Api {
                code: status.as_u16() as i32,
                message: format!("HTTP {}: {}", status, error_text),
            }));
        }

        let result: crate::QueryDomainsResponse = response.json().await?;

        tracing::info!(
            layer_count = result.layers().len(),
            "queryDomains completed"
        );

        Ok(result)
    }
}
