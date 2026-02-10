//! Core HTTP client for ArcGIS services.

use crate::AuthProvider;
use derive_getters::Getters;
use reqwest::Client as ReqwestClient;
use std::sync::Arc;
use tracing::instrument;

/// The main client for interacting with ArcGIS services.
///
/// This client handles HTTP communication, authentication, and common
/// request/response processing for all ArcGIS services.
///
/// # Example
///
/// ```no_run
/// use arcgis::{ApiKeyAuth, ArcGISClient};
///
/// let auth = ApiKeyAuth::new("YOUR_API_KEY");
/// let client = ArcGISClient::new(auth);
/// ```
#[derive(Getters)]
pub struct ArcGISClient {
    /// HTTP client for making requests.
    http: ReqwestClient,
    /// Authentication provider.
    auth: Arc<dyn AuthProvider>,
}

impl ArcGISClient {
    /// Creates a new ArcGIS client with the given authentication provider.
    ///
    /// # Using Environment Variables
    ///
    /// The SDK automatically loads `.env` files when using auth helpers.
    /// Store your credentials in a `.env` file (add to `.gitignore`):
    ///
    /// ```text
    /// # ArcGIS Online keys (tier-separated)
    /// ARCGIS_CONTENT_KEY=your_content_key
    /// ARCGIS_FEATURES_KEY=your_features_key
    /// ARCGIS_LOCATION_KEY=your_location_key
    ///
    /// # ArcGIS Enterprise
    /// ARCGIS_ENTERPRISE_KEY=your_enterprise_key
    /// ARCGIS_ENTERPRISE_PORTAL=https://your-server.com/portal/sharing/rest
    ///
    /// # OAuth (optional)
    /// ARCGIS_CLIENT_ID=your_client_id
    /// ARCGIS_CLIENT_SECRET=your_client_secret
    /// ```
    ///
    /// Then use the `agol()` or `enterprise()` helpers - no manual `dotenvy::dotenv()` call needed:
    ///
    /// ```no_run
    /// use arcgis::{ApiKeyAuth, ApiKeyTier, ArcGISClient};
    ///
    /// # fn example() -> arcgis::Result<()> {
    /// // ArcGIS Online (automatically loads .env and reads ARCGIS_CONTENT_KEY)
    /// let auth = ApiKeyAuth::agol(ApiKeyTier::Content)?;
    /// let client = ArcGISClient::new(auth);
    ///
    /// // ArcGIS Enterprise (reads ARCGIS_ENTERPRISE_KEY)
    /// let auth = ApiKeyAuth::enterprise()?;
    /// let client = ArcGISClient::new(auth);
    /// # Ok(())
    /// # }
    /// ```
    #[instrument(skip(auth))]
    pub fn new(auth: impl AuthProvider + 'static) -> Self {
        tracing::debug!("Creating new ArcGIS client");
        Self {
            http: ReqwestClient::new(),
            auth: Arc::new(auth),
        }
    }

    /// Gets authentication token if required by the provider.
    ///
    /// Returns `Some(token)` if the auth provider requires token parameters
    /// (e.g., ApiKeyAuth, ClientCredentials), or `None` for providers that
    /// don't require tokens (e.g., NoAuth for public services).
    ///
    /// # Example
    ///
    /// ```no_run
    /// use arcgis::{ArcGISClient, NoAuth};
    ///
    /// # async fn example() -> arcgis::Result<()> {
    /// let client = ArcGISClient::new(NoAuth);
    /// let token = client.get_token_if_required().await?;
    /// assert!(token.is_none()); // NoAuth returns None
    /// # Ok(())
    /// # }
    /// ```
    #[instrument(skip(self))]
    pub async fn get_token_if_required(&self) -> crate::Result<Option<String>> {
        if self.auth.requires_token_param() {
            tracing::debug!("Auth provider requires token, retrieving");
            Ok(Some(self.auth.get_token().await?))
        } else {
            tracing::debug!("Auth provider does not require token");
            Ok(None)
        }
    }
}

// TODO: Add request/response helpers
// TODO: Add retry logic
// TODO: Add rate limiting
