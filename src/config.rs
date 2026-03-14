//! Centralized environment configuration.
//!
//! This module provides a single point of access for all environment variables
//! used by the SDK. The configuration is loaded once on first access and cached
//! for the lifetime of the application.
//!
//! # Environment Variables
//!
//! ## ArcGIS Online API Keys (ESRI Tiers)
//!
//! - `ARCGIS_API_KEY` - General-purpose API key (skeleton key, fallback)
//! - `ARCGIS_PUBLIC_KEY` - Public tier (no permissions, minimal security envelope)
//! - `ARCGIS_LOCATION_KEY` - Location services tier (geocoding, routing, basemaps)
//! - `ARCGIS_SPATIAL_KEY` - Spatial Analysis tier (geometry operations, spatial analysis)
//! - `ARCGIS_GENERAL_KEY` - General services tier (feature CRUD, content management)
//! - `ARCGIS_ADMIN_KEY` - Admin services tier (publishing, organization management)
//!
//! ## OAuth Credentials
//!
//! - `ARCGIS_CLIENT_ID` - OAuth client ID for client credentials flow
//! - `ARCGIS_CLIENT_SECRET` - OAuth client secret for client credentials flow
//!
//! ## ArcGIS Enterprise
//!
//! - `ARCGIS_ENTERPRISE_PORTAL` - URL for ArcGIS Enterprise portal (e.g., `https://your-server.com/portal/sharing/rest`)
//! - `ARCGIS_ENTERPRISE_KEY` - API key for ArcGIS Enterprise portal operations
//! - `ARCGIS_FEATURE_URL` - Base URL for a feature service (e.g., `https://your-server.com/arcgis/rest/services/Assets/FeatureServer`)
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
use std::collections::HashMap;
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
    /// General-purpose API key (skeleton key).
    ///
    /// Used as fallback by [`crate::ApiKeyAuth::from_env()`] if tier-specific keys aren't set.
    /// Useful for development when you have a single key with multiple privileges.
    pub arcgis_api_key: Option<SecretString>,

    /// API key for Public tier (no permissions).
    ///
    /// **Permissions:** None - useful for public-facing applications with minimal security envelope.
    ///
    /// **Use Cases:** Testing authentication flow, rate limit bypass, public service access.
    pub arcgis_public_key: Option<SecretString>,

    /// API key for Location services (ESRI Tier 1).
    ///
    /// **Permissions:** Geocoding, routing, basemaps, geoenrichment.
    ///
    /// **Operations:** Geocode addresses, calculate routes, access basemap tiles.
    pub arcgis_location_key: Option<SecretString>,

    /// API key for Spatial Analysis services (ESRI Tier 2).
    ///
    /// **Permissions:** Geometry operations, spatial analysis, elevation services.
    ///
    /// **Operations:** Buffer, intersect, union, project geometries, elevation analysis.
    pub arcgis_spatial_key: Option<SecretString>,

    /// API key for General services (ESRI Tier 3).
    ///
    /// **Permissions:** Feature service CRUD, content management, sharing.
    ///
    /// **Operations:** Query features, create/edit/delete features, manage portal items.
    pub arcgis_general_key: Option<SecretString>,

    /// API key for Admin services (ESRI Tier 4).
    ///
    /// **Permissions:** Service publishing, organization management, advanced admin operations.
    ///
    /// **Operations:** Create hosted services, addToDefinition, manage organization.
    pub arcgis_admin_key: Option<SecretString>,

    /// OAuth client ID for client credentials flow.
    ///
    /// Used by [`crate::ClientCredentialsAuth`] when created via `from_env()`.
    pub arcgis_client_id: Option<SecretString>,

    /// OAuth client secret for client credentials flow.
    ///
    /// Used by [`crate::ClientCredentialsAuth`] when created via `from_env()`.
    pub arcgis_client_secret: Option<SecretString>,

    /// URL for ArcGIS Enterprise portal sharing REST endpoint.
    ///
    /// Example: `https://your-server.com/portal/sharing/rest`
    ///
    /// Required for examples that use Enterprise-only features like branch versioning.
    pub arcgis_enterprise_portal: Option<String>,

    /// API key for ArcGIS Enterprise portal operations.
    ///
    /// General-level permissions for Enterprise portal content management and feature editing.
    /// Separate from ArcGIS Online keys as Enterprise portals use different authentication.
    pub arcgis_enterprise_key: Option<SecretString>,

    /// Base URL for a feature service.
    ///
    /// Example: `https://your-server.com/arcgis/rest/services/Assets/FeatureServer`
    ///
    /// Used for version management examples to construct VersionManagementServer URL.
    /// The VersionManagementServer URL is derived by replacing `FeatureServer` with `VersionManagementServer`.
    pub arcgis_feature_url: Option<String>,

    /// Permission-to-key mappings from environment variables.
    ///
    /// Maps ESRI permission strings (e.g., "portal:user:deleteItem") to the
    /// environment variable names that contain the API key for that permission.
    ///
    /// Populated from `ARCGIS_PERMISSION_*` environment variables:
    /// - `ARCGIS_PERMISSION_portal:user:deleteItem=my_key_value`
    ///
    /// The value is the actual API key, which gets looked up through the
    /// tier-based key fields.
    pub permission_mappings: HashMap<String, SecretString>,
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

        // Load permission mappings from ARCGIS_PERMISSION_* environment variables
        let mut permission_mappings = HashMap::new();
        for (key, value) in std::env::vars() {
            if let Some(permission_str) = key.strip_prefix("ARCGIS_PERMISSION_") {
                tracing::debug!(
                    permission = %permission_str,
                    "Loaded permission mapping from environment"
                );
                permission_mappings.insert(permission_str.to_string(), SecretString::new(value.into()));
            }
        }

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
            arcgis_spatial_key: std::env::var("ARCGIS_SPATIAL_KEY").ok().map(|s| {
                tracing::debug!("ARCGIS_SPATIAL_KEY loaded from environment");
                SecretString::new(s.into())
            }),
            arcgis_general_key: std::env::var("ARCGIS_GENERAL_KEY").ok().map(|s| {
                tracing::debug!("ARCGIS_GENERAL_KEY loaded from environment");
                SecretString::new(s.into())
            }),
            arcgis_admin_key: std::env::var("ARCGIS_ADMIN_KEY").ok().map(|s| {
                tracing::debug!("ARCGIS_ADMIN_KEY loaded from environment");
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
            arcgis_enterprise_portal: std::env::var("ARCGIS_ENTERPRISE_PORTAL").ok().inspect(
                |_| {
                    tracing::debug!("ARCGIS_ENTERPRISE_PORTAL loaded from environment");
                },
            ),
            arcgis_enterprise_key: std::env::var("ARCGIS_ENTERPRISE_KEY").ok().map(|s| {
                tracing::debug!("ARCGIS_ENTERPRISE_KEY loaded from environment");
                SecretString::new(s.into())
            }),
            arcgis_feature_url: std::env::var("ARCGIS_FEATURE_URL").ok().inspect(|_| {
                tracing::debug!("ARCGIS_FEATURE_URL loaded from environment");
            }),
            permission_mappings,
        };

        tracing::debug!(
            permission_count = config.permission_mappings.len(),
            "Environment configuration loaded"
        );
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

    /// Find which API key to use for a given permission (fallback hierarchy).
    ///
    /// This method implements a three-level fallback hierarchy:
    ///
    /// 1. **Specific permission key** (highest priority)
    ///    - `ARCGIS_PERMISSION_portal:user:deleteItem=my_specific_key`
    ///
    /// 2. **Group/tier key** (convenience fallback)
    ///    - Based on the permission's default tier
    ///    - E.g., `portal:user:*` permissions → `ARCGIS_GENERAL_KEY`
    ///
    /// 3. **Skeleton key - AGOL** (broadest fallback)
    ///    - `ARCGIS_API_KEY` - single key with all permissions
    ///
    /// 4. **Skeleton key - Enterprise** (final fallback)
    ///    - `ARCGIS_ENTERPRISE_KEY` - for Enterprise portal operations
    ///
    /// # Arguments
    /// * `perm` - The permission required for the operation
    ///
    /// # Returns
    /// The first matching API key in the fallback hierarchy, or `None` if no key
    /// is configured for this permission.
    ///
    /// # Example
    /// ```no_run
    /// use arcgis::{EnvConfig, Permission};
    ///
    /// let config = EnvConfig::global();
    /// let key = config.get_key_for_permission(Permission::PortalUserDeleteItem);
    /// ```
    pub fn get_key_for_permission(&self, perm: crate::Permission) -> Option<&SecretString> {
        let esri_string = perm.to_esri_string();

        // 1. Check specific permission mapping
        if let Some(key) = self.permission_mappings.get(esri_string) {
            tracing::debug!(
                permission = %esri_string,
                "Using specific permission key"
            );
            return Some(key);
        }

        // 2. Check tier/group key based on permission's default tier
        if let Some(key) = self.get_key_by_tier(perm.default_tier()) {
            tracing::debug!(
                permission = %esri_string,
                tier = ?perm.default_tier(),
                "Using tier fallback key"
            );
            return Some(key);
        }

        // 3. Fall back to skeleton key (AGOL)
        if let Some(key) = &self.arcgis_api_key {
            tracing::debug!(
                permission = %esri_string,
                "Using AGOL skeleton key"
            );
            return Some(key);
        }

        // 4. Fall back to skeleton key (Enterprise)
        if let Some(key) = &self.arcgis_enterprise_key {
            tracing::debug!(
                permission = %esri_string,
                "Using Enterprise skeleton key"
            );
            return Some(key);
        }

        // No key configured for this permission
        tracing::warn!(
            permission = %esri_string,
            tier = ?perm.default_tier(),
            "No API key configured for permission"
        );
        None
    }

    /// Get the API key for a specific tier.
    ///
    /// # Arguments
    /// * `tier` - The API key tier to look up
    ///
    /// # Returns
    /// The API key for the given tier, or `None` if not configured.
    fn get_key_by_tier(&self, tier: crate::ApiKeyTier) -> Option<&SecretString> {
        use crate::ApiKeyTier;

        match tier {
            ApiKeyTier::Public => self.arcgis_public_key.as_ref(),
            ApiKeyTier::Location => self.arcgis_location_key.as_ref(),
            ApiKeyTier::Spatial => self.arcgis_spatial_key.as_ref(),
            ApiKeyTier::General => self.arcgis_general_key.as_ref(),
            ApiKeyTier::Admin => self.arcgis_admin_key.as_ref(),
        }
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
