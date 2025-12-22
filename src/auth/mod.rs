//! Authentication providers for ArcGIS services.

mod api_key;
mod provider;

pub use api_key::ApiKeyAuth;
pub use provider::AuthProvider;
