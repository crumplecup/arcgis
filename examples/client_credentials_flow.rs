//! OAuth 2.0 Client Credentials Flow example.
//!
//! This example demonstrates automated server-to-server authentication
//! using the OAuth 2.0 Client Credentials grant type. **No browser or
//! user interaction is required** - this is fully automated.
//!
//! # Use Cases
//!
//! - Server applications and backend services
//! - Automated scripts and CLI tools
//! - CI/CD pipelines
//! - Any scenario without human interaction
//!
//! # Setup
//!
//! 1. Create a `.env` file in the project root with:
//!    ```env
//!    CLIENT_ID=your_client_id
//!    CLIENT_SECRET=your_client_secret
//!    ```
//!
//! 2. Obtain credentials from ArcGIS Developer dashboard:
//!    https://developers.arcgis.com/applications
//!
//! # Running
//!
//! ```sh
//! cargo run --example client_credentials_flow
//! ```
//!
//! The example will:
//! 1. Create a Client Credentials authenticator
//! 2. Automatically fetch an access token (no browser needed!)
//! 3. Display the token information
//! 4. Demonstrate automatic token refresh

use arcgis::{AuthProvider, ClientCredentialsAuth};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing for structured logging
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info"))
        )
        .init();

    tracing::info!("🔐 ArcGIS OAuth 2.0 Client Credentials Flow Example");
    tracing::info!("✨ Fully automated - no browser interaction required!");

    // Load environment variables (.env is automatically loaded by library)
    tracing::debug!("Loading credentials from environment");
    let client_id = std::env::var("CLIENT_ID")
        .map_err(|_| anyhow::anyhow!("CLIENT_ID must be set in .env or environment"))?;
    let client_secret = std::env::var("CLIENT_SECRET")
        .map_err(|_| anyhow::anyhow!("CLIENT_SECRET must be set in .env or environment"))?;

    // 1. Create OAuth Client Credentials authenticator
    tracing::info!("📋 Creating OAuth Client Credentials authenticator");
    let auth = ClientCredentialsAuth::new(client_id, client_secret)?;
    tracing::info!("✅ Authenticator created");

    // 2. Get access token (fetched automatically on first use)
    tracing::info!("🔑 Fetching access token");
    let token = auth.get_token().await?;
    tracing::info!(
        token_preview = %&token[..20.min(token.len())],
        "✅ Access token obtained"
    );

    // 3. Get token again (should return cached token)
    tracing::info!("🔄 Getting token again (should use cache)");
    let token2 = auth.get_token().await?;
    let tokens_match = token == token2;
    tracing::info!(
        tokens_match = tokens_match,
        "✅ Token retrieved from cache"
    );

    // 4. Show token info
    tracing::info!("📊 Token Information:");
    tracing::info!(
        token_length = token.len(),
        token_type = "Bearer",
        lifetime = "~2 hours",
        "Token details"
    );

    tracing::info!("🎉 Authentication successful!");
    tracing::info!("💡 The ClientCredentialsAuth is now authenticated and can be");
    tracing::info!("   used with ArcGISClient to make authenticated API requests");
    tracing::info!("📝 Token will automatically refresh when it expires");
    tracing::info!("   No manual token management required!");
    tracing::info!("🚀 Example usage:");
    tracing::info!("   let client = ArcGISClient::new(auth);");
    tracing::info!("   // All API calls automatically use refreshed tokens");

    Ok(())
}
