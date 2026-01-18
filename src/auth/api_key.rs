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
    /// This method automatically loads `.env` file and reads the `ARCGIS_API_KEY`
    /// environment variable.
    ///
    /// # Errors
    ///
    /// Returns an error if the `ARCGIS_API_KEY` environment variable is not set.
    /// The error preserves the original `std::env::VarError` in the error chain.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use arcgis::ApiKeyAuth;
    ///
    /// // Reads ARCGIS_API_KEY from .env file
    /// let auth = ApiKeyAuth::from_env()?;
    /// # Ok::<(), arcgis::Error>(())
    /// ```
    #[instrument]
    pub fn from_env() -> Result<Self> {
        tracing::debug!("Loading API key from environment");

        // Load .env file (ignoring errors if it doesn't exist)
        let _ = dotenvy::dotenv();

        // Read the API key - error chain: VarError → EnvError → ErrorKind → Error
        let api_key = match std::env::var("ARCGIS_API_KEY") {
            Ok(key) => {
                tracing::debug!("Successfully loaded API key from environment");
                key
            }
            Err(e) => {
                tracing::error!(
                    error = %e,
                    "ARCGIS_API_KEY environment variable not set or invalid"
                );
                return Err(e.into()); // Automatic conversion through error chain
            }
        };

        Ok(Self::new(api_key))
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
