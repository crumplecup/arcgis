//! API Key authentication provider.

use crate::{AuthProvider, Result};
use async_trait::async_trait;
use secrecy::{ExposeSecret, SecretString};
use tracing::instrument;

/// API key tier for privilege-separated authentication.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ApiKeyTier {
    /// Content management operations (create/publish/share items).
    Content,
    /// Feature editing operations.
    Features,
    /// Location services (geocoding, routing, geometry operations).
    Location,
    /// Public services (read-only operations).
    Public,
}

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
    /// This method automatically loads `.env` file and loads the specified tier's API key.
    /// Falls back to `ARCGIS_API_KEY` (legacy skeleton key) if the tier-specific key is not set.
    ///
    /// # Arguments
    ///
    /// * `tier` - Which API key tier to load (Content, Features, Location, or Public)
    ///
    /// # Key Tiers
    ///
    /// - `ApiKeyTier::Content` → `ARCGIS_CONTENT_KEY` (content management, publishing, sharing)
    /// - `ApiKeyTier::Features` → `ARCGIS_FEATURES_KEY` (feature editing)
    /// - `ApiKeyTier::Location` → `ARCGIS_LOCATION_KEY` (geocoding, routing, geometry)
    /// - `ApiKeyTier::Public` → `ARCGIS_PUBLIC_KEY` (public read-only services)
    ///
    /// # Errors
    ///
    /// Returns an error if neither the tier-specific key nor `ARCGIS_API_KEY` is found.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use arcgis::{ApiKeyAuth, ApiKeyTier};
    ///
    /// // Load content management key for portal operations
    /// let auth = ApiKeyAuth::from_env(ApiKeyTier::Content)?;
    /// # Ok::<(), arcgis::Error>(())
    /// ```
    #[instrument]
    pub fn from_env(tier: ApiKeyTier) -> Result<Self> {
        tracing::debug!("Loading API key from environment: {:?}", tier);

        // Get global configuration (automatically loads .env on first access)
        let config = crate::EnvConfig::global();

        // Match on the tier to get the correct key
        let (api_key, key_name) = match tier {
            ApiKeyTier::Content => (config.arcgis_content_key.as_ref(), "ARCGIS_CONTENT_KEY"),
            ApiKeyTier::Features => (config.arcgis_features_key.as_ref(), "ARCGIS_FEATURES_KEY"),
            ApiKeyTier::Location => (config.arcgis_location_key.as_ref(), "ARCGIS_LOCATION_KEY"),
            ApiKeyTier::Public => (config.arcgis_public_key.as_ref(), "ARCGIS_PUBLIC_KEY"),
        };

        // Fall back to legacy ARCGIS_API_KEY if tier-specific key not set
        let api_key = api_key.or(config.arcgis_api_key.as_ref()).ok_or_else(|| {
            tracing::error!(
                "No API key found in environment. Set {} or ARCGIS_API_KEY",
                key_name
            );
            crate::Error::from(crate::ErrorKind::Env(crate::EnvError::new(
                std::env::VarError::NotPresent,
            )))
        })?;

        tracing::debug!("Successfully loaded API key from environment: {}", key_name);
        Ok(Self::new(api_key.expose_secret().to_string()))
    }

    /// Creates API Key authentication for ArcGIS Online with the specified tier.
    ///
    /// This is a convenience method that delegates to [`ApiKeyAuth::from_env`].
    /// Automatically loads `.env` file and reads the tier-specific API key.
    ///
    /// # Arguments
    ///
    /// * `tier` - Which API key tier to use (Content, Features, Location, or Public)
    ///
    /// # Errors
    ///
    /// Returns an error if the tier-specific key is not found in environment.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use arcgis::{ApiKeyAuth, ApiKeyTier};
    ///
    /// // Load content management key for ArcGIS Online
    /// let auth = ApiKeyAuth::agol(ApiKeyTier::Content)?;
    /// # Ok::<(), arcgis::Error>(())
    /// ```
    #[instrument]
    pub fn agol(tier: ApiKeyTier) -> Result<Self> {
        Self::from_env(tier)
    }

    /// Creates API Key authentication for ArcGIS Enterprise.
    ///
    /// Reads `ARCGIS_ENTERPRISE_KEY` from environment variables.
    /// Automatically loads `.env` file on first access.
    ///
    /// # Environment Variables
    ///
    /// - `ARCGIS_ENTERPRISE_KEY` - API key for Enterprise portal operations
    ///
    /// # Errors
    ///
    /// Returns an error if `ARCGIS_ENTERPRISE_KEY` is not set.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use arcgis::ApiKeyAuth;
    ///
    /// // Load Enterprise API key
    /// let auth = ApiKeyAuth::enterprise()?;
    /// # Ok::<(), arcgis::Error>(())
    /// ```
    #[instrument]
    pub fn enterprise() -> Result<Self> {
        tracing::debug!("Loading Enterprise API key from environment");

        let config = crate::EnvConfig::global();
        let api_key = config.arcgis_enterprise_key.as_ref().ok_or_else(|| {
            tracing::error!("ARCGIS_ENTERPRISE_KEY not found in environment");
            crate::Error::from(crate::ErrorKind::Env(crate::EnvError::new(
                std::env::VarError::NotPresent,
            )))
        })?;

        tracing::debug!("Successfully loaded Enterprise API key");
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
