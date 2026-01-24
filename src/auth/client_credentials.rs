//! OAuth 2.0 Client Credentials Flow for automated authentication.
//!
//! This module implements the OAuth 2.0 Client Credentials grant type,
//! which is designed for server-to-server authentication without requiring
//! user interaction or browser-based authorization.
//!
//! # Use Cases
//!
//! - Server applications and backend services
//! - Automated scripts and CLI tools
//! - CI/CD pipelines
//! - Any scenario without human interaction
//!
//! # Security
//!
//! - Client secret must be kept confidential
//! - Tokens are short-lived (typically 2 hours)
//! - Automatic token refresh before expiration
//! - HTTPS required (enforced by ArcGIS)
//!
//! # Example
//!
//! ```no_run
//! use arcgis::ClientCredentialsAuth;
//!
//! # async fn example() -> arcgis::Result<()> {
//! // Load credentials from .env file (ARCGIS_CLIENT_ID and ARCGIS_CLIENT_SECRET)
//! let auth = ClientCredentialsAuth::from_env()?;
//!
//! // Token is fetched automatically on first use
//! let client = arcgis::ArcGISClient::new(auth);
//! # Ok(())
//! # }
//! ```

use crate::{AuthProvider, Result};
use async_trait::async_trait;
use secrecy::{ExposeSecret, SecretString};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tracing::instrument;

/// Token response from ArcGIS OAuth endpoint.
#[derive(Debug, Clone, Deserialize, Serialize)]
struct TokenResponse {
    /// The access token
    access_token: String,
    /// Token lifetime in seconds (typically 7200 = 2 hours)
    expires_in: u64,
}

/// Stored token with expiration tracking.
#[derive(Debug, Clone)]
struct StoredToken {
    /// The access token
    access_token: String,
    /// When the token was fetched
    fetched_at: Instant,
    /// Token lifetime in seconds
    expires_in: u64,
}

/// OAuth 2.0 Client Credentials authentication provider.
///
/// Implements automated server-to-server authentication using the
/// OAuth 2.0 Client Credentials grant type. This provider:
///
/// - Fetches tokens automatically on first use
/// - Refreshes tokens automatically before expiration
/// - Requires no human interaction or browser
/// - Is thread-safe for concurrent use
///
/// # Security Features
///
/// - Short-lived tokens (typically 2 hours)
/// - Automatic refresh with 5-minute expiration buffer
/// - SSRF prevention (HTTP redirects disabled)
/// - Secure client secret handling
///
/// # Example
///
/// ```no_run
/// use arcgis::{ClientCredentialsAuth, ArcGISClient, AuthProvider};
///
/// # async fn example() -> arcgis::Result<()> {
/// // Create authenticator
/// let auth = ClientCredentialsAuth::new(
///     "your_client_id".to_string(),
///     "your_client_secret".to_string(),
/// )?;
///
/// // Get token (fetched automatically)
/// let token = auth.get_token().await?;
///
/// // Or use with client (token management is automatic)
/// let client = ArcGISClient::new(auth);
/// # Ok(())
/// # }
/// ```
pub struct ClientCredentialsAuth {
    /// Client ID from ArcGIS Developer dashboard
    client_id: String,
    /// Client secret (kept confidential, never logged)
    client_secret: SecretString,
    /// HTTP client with security configuration
    http_client: reqwest::Client,
    /// Stored access token
    token: Arc<RwLock<Option<StoredToken>>>,
}

