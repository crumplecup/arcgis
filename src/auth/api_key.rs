//! API Key authentication provider.

use crate::{AuthProvider, Result};
use async_trait::async_trait;
use secrecy::{ExposeSecret, SecretString};
use tracing::instrument;

/// API Key authentication provider.
///
/// This is the simplest authentication method for ArcGIS services.
/// API keys can be generated in the ArcGIS Developer dashboard.
///
/// # Example
///
/// ```no_run
/// use arcgis::ApiKeyAuth;
///
/// let auth = ApiKeyAuth::new("YOUR_API_KEY");
/// ```
pub struct ApiKeyAuth {
    api_key: SecretString,
}

impl ApiKeyAuth {
    /// Creates a new API Key authentication provider.
    ///
    /// # Arguments
    ///
    /// * `api_key` - Your ArcGIS API key
    ///
    /// # Example
    ///
    /// ```no_run
    /// use arcgis::ApiKeyAuth;
    ///
    /// let auth = ApiKeyAuth::new("YOUR_API_KEY");
    /// ```
    #[instrument(skip(api_key))]
    pub fn new(api_key: impl Into<String>) -> Self {
        tracing::debug!("Creating API Key authentication provider");
        Self {
            api_key: SecretString::new(api_key.into().into_boxed_str()),
        }
    }

    /// Creates a new API Key authentication provider from environment variables.
    ///
    /// This method automatically loads `.env` file and intelligently searches for API keys:
    /// 1. First checks tier-specific keys (privilege separation):
    ///    - `ARCGIS_LOCATION_KEY` - Location services
    ///    - `ARCGIS_CONTENT_KEY` - Content management
    ///    - `ARCGIS_FEATURES_KEY` - Feature editing
    ///    - `ARCGIS_PUBLIC_KEY` - Public services
    /// 2. Falls back to `ARCGIS_API_KEY` (skeleton key with all privileges)
    ///
    /// This allows examples and user code to work seamlessly with the multi-tier system.
    /// Users can provide tier-specific keys for privilege separation, or a skeleton key
    /// for simplicity.
    ///
    /// # Errors
    ///
    /// Returns an error if no API key environment variable is found.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use arcgis::ApiKeyAuth;
    ///
    /// // Automatically finds and uses any available API key
    /// let auth = ApiKeyAuth::from_env()?;
    /// # Ok::<(), arcgis::Error>(())
    /// ```
    #[instrument]
    pub fn from_env() -> Result<Self> {
        tracing::debug!("Loading API key from environment");

        // Get global configuration (automatically loads .env on first access)
        let config = crate::EnvConfig::global();

        // Try tier-specific keys first, then fall back to skeleton key
        let api_key = config
            .arcgis_location_key
            .as_ref()
            .or(config.arcgis_content_key.as_ref())
            .or(config.arcgis_features_key.as_ref())
            .or(config.arcgis_public_key.as_ref())
            .or(config.arcgis_api_key.as_ref())
            .ok_or_else(|| {
                tracing::error!(
                    "No API key found in environment. Set one of: ARCGIS_LOCATION_KEY, \
                     ARCGIS_CONTENT_KEY, ARCGIS_FEATURES_KEY, ARCGIS_PUBLIC_KEY, or ARCGIS_API_KEY"
                );
                crate::Error::from(crate::ErrorKind::Env(crate::EnvError::new(
                    std::env::VarError::NotPresent,
                )))
            })?;

        tracing::debug!("Successfully loaded API key from environment");
        Ok(Self::new(api_key.expose_secret().to_string()))
    }
}

#[async_trait]
impl AuthProvider for ApiKeyAuth {
    #[instrument(skip(self))]
    async fn get_token(&self) -> Result<String> {
        tracing::debug!("Retrieving API key token");
        Ok(self.api_key.expose_secret().to_string())
    }

    #[instrument(skip(self))]
    fn requires_token_param(&self) -> bool {
        true
    }
}
