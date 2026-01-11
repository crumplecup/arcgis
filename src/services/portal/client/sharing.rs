//! Sharing operations for the Portal client.

use super::super::{ShareItemResult, SharingParameters, UnshareItemResult};
use super::PortalClient;
use crate::Result;
use tracing::instrument;

impl<'a> PortalClient<'a> {
    /// Shares an item with groups, organization, or everyone (public).
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use arcgis::{ArcGISClient, ApiKeyAuth, PortalClient, SharingParameters};
    /// # async fn example(portal: &PortalClient<'_>) -> arcgis::Result<()> {
    /// // Share with organization and specific groups
    /// let params = SharingParameters::new()
    ///     .with_org(true)
    ///     .with_groups(vec!["group_id_1".to_string(), "group_id_2".to_string()]);
    ///
    /// let result = portal.share_item("abc123def456", params).await?;
    /// println!("Shared: {}", result.success());
    /// # Ok(())
    /// # }
    /// ```
    #[instrument(skip(self, item_id, params))]
    pub async fn share_item(
        &self,
        item_id: impl AsRef<str>,
        params: SharingParameters,
    ) -> Result<ShareItemResult> {
        let item_id = item_id.as_ref();
        tracing::debug!(item_id = %item_id, "Sharing item");

        // Get authentication token
        let token = self.client.auth().get_token().await?;

        // Get the item to find its owner
        let item = self.get_item(item_id).await?;
        let url = format!(
            "{}/content/users/{}/items/{}/share",
            self.base_url,
            item.owner(),
            item_id
        );

        tracing::debug!(url = %url, owner = %item.owner(), "Sending shareItem request");

        // Build form data
        let mut form = reqwest::multipart::Form::new()
            .text("f", "json")
            .text("token", token.clone());

        if let Some(everyone) = params.everyone() {
            form = form.text("everyone", everyone.to_string());
        }

        if let Some(org) = params.org() {
            form = form.text("org", org.to_string());
        }

        if let Some(groups) = params.groups() {
            form = form.text("groups", groups.join(","));
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
            tracing::error!(status = %status, error = %error_text, "shareItem request failed");
            return Err(crate::Error::from(crate::ErrorKind::Api {
                code: status.as_u16() as i32,
                message: format!("HTTP {}: {}", status, error_text),
            }));
        }

        // Parse response
        let result: ShareItemResult = response.json().await?;

        tracing::debug!(success = result.success(), "Item shared");

        Ok(result)
    }

    /// Unshares an item (removes sharing with groups, org, or public).
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use arcgis::{ArcGISClient, ApiKeyAuth, PortalClient, SharingParameters};
    /// # async fn example(portal: &PortalClient<'_>) -> arcgis::Result<()> {
    /// // Remove organization sharing and specific groups
    /// let params = SharingParameters::new()
    ///     .with_org(false)
    ///     .with_groups(vec!["group_id_1".to_string()]);
    ///
    /// let result = portal.unshare_item("abc123def456", params).await?;
    /// println!("Unshared: {}", result.success());
    /// # Ok(())
    /// # }
    /// ```
    #[instrument(skip(self, item_id, params))]
    pub async fn unshare_item(
        &self,
        item_id: impl AsRef<str>,
        params: SharingParameters,
    ) -> Result<UnshareItemResult> {
        let item_id = item_id.as_ref();
        tracing::debug!(item_id = %item_id, "Unsharing item");

        // Get authentication token
        let token = self.client.auth().get_token().await?;

        // Get the item to find its owner
        let item = self.get_item(item_id).await?;
        let url = format!(
            "{}/content/users/{}/items/{}/unshare",
            self.base_url,
            item.owner(),
            item_id
        );

        tracing::debug!(url = %url, owner = %item.owner(), "Sending unshareItem request");

        // Build form data
        let mut form = reqwest::multipart::Form::new()
            .text("f", "json")
            .text("token", token.clone());

        if let Some(groups) = params.groups() {
            form = form.text("groups", groups.join(","));
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
            tracing::error!(status = %status, error = %error_text, "unshareItem request failed");
            return Err(crate::Error::from(crate::ErrorKind::Api {
                code: status.as_u16() as i32,
                message: format!("HTTP {}: {}", status, error_text),
            }));
        }

        // Parse response
        let result: UnshareItemResult = response.json().await?;

        tracing::debug!(success = result.success(), "Item unshared");

        Ok(result)
    }
}
