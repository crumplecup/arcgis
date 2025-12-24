# ArcGIS Authentication Strategy

## Executive Summary

**Goal**: Implement automated authentication for server-to-server and CLI applications **without requiring human interaction**.

**Authentication Methods**:
1. ✅ **API Key** (Already implemented) - Simple, long-lived tokens
2. ⚠️ **OAuth Client Credentials** (Needs implementation) - Automated server-to-server
3. ❌ **OAuth PKCE** (Wrong approach) - Requires browser/user interaction

**Priority**: P0 - BLOCKING for automated testing and deployment

---

## Authentication Methods Comparison

### 1. API Key Authentication ✅

**Status**: Already implemented in `src/auth/api_key.rs`

**How it works**:
- Long-lived access token (up to 1 year)
- Passed directly in API requests
- No token exchange required

**Use cases**:
- Client-side applications
- Public applications accessing location services
- Quick prototyping and development
- Simple CLI tools

**Pros**:
- Simplest to implement and use
- No authentication flow required
- Good for development/testing
- Built-in key rotation support (as of 2025)

**Cons**:
- Less secure than OAuth for server deployments
- Credential exposure risk if embedded in client apps
- Limited to read-only operations in some contexts

**Example**:
```rust
use arcgis::{ApiKeyAuth, ArcGISClient};

let auth = ApiKeyAuth::new(std::env::var("ARCGIS_API_KEY")?);
let client = ArcGISClient::new(auth);
```

**Configuration**:
```bash
# .env
ARCGIS_API_KEY=your_api_key_here
```

---

### 2. OAuth Client Credentials Flow ⚠️ (NEEDS IMPLEMENTATION)

**Status**: Not yet implemented - **THIS IS WHAT WE NEED**

**How it works**:
1. Application makes POST request to token endpoint
2. Sends `client_id`, `client_secret`, `grant_type=client_credentials`
3. Receives short-lived access token (typically 2 hours)
4. Token refresh handled automatically

**Use cases**:
- **Server applications** (our primary use case)
- **Console scripts and automation**
- **CI/CD pipelines**
- **Backend services**
- **Any scenario without human interaction**

**Pros**:
- ✅ **No human interaction required** - fully automated
- ✅ More secure than API keys for servers
- ✅ Short-lived tokens reduce exposure risk
- ✅ Standard OAuth 2.0 flow
- ✅ Suitable for production deployments

**Cons**:
- Requires token refresh logic
- Slightly more complex than API keys
- Client secret must be secured

**Implementation Required**:

```rust
// src/auth/client_credentials.rs
use oauth2::{
    basic::BasicClient,
    ClientId, ClientSecret, TokenUrl,
    ClientCredentialsFlow,
};

pub struct ClientCredentialsAuth {
    client: BasicClient,
    http_client: reqwest::Client,
    token: Arc<RwLock<Option<StandardTokenResponse>>>,
}

impl ClientCredentialsAuth {
    pub fn new(client_id: String, client_secret: String) -> Result<Self> {
        let token_url = TokenUrl::new(
            "https://www.arcgis.com/sharing/rest/oauth2/token".to_string()
        )?;

        let client = BasicClient::new(ClientId::new(client_id))
            .set_client_secret(ClientSecret::new(client_secret))
            .set_token_uri(token_url);

        let http_client = reqwest::Client::builder()
            .redirect(reqwest::redirect::Policy::none())
            .build()?;

        Ok(Self {
            client,
            http_client,
            token: Arc::new(RwLock::new(None)),
        })
    }

    async fn fetch_token(&self) -> Result<()> {
        let token_response = self.client
            .exchange_client_credentials()
            .request_async(&self.http_client)
            .await?;

        *self.token.write().await = Some(token_response);
        Ok(())
    }
}

#[async_trait]
impl AuthProvider for ClientCredentialsAuth {
    async fn get_token(&self) -> Result<String> {
        let token_guard = self.token.read().await;

        if let Some(token) = token_guard.as_ref() {
            // Check if expired (5-minute buffer)
            if let Some(expires_in) = token.expires_in() {
                if expires_in < Duration::from_secs(300) {
                    drop(token_guard);
                    self.fetch_token().await?;

                    let new_guard = self.token.read().await;
                    return Ok(new_guard.as_ref().unwrap()
                        .access_token().secret().clone());
                }
            }
            Ok(token.access_token().secret().clone())
        } else {
            drop(token_guard);
            self.fetch_token().await?;

            let guard = self.token.read().await;
            Ok(guard.as_ref().unwrap().access_token().secret().clone())
        }
    }
}
```

