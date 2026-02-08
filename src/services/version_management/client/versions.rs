//! Version CRUD and metadata operations for the Version Management client.

use super::super::{
    AlterResponse, AlterVersionParams, CreateVersionParams, CreateVersionResponse, DeleteResponse,
    VersionGuid, VersionInfo, VersionInfosResponse,
};
use super::VersionManagementClient;
use crate::Result;
use tracing::instrument;

impl<'a> VersionManagementClient<'a> {
    /// Creates a new version from the DEFAULT version.
    ///
    /// Creates a named version that branches from the DEFAULT version. This is the
    /// starting point for versioned editing workflows.
    ///
    /// # Arguments
    ///
    /// * `params` - Parameters for creating the version (name, access level, description)
    ///
    /// # Returns
    ///
    /// Returns a [`CreateVersionResponse`] with the newly created version's information.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - A version with the same name already exists
    /// - The user doesn't have permission to create versions
    /// - Authentication fails
    /// - Network error occurs
    ///
    /// # Example
    ///
    /// ```no_run
    /// use arcgis::{
    ///     ArcGISClient, ClientCredentialsAuth, VersionManagementClient,
    ///     CreateVersionParams, VersionPermission,
    /// };
    ///
    /// # async fn example() -> arcgis::Result<()> {
    /// # let auth = ClientCredentialsAuth::new("id".to_string(), "secret".to_string()).expect("Valid");
    /// # let client = ArcGISClient::new(auth);
    /// # let vm_client = VersionManagementClient::new("url", &client);
    /// let params = CreateVersionParams::new("workplan_2024", VersionPermission::Public)
    ///     .with_description("Work plan for 2024 projects");
    ///
    /// let response = vm_client.create(params).await?;
    ///
    /// if response.success().unwrap_or(false) {
    ///     if let Some(version_info) = response.version_info() {
    ///         println!("Created version: {}", version_info.version_name());
    ///     }
    /// }
    /// # Ok(())
    /// # }
    /// ```
    #[instrument(skip(self, params), fields(base_url = %self.base_url, version_name = %params.version_name()))]
    pub async fn create(&self, params: CreateVersionParams) -> Result<CreateVersionResponse> {
        tracing::debug!(
            version_name = %params.version_name(),
            access = %params.access(),
            "Creating new version"
        );

        let url = format!("{}/create", self.base_url);

        let access_str = params.access().to_string();
        let mut form = vec![
            ("versionName", params.version_name().as_str()),
            ("access", access_str.as_str()),
            ("f", "json"),
        ];

        if let Some(description) = params.description() {
            form.push(("description", description.as_str()));
        }

        // Add token if required by auth provider
        let token_opt = self.client.get_token_if_required().await?;
        let token_str;
        if let Some(token) = token_opt {
            token_str = token;
            form.push(("token", token_str.as_str()));
        }
        tracing::debug!(url = %url, "Sending create request");

        let response = self.client.http().post(&url).form(&form).send().await?;

        let status = response.status();
        if !status.is_success() {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|e| format!("Failed to read error: {}", e));
            tracing::error!(status = %status, error = %error_text, "create failed");
            return Err(crate::Error::from(crate::ErrorKind::Api {
                code: status.as_u16() as i32,
                message: format!("HTTP {}: {}", status, error_text),
            }));
        }

        // Get raw response text for debugging
        let response_text = response.text().await?;
        tracing::debug!(response = %response_text, "Raw create version response");

        // Try to deserialize
        let create_response: CreateVersionResponse = serde_json::from_str(&response_text)?;

        if create_response.success().is_some_and(|s| s) {
            tracing::info!(
                version_name = %params.version_name(),
                "Version created successfully"
            );
        } else {
            tracing::warn!(
                version_name = %params.version_name(),
                error = ?create_response.error(),
                "create reported failure"
            );
        }

