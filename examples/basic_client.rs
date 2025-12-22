//! Basic example showing how to create an ArcGIS client.
//!
//! This example demonstrates:
//! - Creating an API Key authentication provider
//! - Creating an ArcGIS client
//!
//! Run with:
//! ```bash
//! cargo run --example basic_client
//! ```

use arcgis::{ApiKeyAuth, ArcGISClient};

fn main() {
    // Set up tracing subscriber for logging
    tracing_subscriber::fmt::init();

    // Create an API Key authentication provider
    // In a real application, load this from environment or config
    let auth = ApiKeyAuth::new("YOUR_API_KEY_HERE");

    // Create the ArcGIS client
    let client = ArcGISClient::new(auth);

    println!("ArcGIS client created successfully!");
    println!("HTTP client: {:?}", client.http());
    println!("Note: This is a basic example. Feature Service support coming soon!");
}
