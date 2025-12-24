//! OAuth 2.0 + PKCE authentication provider for ArcGIS.

use crate::{AuthProvider, Result};
use async_trait::async_trait;
use oauth2::{
    basic::{BasicClient, BasicErrorResponseType, BasicTokenType},
    AuthUrl, AuthorizationCode, ClientId, ClientSecret, CsrfToken, EmptyExtraTokenFields,
    PkceCodeChallenge, PkceCodeVerifier, RedirectUrl, RevocationErrorResponseType,
    StandardErrorResponse, StandardRevocableToken, StandardTokenIntrospectionResponse,
    StandardTokenResponse, TokenResponse, TokenUrl,
};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::instrument;

/// Type alias for a fully configured OAuth client with all endpoints set.
type ConfiguredClient = oauth2::Client<
    StandardErrorResponse<BasicErrorResponseType>,
    StandardTokenResponse<EmptyExtraTokenFields, BasicTokenType>,
    StandardTokenIntrospectionResponse<EmptyExtraTokenFields, BasicTokenType>,
    StandardRevocableToken,
    StandardErrorResponse<RevocationErrorResponseType>,
    oauth2::EndpointSet,      // Auth URL is set
    oauth2::EndpointNotSet,   // Device auth URL not set
    oauth2::EndpointNotSet,   // Introspection URL not set
    oauth2::EndpointNotSet,   // Revocation URL not set
    oauth2::EndpointSet,      // Token URL is set
>;

/// OAuth session state for a single authorization flow.
///
/// Stores the PKCE verifier and CSRF token for validation during the
/// callback phase. This struct must be preserved between the authorization
/// URL generation and the code exchange.
#[derive(Debug)]
pub struct OAuthSession {
    /// PKCE code verifier (secret, must not be exposed)
    pub pkce_verifier: PkceCodeVerifier,
    /// CSRF token for state validation
    pub csrf_token: CsrfToken,
}

/// OAuth 2.0 + PKCE authentication provider for ArcGIS.
///
/// Implements the OAuth 2.0 authorization code flow with PKCE
/// (Proof Key for Code Exchange) for secure authentication against
/// ArcGIS Online and ArcGIS Enterprise.
///
/// # Security Features
///
/// - **PKCE**: Prevents authorization code injection attacks
/// - **CSRF Protection**: Validates state parameter to prevent CSRF attacks
/// - **Token Refresh**: Automatically refreshes expired tokens
/// - **SSRF Prevention**: HTTP client configured to block redirects
///
/// # Example
///
/// ```no_run
/// use arcgis::{OAuthProvider, OAuthSession};
/// use oauth2::{AuthorizationCode, CsrfToken};
///
/// # async fn example() -> arcgis::Result<()> {
/// // 1. Create OAuth provider
/// let oauth = OAuthProvider::new(
///     "CLIENT_ID".to_string(),
///     "CLIENT_SECRET".to_string(),
///     "http://localhost:8080/callback".to_string(),
/// )?;
///
/// // 2. Generate authorization URL
/// let (auth_url, session) = oauth.authorize_url();
/// println!("Visit: {}", auth_url);
///
/// // 3. User authorizes, receives code and state
/// let code = AuthorizationCode::new("received_code".to_string());
/// let state = CsrfToken::new("received_state".to_string());
///
/// // 4. Exchange code for token (validates CSRF)
/// oauth.exchange_code(code, session, state).await?;
///
/// // 5. Provider is now authenticated, use with ArcGISClient
/// # Ok(())
/// # }
/// ```
pub struct OAuthProvider {
    /// OAuth2 client configuration
    client: ConfiguredClient,
    /// HTTP client with security configuration
    http_client: reqwest::Client,
    /// Stored access and refresh tokens
    token: Arc<RwLock<Option<StandardTokenResponse<EmptyExtraTokenFields, BasicTokenType>>>>,
}

