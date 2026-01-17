//! Group operations for the Portal client.

use super::super::{
    CreateGroupParams, GroupInfo, GroupResult, GroupSearchParameters, GroupSearchResult, SortOrder,
    UpdateGroupParams,
};
use super::PortalClient;
use crate::Result;
use serde::Serialize;
use tracing::instrument;

impl<'a> PortalClient<'a> {
    /// Searches for portal groups.
    ///
    /// Supports Lucene query syntax for flexible group discovery.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use arcgis::{ArcGISClient, ApiKeyAuth, PortalClient, GroupSearchParameters};
    /// # async fn example(portal: &PortalClient<'_>) -> arcgis::Result<()> {
    /// // Search for public groups with "GIS" in the title
    /// let results = portal
    ///     .search_groups(GroupSearchParameters::new("title:GIS AND access:public"))
    ///     .await?;
    ///
    /// for group in results.results() {
    ///     println!("{}: {}", group.title(), group.id());
    /// }
    /// # Ok(())
    /// # }
    /// ```
    #[instrument(skip(self))]
    pub async fn search_groups(&self, params: GroupSearchParameters) -> Result<GroupSearchResult> {
        tracing::debug!(query = %params.query(), "Searching groups");

        let url = format!("{}/community/groups", self.base_url);

        // Get authentication token
        tracing::debug!(url = %url, "Sending searchGroups request");

        // Build query parameters
        #[derive(Serialize)]
        struct GroupSearchQuery<'a> {
            q: &'a str,
            #[serde(rename = "sortField", skip_serializing_if = "Option::is_none")]
            sort_field: Option<&'a str>,
            #[serde(rename = "sortOrder", skip_serializing_if = "Option::is_none")]
            sort_order: Option<&'static str>,
            #[serde(skip_serializing_if = "Option::is_none")]
            start: Option<u32>,
            #[serde(skip_serializing_if = "Option::is_none")]
            num: Option<u32>,
            f: &'static str,
            #[serde(skip_serializing_if = "Option::is_none")]
            token: Option<String>,
        }


        let token_opt = self.client.get_token_if_required().await?;

        let query = GroupSearchQuery {
            q: params.query(),
            sort_field: params.sort_field().as_deref(),
            sort_order: params.sort_order().map(|o| match o {
                SortOrder::Asc => "asc",
                SortOrder::Desc => "desc",
            }),
            start: *params.start(),
            num: *params.num(),
            f: "json",
            token: token_opt,
        };

        // Build request
        let response = self.client.http().get(&url).query(&query).send().await?;

