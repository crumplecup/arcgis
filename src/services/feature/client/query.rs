//! Query operations for the Feature Service client.

use super::super::{FeatureQueryParams, FeatureSet};
use super::FeatureServiceClient;
use crate::{LayerId, Result};
use tracing::instrument;

impl<'a> FeatureServiceClient<'a> {
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
    /// println!("Retrieved {} features", features.features().len());
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

        // Parse the response based on the requested format
        let feature_set = match params.format() {
            crate::ResponseFormat::Pbf => {
                // PBF format - decode binary protocol buffer
                let bytes = response.bytes().await?;
                tracing::debug!(bytes_len = bytes.len(), "Received PBF response");
                super::super::pbf::decode_feature_collection(&bytes)?
            }
            _ => {
                // JSON or GeoJSON format - use standard JSON parsing
                response.json().await?
            }
        };

        tracing::debug!(
            feature_count = feature_set.features().len(),
            exceeded_limit = feature_set.exceeded_transfer_limit(),
            format = ?params.format(),
            "Query completed successfully"
        );

        Ok(feature_set)
    }

    /// Queries related records for specified object IDs.
    ///
    /// This method retrieves records from related tables/layers based on relationship classes.
    /// Results are grouped by source object ID.
    ///
    /// # Arguments
    ///
    /// * `layer_id` - The layer to query from
    /// * `params` - Related records query parameters
    ///
    /// # Example
    ///
    /// ```no_run
    /// use arcgis::{ApiKeyAuth, ArcGISClient, FeatureServiceClient, LayerId, ObjectId, RelatedRecordsParams};
    ///
    /// # async fn example() -> arcgis::Result<()> {
    /// let auth = ApiKeyAuth::new("YOUR_API_KEY");
    /// let client = ArcGISClient::new(auth);
    /// let service = FeatureServiceClient::new(
    ///     "https://services.arcgis.com/org/arcgis/rest/services/Dataset/FeatureServer",
    ///     &client,
    /// );
    ///
    /// let params = RelatedRecordsParams::builder()
    ///     .object_ids(vec![ObjectId::new(1), ObjectId::new(2)])
    ///     .relationship_id(3u32)
    ///     .out_fields(vec!["NAME".to_string(), "STATUS".to_string()])
    ///     .build()
    ///     .expect("Valid params");
    ///
    /// let response = service.query_related_records(LayerId::new(0), params).await?;
    /// for group in response.related_record_groups() {
    ///     println!("Object {}: {} related records",
    ///         group.object_id(),
    ///         group.related_records().len());
    /// }
    /// # Ok(())
    /// # }
    /// ```
    #[instrument(skip(self, params), fields(layer_id = %layer_id, base_url = %self.base_url))]
    pub async fn query_related_records(
        &self,
        layer_id: LayerId,
        params: crate::RelatedRecordsParams,
    ) -> Result<crate::RelatedRecordsResponse> {
        tracing::debug!("Querying related records");

        // Construct the URL
        let url = format!("{}/{}/queryRelatedRecords", self.base_url, layer_id);

        // Get authentication token
        let token = self.client.auth().get_token().await?;

        tracing::debug!(url = %url, "Sending query related records request");

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
            tracing::error!(status = %status, error = %error_text, "Query related records request failed");
            return Err(crate::Error::from(crate::ErrorKind::Api {
                code: status.as_u16() as i32,
                message: format!("HTTP {}: {}", status, error_text),
            }));
        }

        // Parse the response
        let result: crate::RelatedRecordsResponse = response.json().await?;

        tracing::debug!(
            groups_count = result.related_record_groups().len(),
            "Query related records completed successfully"
        );

        Ok(result)
    }

    /// Queries top features from a layer based on ranking within groups.
    ///
    /// The queryTopFeatures operation returns features based on top features by order within a group.
    /// For example, you can query the top 3 most populous cities from each state.
    ///
    /// # Arguments
    ///
    /// * `layer_id` - The layer to query
    /// * `params` - Top features query parameters including topFilter (required)
    ///
    /// # Example
    ///
    /// ```no_run
    /// use arcgis::{ApiKeyAuth, ArcGISClient, FeatureServiceClient, LayerId, TopFeaturesParams, TopFilter};
    ///
    /// # async fn example() -> arcgis::Result<()> {
    /// let auth = ApiKeyAuth::new("YOUR_API_KEY");
    /// let client = ArcGISClient::new(auth);
    /// let service = FeatureServiceClient::new(
    ///     "https://services.arcgis.com/org/arcgis/rest/services/Dataset/FeatureServer",
    ///     &client,
    /// );
    ///
    /// // Get top 3 most populous cities from each state
    /// let filter = TopFilter::new(
    ///     vec!["State".to_string()],
    ///     3,
    ///     vec!["Population DESC".to_string()],
    /// );
    ///
    /// let params = TopFeaturesParams::builder()
    ///     .top_filter(filter)
    ///     .out_fields(vec!["Name".to_string(), "State".to_string(), "Population".to_string()])
    ///     .build()
    ///     .expect("Valid params");
    ///
    /// let feature_set = service.query_top_features(LayerId::new(0), params).await?;
    /// for feature in feature_set.features() {
    ///     println!("City: {:?}", feature.attributes().get("Name"));
    /// }
    /// # Ok(())
    /// # }
    /// ```
    #[instrument(skip(self, params), fields(layer_id = %layer_id, base_url = %self.base_url))]
    pub async fn query_top_features(
        &self,
        layer_id: LayerId,
        params: crate::TopFeaturesParams,
    ) -> Result<crate::FeatureSet> {
        tracing::debug!("Querying top features");

        // Construct the URL
        let url = format!("{}/{}/queryTopFeatures", self.base_url, layer_id);

        // Get authentication token
        let token = self.client.auth().get_token().await?;

        tracing::debug!(url = %url, "Sending query top features request");

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
            tracing::error!(status = %status, error = %error_text, "Query top features request failed");
            return Err(crate::Error::from(crate::ErrorKind::Api {
                code: status.as_u16() as i32,
                message: format!("HTTP {}: {}", status, error_text),
            }));
        }

        // Parse the response based on the requested format
        let result = if let Some(format_str) = params.f() {
            if format_str == "pbf" {
                // PBF format - decode binary protocol buffer
                let bytes = response.bytes().await?;
                tracing::debug!(bytes_len = bytes.len(), "Received PBF response");
                super::super::pbf::decode_feature_collection(&bytes)?
            } else {
                // JSON or GeoJSON format - use standard JSON parsing
                response.json().await?
            }
        } else {
            // Default to JSON parsing
            response.json().await?
        };

        tracing::debug!(
            features_count = result.features().len(),
            format = ?params.f(),
            "Query top features completed successfully"
        );

        Ok(result)
    }

    /// Efficiently counts features matching a query without returning feature data.
    ///
    /// This operation returns only the count of features matching the query criteria,
    /// making it much more efficient than querying all features and counting them.
    ///
    /// # Arguments
    ///
    /// * `layer_id` - The layer to query
    /// * `where_clause` - SQL WHERE clause to filter features (default: "1=1")
    ///
    /// # Returns
    ///
    /// The count of features matching the query.
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
    /// // Count all features
    /// let total_count = service.query_feature_count(LayerId::new(0), "1=1").await?;
    ///
    /// // Count features matching criteria
    /// let filtered_count = service
    ///     .query_feature_count(LayerId::new(0), "STATE = 'CA' AND POPULATION > 100000")
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    #[instrument(skip(self, where_clause), fields(layer_id = %layer_id))]
    pub async fn query_feature_count(
        &self,
        layer_id: LayerId,
        where_clause: impl Into<String>,
    ) -> Result<u32> {
        tracing::debug!("Querying feature count");

        let params = FeatureQueryParams::builder()
            .where_clause(where_clause)
            .return_count_only(true)
            .return_geometry(false)
            .out_fields(vec![]) // No fields needed for count
            .build()
            .expect("Valid query params");

        let result = self.query_with_params(layer_id, params).await?;

        let count = (*result.count()).unwrap_or(0);
        tracing::info!(count = count, "Feature count query completed");

        Ok(count)
    }
}
