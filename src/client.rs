//! Core HTTP client for ArcGIS services.

use crate::AuthProvider;
use derive_getters::Getters;
use reqwest::Client as ReqwestClient;
use std::sync::Arc;
use tracing::instrument;

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
    /// # Example
    ///
    /// ```no_run
    /// use arcgis::{ArcGISClient, auth::ApiKeyAuth};
    ///
    /// let auth = ApiKeyAuth::new("YOUR_API_KEY");
    /// let client = ArcGISClient::new(auth);
    /// ```
    #[instrument(skip(auth))]
    pub fn new(auth: impl AuthProvider + 'static) -> Self {
        tracing::debug!("Creating new ArcGIS client");
        Self {
            http: ReqwestClient::new(),
            auth: Arc::new(auth),
        }
    }
}

// TODO: Add request/response helpers
// TODO: Add retry logic
// TODO: Add rate limiting