        Ok(create_response)
    }

    /// Alters an existing version's properties.
    ///
    /// Allows changing a version's name, description, or access permission level.
    ///
    /// # Arguments
    ///
    /// * `version_guid` - The GUID of the version to alter
    /// * `params` - Parameters specifying which properties to change
    ///
    /// # Returns
    ///
    /// Returns an [`AlterResponse`] indicating success or failure.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The version doesn't exist
    /// - The user doesn't have permission to alter the version
    /// - Authentication fails
    /// - Network error occurs
    ///
    /// # Example
    ///
    /// ```no_run
    /// use arcgis::{
    ///     ArcGISClient, ClientCredentialsAuth, VersionManagementClient,
    ///     AlterVersionParams, VersionPermission,
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
    /// let params = AlterVersionParams::new()
    ///     .with_access(VersionPermission::Protected)
    ///     .with_description("Updated description");
    ///
    /// let response = vm_client.alter(version_guid.into(), params).await?;
    ///
    /// if *response.success() {
    ///     println!("Version altered successfully");
    /// }
    /// # Ok(())
    /// # }
    /// ```
    #[instrument(skip(self, params), fields(base_url = %self.base_url, version_guid = %version_guid))]
    pub async fn alter(
        &self,
        version_guid: VersionGuid,
        params: AlterVersionParams,
    ) -> Result<AlterResponse> {
        tracing::debug!(version_guid = %version_guid, "Altering version");

        let url = format!("{}/versions/{}/alter", self.base_url, version_guid);

        let mut form = vec![("f", "json")];

        let version_name_str;
        if let Some(version_name) = params.version_name() {
            version_name_str = version_name.clone();
            form.push(("versionName", version_name_str.as_str()));
        }

        // Add token if required by auth provider
        let token_opt = self.client.get_token_if_required().await?;
        let token_str;
        if let Some(token) = token_opt {
            token_str = token;
            form.push(("token", token_str.as_str()));
        }

        let access_str;
        if let Some(access) = params.access() {
            access_str = access.to_string();
            form.push(("access", access_str.as_str()));
        }

        let description_str;
        if let Some(description) = params.description() {
            description_str = description.clone();
            form.push(("description", description_str.as_str()));
        }

        tracing::debug!(url = %url, "Sending alter request");

        let response = self.client.http().post(&url).form(&form).send().await?;

        let status = response.status();
        if !status.is_success() {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|e| format!("Failed to read error: {}", e));
            tracing::error!(status = %status, error = %error_text, "alter failed");
            return Err(crate::Error::from(crate::ErrorKind::Api {
                code: status.as_u16() as i32,
                message: format!("HTTP {}: {}", status, error_text),
            }));
        }

        let alter_response: AlterResponse = response.json().await?;

        if *alter_response.success() {
            tracing::info!(
                version_guid = %version_guid,
                moment = ?alter_response.moment(),
                "Version altered successfully"
            );
        } else {
            tracing::warn!(
                version_guid = %version_guid,
                error = ?alter_response.error(),
                "alter reported failure"
            );
        }

        Ok(alter_response)
    }

    /// Deletes a version.
    ///
    /// Permanently removes a version from the geodatabase. This operation cannot be undone.
    ///
    /// **Warning**: This operation is irreversible. All edits in the version that have
    /// not been posted to the parent version will be lost.
    ///
    /// # Arguments
    ///
    /// * `version_guid` - The GUID of the version to delete
    ///
    /// # Returns
    ///
    /// Returns a [`DeleteResponse`] indicating success or failure.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The version doesn't exist
    /// - The version is the DEFAULT version (cannot be deleted)
    /// - The user doesn't have permission to delete the version
    /// - Authentication fails
    /// - Network error occurs
    ///
    /// # Example
    ///
    /// ```no_run
    /// use arcgis::{ArcGISClient, ClientCredentialsAuth, VersionManagementClient};
    /// use uuid::Uuid;
    ///
    /// # async fn example() -> arcgis::Result<()> {
    /// # let auth = ClientCredentialsAuth::new("id".to_string(), "secret".to_string()).expect("Valid");
    /// # let client = ArcGISClient::new(auth);
    /// # let vm_client = VersionManagementClient::new("url", &client);
    /// let version_guid = Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000")
    ///     .expect("Valid UUID");
    ///
    /// let response = vm_client.delete(version_guid.into()).await?;
    ///
    /// if *response.success() {
    ///     println!("Version deleted successfully");
    /// } else {
    ///     eprintln!("Failed to delete version: {:?}", response.error());
    /// }
    /// # Ok(())
    /// # }
    /// ```
    #[instrument(skip(self), fields(base_url = %self.base_url, version_guid = %version_guid))]
    pub async fn delete(&self, version_guid: VersionGuid) -> Result<DeleteResponse> {
        tracing::debug!(version_guid = %version_guid, "Deleting version");

        let url = format!("{}/versions/{}/delete", self.base_url, version_guid);

        tracing::debug!(url = %url, "Sending delete request");

        let mut form = vec![];

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
            tracing::error!(status = %status, error = %error_text, "delete failed");
            return Err(crate::Error::from(crate::ErrorKind::Api {
                code: status.as_u16() as i32,
                message: format!("HTTP {}: {}", status, error_text),
            }));
        }

        let delete_response: DeleteResponse = response.json().await?;

        if *delete_response.success() {
            tracing::info!(
                version_guid = %version_guid,
                moment = ?delete_response.moment(),
                "Version deleted successfully"
            );
        } else {
            tracing::warn!(
                version_guid = %version_guid,
                error = ?delete_response.error(),
                "delete reported failure"
            );
        }

        Ok(delete_response)
    }

    /// Gets information about a specific version.
    ///
    /// Retrieves metadata for a version including its name, description, access level,
    /// and creation/modification timestamps.
    ///
    /// # Arguments
    ///
    /// * `version_guid` - The GUID of the version to query
    ///
    /// # Returns
    ///
    /// Returns a [`VersionInfo`] struct with the version's metadata.
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
    /// use arcgis::{ArcGISClient, ClientCredentialsAuth, VersionManagementClient};
    /// use uuid::Uuid;
    ///
    /// # async fn example() -> arcgis::Result<()> {
    /// # let auth = ClientCredentialsAuth::new("id".to_string(), "secret".to_string()).expect("Valid");
    /// # let client = ArcGISClient::new(auth);
    /// # let vm_client = VersionManagementClient::new("url", &client);
    /// let version_guid = Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000")
    ///     .expect("Valid UUID");
    ///
    /// let version_info = vm_client.get_info(version_guid.into()).await?;
    ///
    /// println!("Version: {}", version_info.version_name());
    /// println!("Description: {:?}", version_info.description());
    /// # Ok(())
    /// # }
    /// ```
    #[instrument(skip(self), fields(base_url = %self.base_url, version_guid = %version_guid))]
    pub async fn get_info(&self, version_guid: VersionGuid) -> Result<VersionInfo> {
        tracing::debug!(version_guid = %version_guid, "Getting version info");

        let url = format!("{}/versions/{}", self.base_url, version_guid);

        tracing::debug!(url = %url, "Sending get version info request");

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
                .unwrap_or_else(|e| format!("Failed to read error: {}", e));
            tracing::error!(status = %status, error = %error_text, "get version info failed");
            return Err(crate::Error::from(crate::ErrorKind::Api {
                code: status.as_u16() as i32,
                message: format!("HTTP {}: {}", status, error_text),
            }));
        }

        let version_info: VersionInfo = response.json().await?;

        tracing::info!(
            version_guid = %version_guid,
            version_name = version_info.version_name(),
            "Version info retrieved"
        );

        Ok(version_info)
    }

    /// Lists all versions available in the Version Management Service.
    ///
    /// Returns a list of all versions that the authenticated user has access to view.
    ///
    /// # Returns
    ///
    /// Returns a [`VersionInfosResponse`] containing a vector of [`VersionInfo`] structs.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Authentication fails
    /// - Network error occurs
    ///
    /// # Example
    ///
    /// ```no_run
    /// use arcgis::{ArcGISClient, ClientCredentialsAuth, VersionManagementClient};
    ///
    /// # async fn example() -> arcgis::Result<()> {
    /// # let auth = ClientCredentialsAuth::new("id".to_string(), "secret".to_string()).expect("Valid");
    /// # let client = ArcGISClient::new(auth);
    /// # let vm_client = VersionManagementClient::new("url", &client);
    /// let response = vm_client.list_versions().await?;
    ///
    /// println!("Found {} versions", response.versions().len());
    ///
    /// for version in response.versions() {
    ///     println!("  - {}: {:?}", version.version_name(), version.description());
    /// }
    /// # Ok(())
    /// # }
    /// ```
    #[instrument(skip(self), fields(base_url = %self.base_url))]
    pub async fn list_versions(&self) -> Result<VersionInfosResponse> {
        tracing::debug!("Listing all versions");

        let url = format!("{}/versionInfos", self.base_url);

        tracing::debug!(url = %url, "Sending list versions request");

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
                .unwrap_or_else(|e| format!("Failed to read error: {}", e));
            tracing::error!(status = %status, error = %error_text, "list versions failed");
            return Err(crate::Error::from(crate::ErrorKind::Api {
                code: status.as_u16() as i32,
                message: format!("HTTP {}: {}", status, error_text),
            }));
        }

        let versions_response: VersionInfosResponse = response.json().await?;

        tracing::info!(
            version_count = versions_response.versions().len(),
            "Versions list retrieved"
        );

        Ok(versions_response)
    }
}
