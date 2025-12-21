//! Core HTTP client for ArcGIS services.

use crate::auth::AuthProvider;
use reqwest::Client as ReqwestClient;
use std::sync::Arc;

/// The main client for interacting with ArcGIS services.
///
/// This client handles HTTP communication, authentication, and common
/// request/response processing for all ArcGIS services.
///
/// # Example
///
/// ```no_run
/// use arcgis::{ArcGISClient, auth::ApiKeyAuth};
///
/// let auth = ApiKeyAuth::new("YOUR_API_KEY");
/// let client = ArcGISClient::new(auth);
/// ```
pub struct ArcGISClient {
    http: ReqwestClient,
    auth: Arc<dyn AuthProvider>,
}

impl ArcGISClient {
    /// Creates a new ArcGIS client with the given authentication provider.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use arcgis::{ArcGISClient, auth::ApiKeyAuth};
    ///
    /// let auth = ApiKeyAuth::new("YOUR_API_KEY");
    /// let client = ArcGISClient::new(auth);
    /// ```
    pub fn new(auth: impl AuthProvider + 'static) -> Self {
        Self {
            http: ReqwestClient::new(),
            auth: Arc::new(auth),
        }
    }

    /// Returns a reference to the underlying HTTP client.
    pub fn http(&self) -> &ReqwestClient {
        &self.http
    }

    /// Returns a reference to the authentication provider.
    pub fn auth(&self) -> &Arc<dyn AuthProvider> {
        &self.auth
    }
}

// TODO: Add request/response helpers
// TODO: Add retry logic
// TODO: Add rate limiting