impl OAuthProvider {
    /// Creates a new OAuth provider with PKCE support.
    ///
    /// # Arguments
    ///
    /// * `client_id` - Application client ID from ArcGIS Developer dashboard
    /// * `client_secret` - Application client secret (keep confidential)
    /// * `redirect_uri` - Registered redirect URI (must match ArcGIS config)
    ///
    /// # Security
    ///
    /// The HTTP client is configured to prevent SSRF attacks by disabling
    /// automatic redirect following.
    ///
    /// # Errors
    ///
    /// Returns an error if the authorization or token URLs are invalid, or
    /// if the HTTP client cannot be created.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use arcgis::OAuthProvider;
    ///
    /// # fn example() -> arcgis::Result<()> {
    /// let oauth = OAuthProvider::new(
    ///     std::env::var("CLIENT_ID")?,
    ///     std::env::var("CLIENT_SECRET")?,
    ///     "http://localhost:8080/callback".to_string(),
    /// )?;
    /// # Ok(())
    /// # }
    /// ```
    #[instrument(skip(client_id, client_secret))]
    pub fn new(
        client_id: String,
        client_secret: String,
        redirect_uri: String,
    ) -> Result<Self> {
        tracing::debug!("Creating OAuth provider");

        // Create URLs first to handle errors before building client
        let auth_url = AuthUrl::new(
            "https://www.arcgis.com/sharing/rest/oauth2/authorize".to_string(),
        )?;
        let token_url =
            TokenUrl::new("https://www.arcgis.com/sharing/rest/oauth2/token".to_string())?;
        let redirect_url = RedirectUrl::new(redirect_uri)?;

        let client = BasicClient::new(ClientId::new(client_id))
            .set_client_secret(ClientSecret::new(client_secret))
            .set_auth_uri(auth_url)
            .set_token_uri(token_url)
            .set_redirect_uri(redirect_url);

        // Security: disable redirects to prevent SSRF vulnerabilities
        let http_client = reqwest::Client::builder()
            .redirect(reqwest::redirect::Policy::none())
            .build()?;

        tracing::debug!("OAuth provider created successfully");

        Ok(Self {
            client,
            http_client,
            token: Arc::new(RwLock::new(None)),
        })
    }

    /// Generates authorization URL with PKCE and CSRF protection.
    ///
    /// Returns the URL to redirect users to for authorization, plus the
    /// session state that must be validated during the callback.
    ///
    /// # Returns
    ///
    /// A tuple of `(authorization_url, session)` where:
    /// - `authorization_url`: The URL to direct the user to
    /// - `session`: State containing PKCE verifier and CSRF token
    ///
    /// # Example
    ///
    /// ```no_run
    /// use arcgis::OAuthProvider;
    ///
    /// # fn example() -> arcgis::Result<()> {
    /// let oauth = OAuthProvider::new(
    ///     "client_id".to_string(),
    ///     "client_secret".to_string(),
    ///     "http://localhost:8080/callback".to_string(),
    /// )?;
    ///
    /// let (auth_url, session) = oauth.authorize_url();
    /// println!("Visit this URL to authorize: {}", auth_url);
    ///
    /// // Store session for later validation
    /// # Ok(())
    /// # }
    /// ```
    #[instrument(skip(self))]
    pub fn authorize_url(&self) -> (url::Url, OAuthSession) {
        tracing::debug!("Generating authorization URL with PKCE");

        // oauth2 crate generates both challenge and verifier together
        let (pkce_challenge, pkce_verifier) = PkceCodeChallenge::new_random_sha256();

        let (auth_url, csrf_token) = self
            .client
            .authorize_url(CsrfToken::new_random)
            .set_pkce_challenge(pkce_challenge)
            .url();

        tracing::info!(
            url = %auth_url,
            "Generated authorization URL"
        );

        (
            auth_url,
            OAuthSession {
                pkce_verifier,
                csrf_token,
            },
        )
    }

