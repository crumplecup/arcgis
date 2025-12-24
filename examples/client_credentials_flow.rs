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
async fn main() -> arcgis::Result<()> {
    // Initialize tracing for observability
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    // Load environment variables from .env file
    dotenvy::dotenv().ok();

    let client_id =
        std::env::var("CLIENT_ID").expect("CLIENT_ID must be set in .env or environment");
    let client_secret =
        std::env::var("CLIENT_SECRET").expect("CLIENT_SECRET must be set in .env or environment");

    println!("🔐 ArcGIS OAuth 2.0 Client Credentials Flow Example\n");
    println!("✨ Fully automated - no browser interaction required!\n");

    // 1. Create OAuth Client Credentials authenticator
    println!("📋 Creating OAuth Client Credentials authenticator...");
    let auth = ClientCredentialsAuth::new(client_id, client_secret)?;
    println!("✅ Authenticator created\n");

    // 2. Get access token (fetched automatically on first use)
    println!("🔑 Fetching access token...");
    let token = auth.get_token().await?;
    println!("✅ Access token obtained");
    println!("   Token (first 20 chars): {}...\n", &token[..20]);

    // 3. Get token again (should return cached token)
    println!("🔄 Getting token again (should use cache)...");
    let token2 = auth.get_token().await?;
    println!("✅ Token retrieved from cache");
    println!("   Tokens match: {}\n", token == token2);

    // 4. Show token info
    println!("📊 Token Information:");
    println!("   Length: {} characters", token.len());
    println!("   Type: Bearer token (for Authorization header)");
    println!("   Lifetime: ~2 hours (refreshed automatically)\n");

    println!("🎉 Authentication successful!");
    println!("\n💡 The ClientCredentialsAuth is now authenticated and can be");
    println!("   used with ArcGISClient to make authenticated API requests.");
    println!("\n📝 Token will automatically refresh when it expires.");
    println!("   No manual token management required!");

    println!("\n🚀 Example usage with ArcGIS client:");
    println!("   let client = ArcGISClient::new(auth);");
    println!("   // All API calls automatically use refreshed tokens");

    Ok(())
}
