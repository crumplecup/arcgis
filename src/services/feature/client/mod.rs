//! Feature Service client for querying and editing features.

mod admin;
mod attachment;
mod definition;
mod edit;
mod query;

use crate::{ArcGISClient, LayerId, QueryBuilder};
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
    pub(super) base_url: String,
    /// Reference to the ArcGIS client for HTTP operations.
    pub(super) client: &'a ArcGISClient,
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
    /// println!("Retrieved {} features", features.features().len());
    /// # Ok(())
    /// # }
    /// ```
    #[instrument(skip(self), fields(layer_id = %layer_id, base_url = %self.base_url))]
    pub fn query(&'a self, layer_id: LayerId) -> QueryBuilder<'a> {
        tracing::debug!(layer_id = %layer_id, "Creating query builder");
        QueryBuilder::new(self, layer_id)
    }
}
