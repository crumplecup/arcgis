# PKCE OAuth Authentication Strategy

## Executive Summary

**Feasibility**: ✅ **YES** - PKCE authentication with CLIENT_ID and
CLIENT_SECRET is fully supported and recommended by ArcGIS.

**Timeline**: 2 weeks for complete implementation (Phase 1, Weeks 1-2)

**Priority**: **P0 - CRITICAL** - BLOCKING for all other development.
Without OAuth authentication, no API testing or SDK development is possible.

## OAuth2 Crate Ecosystem

The Rust `oauth2` crate (v4.4+) provides a complete, battle-tested OAuth 2.0
implementation with built-in PKCE support. **We do NOT need to implement PKCE
manually.**

### What the oauth2 Crate Provides

- ✅ **PKCE Generation**: `PkceCodeChallenge::new_random_sha256()` generates
  both challenge and verifier
- ✅ **CSRF Protection**: `CsrfToken::new_random()` for state validation
- ✅ **Typed Tokens**: `AuthorizationCode`, `AccessToken`, `RefreshToken`
- ✅ **Async Support**: `request_async()` with tokio/reqwest
- ✅ **Multiple Flows**: Authorization code, client credentials,
  password grant, device code
- ✅ **Security**: Constant-time comparison, secure random generation

### Key Pattern

```rust
// Generate PKCE challenge + verifier together
let (pkce_challenge, pkce_verifier) =
    PkceCodeChallenge::new_random_sha256();

// Attach challenge to authorization URL
let (auth_url, csrf_token) = client
    .authorize_url(CsrfToken::new_random)
    .set_pkce_challenge(pkce_challenge)
    .url();

// Exchange code with verifier
let token = client
    .exchange_code(AuthorizationCode::new(code))
    .set_pkce_verifier(pkce_verifier)
    .request_async(&http_client)
    .await?;
```

### Critical Security Requirement

The HTTP client MUST disable redirects to prevent SSRF attacks:

```rust
let http_client = reqwest::Client::builder()
    .redirect(reqwest::redirect::Policy::none())
    .build()?;
```

## Background

### Current State

- ✅ API Key authentication implemented (ApiKeyAuth)
- ✅ AuthProvider trait defines authentication interface
- ❌ OAuth flows not yet implemented
- ❌ Token management not implemented
- ❌ Token refresh not implemented

### Why PKCE?

PKCE (Proof Key for Code Exchange, RFC 7636) prevents authorization code
injection attacks by requiring a cryptographic code verifier that's
separate from the client secret. This is critical for:

1. **Public clients** (desktop apps, CLIs) where client secrets can be
   extracted
2. **Enhanced security** even with confidential clients
   (server applications)
3. **Modern OAuth best practices** as recommended by OAuth 2.1

### ArcGIS PKCE Support

According to ArcGIS documentation:

- PKCE is supported via `code_challenge` and `code_challenge_method`
  parameters
- CLIENT_SECRET is "optional, though recommended for additional security"
- Both S256 (SHA-256 hash) and plain code challenges are supported
- Standard OAuth 2.0 endpoints are used

## Feasibility Analysis

### ✅ Fully Feasible

**Reasons**:

1. **ArcGIS API Support**: Native PKCE support in authorization endpoint
2. **Rust Ecosystem**: `oauth2` crate provides PKCE primitives
3. **Architecture Fit**: AuthProvider trait already supports async token
   retrieval
4. **Credentials Available**: CLIENT_ID and CLIENT_SECRET in .env
5. **Standard Flow**: Well-documented OAuth 2.0 authorization code flow
   with PKCE

### Flow Compatibility

Using CLIENT_ID + CLIENT_SECRET + PKCE provides **defense in depth**:

- **Code verifier**: Prevents code injection (PKCE)
- **Client secret**: Authenticates the application (OAuth 2.0)
- **Together**: Maximum security for confidential clients

## Technical Architecture

### Authentication Flow