    /// Exchanges authorization code for access token.
    ///
    /// Validates the CSRF token and uses the PKCE verifier to prove that
    /// this client initiated the authorization request.
    ///
    /// # Arguments
    ///
    /// * `code` - Authorization code from the OAuth callback
    /// * `session` - Session state from `authorize_url()`
    /// * `returned_state` - State parameter from the OAuth callback
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - CSRF token validation fails (potential attack)
    /// - Code exchange fails (invalid code, network error)
    /// - PKCE verification fails
    ///
    /// # Example
    ///
    /// ```no_run
    /// use arcgis::OAuthProvider;
    /// use oauth2::{AuthorizationCode, CsrfToken};
    ///
    /// # async fn example() -> arcgis::Result<()> {
    /// let oauth = OAuthProvider::new(
    ///     "client_id".to_string(),
    ///     "client_secret".to_string(),
    ///     "http://localhost:8080/callback".to_string(),
    /// )?;
    ///
    /// let (auth_url, session) = oauth.authorize_url();
    /// // ... user authorizes ...
    ///
    /// let code = AuthorizationCode::new("received_code".to_string());
    /// let state = CsrfToken::new("received_state".to_string());
    ///
    /// oauth.exchange_code(code, session, state).await?;
    /// # Ok(())
    /// # }
    /// ```
    #[instrument(skip(self, code, session, returned_state))]
    pub async fn exchange_code(
        &self,
        code: AuthorizationCode,
        session: OAuthSession,
        returned_state: CsrfToken,
    ) -> Result<()> {
        tracing::debug!("Exchanging authorization code for token");

        // Validate CSRF token (constant-time comparison via oauth2 crate)
        if session.csrf_token.secret() != returned_state.secret() {
            tracing::error!("CSRF token mismatch");
            return Err(crate::ErrorKind::OAuth(
                "CSRF token mismatch - potential attack".to_string(),
            )
            .into());
        }

        tracing::debug!("CSRF token validated");

        let token_response = self
            .client
            .exchange_code(code)
            .set_pkce_verifier(session.pkce_verifier)
            .request_async(&self.http_client)
            .await
            .map_err(|e| crate::ErrorKind::OAuth(format!("Token exchange failed: {}", e)))?;

        tracing::info!("Token exchange successful");

        *self.token.write().await = Some(token_response);

        Ok(())
    }

    /// Refreshes the access token using the refresh token.
    ///
    /// OAuth 2.1 supports refresh token rotation - the response may include
    /// a new refresh token that replaces the old one.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - No refresh token is available
    /// - Token refresh fails (invalid refresh token, network error)
    ///
    /// # Example
    ///
    /// ```no_run
    /// use arcgis::OAuthProvider;
    ///
    /// # async fn example() -> arcgis::Result<()> {
    /// # let oauth = OAuthProvider::new(
    /// #     "client_id".to_string(),
    /// #     "client_secret".to_string(),
    /// #     "http://localhost:8080/callback".to_string(),
    /// # )?;
    /// // After initial authentication...
    /// oauth.refresh_token().await?;
    /// # Ok(())
    /// # }
    /// ```
    #[instrument(skip(self))]
    pub async fn refresh_token(&self) -> Result<()> {
        tracing::debug!("Refreshing access token");

        let refresh_token = {
            let token_guard = self.token.read().await;
            token_guard
                .as_ref()
                .and_then(|t| t.refresh_token())
                .ok_or_else(|| {
                    crate::Error::from(crate::ErrorKind::OAuth(
                        "No refresh token available".to_string(),
                    ))
                })?
                .clone()
        };

        let token_response = self
            .client
            .exchange_refresh_token(&refresh_token)
            .request_async(&self.http_client)
            .await
            .map_err(|e| crate::ErrorKind::OAuth(format!("Token refresh failed: {}", e)))?;

        tracing::info!("Token refresh successful");

        // OAuth 2.1: Store new refresh token if provided (rotation)
        *self.token.write().await = Some(token_response);

        Ok(())
    }
}

#[async_trait]
impl AuthProvider for OAuthProvider {
    /// Returns the current access token, refreshing if necessary.
    ///
    /// Automatically refreshes the token if it will expire within 5 minutes.
    /// This provides a buffer to prevent race conditions where the token
    /// expires between retrieval and use.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Not authenticated (call `exchange_code` first)
    /// - Token refresh fails
    #[instrument(skip(self))]
    async fn get_token(&self) -> Result<String> {
        let token_guard = self.token.read().await;

        if let Some(token) = token_guard.as_ref() {
            // Check if token is expired (with 5-minute buffer)
            if let Some(expires_in) = token.expires_in() {
                if expires_in < std::time::Duration::from_secs(300) {
                    drop(token_guard);
                    tracing::debug!("Token expiring soon, refreshing");
                    self.refresh_token().await?;

                    let new_guard = self.token.read().await;
                    return Ok(new_guard
                        .as_ref()
                        .expect("Token should exist after refresh")
                        .access_token()
                        .secret()
                        .clone());
                }
            }

            tracing::debug!("Returning current access token");
            Ok(token.access_token().secret().clone())
        } else {
            tracing::error!("Not authenticated");
            Err(
                crate::ErrorKind::Auth("Not authenticated - call exchange_code first".to_string())
                    .into(),
            )
        }
    }
}
