//! Core HTTP client for ArcGIS services.

use crate::AuthProvider;
use derive_getters::Getters;
use reqwest::Client as ReqwestClient;
use std::sync::{Arc, OnceLock};
use tracing::instrument;

/// Initialize environment variables from .env file.
///
/// This function is called automatically the first time an ArcGIS client is created.
/// It loads environment variables from a `.env` file in the current directory or
/// any parent directory.
///
/// If no `.env` file is found, this function silently succeeds - it's optional.
/// This allows users to:
/// - Store API keys and credentials securely in `.env` (gitignored)
/// - Use system environment variables instead
/// - Deploy with different env management systems
fn init_env() {
    static INIT: OnceLock<()> = OnceLock::new();
    INIT.get_or_init(|| {
        // Silently load .env if it exists
        // Users can override with system environment variables
        dotenvy::dotenv().ok();
        tracing::debug!("Environment initialization complete");
    });
}

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
    /// This function automatically loads environment variables from a `.env` file
    /// on first use. This means you can store your API keys and credentials in a
    /// `.env` file (which should be in `.gitignore`):
    ///
    /// ```text
    /// ARCGIS_API_KEY=your_api_key_here
    /// ARCGIS_CLIENT_ID=your_client_id
    /// ARCGIS_CLIENT_SECRET=your_client_secret
    /// ```
    ///
    /// Then use them in your code:
    ///
    /// ```no_run
    /// use arcgis::{ApiKeyAuth, ArcGISClient};
    ///
    /// let api_key = std::env::var("ARCGIS_API_KEY")
    ///     .expect("ARCGIS_API_KEY must be set in .env or environment");
    /// let auth = ApiKeyAuth::new(api_key);
    /// let client = ArcGISClient::new(auth);
    /// ```
    #[instrument(skip(auth))]
    pub fn new(auth: impl AuthProvider + 'static) -> Self {
        // Initialize environment on first client creation
        init_env();

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
