//! Item operations for the Portal client.

use super::super::{
    AddItemParams, AddItemResult, DeleteItemResult, ItemInfo, UpdateItemParams, UpdateItemResult,
};
use super::PortalClient;
use crate::Result;
use tracing::instrument;

impl<'a> PortalClient<'a> {
    /// Gets a portal item by ID.
    ///
    /// Retrieves detailed metadata for a specific item.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use arcgis::{ArcGISClient, ApiKeyAuth, PortalClient};
    /// # async fn example(portal: &PortalClient<'_>) -> arcgis::Result<()> {
    /// let item = portal.get_item("abc123def456").await?;
    /// println!("Title: {}", item.title());
    /// println!("Type: {}", item.item_type());
    /// println!("Owner: {}", item.owner());
    /// # Ok(())
    /// # }
    /// ```
    #[instrument(skip(self, item_id))]
    pub async fn get_item(&self, item_id: impl AsRef<str>) -> Result<ItemInfo> {
        let item_id = item_id.as_ref();
        tracing::debug!(item_id = %item_id, "Getting item");

        let url = format!("{}/content/items/{}", self.base_url, item_id);

        // Get authentication token

        tracing::debug!(url = %url, "Sending getItem request");

        // Build request
        let mut request = self.client.http().get(&url).query(&[("f", "json")]);

        if let Some(token) = self.client.get_token_if_required().await? {

            request = request.query(&[("token", token)]);

        }

        let response = request.send().await?;

        // Check for HTTP errors
        let status = response.status();
        if !status.is_success() {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|e| format!("Failed to read error response: {}", e));
            tracing::error!(status = %status, error = %error_text, "getItem request failed");
            return Err(crate::Error::from(crate::ErrorKind::Api {
                code: status.as_u16() as i32,
                message: format!("HTTP {}: {}", status, error_text),
            }));
        }

        // Parse response
        let item: ItemInfo = response.json().await?;

        tracing::debug!(title = %item.title(), item_type = %item.item_type(), "Got item");