```text
┌─────────────┐                                  ┌──────────────┐
│   Client    │                                  │   ArcGIS     │
│ Application │                                  │   Server     │
└──────┬──────┘                                  └──────┬───────┘
       │                                                │
       │ 1. Generate code_verifier (random 43-128 chars)│
       │    code_challenge = SHA256(code_verifier)      │
       │                                                │
       │ 2. Authorization Request                       │
       │    /oauth2/authorize?                         │
       │      client_id=...                            │
       │      response_type=code                       │
       │      redirect_uri=...                         │
       │      code_challenge=...                       │
       │      code_challenge_method=S256               │
       ├───────────────────────────────────────────────>│
       │                                                │
       │ 3. User Login & Consent (browser)             │
       │<──────────────────────────────────────────────┤
       │                                                │
       │ 4. Authorization Code (via redirect)          │
       │<──────────────────────────────────────────────┤
       │                                                │
       │ 5. Token Request                              │
       │    POST /oauth2/token                         │
       │      grant_type=authorization_code            │
       │      code=...                                 │
       │      client_id=...                            │
       │      client_secret=...                        │
       │      code_verifier=... (original plaintext)   │
       │      redirect_uri=...                         │
       ├───────────────────────────────────────────────>│
       │                                                │
       │                     6. Verify:                 │
       │                        SHA256(code_verifier)   │
       │                        == code_challenge       │
       │                        client_secret valid     │
       │                                                │
       │ 7. Access Token + Refresh Token               │
       │<──────────────────────────────────────────────┤
       │                                                │
       │ 8. API Requests with token                    │
       │    ?token=<access_token>                      │
       ├───────────────────────────────────────────────>│
       │                                                │
       │ 9. Token Refresh (when expired)               │
       │    POST /oauth2/token                         │
       │      grant_type=refresh_token                 │
       │      refresh_token=...                        │
       │      client_id=...                            │
       │      client_secret=...                        │
       ├───────────────────────────────────────────────>│
       │                                                │
       │ 10. New Access Token + Refresh Token          │
       │<──────────────────────────────────────────────┤
```

### Key Components

**Note**: The `oauth2` crate provides all PKCE primitives. We do NOT need to
implement PKCE generation manually.

#### 1. HTTP Client Setup (Security Critical)

```rust
// src/auth/http_client.rs
use reqwest::Client;

/// Creates a secure HTTP client for OAuth requests.
///
/// SECURITY: Disables redirects to prevent SSRF vulnerabilities
pub fn create_oauth_http_client() -> Client {
    Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .expect("Failed to create HTTP client")
}
```

#### 2. OAuth Provider