**Configuration**:
```bash
# .env
CLIENT_ID=your_client_id
CLIENT_SECRET=your_client_secret
```

**API Request Details**:

Endpoint: `https://www.arcgis.com/sharing/rest/oauth2/token`

Method: POST

Headers: `Content-Type: application/x-www-form-urlencoded`

Body:
```
client_id=YOUR_CLIENT_ID
client_secret=YOUR_CLIENT_SECRET
grant_type=client_credentials
```

Response:
```json
{
  "access_token": "...",
  "expires_in": 7200,
  "token_type": "Bearer"
}
```

---

### 3. OAuth Authorization Code with PKCE ❌ (WRONG APPROACH)

**Status**: Currently implemented in `src/auth/oauth.rs` - **SHOULD BE REMOVED OR ARCHIVED**

**Why it's wrong**:
- ❌ **Requires human browser interaction**
- ❌ User must manually authorize in browser
- ❌ Cannot be automated
- ❌ Unsuitable for CI/CD, servers, scripts
- ❌ Blocks our development workflow

**When PKCE IS appropriate**:
- User-facing web applications
- Mobile applications
- Desktop applications with GUI
- Applications distributed to end users
- **NOT for our use case**

**Action required**: Archive or remove this implementation

---

## Implementation Plan

### Phase 1: Client Credentials Flow (Week 1)

**Tasks**:
- [ ] Create `src/auth/client_credentials.rs`
- [ ] Implement `ClientCredentialsAuth` struct
- [ ] Implement automatic token fetching on first use
- [ ] Implement automatic token refresh (5-minute buffer)
- [ ] Add `#[instrument]` to all methods
- [ ] Thread-safe token storage with `Arc<RwLock>`
- [ ] Implement `AuthProvider` trait
- [ ] Add error handling for token fetch failures

**Deliverables**:
```rust
pub use auth::{ApiKeyAuth, ClientCredentialsAuth, AuthProvider};
```

### Phase 2: Testing & Examples (Week 1)

**Tasks**:
- [ ] Create example `examples/client_credentials_flow.rs`
- [ ] Test with real ArcGIS credentials
- [ ] Verify token refresh works automatically
- [ ] Add integration test (manual)
- [ ] Document usage in README

**Example**:
```rust
// examples/client_credentials_flow.rs
use arcgis::{ClientCredentialsAuth, ArcGISClient, AuthProvider};

#[tokio::main]
async fn main() -> arcgis::Result<()> {
    dotenvy::dotenv().ok();

    let client_id = std::env::var("CLIENT_ID")?;
    let client_secret = std::env::var("CLIENT_SECRET")?;

    println!("Creating OAuth client credentials authenticator...");
    let auth = ClientCredentialsAuth::new(client_id, client_secret)?;

    println!("Fetching access token...");
    let token = auth.get_token().await?;
    println!("Token obtained: {}...", &token[..20]);

    println!("Creating ArcGIS client...");
    let client = ArcGISClient::new(auth);

    // Use client for API calls - token refresh is automatic
    // let features = client.feature_service(url).layer(0).query()...

    println!("Authentication successful!");
    Ok(())
}
```

### Phase 3: Cleanup (Week 1)

