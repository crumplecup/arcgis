//! Portal client for ArcGIS Online and Portal for ArcGIS operations.

use crate::{ArcGISClient, Result};
use super::{ItemInfo, SearchParameters, SearchResult, SortOrder, UserInfo};
use serde::Serialize;
use tracing::instrument;

/// Client for interacting with ArcGIS Portal (ArcGIS Online or Portal for ArcGIS).
///
/// Provides access to user management, content search, item operations,
/// and other portal-specific functionality.
///
/// # Example
///
/// ```no_run
/// use arcgis::{ArcGISClient, ApiKeyAuth, PortalClient};
///
/// # async fn example() -> arcgis::Result<()> {
/// let auth = ApiKeyAuth::new("YOUR_API_KEY");
/// let client = ArcGISClient::new(auth);
/// let portal = PortalClient::new("https://www.arcgis.com/sharing/rest", &client);
///
/// // Get current user info
/// let user = portal.get_self().await?;
/// println!("Logged in as: {}", user.username());
///
/// // Search for items
/// let results = portal
///     .search(SearchParameters::new("type:\"Feature Service\""))
///     .await?;
/// println!("Found {} items", results.total());
/// # Ok(())
/// # }
/// ```
#[derive(Clone)]
pub struct PortalClient<'a> {
    /// Base URL of the portal (e.g., "https://www.arcgis.com/sharing/rest").
    base_url: String,
    /// Reference to the ArcGIS client for authentication and HTTP.
    client: &'a ArcGISClient,
}

impl<'a> PortalClient<'a> {
    /// Creates a new PortalClient.
    ///
    /// # Arguments
    ///
    /// * `base_url` - Portal sharing REST API URL (e.g., "https://www.arcgis.com/sharing/rest")
    /// * `client` - Reference to ArcGIS client for authentication
    ///
    /// # Example
    ///
    /// ```no_run
    /// use arcgis::{ArcGISClient, ApiKeyAuth, PortalClient};
    ///
    /// # fn example() {
    /// let auth = ApiKeyAuth::new("YOUR_API_KEY");
    /// let client = ArcGISClient::new(auth);
    ///
    /// // ArcGIS Online
    /// let portal = PortalClient::new("https://www.arcgis.com/sharing/rest", &client);
    ///
    /// // Portal for ArcGIS (on-premises)
    /// let portal = PortalClient::new("https://myportal.example.com/sharing/rest", &client);
    /// # }
    /// ```
    #[instrument(skip(base_url, client))]
    pub fn new(base_url: impl Into<String>, client: &'a ArcGISClient) -> Self {
        let base_url = base_url.into();
        tracing::debug!(base_url = %base_url, "Creating PortalClient");
        Self { base_url, client }
    }

    /// Returns the base URL of the portal.
    pub fn base_url(&self) -> &str {
        &self.base_url
    }

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

        // Get authentication token
        let token = self.client.auth().get_token().await?;

        tracing::debug!(url = %url, "Sending getSelf request");

        // Build request
        let response = self
            .client
            .http()
            .get(&url)
            .query(&[("f", "json"), ("token", &token)])
            .send()
            .await?;

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

    /// Searches for portal items.
    ///
    /// Supports Lucene query syntax for flexible item discovery.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use arcgis::{ArcGISClient, ApiKeyAuth, PortalClient, SearchParameters};
    /// # async fn example(portal: &PortalClient<'_>) -> arcgis::Result<()> {
    /// // Search for feature services
    /// let results = portal
    ///     .search(SearchParameters::new("type:\"Feature Service\""))
    ///     .await?;
    ///
    /// for item in results.results() {
    ///     println!("{}: {}", item.title(), item.id());
    /// }
    ///
    /// // Search with filters
    /// let results = portal
    ///     .search(
    ///         SearchParameters::new("tags:transportation")
    ///             .with_pagination(1, 20)
    ///             .with_sort("modified", SortOrder::Desc)
    ///     )
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    #[instrument(skip(self))]
    pub async fn search(&self, params: SearchParameters) -> Result<SearchResult> {
        tracing::debug!(query = %params.query(), "Searching portal items");

        let url = format!("{}/search", self.base_url);

        // Get authentication token
        let token = self.client.auth().get_token().await?;

        tracing::debug!(url = %url, "Sending search request");

        // Build query parameters
        #[derive(Serialize)]
        struct SearchQuery<'a> {
            q: &'a str,
            #[serde(skip_serializing_if = "Option::is_none")]
            bbox: Option<String>,
            #[serde(skip_serializing_if = "Option::is_none")]
            categories: Option<&'a str>,
            #[serde(rename = "sortField", skip_serializing_if = "Option::is_none")]
            sort_field: Option<&'a str>,
            #[serde(rename = "sortOrder", skip_serializing_if = "Option::is_none")]
            sort_order: Option<&'static str>,
            #[serde(skip_serializing_if = "Option::is_none")]
            start: Option<u32>,
            #[serde(skip_serializing_if = "Option::is_none")]
            num: Option<u32>,
            f: &'static str,
            token: &'a str,
        }

        let query = SearchQuery {
            q: params.query(),
            bbox: params.bbox().as_ref().map(|b| {
                b.iter()
                    .map(|v| v.to_string())
                    .collect::<Vec<_>>()
                    .join(",")
            }),
            categories: params.categories().as_deref(),
            sort_field: params.sort_field().as_deref(),
            sort_order: params.sort_order().map(|o| match o {
                SortOrder::Asc => "asc",
                SortOrder::Desc => "desc",
            }),
            start: *params.start(),
            num: *params.num(),
            f: "json",
            token: &token,
        };

        // Build request
        let response = self
            .client
            .http()
            .get(&url)
            .query(&query)
            .send()
            .await?;

        // Check for HTTP errors
        let status = response.status();
        if !status.is_success() {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|e| format!("Failed to read error response: {}", e));
            tracing::error!(status = %status, error = %error_text, "Search request failed");
            return Err(crate::Error::from(crate::ErrorKind::Api {
                code: status.as_u16() as i32,
                message: format!("HTTP {}: {}", status, error_text),
            }));
        }

        // Parse response
        let result: SearchResult = response.json().await?;

        tracing::debug!(
            total = result.total(),
            count = result.results().len(),
            "Search completed"
        );

        Ok(result)
    }

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
        let token = self.client.auth().get_token().await?;

        tracing::debug!(url = %url, "Sending getItem request");

        // Build request
        let response = self
            .client
            .http()
            .get(&url)
            .query(&[("f", "json"), ("token", &token)])
            .send()
            .await?;

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
}
