//! Feature Service client for querying and editing features.

use crate::{ArcGISClient, FeatureQueryParams, FeatureSet, LayerId, QueryBuilder, Result};
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
            let error_text = response.text().await.unwrap_or_default();
            tracing::error!(status = %status, error = %error_text, "Query request failed");
            return Err(crate::Error::api(
                status.as_u16() as i32,
                format!("HTTP {}: {}", status, error_text),
            ));
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
}
