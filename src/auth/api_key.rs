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
/// use arcgis::auth::ApiKeyAuth;
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
    /// use arcgis::auth::ApiKeyAuth;
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

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_api_key_auth() {
        let auth = ApiKeyAuth::new("test_api_key");
        let token = auth.get_token().await.unwrap();
        assert_eq!(token, "test_api_key");
    }
}