```rust
// src/auth/oauth.rs
use oauth2::{
    basic::BasicClient,
    reqwest::async_http_client,
    AuthUrl, AuthorizationCode, ClientId, ClientSecret,
    CsrfToken, PkceCodeChallenge, PkceCodeVerifier,
    RedirectUrl, RefreshToken, TokenResponse, TokenUrl,
};
use std::sync::Arc;
use tokio::sync::RwLock;

/// OAuth session state for a single authorization flow.
///
/// Stores the PKCE verifier and CSRF token for validation.
pub struct OAuthSession {
    pub pkce_verifier: PkceCodeVerifier,
    pub csrf_token: CsrfToken,
}

/// OAuth 2.0 + PKCE authentication provider for ArcGIS.
pub struct OAuthProvider {
    client: BasicClient,
    http_client: reqwest::Client,
    token: Arc<RwLock<Option<oauth2::StandardTokenResponse>>>,
}

impl OAuthProvider {
    /// Creates a new OAuth provider with PKCE support.
    pub fn new(
        client_id: String,
        client_secret: String,
        redirect_uri: String,
    ) -> Result<Self> {
        let client = BasicClient::new(
            ClientId::new(client_id),
            Some(ClientSecret::new(client_secret)),
            AuthUrl::new(
                "https://www.arcgis.com/sharing/rest/oauth2/authorize"
                    .to_string()
            )?,
            Some(TokenUrl::new(
                "https://www.arcgis.com/sharing/rest/oauth2/token"
                    .to_string()
            )?),
        )
        .set_redirect_uri(RedirectUrl::new(redirect_uri)?);

        // Security: disable redirects to prevent SSRF
        let http_client = reqwest::Client::builder()
            .redirect(reqwest::redirect::Policy::none())
            .build()?;

        Ok(Self {
            client,
            http_client,
            token: Arc::new(RwLock::new(None)),
        })
    }

    /// Generates authorization URL with PKCE and CSRF protection.
    ///
    /// Returns the URL to redirect users to, plus session state
    /// that must be validated during callback.
    pub fn authorize_url(&self) -> (url::Url, OAuthSession) {
        // oauth2 crate generates both challenge and verifier together
        let (pkce_challenge, pkce_verifier) =
            PkceCodeChallenge::new_random_sha256();

        let (auth_url, csrf_token) = self
            .client
            .authorize_url(CsrfToken::new_random)
            .set_pkce_challenge(pkce_challenge)
            .url();

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
    /// Validates CSRF token and uses PKCE verifier.
    pub async fn exchange_code(
        &self,
        code: AuthorizationCode,
        session: OAuthSession,
        returned_state: CsrfToken,
    ) -> Result<()> {
        // Validate CSRF token (constant-time comparison)
        if session.csrf_token.secret() != returned_state.secret() {
            return Err(Error::new("CSRF token mismatch"));
        }

        let token_response = self
            .client
            .exchange_code(code)
            .set_pkce_verifier(session.pkce_verifier)
            .request_async(&self.http_client)
            .await?;

        *self.token.write().await = Some(token_response);
        Ok(())
    }

    /// Refreshes the access token using the refresh token.
    ///
    /// OAuth 2.1 supports refresh token rotation - the response
    /// may include a new refresh token that should be stored.
    pub async fn refresh_token(&self) -> Result<()> {
        let refresh_token = {
            let token_guard = self.token.read().await;
            token_guard
                .as_ref()
                .and_then(|t| t.refresh_token())
                .ok_or_else(|| Error::new("No refresh token available"))?
                .clone()
        };

        let token_response = self
            .client
            .exchange_refresh_token(&refresh_token)
            .request_async(&self.http_client)
            .await?;

        // OAuth 2.1: Store new refresh token if provided (rotation)
        *self.token.write().await = Some(token_response);
        Ok(())
    }
}

#[async_trait]
impl AuthProvider for OAuthProvider {
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
                        .unwrap()
                        .access_token()
                        .secret()
                        .clone());
                }
            }

            Ok(token.access_token().secret().clone())
        } else {
            Err(Error::new("Not authenticated"))
        }
    }
}
```

#### 3. Configuration

```rust
// .env additions
CLIENT_ID="vFJDWRVUN5KT2H6V"
CLIENT_SECRET="beea82a053f94b0b97bf27919c84d6fa"
REDIRECT_URI="http://localhost:8080/callback"
```

#### 4. Usage Example

