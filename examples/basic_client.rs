//! Basic example showing how to create an ArcGIS client with API key authentication.
//!
//! This example demonstrates:
//! - Loading API keys from environment variables (secure pattern)
//! - Creating an API Key authentication provider
//! - Creating an ArcGIS client
//!
//! # Setup
//!
//! Create a `.env` file in the project root:
//!
//! ```env
//! ARCGIS_API_KEY=your_api_key_here
//! ```
//!
//! The `.env` file is automatically loaded when you create an ArcGIS client.
//! This keeps your credentials out of version control (make sure `.env` is in `.gitignore`).
//!
//! # Running
//!
//! ```bash
//! cargo run --example basic_client
//! ```

use arcgis::{ApiKeyAuth, ArcGISClient};

fn main() {
    // Set up tracing subscriber for logging
    tracing_subscriber::fmt::init();

    // Load API key from environment
    // The .env file is automatically loaded by ArcGISClient::new()
    let api_key = std::env::var("ARCGIS_API_KEY").expect(
        "ARCGIS_API_KEY must be set in .env file or environment.\n\
         Create a .env file with: ARCGIS_API_KEY=your_api_key_here"
    );

    // Create an API Key authentication provider
    let auth = ApiKeyAuth::new(api_key);

    // Create the ArcGIS client (automatically loads .env on first use)
    let client = ArcGISClient::new(auth);

    println!("✅ ArcGIS client created successfully!");
    println!("📡 HTTP client ready: {:?}", client.http());
    println!("\n💡 Next steps:");
    println!("   - Check out examples/client_credentials_flow.rs for OAuth");
    println!("   - See examples/edit_session.rs for feature editing");
}
