//! Centralized environment configuration.
//!
//! This module provides a single point of access for all environment variables
//! used by the SDK. The configuration is loaded once on first access and cached
//! for the lifetime of the application.
//!
//! # Environment Variables
//!
//! - `ARCGIS_API_KEY` - General-purpose API key for basic services
//! - `ARCGIS_CONTENT_KEY` - API key with content management privileges
//! - `ARCGIS_FEATURES_KEY` - API key with feature editing privileges
//! - `ARCGIS_CLIENT_ID` - OAuth client ID for client credentials flow
//! - `ARCGIS_CLIENT_SECRET` - OAuth client secret for client credentials flow
//!
//! # Example
//!
//! ```no_run
//! use arcgis::EnvConfig;
//!
//! // Configuration is loaded automatically on first access
//! let config = EnvConfig::global();
//!
//! // Check if API key is available
//! if let Some(api_key) = &config.arcgis_api_key {
//!     println!("API key is configured");
//! }
//! ```

use secrecy::SecretString;
use std::sync::OnceLock;

/// Global environment configuration singleton.
static ENV_CONFIG: OnceLock<EnvConfig> = OnceLock::new();

/// Environment configuration for ArcGIS SDK.
///
/// Contains all supported environment variables as `Option<SecretString>`.
/// Secrets are wrapped in `secrecy::Secret` to prevent accidental logging or
/// exposure.
///
/// This struct is loaded once on first access via [`EnvConfig::global()`] and
/// cached for the lifetime of the application.
#[derive(Debug, Clone)]
pub struct EnvConfig {
    /// General-purpose API key for Tier 1+ services (skeleton key).
    ///
    /// Used as fallback by [`crate::ApiKeyAuth::from_env()`] if tier-specific keys aren't set.
    pub arcgis_api_key: Option<SecretString>,

    /// API key for public services (Tier 0).
    pub arcgis_public_key: Option<SecretString>,

    /// API key for location services (Tier 2).
    pub arcgis_location_key: Option<SecretString>,

    /// API key with content management privileges (Tier 3).
    ///
    /// Required for portal operations like creating/deleting services.
    pub arcgis_content_key: Option<SecretString>,

    /// API key with feature editing privileges (Tier 3).
    ///
    /// Required for feature service editing operations.
    pub arcgis_features_key: Option<SecretString>,

    /// OAuth client ID for client credentials flow.
    ///
    /// Used by [`crate::ClientCredentialsAuth`] when created via `from_env()`.
    pub arcgis_client_id: Option<SecretString>,

    /// OAuth client secret for client credentials flow.
    ///
    /// Used by [`crate::ClientCredentialsAuth`] when created via `from_env()`.
    pub arcgis_client_secret: Option<SecretString>,
}

impl EnvConfig {
    /// Loads configuration from environment variables.
    ///
    /// Automatically loads `.env` file if present using `dotenvy`.
    /// Missing variables result in `None` values (not errors).
    ///
    /// This method is called automatically on first access to [`EnvConfig::global()`].
    fn load() -> Self {
        // Load .env file (ignore errors if it doesn't exist)
        let _ = dotenvy::dotenv();

        tracing::debug!("Loading environment configuration");

        let config = Self {
            arcgis_api_key: std::env::var("ARCGIS_API_KEY").ok().map(|s| {
                tracing::debug!("ARCGIS_API_KEY loaded from environment");
                SecretString::new(s.into())
            }),
            arcgis_public_key: std::env::var("ARCGIS_PUBLIC_KEY").ok().map(|s| {
                tracing::debug!("ARCGIS_PUBLIC_KEY loaded from environment");
                SecretString::new(s.into())
            }),
            arcgis_location_key: std::env::var("ARCGIS_LOCATION_KEY").ok().map(|s| {
                tracing::debug!("ARCGIS_LOCATION_KEY loaded from environment");
                SecretString::new(s.into())
            }),
            arcgis_content_key: std::env::var("ARCGIS_CONTENT_KEY").ok().map(|s| {
                tracing::debug!("ARCGIS_CONTENT_KEY loaded from environment");
                SecretString::new(s.into())
            }),
            arcgis_features_key: std::env::var("ARCGIS_FEATURES_KEY").ok().map(|s| {
                tracing::debug!("ARCGIS_FEATURES_KEY loaded from environment");
                SecretString::new(s.into())
            }),
            arcgis_client_id: std::env::var("ARCGIS_CLIENT_ID").ok().map(|s| {
                tracing::debug!("ARCGIS_CLIENT_ID loaded from environment");
                SecretString::new(s.into())
            }),
            arcgis_client_secret: std::env::var("ARCGIS_CLIENT_SECRET").ok().map(|s| {
                tracing::debug!("ARCGIS_CLIENT_SECRET loaded from environment");
                SecretString::new(s.into())
            }),
        };

        tracing::debug!("Environment configuration loaded");
        config
    }

    /// Gets the global environment configuration.
    ///
    /// Loads configuration on first access and caches it for subsequent calls.
    /// The `.env` file is loaded automatically - users never need to call
    /// `dotenvy::dotenv()` manually.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use arcgis::EnvConfig;
    ///
    /// let config = EnvConfig::global();
    /// if config.arcgis_api_key.is_some() {
    ///     println!("API key is configured");
    /// }
    /// ```
    pub fn global() -> &'static Self {
        ENV_CONFIG.get_or_init(Self::load)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_env_config_loads_without_panic() {
        // Should not panic even if no env vars are set
        let config = EnvConfig::global();

        // Just verify it's callable - actual values depend on environment
        let _ = &config.arcgis_api_key;
    }
}