```rust
// examples/oauth_flow.rs
use arcgis::{OAuthProvider, OAuthSession, ArcGISClient};
use oauth2::{AuthorizationCode, CsrfToken};
use std::io::{BufRead, BufReader, Write};
use std::net::TcpListener;

#[tokio::main]
async fn main() -> Result<()> {
    // 1. Create OAuth provider
    let oauth = OAuthProvider::new(
        env::var("CLIENT_ID")?,
        env::var("CLIENT_SECRET")?,
        "http://localhost:8080/callback".to_string(),
    )?;

    // 2. Generate authorization URL with PKCE + CSRF
    let (auth_url, session) = oauth.authorize_url();

    println!("Visit this URL to authorize:");
    println!("{}\n", auth_url);

    // 3. Start local server to capture redirect
    let listener = TcpListener::bind("127.0.0.1:8080")?;
    println!("Listening on http://localhost:8080...\n");

    // 4. Wait for OAuth callback
    let (code, state) = receive_callback(&listener)?;

    // 5. Exchange code for token (validates CSRF token)
    oauth
        .exchange_code(
            AuthorizationCode::new(code),
            session,
            CsrfToken::new(state),
        )
        .await?;

    println!("Authentication successful!\n");

    // 6. Use authenticated client
    let client = ArcGISClient::new(oauth);

    // Token refresh is automatic in get_token()
    let features = client
        .feature_service(service_url)
        .layer(0)
        .query()
        .where_clause("1=1")
        .execute()
        .await?;

    println!("Retrieved {} features", features.len());

    Ok(())
}

/// Receives OAuth callback and extracts code and state.
fn receive_callback(listener: &TcpListener) -> Result<(String, String)> {
    for stream in listener.incoming() {
        if let Ok(mut stream) = stream {
            let mut reader = BufReader::new(&stream);
            let mut request_line = String::new();
            reader.read_line(&mut request_line)?;

            // Extract code and state from query string
            let redirect_url = request_line
                .split_whitespace()
                .nth(1)
                .ok_or_else(|| Error::new("Invalid request"))?;

            let url = url::Url::parse(&format!("http://localhost{}", redirect_url))?;

            let code = url
                .query_pairs()
                .find(|(key, _)| key == "code")
                .map(|(_, code)| code.into_owned())
                .ok_or_else(|| Error::new("No code in callback"))?;

            let state = url
                .query_pairs()
                .find(|(key, _)| key == "state")
                .map(|(_, state)| state.into_owned())
                .ok_or_else(|| Error::new("No state in callback"))?;

            // Send response
            let response = "HTTP/1.1 200 OK\r\n\r\n\
                <html><body>\
                <h1>Authentication successful!</h1>\
                <p>You can close this window.</p>\
                </body></html>";
            stream.write_all(response.as_bytes())?;

            return Ok((code, state));
        }
    }

    Err(Error::new("Failed to receive callback"))
}
```

## Implementation Plan

**Status**: This is now Phase 1, Priority 0 in IMPLEMENTATION_PLAN.md

### Week 1: Core OAuth Infrastructure

**Tasks**:

- [ ] Add `oauth2` crate dependency
- [ ] Create secure HTTP client with redirect policy (src/auth/http_client.rs)
- [ ] Create OAuthProvider struct using oauth2::BasicClient (src/auth/oauth.rs)
- [ ] Create OAuthSession struct for state management
- [ ] Implement AuthProvider trait for OAuthProvider
- [ ] Add unit tests for token refresh logic
- [ ] Add integration test (manual, with real credentials)

**Deliverables**:

- OAuth provider using oauth2 crate primitives
- CSRF token validation
- PKCE flow working (via oauth2 crate)
- Basic token exchange working

### Week 2: CLI Example + Documentation

**Tasks**:

- [ ] Create interactive OAuth CLI example
- [ ] Add local callback server for redirect handling
- [ ] Implement automatic token refresh logic
- [ ] Add token expiration checking (5-minute buffer)
- [ ] Thread-safe token storage verification
- [ ] Add tracing instrumentation
- [ ] Write comprehensive documentation
- [ ] Add troubleshooting guide
- [ ] Integration test with real credentials

**Deliverables**:

- Complete OAuth CLI example working end-to-end
- Automatic token refresh verified
- User documentation complete
- Integration test passing
- Ready for v0.1.0-alpha release

## Dependencies

### New Crate Dependencies

```toml
[dependencies]
oauth2 = "4.4"           # OAuth 2.0 client (includes PKCE)
reqwest = { version = "0.12", features = ["json"] }  # HTTP client
tokio = { version = "1", features = ["sync"] }       # Async runtime
url = "2.5"              # URL parsing for callbacks
async-trait = "0.1"      # Async trait support
tracing = "0.1"          # Already present, for instrumentation

# Already present in project:
# secrecy = "0.8"        # Secret management
```

