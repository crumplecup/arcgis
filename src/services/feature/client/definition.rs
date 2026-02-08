//! Service definition retrieval operations for the Feature Service client.

use super::FeatureServiceClient;
use crate::{LayerDefinition, LayerId, Result, ServiceDefinition, TableDefinition};
use tracing::instrument;

impl<'a> FeatureServiceClient<'a> {
    /// Retrieves the service-level definition from an existing Feature Service.
    ///
    /// Fetches the service root endpoint (`GET {serviceUrl}?f=json`) and deserializes
    /// the JSON response into a [`ServiceDefinition`]. This is useful for inspecting
    /// service metadata and discovering layer/table IDs before fetching full definitions.
    ///
    /// # Layer Stubs
    ///
    /// The ESRI service root response contains only layer stubs (id, name, geometry type)
    /// without full field definitions. To get complete field definitions for a specific
    /// layer, use [`get_layer_definition`](Self::get_layer_definition).
    ///
    /// # Service Name
    ///
    /// The service name is encoded in the URL path, not the JSON response. The returned
    /// `ServiceDefinition` will have an empty `name`. Set it from your URL if needed.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use arcgis::{ArcGISClient, ApiKeyAuth, FeatureServiceClient};
    ///
    /// # async fn example() -> arcgis::Result<()> {
    /// let auth = ApiKeyAuth::new("YOUR_API_KEY");
    /// let client = ArcGISClient::new(auth);
    /// let service = FeatureServiceClient::new(
    ///     "https://services.arcgis.com/org/arcgis/rest/services/MyService/FeatureServer",
    ///     &client,
    /// );
    ///
    /// let definition = service.get_definition().await?;
    /// println!("Max record count: {:?}", definition.max_record_count());
    /// println!("Layers: {}", definition.layers().len());
    /// # Ok(())
    /// # }
    /// ```
    #[instrument(skip(self), fields(base_url = %self.base_url))]
    pub async fn get_definition(&self) -> Result<ServiceDefinition> {
        tracing::debug!("Fetching service definition");

        let url = &self.base_url;
        tracing::debug!(url = %url, "Sending service definition request");

        let mut request = self.client.http().get(url).query(&[("f", "json")]);

        if let Some(token) = self.client.get_token_if_required().await? {
            request = request.query(&[("token", token)]);
        }

        let response = request.send().await?;

        let status = response.status();
        if !status.is_success() {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|e| format!("Failed to read error response: {}", e));
            tracing::error!(status = %status, error = %error_text, "get_definition request failed");
            return Err(crate::Error::from(crate::ErrorKind::Api {
                code: status.as_u16() as i32,
                message: format!("HTTP {}: {}", status, error_text),
            }));
        }

        let definition: ServiceDefinition = response.json().await?;

        tracing::info!(
            layer_count = definition.layers().len(),
            table_count = definition.tables().len(),
            "Service definition retrieved"
        );

        Ok(definition)
    }

    /// Retrieves the full definition for a specific layer.
    ///
    /// Fetches `GET {serviceUrl}/{layerId}?f=json` and deserializes the full
    /// [`LayerDefinition`], including all field definitions, relationships,
    /// templates, and indexes.
    ///
    /// This is the recommended way to read an existing layer's schema before
    /// making updates via `addToDefinition` or `updateDefinition`.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use arcgis::{ArcGISClient, ApiKeyAuth, FeatureServiceClient, LayerId};
    ///
    /// # async fn example() -> arcgis::Result<()> {
    /// let auth = ApiKeyAuth::new("YOUR_API_KEY");
    /// let client = ArcGISClient::new(auth);
    /// let service = FeatureServiceClient::new(
    ///     "https://services.arcgis.com/org/arcgis/rest/services/MyService/FeatureServer",
    ///     &client,
    /// );
    ///
    /// let layer = service.get_layer_definition(LayerId::new(0)).await?;
    /// println!("Layer: {}", layer.name());
    /// println!("Fields: {}", layer.fields().len());
    /// for field in layer.fields() {
    ///     println!("  - {} ({:?})", field.name(), field.field_type());
    /// }
    /// # Ok(())
    /// # }
    /// ```
    #[instrument(skip(self), fields(base_url = %self.base_url, layer_id = %layer_id))]
    pub async fn get_layer_definition(&self, layer_id: LayerId) -> Result<LayerDefinition> {
        tracing::debug!("Fetching layer definition");

        let url = format!("{}/{}", self.base_url, layer_id);
        tracing::debug!(url = %url, "Sending layer definition request");

        let mut request = self.client.http().get(&url).query(&[("f", "json")]);

        if let Some(token) = self.client.get_token_if_required().await? {
            request = request.query(&[("token", token)]);
        }

        let response = request.send().await?;

        let status = response.status();
        if !status.is_success() {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|e| format!("Failed to read error response: {}", e));
            tracing::error!(status = %status, error = %error_text, "get_layer_definition request failed");
            return Err(crate::Error::from(crate::ErrorKind::Api {
                code: status.as_u16() as i32,
                message: format!("HTTP {}: {}", status, error_text),
            }));
        }

        let layer: LayerDefinition = response.json().await?;

        tracing::info!(
            name = %layer.name(),
            field_count = layer.fields().len(),
            "Layer definition retrieved"
        );

        Ok(layer)
    }

    /// Retrieves the full definition for a specific table.
    ///
    /// Fetches `GET {serviceUrl}/{tableId}?f=json` and deserializes the full
    /// [`TableDefinition`], including all field definitions and relationships.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use arcgis::{ArcGISClient, ApiKeyAuth, FeatureServiceClient, LayerId};
    ///
    /// # async fn example() -> arcgis::Result<()> {
    /// let auth = ApiKeyAuth::new("YOUR_API_KEY");
    /// let client = ArcGISClient::new(auth);
    /// let service = FeatureServiceClient::new(
    ///     "https://services.arcgis.com/org/arcgis/rest/services/MyService/FeatureServer",
    ///     &client,
    /// );
    ///
    /// // Tables use the same numeric IDs as layers in the URL path
    /// let table = service.get_table_definition(LayerId::new(1)).await?;
    /// println!("Table: {}", table.name());
    /// println!("Fields: {}", table.fields().len());
    /// # Ok(())
    /// # }
    /// ```
    #[instrument(skip(self), fields(base_url = %self.base_url, table_id = %table_id))]
    pub async fn get_table_definition(&self, table_id: LayerId) -> Result<TableDefinition> {
        tracing::debug!("Fetching table definition");

        let url = format!("{}/{}", self.base_url, table_id);
        tracing::debug!(url = %url, "Sending table definition request");

        let mut request = self.client.http().get(&url).query(&[("f", "json")]);

        if let Some(token) = self.client.get_token_if_required().await? {
            request = request.query(&[("token", token)]);
        }

        let response = request.send().await?;

        let status = response.status();
        if !status.is_success() {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|e| format!("Failed to read error response: {}", e));
            tracing::error!(status = %status, error = %error_text, "get_table_definition request failed");
            return Err(crate::Error::from(crate::ErrorKind::Api {
                code: status.as_u16() as i32,
                message: format!("HTTP {}: {}", status, error_text),
            }));
        }

        let table: TableDefinition = response.json().await?;

        tracing::info!(
            name = %table.name(),
            field_count = table.fields().len(),
            "Table definition retrieved"
        );

        Ok(table)
    }
}
