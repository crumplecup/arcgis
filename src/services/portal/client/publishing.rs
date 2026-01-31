//! Service publishing operations for the Portal client.

use super::super::{
    CreateServiceParams, CreateServiceResult, DeleteServiceResult, OverwriteParameters,
    OverwriteResult, PublishParameters, PublishResult, PublishStatus,
    UpdateServiceDefinitionParams, UpdateServiceDefinitionResult,
};
use super::PortalClient;
use crate::Result;
use tracing::instrument;

impl<'a> PortalClient<'a> {
    /// Creates a new hosted feature service.
    ///
    /// Directly creates a new hosted feature service with the specified configuration.
    /// Unlike `publish()` which requires an existing item, this creates a service from scratch.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use arcgis::{ArcGISClient, ApiKeyAuth, PortalClient, CreateServiceParams};
    /// # async fn example(portal: &PortalClient<'_>) -> arcgis::Result<()> {
    /// // Create a simple feature service with a point layer
    /// let service_def = serde_json::json!({
    ///     "layers": [{
    ///         "name": "MyPoints",
    ///         "type": "Feature Layer",
    ///         "geometryType": "esriGeometryPoint",
    ///         "hasAttachments": true,
    ///         "fields": [
    ///             {
    ///                 "name": "OBJECTID",
    ///                 "type": "esriFieldTypeOID",
    ///                 "alias": "Object ID"
    ///             },
    ///             {
    ///                 "name": "Name",
    ///                 "type": "esriFieldTypeString",
    ///                 "alias": "Name",
    ///                 "length": 256
    ///             }
    ///         ]
    ///     }]
    /// });
    ///
    /// let params = CreateServiceParams::new("MyFeatureService")
    ///     .with_description("A hosted feature service")
    ///     .with_capabilities("Query,Create,Update,Delete,Editing")
    ///     .with_service_definition(service_def);
    ///
    /// let result = portal.create_service(params).await?;
    /// if let Some(service_url) = result.service_url() {
    ///     println!("Created service: {}", service_url);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    #[instrument(skip(self, params))]
    pub async fn create_service(&self, params: CreateServiceParams) -> Result<CreateServiceResult> {
        tracing::debug!(name = %params.name(), "Creating hosted feature service");

        let url = format!("{}/content/users/{{username}}/createService", self.base_url);

        // Get authentication token and user info
        let user = self.get_self().await?;
        let username = user.effective_username().ok_or_else(|| {
            crate::Error::from(crate::ErrorKind::Api {
                code: 401,
                message: "Username not available in user info".to_string(),
            })
        })?;
        let url = url.replace("{username}", username);

        tracing::debug!(url = %url, "Sending createService request");

        // Build the create parameters with full service definition
        let mut create_params_obj = serde_json::json!({
            "name": params.name(),
        });

        if let Some(desc) = params.description() {
            create_params_obj["description"] = serde_json::json!(desc);
        }

        if let Some(has_static) = params.has_static_data() {
            create_params_obj["hasStaticData"] = serde_json::json!(has_static);
        }

        if let Some(max_records) = params.max_record_count() {
            create_params_obj["maxRecordCount"] = serde_json::json!(max_records);
        }

        if let Some(formats) = params.supported_query_formats() {
            create_params_obj["supportedQueryFormats"] = serde_json::json!(formats);
        }

        if let Some(caps) = params.capabilities() {
            create_params_obj["capabilities"] = serde_json::json!(caps);
        }

        // Add layer definitions if provided
        if let Some(layers) = params.service_definition() {
            create_params_obj["layers"] = layers.clone();
        }

        // Build form data
        let mut form = reqwest::multipart::Form::new()
            .text("f", "json")
            .text("outputType", "featureService")
            .text("createParameters", create_params_obj.to_string());

        // Add token if required
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
            tracing::error!(status = %status, error = %error_text, "createService request failed");
            return Err(crate::Error::from(crate::ErrorKind::Api {
                code: status.as_u16() as i32,
                message: format!("HTTP {}: {}", status, error_text),
            }));
        }