        Ok(item)
    }

    /// Adds a new item to the portal.
    ///
    /// Creates a new item in the authenticated user's content area.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use arcgis::{ArcGISClient, ApiKeyAuth, PortalClient, AddItemParams};
    /// # async fn example(portal: &PortalClient<'_>) -> arcgis::Result<()> {
    /// let params = AddItemParams::new("My Map", "Web Map")
    ///     .with_description("A sample web map")
    ///     .with_tags(vec!["sample".to_string(), "demo".to_string()]);
    ///
    /// let result = portal.add_item(params).await?;
    /// println!("Created item: {}", result.id());
    /// # Ok(())
    /// # }
    /// ```
    #[instrument(skip(self, params))]
    pub async fn add_item(&self, params: AddItemParams) -> Result<AddItemResult> {
        tracing::debug!(title = %params.title(), item_type = %params.item_type(), "Adding item");

        // Get authentication token

        // We need the username to construct the URL
        let user = self.get_self().await?;
        let url = format!(
            "{}/content/users/{}/addItem",
            self.base_url,
            user.username()
        );

        tracing::debug!(url = %url, username = %user.username(), "Sending addItem request");

        // Build form data
        let mut form = reqwest::multipart::Form::new()
            .text("f", "json")
            .text("title", params.title().to_string())
            .text("type", params.item_type().to_string());

        if let Some(desc) = params.description() {
            form = form.text("description", desc.to_string());
        }

        if let Some(tags) = params.tags() {
            form = form.text("tags", tags.join(","));
        }

        if let Some(snippet) = params.snippet() {
            form = form.text("snippet", snippet.to_string());
        }

        if let Some(categories) = params.categories() {
            form = form.text("categories", categories.join(","));
        }

        if let Some(keywords) = params.type_keywords() {
            form = form.text("typeKeywords", keywords.join(","));
        }

        if let Some(url_str) = params.url() {
            form = form.text("url", url_str.to_string());
        }

        if let Some(wkid) = params.spatial_reference() {
            form = form.text("spatialReference", wkid.to_string());
        }

        if let Some(extent) = params.extent() {
            let extent_str = serde_json::to_string(extent)?;
            form = form.text("extent", extent_str);
        }

        if let Some(access) = params.access() {
            form = form.text("access", access.to_string());
        }

        if let Some(properties) = params.properties() {
            let props_str = serde_json::to_string(properties)?;
            form = form.text("properties", props_str);
        }

        // Build request
        // Add token if required by auth provider

        if let Some(token) = self.client.get_token_if_required().await? {

            form = form.text("token", token);

        }


        let response = self.client.http().post(&url).multipart(form).send().await?;

        // Check for HTTP errors
        let status = response.status();
        if !status.is_success() {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|e| format!("Failed to read error response: {}", e));
            tracing::error!(status = %status, error = %error_text, "addItem request failed");
            return Err(crate::Error::from(crate::ErrorKind::Api {
                code: status.as_u16() as i32,
                message: format!("HTTP {}: {}", status, error_text),
            }));
        }

        // Parse response
        let result: AddItemResult = response.json().await?;

        tracing::debug!(item_id = %result.id(), success = result.success(), "Item added");

        Ok(result)
    }

    /// Updates an existing portal item's metadata.
    ///
    /// Updates properties like title, description, tags, etc. for an existing item.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use arcgis::{ArcGISClient, ApiKeyAuth, PortalClient, UpdateItemParams};
    /// # async fn example(portal: &PortalClient<'_>) -> arcgis::Result<()> {
    /// let params = UpdateItemParams::new()
    ///     .with_title("Updated Title")
    ///     .with_description("Updated description");
    ///
    /// let result = portal.update_item("abc123def456", params).await?;
    /// println!("Update success: {}", result.success());
    /// # Ok(())
    /// # }
    /// ```
    #[instrument(skip(self, item_id, params))]
    pub async fn update_item(
        &self,
        item_id: impl AsRef<str>,
        params: UpdateItemParams,
    ) -> Result<UpdateItemResult> {
        let item_id = item_id.as_ref();
        tracing::debug!(item_id = %item_id, "Updating item");

        // Get authentication token

        // Get the item to find its owner
        let item = self.get_item(item_id).await?;
        let url = format!(
            "{}/content/users/{}/items/{}/update",
            self.base_url,
            item.owner(),
            item_id
        );

        tracing::debug!(url = %url, owner = %item.owner(), "Sending updateItem request");

        // Build form data
        let mut form = reqwest::multipart::Form::new()
            .text("f", "json");

        if let Some(title) = params.title() {
            form = form.text("title", title.to_string());
        }

        if let Some(desc) = params.description() {
            form = form.text("description", desc.to_string());
        }

        if let Some(tags) = params.tags() {
            form = form.text("tags", tags.join(","));
        }

        if let Some(snippet) = params.snippet() {
            form = form.text("snippet", snippet.to_string());
        }

        if let Some(categories) = params.categories() {
            form = form.text("categories", categories.join(","));
        }

        if let Some(keywords) = params.type_keywords() {
            form = form.text("typeKeywords", keywords.join(","));
        }

        if let Some(url_str) = params.url() {
            form = form.text("url", url_str.to_string());
        }

        if let Some(wkid) = params.spatial_reference() {
            form = form.text("spatialReference", wkid.to_string());
        }

        if let Some(extent) = params.extent() {
            let extent_str = serde_json::to_string(extent)?;
            form = form.text("extent", extent_str);
        }

        if let Some(access) = params.access() {
            form = form.text("access", access.to_string());
        }

        // Build request
        // Add token if required by auth provider

        if let Some(token) = self.client.get_token_if_required().await? {

            form = form.text("token", token);

        }


        let response = self.client.http().post(&url).multipart(form).send().await?;

        // Check for HTTP errors
        let status = response.status();
        if !status.is_success() {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|e| format!("Failed to read error response: {}", e));
            tracing::error!(status = %status, error = %error_text, "updateItem request failed");
            return Err(crate::Error::from(crate::ErrorKind::Api {
                code: status.as_u16() as i32,
                message: format!("HTTP {}: {}", status, error_text),
            }));
        }

        // Parse response
        let result: UpdateItemResult = response.json().await?;

        tracing::debug!(success = result.success(), "Item updated");

        Ok(result)
    }

    /// Deletes a portal item.
    ///
    /// Permanently removes an item from the portal.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use arcgis::{ArcGISClient, ApiKeyAuth, PortalClient};
    /// # async fn example(portal: &PortalClient<'_>) -> arcgis::Result<()> {
    /// let result = portal.delete_item("abc123def456").await?;
    /// println!("Delete success: {}", result.success());
    /// # Ok(())
    /// # }
    /// ```
    #[instrument(skip(self, item_id))]
    pub async fn delete_item(&self, item_id: impl AsRef<str>) -> Result<DeleteItemResult> {
        let item_id = item_id.as_ref();
        tracing::debug!(item_id = %item_id, "Deleting item");

        // Get authentication token

        // Get the item to find its owner
        let item = self.get_item(item_id).await?;
        let url = format!(
            "{}/content/users/{}/items/{}/delete",
            self.base_url,
            item.owner(),
            item_id
        );

        tracing::debug!(url = %url, owner = %item.owner(), "Sending deleteItem request");

        // Build request
        let mut form_data = vec![("f", "json")];


        // Add token if required by auth provider

        let token_opt = self.client.get_token_if_required().await?;

        let token_str;

        if let Some(token) = token_opt {

            token_str = token;

            form_data.push(("token", token_str.as_str()));

        }


        let response = self
            .client
            .http()
            .post(&url)
            .form(&form_data)
            .send()
            .await?;

        // Check for HTTP errors
        let status = response.status();
        if !status.is_success() {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|e| format!("Failed to read error response: {}", e));
            tracing::error!(status = %status, error = %error_text, "deleteItem request failed");
            return Err(crate::Error::from(crate::ErrorKind::Api {
                code: status.as_u16() as i32,
                message: format!("HTTP {}: {}", status, error_text),
            }));
        }

        // Parse response
        let result: DeleteItemResult = response.json().await?;

        tracing::debug!(success = result.success(), "Item deleted");

        Ok(result)
    }

    /// Downloads the data file associated with a portal item.
    ///
    /// Returns the raw bytes of the item's data file (e.g., the actual web map JSON,
    /// a GeoJSON file, or other item data).
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use arcgis::{ArcGISClient, ApiKeyAuth, PortalClient};
    /// # async fn example(portal: &PortalClient<'_>) -> arcgis::Result<()> {
    /// let data = portal.get_item_data("abc123def456").await?;
    /// println!("Downloaded {} bytes", data.len());
    /// # Ok(())
    /// # }
    /// ```
    #[instrument(skip(self, item_id))]
    pub async fn get_item_data(&self, item_id: impl AsRef<str>) -> Result<bytes::Bytes> {
        let item_id = item_id.as_ref();
        tracing::debug!(item_id = %item_id, "Getting item data");

        let url = format!("{}/content/items/{}/data", self.base_url, item_id);

        // Get authentication token

        tracing::debug!(url = %url, "Sending getItemData request");

        // Build request
        let mut request = self.client.http().get(&url).query(&[("f", "json")]);

        if let Some(token) = self.client.get_token_if_required().await? {

            request = request.query(&[("token", token)]);

        }

        let response = request.send().await?;

        // Check for HTTP errors
        let status = response.status();
        if !status.is_success() {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|e| format!("Failed to read error response: {}", e));
            tracing::error!(status = %status, error = %error_text, "getItemData request failed");
            return Err(crate::Error::from(crate::ErrorKind::Api {
                code: status.as_u16() as i32,
                message: format!("HTTP {}: {}", status, error_text),
            }));
        }

        // Get bytes
        let bytes = response.bytes().await?;

        tracing::debug!(size = bytes.len(), "Retrieved item data");

        Ok(bytes)
    }

    /// Uploads or updates the data file for a portal item.
    ///
    /// Updates the item's data content (e.g., web map definition, GeoJSON, etc.).
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use arcgis::{ArcGISClient, ApiKeyAuth, PortalClient};
    /// # async fn example(portal: &PortalClient<'_>) -> arcgis::Result<()> {
    /// let data = br#"{"type":"FeatureCollection","features":[]}"#.to_vec();
    /// let result = portal.update_item_data("abc123def456", data).await?;
    /// println!("Update success: {}", result.success());
    /// # Ok(())
    /// # }
    /// ```
    #[instrument(skip(self, item_id, data))]
    pub async fn update_item_data(
        &self,
        item_id: impl AsRef<str>,
        data: Vec<u8>,
    ) -> Result<UpdateItemResult> {
        let item_id = item_id.as_ref();
        tracing::debug!(item_id = %item_id, size = data.len(), "Updating item data");

        // Get authentication token

        // Get the item to find its owner
        let item = self.get_item(item_id).await?;
        let url = format!(
            "{}/content/users/{}/items/{}/update",
            self.base_url,
            item.owner(),
            item_id
        );

        tracing::debug!(url = %url, owner = %item.owner(), "Sending updateItemData request");

        // Create multipart form with file
        let part = reqwest::multipart::Part::bytes(data)
            .file_name("data.json")
            .mime_str("application/json")?;

        let mut form = reqwest::multipart::Form::new()
            .text("f", "json")
            .part("file", part);

        // Add token if required by auth provider
        if let Some(token) = self.client.get_token_if_required().await? {
            form = form.text("token", token);
        }


        let response = self.client.http().post(&url).multipart(form).send().await?;

        // Check for HTTP errors
        let status = response.status();
        if !status.is_success() {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|e| format!("Failed to read error response: {}", e));
            tracing::error!(status = %status, error = %error_text, "updateItemData request failed");
            return Err(crate::Error::from(crate::ErrorKind::Api {
                code: status.as_u16() as i32,
                message: format!("HTTP {}: {}", status, error_text),
            }));
        }

        // Parse response
        let result: UpdateItemResult = response.json().await?;

        tracing::debug!(success = result.success(), "Item data updated");

        Ok(result)
    }
}