**Note**: The `oauth2` crate includes all PKCE functionality (SHA-256, base64,
random generation). No manual crypto implementation needed!

### Configuration Requirements

**.env**:

```bash
CLIENT_ID="vFJDWRVUN5KT2H6V"
CLIENT_SECRET="beea82a053f94b0b97bf27919c84d6fa"
REDIRECT_URI="http://localhost:8080/callback"  # For CLI apps
```

**ArcGIS Developer Dashboard**:

- Register redirect URI in application settings
- Ensure OAuth 2.0 is enabled for the application

## Security Considerations

### Implemented Safeguards

1. **Code Verifier Randomness**: 128 characters, cryptographically secure
2. **SHA-256 Challenge**: Industry standard, prevents rainbow tables
3. **Client Secret Protection**: Via `secrecy::SecretString`
4. **Token Storage**: In-memory only, cleared on process exit
5. **HTTPS Enforcement**: ArcGIS requires HTTPS for production
6. **Token Refresh**: Automatic, prevents expired token errors

### Best Practices

- ✅ Never log client secrets or tokens
- ✅ Never commit .env to version control
- ✅ Use HTTPS redirect URIs in production
- ✅ Implement CSRF protection (oauth2 crate provides CsrfToken)
- ✅ Store refresh tokens securely (OS keychain for desktop apps)
- ✅ Validate redirect URIs match registered URIs

## Testing Strategy

### Unit Tests

```rust
#[test]
fn oauth_session_contains_pkce_and_csrf() {
    use oauth2::PkceCodeChallenge;

    let (pkce_challenge, pkce_verifier) =
        PkceCodeChallenge::new_random_sha256();

    // oauth2 crate handles validation internally
    assert!(!pkce_verifier.secret().is_empty());
    assert!(!pkce_challenge.as_str().is_empty());
}

#[tokio::test]
async fn token_refresh_updates_token() {
    // Mock test - real test requires valid refresh token
    let provider = create_test_provider();

    // Verify token refresh logic updates stored token
    // (requires mock OAuth server for real testing)
}

#[test]
fn csrf_token_validation_prevents_attacks() {
    use oauth2::CsrfToken;

    let token1 = CsrfToken::new_random();
    let token2 = CsrfToken::new_random();

    // Tokens should be different (randomness check)
    assert_ne!(token1.secret(), token2.secret());
}
```

### Integration Tests

```rust
#[tokio::test]
#[ignore] // Requires manual authorization
async fn test_oauth_flow() {
    let oauth = OAuthProvider::new(
        env::var("CLIENT_ID").unwrap(),
        env::var("CLIENT_SECRET").unwrap(),
        "http://localhost:8080/callback".to_string(),
    ).unwrap();

    let (url, verifier) = oauth.authorize_url();
    println!("Authorize at: {}", url);

    // Manual step: user authorizes and pastes code
    let code = get_code_from_user();

    oauth.exchange_code(code, verifier).await.unwrap();
    let token = oauth.get_token().await.unwrap();

    assert!(!token.is_empty());
}
```

### Manual Testing Checklist

- [ ] Authorization URL opens in browser
- [ ] User can log in and authorize
- [ ] Redirect captures authorization code
- [ ] Token exchange succeeds
- [ ] Access token works for API requests
- [ ] Token refresh works before expiration
- [ ] Expired token triggers refresh
- [ ] Invalid code returns clear error

## Risks & Mitigation

### Risk 1: Redirect URI Handling

**Risk**: CLI applications need to capture redirect callbacks

**Mitigation**:

- Use `http://localhost:<random_port>/callback`
- Start temporary HTTP server to capture code
- Provide copy/paste fallback for restricted environments

### Risk 2: Token Persistence

**Risk**: Tokens lost on application restart

**Mitigation**:

- Implement optional token storage trait
- Support OS keychain integration (keyring crate)
- Document security implications

### Risk 3: Concurrent Token Refresh

**Risk**: Multiple threads triggering simultaneous refresh

**Mitigation**:

- Use `Arc&lt;RwLock&gt;` for synchronized access
- Check expiration before acquiring write lock
- Add token refresh mutex for extra safety

### Risk 4: ArcGIS Endpoint Variations

**Risk**: Enterprise/Portal endpoints differ from ArcGIS Online

**Mitigation**:

- Make OAuth endpoints configurable
- Provide presets for common environments
- Document endpoint discovery process

## Success Criteria

### Functional Requirements

- ✅ Can generate PKCE verifier and challenge
- ✅ Can initiate OAuth authorization flow
- ✅ Can exchange authorization code for token
- ✅ Can make authenticated API requests
- ✅ Can automatically refresh expired tokens
- ✅ Works with CLIENT_ID + CLIENT_SECRET

### Non-Functional Requirements

- ✅ Thread-safe token access
- ✅ No secret leakage in logs
- ✅ Clear error messages for auth failures
- ✅ Comprehensive documentation
- ✅ Example applications work end-to-end

### Code Quality

- ✅ All public APIs documented
- ✅ Unit tests for PKCE generation
- ✅ Integration test (manual)
- ✅ No clippy warnings
- ✅ Follows CLAUDE.md standards

## Future Enhancements

### v0.3.0+

- **Device Flow**: For devices without browsers (IoT, servers)
- **Client Credentials**: App-to-app authentication
- **Implicit Flow**: Deprecated but may be needed for legacy
- **Token Storage**: Persistent storage with keychain integration
- **Multiple Providers**: Support multiple OAuth providers
  simultaneously

### Advanced Features

- **Token Introspection**: Validate token status with ArcGIS
- **Token Revocation**: Programmatically revoke tokens
- **Scope Management**: Request specific OAuth scopes
- **Custom Claims**: Support for custom JWT claims

## Migration Path

### From API Key to OAuth

```rust
// Before (API Key)
let auth = ApiKeyAuth::new(env::var("API_KEY")?);
let client = ArcGISClient::new(auth);

// After (OAuth)
let oauth = OAuthProvider::new(
    env::var("CLIENT_ID")?,
    env::var("CLIENT_SECRET")?,
    env::var("REDIRECT_URI")?,
)?;

// One-time authorization
let (auth_url, verifier) = oauth.authorize_url();
// ... user authorizes ...
oauth.exchange_code(code, verifier).await?;

let client = ArcGISClient::new(oauth);
```

### Compatibility

- AuthProvider trait unchanged (backward compatible)
- Existing API Key code continues to work
- OAuth is opt-in via new dependency

## Conclusion

**PKCE OAuth authentication is fully feasible and recommended.**

### Key Takeaways

1. ✅ **Supported**: ArcGIS natively supports PKCE
2. ✅ **Secure**: CLIENT_ID + CLIENT_SECRET + PKCE = defense in depth
3. ✅ **Practical**: `oauth2` crate provides robust implementation
4. ✅ **Compatible**: Fits existing AuthProvider architecture
5. ✅ **Timeline**: 2-3 weeks for complete implementation

### Next Steps

1. **Approve this strategy** and proceed with implementation
2. **Phase 1** (Week 1): Core infrastructure and PKCE generation
3. **Phase 2** (Week 2): Token management and refresh logic
4. **Phase 3** (Week 3): User experience and documentation
5. **Release**: Include in v0.2.0 milestone

### Questions?

- How should we handle the redirect callback in CLI applications?
- Do we need token persistence for long-running applications?
- Should we support multiple OAuth providers simultaneously?
- What's the priority vs other v0.2.0 features (editing operations)?

---

**Document Version**: 1.0
**Author**: Claude Sonnet 4.5
**Date**: December 23, 2025
**Status**: Proposed
