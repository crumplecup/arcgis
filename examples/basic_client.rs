//! Basic example showing how to create an ArcGIS client with API key authentication.
//!
//! This example demonstrates:
//! - Loading API keys from environment variables (secure pattern)
//! - Creating an API Key authentication provider
//! - Creating an ArcGIS client
//! - Proper error handling with anyhow
//! - Structured logging with tracing
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

fn main() -> anyhow::Result<()> {
    // Initialize tracing subscriber for structured logging
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    tracing::info!("Starting basic_client example");

    // Load API key from environment (.env file automatically loaded)
    tracing::info!("Creating API key authentication provider from environment");
    let auth = ApiKeyAuth::from_env()?;

    // Create the ArcGIS client
    tracing::info!("Creating ArcGIS client");
    let client = ArcGISClient::new(auth);

    tracing::info!("ArcGIS client created successfully");
    tracing::debug!(http_client = ?client.http(), "HTTP client ready");

    tracing::info!("Next steps:");
    tracing::info!("  - Check out examples/client_credentials_flow.rs for OAuth");
    tracing::info!("  - See examples/edit_session.rs for feature editing");

    Ok(())
}