        // Check for HTTP errors
        let status = response.status();
        if !status.is_success() {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|e| format!("Failed to read error response: {}", e));
            tracing::error!(status = %status, error = %error_text, "searchGroups request failed");
            return Err(crate::Error::from(crate::ErrorKind::Api {
                code: status.as_u16() as i32,
                message: format!("HTTP {}: {}", status, error_text),
            }));
        }

        // Parse response
        let result: GroupSearchResult = response.json().await?;

        tracing::debug!(
            total = result.total(),
            count = result.results().len(),
            "Group search completed"
        );

        Ok(result)
    }

    /// Gets a portal group by ID.
    ///
    /// Retrieves detailed metadata for a specific group.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use arcgis::{ArcGISClient, ApiKeyAuth, PortalClient};
    /// # async fn example(portal: &PortalClient<'_>) -> arcgis::Result<()> {
    /// let group = portal.get_group("abc123def456").await?;
    /// println!("Title: {}", group.title());
    /// println!("Owner: {}", group.owner());
    /// println!("Access: {}", group.access());
    /// # Ok(())
    /// # }
    /// ```
    #[instrument(skip(self, group_id))]
    pub async fn get_group(&self, group_id: impl AsRef<str>) -> Result<GroupInfo> {
        let group_id = group_id.as_ref();
        tracing::debug!(group_id = %group_id, "Getting group");

        let url = format!("{}/community/groups/{}", self.base_url, group_id);

        // Get authentication token
        tracing::debug!(url = %url, "Sending getGroup request");

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
            tracing::error!(status = %status, error = %error_text, "getGroup request failed");
            return Err(crate::Error::from(crate::ErrorKind::Api {
                code: status.as_u16() as i32,
                message: format!("HTTP {}: {}", status, error_text),
            }));
        }

        // Parse response
        let group: GroupInfo = response.json().await?;

        tracing::debug!(title = %group.title(), owner = %group.owner(), "Got group");

        Ok(group)
    }

    /// Creates a new portal group.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use arcgis::{ArcGISClient, ApiKeyAuth, PortalClient, CreateGroupParams};
    /// # async fn example(portal: &PortalClient<'_>) -> arcgis::Result<()> {
    /// let params = CreateGroupParams::new("My Project Group")
    ///     .with_description("Collaboration group for GIS project")
    ///     .with_tags(vec!["project".to_string(), "collaboration".to_string()])
    ///     .with_access("org".to_string());
    ///
    /// let result = portal.create_group(params).await?;
    /// println!("Created group: {:?}", result.group_id());
    /// # Ok(())
    /// # }
    /// ```
    #[instrument(skip(self, params))]
    pub async fn create_group(&self, params: CreateGroupParams) -> Result<GroupResult> {
        tracing::debug!(title = %params.title(), "Creating group");

        let url = format!("{}/community/createGroup", self.base_url);

        // Get authentication token
        tracing::debug!(url = %url, "Sending createGroup request");

        // Build form data
        let mut form = reqwest::multipart::Form::new()
            .text("f", "json")
            .text("title", params.title().to_string());

        // Add token if required by auth provider
        if let Some(token) = self.client.get_token_if_required().await? {
            form = form.text("token", token);
        }

        if let Some(desc) = params.description() {
            form = form.text("description", desc.to_string());
        }

        if let Some(snippet) = params.snippet() {
            form = form.text("snippet", snippet.to_string());
        }

        if let Some(tags) = params.tags() {
            form = form.text("tags", tags.join(","));
        }

        if let Some(access) = params.access() {
            form = form.text("access", access.to_string());
        }

        if let Some(invitation_only) = params.is_invitation_only() {
            form = form.text("isInvitationOnly", invitation_only.to_string());
        }

        if let Some(view_only) = params.is_view_only() {
            form = form.text("isViewOnly", view_only.to_string());
        }

        if let Some(auto_join) = params.auto_join() {
            form = form.text("autoJoin", auto_join.to_string());
        }

        if let Some(sort_field) = params.sort_field() {
            form = form.text("sortField", sort_field.to_string());
        }

        if let Some(sort_order) = params.sort_order() {
            form = form.text("sortOrder", sort_order.to_string());
        }

        // Build request
        let response = self.client.http().post(&url).multipart(form).send().await?;

        // Check for HTTP errors
        let status = response.status();
        if !status.is_success() {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|e| format!("Failed to read error response: {}", e));
            tracing::error!(status = %status, error = %error_text, "createGroup request failed");
            return Err(crate::Error::from(crate::ErrorKind::Api {
                code: status.as_u16() as i32,
                message: format!("HTTP {}: {}", status, error_text),
            }));
        }

        // Parse response
        let result: GroupResult = response.json().await?;

        tracing::debug!(success = result.success(), group_id = ?result.group_id(), "Group created");

        Ok(result)
    }

    /// Updates an existing group's metadata.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use arcgis::{ArcGISClient, ApiKeyAuth, PortalClient, UpdateGroupParams};
    /// # async fn example(portal: &PortalClient<'_>) -> arcgis::Result<()> {
    /// let params = UpdateGroupParams::new()
    ///     .with_title("Updated Group Title")
    ///     .with_description("Updated description");
    ///
    /// let result = portal.update_group("abc123def456", params).await?;
    /// println!("Update success: {}", result.success());
    /// # Ok(())
    /// # }
    /// ```
    #[instrument(skip(self, group_id, params))]
    pub async fn update_group(
        &self,
        group_id: impl AsRef<str>,
        params: UpdateGroupParams,
    ) -> Result<GroupResult> {
        let group_id = group_id.as_ref();
        tracing::debug!(group_id = %group_id, "Updating group");

        let url = format!("{}/community/groups/{}/update", self.base_url, group_id);

        // Get authentication token
        tracing::debug!(url = %url, "Sending updateGroup request");

        // Build form data
        let mut form = reqwest::multipart::Form::new()
            .text("f", "json");

        // Add token if required by auth provider
        if let Some(token) = self.client.get_token_if_required().await? {
            form = form.text("token", token);
        }

        if let Some(title) = params.title() {
            form = form.text("title", title.to_string());
        }

        if let Some(desc) = params.description() {
            form = form.text("description", desc.to_string());
        }

        if let Some(snippet) = params.snippet() {
            form = form.text("snippet", snippet.to_string());
        }

        if let Some(tags) = params.tags() {
            form = form.text("tags", tags.join(","));
        }

        if let Some(access) = params.access() {
            form = form.text("access", access.to_string());
        }

        if let Some(invitation_only) = params.is_invitation_only() {
            form = form.text("isInvitationOnly", invitation_only.to_string());
        }

        if let Some(view_only) = params.is_view_only() {
            form = form.text("isViewOnly", view_only.to_string());
        }

        if let Some(sort_field) = params.sort_field() {
            form = form.text("sortField", sort_field.to_string());
        }

        if let Some(sort_order) = params.sort_order() {
            form = form.text("sortOrder", sort_order.to_string());
        }

        // Build request
        let response = self.client.http().post(&url).multipart(form).send().await?;

        // Check for HTTP errors
        let status = response.status();
        if !status.is_success() {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|e| format!("Failed to read error response: {}", e));
            tracing::error!(status = %status, error = %error_text, "updateGroup request failed");
            return Err(crate::Error::from(crate::ErrorKind::Api {
                code: status.as_u16() as i32,
                message: format!("HTTP {}: {}", status, error_text),
            }));
        }

        // Parse response
        let result: GroupResult = response.json().await?;

        tracing::debug!(success = result.success(), "Group updated");

        Ok(result)
    }

    /// Deletes a portal group.
    ///
    /// Permanently removes a group from the portal.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use arcgis::{ArcGISClient, ApiKeyAuth, PortalClient};
    /// # async fn example(portal: &PortalClient<'_>) -> arcgis::Result<()> {
    /// let result = portal.delete_group("abc123def456").await?;
    /// println!("Delete success: {}", result.success());
    /// # Ok(())
    /// # }
    /// ```
    #[instrument(skip(self, group_id))]
    pub async fn delete_group(&self, group_id: impl AsRef<str>) -> Result<GroupResult> {
        let group_id = group_id.as_ref();
        tracing::debug!(group_id = %group_id, "Deleting group");

        let url = format!("{}/community/groups/{}/delete", self.base_url, group_id);

        // Get authentication token
        tracing::debug!(url = %url, "Sending deleteGroup request");

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
            tracing::error!(status = %status, error = %error_text, "deleteGroup request failed");
            return Err(crate::Error::from(crate::ErrorKind::Api {
                code: status.as_u16() as i32,
                message: format!("HTTP {}: {}", status, error_text),
            }));
        }

        // Parse response
        let result: GroupResult = response.json().await?;

        tracing::debug!(success = result.success(), "Group deleted");

        Ok(result)
    }

    /// Joins a group (adds current user as member).
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use arcgis::{ArcGISClient, ApiKeyAuth, PortalClient};
    /// # async fn example(portal: &PortalClient<'_>) -> arcgis::Result<()> {
    /// let result = portal.join_group("abc123def456").await?;
    /// println!("Joined: {}", result.success());
    /// # Ok(())
    /// # }
    /// ```
    #[instrument(skip(self, group_id))]
    pub async fn join_group(&self, group_id: impl AsRef<str>) -> Result<GroupResult> {
        let group_id = group_id.as_ref();
        tracing::debug!(group_id = %group_id, "Joining group");

        let url = format!("{}/community/groups/{}/join", self.base_url, group_id);

        // Get authentication token
        tracing::debug!(url = %url, "Sending joinGroup request");

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
            tracing::error!(status = %status, error = %error_text, "joinGroup request failed");
            return Err(crate::Error::from(crate::ErrorKind::Api {
                code: status.as_u16() as i32,
                message: format!("HTTP {}: {}", status, error_text),
            }));
        }

        // Parse response
        let result: GroupResult = response.json().await?;

        tracing::debug!(success = result.success(), "Joined group");

        Ok(result)
    }

    /// Leaves a group (removes current user from membership).
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use arcgis::{ArcGISClient, ApiKeyAuth, PortalClient};
    /// # async fn example(portal: &PortalClient<'_>) -> arcgis::Result<()> {
    /// let result = portal.leave_group("abc123def456").await?;
    /// println!("Left: {}", result.success());
    /// # Ok(())
    /// # }
    /// ```
    #[instrument(skip(self, group_id))]
    pub async fn leave_group(&self, group_id: impl AsRef<str>) -> Result<GroupResult> {
        let group_id = group_id.as_ref();
        tracing::debug!(group_id = %group_id, "Leaving group");

        let url = format!("{}/community/groups/{}/leave", self.base_url, group_id);

        // Get authentication token
        tracing::debug!(url = %url, "Sending leaveGroup request");

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
            tracing::error!(status = %status, error = %error_text, "leaveGroup request failed");
            return Err(crate::Error::from(crate::ErrorKind::Api {
                code: status.as_u16() as i32,
                message: format!("HTTP {}: {}", status, error_text),
            }));
        }

        // Parse response
        let result: GroupResult = response.json().await?;

        tracing::debug!(success = result.success(), "Left group");

        Ok(result)
    }

    /// Adds an item to a group.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use arcgis::{ArcGISClient, ApiKeyAuth, PortalClient};
    /// # async fn example(portal: &PortalClient<'_>) -> arcgis::Result<()> {
    /// let result = portal.add_to_group("group_id", "item_id").await?;
    /// println!("Added to group: {}", result.success());
    /// # Ok(())
    /// # }
    /// ```
    #[instrument(skip(self, group_id, item_id))]
    pub async fn add_to_group(
        &self,
        group_id: impl AsRef<str>,
        item_id: impl AsRef<str>,
    ) -> Result<GroupResult> {
        let group_id = group_id.as_ref();
        let item_id = item_id.as_ref();
        tracing::debug!(group_id = %group_id, item_id = %item_id, "Adding item to group");

        let url = format!("{}/content/items/{}/share", self.base_url, item_id);

        // Get authentication token
        tracing::debug!(url = %url, "Sending addToGroup request");

        // Build request
        let mut form_data = vec![("f", "json"), ("groups", group_id)];


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
            tracing::error!(status = %status, error = %error_text, "addToGroup request failed");
            return Err(crate::Error::from(crate::ErrorKind::Api {
                code: status.as_u16() as i32,
                message: format!("HTTP {}: {}", status, error_text),
            }));
        }

        // Parse response
        let result: GroupResult = response.json().await?;

        tracing::debug!(success = result.success(), "Added item to group");

        Ok(result)
    }

    /// Removes an item from a group.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use arcgis::{ArcGISClient, ApiKeyAuth, PortalClient};
    /// # async fn example(portal: &PortalClient<'_>) -> arcgis::Result<()> {
    /// let result = portal.remove_from_group("group_id", "item_id").await?;
    /// println!("Removed from group: {}", result.success());
    /// # Ok(())
    /// # }
    /// ```
    #[instrument(skip(self, group_id, item_id))]
    pub async fn remove_from_group(
        &self,
        group_id: impl AsRef<str>,
        item_id: impl AsRef<str>,
    ) -> Result<GroupResult> {
        let group_id = group_id.as_ref();
        let item_id = item_id.as_ref();
        tracing::debug!(group_id = %group_id, item_id = %item_id, "Removing item from group");

        let url = format!("{}/content/items/{}/unshare", self.base_url, item_id);

        // Get authentication token
        tracing::debug!(url = %url, "Sending removeFromGroup request");

        // Build request
        let mut form_data = vec![("f", "json"), ("groups", group_id)];


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
            tracing::error!(status = %status, error = %error_text, "removeFromGroup request failed");
            return Err(crate::Error::from(crate::ErrorKind::Api {
                code: status.as_u16() as i32,
                message: format!("HTTP {}: {}", status, error_text),
            }));
        }

        // Parse response
        let result: GroupResult = response.json().await?;

        tracing::debug!(success = result.success(), "Removed item from group");

        Ok(result)
    }
}
