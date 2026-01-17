//! User operations for the Portal client.

use super::super::UserInfo;
use super::PortalClient;
use crate::Result;
use tracing::instrument;

impl<'a> PortalClient<'a> {
    /// Gets information about the currently authenticated user.
    ///
    /// Calls the `/community/self` endpoint to retrieve user properties,
    /// including username, role, privileges, groups, and storage quota.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use arcgis::{ArcGISClient, ApiKeyAuth, PortalClient};
    /// # async fn example(portal: &PortalClient<'_>) -> arcgis::Result<()> {
    /// let user = portal.get_self().await?;
    /// println!("Username: {}", user.username());
    /// println!("Role: {:?}", user.role());
    /// println!("Groups: {}", user.groups().len());
    /// # Ok(())
    /// # }
    /// ```
    #[instrument(skip(self))]
    pub async fn get_self(&self) -> Result<UserInfo> {
        tracing::debug!("Getting current user info");

        let url = format!("{}/community/self", self.base_url);

        tracing::debug!(url = %url, "Sending getSelf request");

        // Build request with query parameters
        let mut request = self.client.http().get(&url).query(&[("f", "json")]);

        // Add token if required by auth provider
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
            tracing::error!(status = %status, error = %error_text, "getSelf request failed");
            return Err(crate::Error::from(crate::ErrorKind::Api {
                code: status.as_u16() as i32,
                message: format!("HTTP {}: {}", status, error_text),
            }));
        }

        // Parse response
        let user: UserInfo = response.json().await?;

        tracing::debug!(username = %user.username(), "Got user info");

        Ok(user)
    }
}