        // Get response text for debugging
        let response_text = response.text().await?;
        tracing::debug!(response = %response_text, "createService raw response");

        // Parse response
        let result: CreateServiceResult = serde_json::from_str(&response_text)?;

        tracing::debug!(
            success = result.success(),
            service_item_id = ?result.service_item_id(),
            service_url = ?result.service_url(),
            "Service created"
        );

        Ok(result)
    }

    /// Publishes a hosted service from an item.
    ///
    /// Creates a new hosted feature layer or other service from a source item
    /// (e.g., file geodatabase, shapefile, CSV, GeoJSON).
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use arcgis::{ArcGISClient, ApiKeyAuth, PortalClient, PublishParameters};
    /// # async fn example(portal: &PortalClient<'_>) -> arcgis::Result<()> {
    /// let params = PublishParameters::new("MyFeatureService")
    ///     .with_description("Published from shapefile")
    ///     .with_max_record_count(1000)
    ///     .with_capabilities("Query,Create,Update,Delete".to_string());
    ///
    /// let result = portal.publish("source_item_id", params).await?;
    /// if let Some(service_url) = result.service_url() {
    ///     println!("Published service: {}", service_url);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    #[instrument(skip(self, item_id, params))]
    pub async fn publish(
        &self,
        item_id: impl AsRef<str>,
        params: PublishParameters,
    ) -> Result<PublishResult> {
        let item_id = item_id.as_ref();
        tracing::debug!(item_id = %item_id, name = %params.name(), "Publishing service");

        let url = format!("{}/content/users/{{username}}/publish", self.base_url);

        // Get authentication token and user info
        let user = self.get_self().await?;
        let username = user.effective_username().ok_or_else(|| {
            crate::Error::from(crate::ErrorKind::Api {
                code: 401,
                message: "Username not available in user info".to_string(),
            })
        })?;
        let url = url.replace("{username}", username);

        tracing::debug!(url = %url, "Sending publish request");

        // Build publish parameters as JSON
        let mut publish_params = serde_json::json!({
            "itemId": item_id,
            "filetype": "serviceDefinition",
            "publishParameters": {
                "name": params.name(),
            }
        });

        if let Some(desc) = params.description() {
            publish_params["publishParameters"]["description"] = serde_json::json!(desc);
        }

        if let Some(copyright) = params.copyright_text() {
            publish_params["publishParameters"]["copyrightText"] = serde_json::json!(copyright);
        }

        if let Some(has_static) = params.has_static_data() {
            publish_params["publishParameters"]["hasStaticData"] = serde_json::json!(has_static);
        }

        if let Some(max_records) = params.max_record_count() {
            publish_params["publishParameters"]["maxRecordCount"] = serde_json::json!(max_records);
        }

        if let Some(caps) = params.capabilities() {
            publish_params["publishParameters"]["capabilities"] = serde_json::json!(caps);
        }

        if let Some(wkid) = params.spatial_reference() {
            publish_params["publishParameters"]["spatialReference"] =
                serde_json::json!({ "wkid": wkid });
        }

        if let Some(extent) = params.initial_extent() {
            publish_params["publishParameters"]["initialExtent"] = serde_json::json!(extent);
        }

        if let Some(extent) = params.full_extent() {
            publish_params["publishParameters"]["fullExtent"] = serde_json::json!(extent);
        }

        if let Some(allow) = params.allow_geometry_updates() {
            publish_params["publishParameters"]["allowGeometryUpdates"] = serde_json::json!(allow);
        }

        if let Some(enable) = params.enable_versioning() {
            publish_params["publishParameters"]["enableVersioning"] = serde_json::json!(enable);
        }

        if let Some(units) = params.units() {
            publish_params["publishParameters"]["units"] = serde_json::json!(units);
        }

        if let Some(xss) = params.xss_prevention_enabled() {
            publish_params["publishParameters"]["xssPreventionEnabled"] = serde_json::json!(xss);
        }

        // Build form data
        let mut form = reqwest::multipart::Form::new()
            .text("f", "json")
            .text("itemId", item_id.to_string())
            .text("filetype", "serviceDefinition")
            .text(
                "publishParameters",
                publish_params["publishParameters"].to_string(),
            );

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
            tracing::error!(status = %status, error = %error_text, "publish request failed");
            return Err(crate::Error::from(crate::ErrorKind::Api {
                code: status.as_u16() as i32,
                message: format!("HTTP {}: {}", status, error_text),
            }));
        }

        // Get response text for debugging
        let response_text = response.text().await?;
        tracing::debug!(response = %response_text, "publish raw response");

        // Parse response
        let result: PublishResult = serde_json::from_str(&response_text)?;

        tracing::debug!(
            success = result.success(),
            service_item_id = ?result.service_item_id(),
            "Service published"
        );

        Ok(result)
    }

    /// Gets the status of a publishing job.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use arcgis::{ArcGISClient, ApiKeyAuth, PortalClient};
    /// # async fn example(portal: &PortalClient<'_>) -> arcgis::Result<()> {
    /// let status = portal.get_publish_status("job_id").await?;
    /// if let Some(job_status) = status.job_status() {
    ///     println!("Job status: {}", job_status);
    /// }
    /// if let Some(progress) = status.progress() {
    ///     println!("Progress: {}%", progress);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    #[instrument(skip(self, job_id))]
    pub async fn get_publish_status(&self, job_id: impl AsRef<str>) -> Result<PublishStatus> {
        let job_id = job_id.as_ref();
        tracing::debug!(job_id = %job_id, "Getting publish status");

        let url = format!(
            "{}/content/users/{{username}}/jobs/{}",
            self.base_url, job_id
        );

        // Get authentication token and user info
        let user = self.get_self().await?;
        let username = user.effective_username().ok_or_else(|| {
            crate::Error::from(crate::ErrorKind::Api {
                code: 401,
                message: "Username not available in user info".to_string(),
            })
        })?;
        let url = url.replace("{username}", username);

        tracing::debug!(url = %url, "Sending getPublishStatus request");

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
            tracing::error!(status = %status, error = %error_text, "getPublishStatus request failed");
            return Err(crate::Error::from(crate::ErrorKind::Api {
                code: status.as_u16() as i32,
                message: format!("HTTP {}: {}", status, error_text),
            }));
        }

        // Parse response
        let result: PublishStatus = response.json().await?;

        tracing::debug!(
            job_status = ?result.job_status(),
            progress = ?result.progress(),
            "Got publish status"
        );

        Ok(result)
    }

    /// Updates a service definition.
    ///
    /// Modifies the configuration of an existing hosted service.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use arcgis::{ArcGISClient, ApiKeyAuth, PortalClient, UpdateServiceDefinitionParams};
    /// # async fn example(portal: &PortalClient<'_>) -> arcgis::Result<()> {
    /// let params = UpdateServiceDefinitionParams::new()
    ///     .with_description("Updated service description")
    ///     .with_max_record_count(2000);
    ///
    /// let result = portal.update_service_definition("service_item_id", params).await?;
    /// println!("Update success: {}", result.success());
    /// # Ok(())
    /// # }
    /// ```
    #[instrument(skip(self, service_item_id, params))]
    pub async fn update_service_definition(
        &self,
        service_item_id: impl AsRef<str>,
        params: UpdateServiceDefinitionParams,
    ) -> Result<UpdateServiceDefinitionResult> {
        let service_item_id = service_item_id.as_ref();
        tracing::debug!(service_item_id = %service_item_id, "Updating service definition");

        // Get the service item to find its URL
        let item = self.get_item(service_item_id).await?;

        let service_url = item.url().clone().ok_or_else(|| {
            crate::Error::from(crate::ErrorKind::Api {
                code: 400,
                message: "Item does not have a service URL".to_string(),
            })
        })?;

        let url = format!("{}/updateDefinition", service_url);

        // Get authentication token
        tracing::debug!(url = %url, "Sending updateServiceDefinition request");

        // Build update parameters
        let mut form = reqwest::multipart::Form::new().text("f", "json");

        if let Some(def) = params.service_definition() {
            form = form.text("updateDefinition", def.to_string());
        }

        if let Some(desc) = params.description() {
            form = form.text("description", desc.to_string());
        }

        if let Some(caps) = params.capabilities() {
            form = form.text("capabilities", caps.to_string());
        }

        if let Some(max_records) = params.max_record_count() {
            form = form.text("maxRecordCount", max_records.to_string());
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
            tracing::error!(status = %status, error = %error_text, "updateServiceDefinition request failed");
            return Err(crate::Error::from(crate::ErrorKind::Api {
                code: status.as_u16() as i32,
                message: format!("HTTP {}: {}", status, error_text),
            }));
        }

        // Parse response
        let result: UpdateServiceDefinitionResult = response.json().await?;

        tracing::debug!(success = result.success(), "Service definition updated");

        Ok(result)
    }

    /// Deletes a hosted service.
    ///
    /// Permanently removes a hosted service and its associated item.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use arcgis::{ArcGISClient, ApiKeyAuth, PortalClient};
    /// # async fn example(portal: &PortalClient<'_>) -> arcgis::Result<()> {
    /// let result = portal.delete_service("service_item_id").await?;
    /// println!("Delete success: {}", result.success());
    /// # Ok(())
    /// # }
    /// ```
    #[instrument(skip(self, service_item_id))]
    pub async fn delete_service(
        &self,
        service_item_id: impl AsRef<str>,
    ) -> Result<DeleteServiceResult> {
        let service_item_id = service_item_id.as_ref();
        tracing::debug!(service_item_id = %service_item_id, "Deleting service");

        // Simply delegate to delete_item as services are deleted the same way
        let delete_result = self.delete_item(service_item_id).await?;

        // Convert DeleteItemResult to DeleteServiceResult
        let result =
            DeleteServiceResult::new(*delete_result.success(), delete_result.item_id().clone());

        tracing::debug!(success = result.success(), "Service deleted");

        Ok(result)
    }

    /// Overwrites an existing service with new data.
    ///
    /// Replaces the data in an existing hosted service while preserving
    /// the service URL and item ID.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use arcgis::{ArcGISClient, ApiKeyAuth, PortalClient, OverwriteParameters};
    /// # async fn example(portal: &PortalClient<'_>) -> arcgis::Result<()> {
    /// let params = OverwriteParameters::new("new_data_item_id", "existing_service_id")
    ///     .with_preserve_item_id(true);
    ///
    /// let result = portal.overwrite_service(params).await?;
    /// println!("Overwrite success: {}", result.success());
    /// # Ok(())
    /// # }
    /// ```
    #[instrument(skip(self, params))]
    pub async fn overwrite_service(&self, params: OverwriteParameters) -> Result<OverwriteResult> {
        tracing::debug!(
            source_item_id = %params.source_item_id(),
            target_service_id = %params.target_service_id(),
            "Overwriting service"
        );

        let url = format!(
            "{}/content/users/{{username}}/items/{}/update",
            self.base_url,
            params.target_service_id()
        );

        // Get authentication token and user info
        let user = self.get_self().await?;
        let username = user.effective_username().ok_or_else(|| {
            crate::Error::from(crate::ErrorKind::Api {
                code: 401,
                message: "Username not available in user info".to_string(),
            })
        })?;
        let url = url.replace("{username}", username);

        tracing::debug!(url = %url, "Sending overwriteService request");

        // Build form data
        let mut form = reqwest::multipart::Form::new()
            .text("f", "json")
            .text("sourceItemId", params.source_item_id().to_string())
            .text("overwrite", "true");

        if let Some(preserve) = params.preserve_item_id() {
            form = form.text("preserveItemId", preserve.to_string());
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
            tracing::error!(status = %status, error = %error_text, "overwriteService request failed");
            return Err(crate::Error::from(crate::ErrorKind::Api {
                code: status.as_u16() as i32,
                message: format!("HTTP {}: {}", status, error_text),
            }));
        }

        // Parse response
        let result: OverwriteResult = response.json().await?;

        tracing::debug!(success = result.success(), "Service overwritten");

        Ok(result)
    }
}
