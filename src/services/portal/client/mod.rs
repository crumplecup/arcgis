//! Portal client for ArcGIS Online and Portal for ArcGIS operations.

mod groups;
mod items;
mod publishing;
mod search;
mod sharing;
mod users;

use crate::ArcGISClient;
use tracing::instrument;

/// Client for interacting with ArcGIS Portal (ArcGIS Online or Portal for ArcGIS).
///
/// Provides access to user management, content search, item operations,
/// and other portal-specific functionality.
///
/// # Example
///
/// ```no_run
/// use arcgis::{ArcGISClient, ApiKeyAuth, PortalClient, SearchParameters};
///
/// # async fn example() -> arcgis::Result<()> {
/// let auth = ApiKeyAuth::new("YOUR_API_KEY");
/// let client = ArcGISClient::new(auth);
/// let portal = PortalClient::new("https://www.arcgis.com/sharing/rest", &client);
///
/// // Get current user info
/// let user = portal.get_self().await?;
/// if let Some(name) = user.effective_username() {
///     println!("Logged in as: {}", name);
/// }
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
    pub(super) base_url: String,
    /// Reference to the ArcGIS client for authentication and HTTP.
    pub(super) client: &'a ArcGISClient,
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
    /// use arcgis::{ArcGISClient, ApiKeyAuth, PortalClient, SearchParameters};
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
}
