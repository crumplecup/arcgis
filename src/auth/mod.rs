//! Authentication providers for ArcGIS services.

use crate::Result;
use async_trait::async_trait;

pub mod api_key;

pub use api_key::ApiKeyAuth;

/// Trait for authentication providers.
///
/// Implementors of this trait can provide authentication credentials
/// for ArcGIS API requests.
#[async_trait]
pub trait AuthProvider: Send + Sync {
    /// Returns the authentication token or API key to use for requests.
    async fn get_token(&self) -> Result<String>;

    /// Returns whether this provider requires a token parameter.
    fn requires_token_param(&self) -> bool {
        true
    }
}