**Tasks**:
- [ ] Archive PKCE implementation or move to `auth/user_auth.rs`
- [ ] Update documentation to clarify authentication methods
- [ ] Remove `examples/oauth_flow.rs` or mark as "user auth only"
- [ ] Update PLANNING_INDEX.md
- [ ] Run full test suite

---

## Decision Matrix: When to Use Each Method

| Scenario | Recommended Auth | Reason |
|----------|------------------|--------|
| **Server application** | Client Credentials | Automated, no human interaction |
| **CLI automation/scripts** | Client Credentials | Automated, no human interaction |
| **CI/CD pipeline** | Client Credentials | Automated, no human interaction |
| **Development/testing** | API Key | Simplest, fastest setup |
| **Simple read-only client** | API Key | Adequate security, easy to use |
| **User-facing web app** | PKCE (future) | User login required |
| **Mobile app** | PKCE (future) | User login required |
| **Desktop GUI app** | PKCE (future) | User login required |

**For this project**: We need **Client Credentials** because we're building server tools and automation.

---

## Security Considerations

### API Key
- ✅ Store in `.env`, never commit
- ✅ Rotate keys regularly (new feature supports this)
- ✅ Use different keys for dev/prod
- ⚠️ Avoid embedding in client-side code

### Client Credentials
- ✅ Store `CLIENT_SECRET` securely (environment variable, secrets manager)
- ✅ Never log or expose client secret
- ✅ Use HTTPS only (enforced by ArcGIS)
- ✅ Rotate credentials periodically
- ✅ Monitor token refresh failures
- ✅ Implement exponential backoff for token requests

---

## Testing Strategy

### Manual Testing
1. Verify client credentials auth works with real ArcGIS account
2. Verify token refresh happens automatically
3. Verify token expiration is handled correctly
4. Test error scenarios (invalid credentials, network failure)

### Integration Testing
```rust
#[tokio::test]
#[ignore] // Requires real credentials
async fn test_client_credentials_flow() -> arcgis::Result<()> {
    let auth = ClientCredentialsAuth::new(
        std::env::var("CLIENT_ID")?,
        std::env::var("CLIENT_SECRET")?,
    )?;

    // First token fetch
    let token1 = auth.get_token().await?;
    assert!(!token1.is_empty());

    // Second call should use cached token
    let token2 = auth.get_token().await?;
    assert_eq!(token1, token2);

    Ok(())
}
```

---

## Success Criteria

- ✅ Can authenticate without human interaction
- ✅ Token refresh is automatic
- ✅ Works in CI/CD environments
- ✅ Works in server deployments
- ✅ Example demonstrates usage
- ✅ Documentation explains both auth methods
- ✅ Integration test passes with real credentials

---

## Migration Path

### From Current (Broken) PKCE Implementation

**Before** (requires browser):
```rust
let oauth = OAuthProvider::new(...)?;
let (url, session) = oauth.authorize_url();
// ❌ User must click URL and authorize in browser
```

**After** (fully automated):
```rust
let auth = ClientCredentialsAuth::new(
    env::var("CLIENT_ID")?,
    env::var("CLIENT_SECRET")?,
)?;
// ✅ No user interaction required
let client = ArcGISClient::new(auth);
```

---

## References

**ArcGIS Documentation**:
- [Client credentials flow](https://developers.arcgis.com/documentation/security-and-authentication/app-authentication/client-credentials-flow/)
- [Types of authentication](https://developers.arcgis.com/documentation/security-and-authentication/types-of-authentication/)
- [Introduction to app authentication](https://developers.arcgis.com/documentation/security-and-authentication/app-authentication/)
- [API key authentication](https://developers.arcgis.com/documentation/security-and-authentication/api-key-authentication/)

**oauth2 Crate**:
- Documentation: https://docs.rs/oauth2/
- Client Credentials: `ClientCredentialsFlow` trait

---

**Document Version**: 1.0
**Author**: Claude Sonnet 4.5
**Date**: December 24, 2025
**Status**: Active