impl ClientCredentialsAuth {
    /// Creates a new OAuth Client Credentials authenticator.
    ///
    /// # Arguments
    ///
    /// * `client_id` - Application client ID from ArcGIS Developer dashboard
    /// * `client_secret` - Application client secret (keep confidential)
    ///
    /// # Security
    ///
    /// The HTTP client is configured to prevent SSRF attacks by disabling
    /// automatic redirect following.
    ///
    /// # Errors
    ///
    /// Returns an error if the token URL is invalid or if the HTTP client
    /// cannot be created.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use arcgis::ClientCredentialsAuth;
    ///
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let auth = ClientCredentialsAuth::new(
    ///     std::env::var("ARCGIS_CLIENT_ID")?,
    ///     std::env::var("ARCGIS_CLIENT_SECRET")?,
    /// )?;
    /// # Ok(())
    /// # }
    /// ```
    #[instrument(skip(client_id, client_secret))]
    pub fn new(client_id: String, client_secret: String) -> Result<Self> {
        tracing::debug!("Creating OAuth Client Credentials authenticator");

        // Security: disable redirects to prevent SSRF vulnerabilities
        let http_client = reqwest::Client::builder()
            .redirect(reqwest::redirect::Policy::none())
            .build()?;

        tracing::debug!("OAuth Client Credentials authenticator created");

        Ok(Self {
            client_id,
            client_secret: SecretString::new(client_secret.into_boxed_str()),
            http_client,
            token: Arc::new(RwLock::new(None)),
        })
    }

    /// Creates a new OAuth Client Credentials authenticator from environment variables.
    ///
    /// This method automatically loads `.env` file and reads the `ARCGIS_CLIENT_ID` and
    /// `ARCGIS_CLIENT_SECRET` environment variables.
    ///
    /// # Environment Variables
    ///
    /// - `ARCGIS_CLIENT_ID` - Application client ID from ArcGIS Developer dashboard
    /// - `ARCGIS_CLIENT_SECRET` - Application client secret (keep confidential)
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The `ARCGIS_CLIENT_ID` environment variable is not set
    /// - The `ARCGIS_CLIENT_SECRET` environment variable is not set
    /// - The HTTP client cannot be created
    ///
    /// The error preserves the original `std::env::VarError` in the error chain.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use arcgis::ClientCredentialsAuth;
    ///
    /// # fn example() -> arcgis::Result<()> {
    /// // Reads ARCGIS_CLIENT_ID and ARCGIS_CLIENT_SECRET from .env file
    /// let auth = ClientCredentialsAuth::from_env()?;
    /// # Ok(())
    /// # }
    /// ```
    #[instrument]
    pub fn from_env() -> Result<Self> {
        tracing::debug!("Loading OAuth credentials from environment");

        // Load .env file (ignoring errors if it doesn't exist)
        let _ = dotenvy::dotenv();

        // Read ARCGIS_CLIENT_ID - error chain: VarError → EnvError → ErrorKind → Error
        let client_id = match std::env::var("ARCGIS_CLIENT_ID") {
            Ok(id) => {
                tracing::debug!("Successfully loaded ARCGIS_CLIENT_ID from environment");
                id
            }
            Err(e) => {
                tracing::error!(
                    error = %e,
                    "ARCGIS_CLIENT_ID environment variable not set or invalid"
                );
                return Err(e.into()); // Automatic conversion through error chain
            }
        };

        // Read ARCGIS_CLIENT_SECRET - error chain: VarError → EnvError → ErrorKind → Error
        let client_secret = match std::env::var("ARCGIS_CLIENT_SECRET") {
            Ok(secret) => {
                tracing::debug!("Successfully loaded ARCGIS_CLIENT_SECRET from environment");
                secret
            }
            Err(e) => {
                tracing::error!(
                    error = %e,
                    "ARCGIS_CLIENT_SECRET environment variable not set or invalid"
                );
                return Err(e.into()); // Automatic conversion through error chain
            }
        };

        Self::new(client_id, client_secret)
    }

    /// Fetches a new access token from the ArcGIS token endpoint.
    ///
    /// This method makes a POST request to the OAuth token endpoint with
    /// the client credentials to obtain a new access token.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The HTTP request fails
    /// - The credentials are invalid
    /// - The ArcGIS server returns an error
    #[instrument(skip(self))]
    async fn fetch_token(&self) -> Result<()> {
        tracing::debug!("Fetching new access token via client credentials flow");

        let params = [
            ("client_id", self.client_id.as_str()),
            ("client_secret", self.client_secret.expose_secret()),
            ("grant_type", "client_credentials"),
            ("f", "json"), // ArcGIS requires this for JSON response
        ];

        let response = self
            .http_client
            .post("https://www.arcgis.com/sharing/rest/oauth2/token")
            .form(&params)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response
                .text()
                .await
                .unwrap_or_else(|_| "<no body>".to_string());
            return Err(crate::ErrorKind::OAuth(format!(
                "Token request failed with status {}: {}",
                status, body
            ))
            .into());
        }

        let token_response: TokenResponse = response.json().await?;

        tracing::info!(
            expires_in = token_response.expires_in,
            "Access token obtained successfully"
        );

        let stored_token = StoredToken {
            access_token: token_response.access_token,
            fetched_at: Instant::now(),
            expires_in: token_response.expires_in,
        };

        *self.token.write().await = Some(stored_token);

        Ok(())
    }

    /// Checks if the current token is expired or will expire soon.
    ///
    /// Returns `true` if:
    /// - Token will expire within 5 minutes
    ///
    /// The 5-minute buffer prevents race conditions where a token
    /// expires between retrieval and use.
    fn is_token_expired(token: &StoredToken) -> bool {
        let age = token.fetched_at.elapsed();
        let expires_in = Duration::from_secs(token.expires_in);
        let buffer = Duration::from_secs(300); // 5-minute buffer

        age + buffer >= expires_in
    }
}

#[async_trait]
impl AuthProvider for ClientCredentialsAuth {
    /// Returns the current access token, fetching or refreshing if necessary.
    ///
    /// This method:
    /// - Fetches a token on first use
    /// - Returns the cached token if valid
    /// - Automatically refreshes tokens expiring within 5 minutes
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Token fetch fails (invalid credentials, network error)
    /// - Token refresh fails
    ///
    /// # Example
    ///
    /// ```no_run
    /// use arcgis::{ClientCredentialsAuth, AuthProvider};
    ///
    /// # async fn example() -> arcgis::Result<()> {
    /// let auth = ClientCredentialsAuth::new(
    ///     "client_id".to_string(),
    ///     "client_secret".to_string(),
    /// )?;
    ///
    /// // First call fetches token
    /// let token1 = auth.get_token().await?;
    ///
    /// // Second call returns cached token
    /// let token2 = auth.get_token().await?;
    ///
    /// assert_eq!(token1, token2);
    /// # Ok(())
    /// # }
    /// ```
    #[instrument(skip(self))]
    async fn get_token(&self) -> Result<String> {
        let token_guard = self.token.read().await;

        if let Some(token) = token_guard.as_ref() {
            // Check if token is expired or expiring soon
            if Self::is_token_expired(token) {
                drop(token_guard);
                tracing::debug!("Token expiring soon, fetching new token");
                self.fetch_token().await?;

                let new_guard = self.token.read().await;
                let token = new_guard.as_ref().ok_or_else(|| {
                    crate::ErrorKind::OAuth("Token missing after successful fetch".to_string())
                })?;
                return Ok(token.access_token.clone());
            }

            tracing::debug!("Returning cached access token");
            Ok(token.access_token.clone())
        } else {
            // No token exists - fetch one
            drop(token_guard);
            tracing::debug!("No token exists, fetching initial token");
            self.fetch_token().await?;

            let guard = self.token.read().await;
            let token = guard.as_ref().ok_or_else(|| {
                crate::ErrorKind::OAuth("Token missing after successful fetch".to_string())
            })?;
            Ok(token.access_token.clone())
        }
    }
}
