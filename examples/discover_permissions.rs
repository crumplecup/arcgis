//! Discover what permissions each API key has by calling /community/self
//!
//! This tool helps you:
//! 1. See which privileges each configured key has
//! 2. Identify which permissions to add to the Permission enum
//! 3. Verify your key configuration is correct
//!
//! # Running
//!
//! ```bash
//! cargo run --example discover_permissions
//!
//! # Save output to file
//! cargo run --example discover_permissions > discovered_permissions.txt
//! ```

use arcgis::EnvConfig;
use serde::Deserialize;
use std::collections::HashSet;

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum SelfResponse {
    User(UserResponse),
    App(AppResponse),
}

#[derive(Debug, Deserialize)]
struct UserResponse {
    username: String,
    role: String,
    privileges: Vec<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct AppResponse {
    app_info: AppInfo,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct AppInfo {
    app_id: String,
    app_title: String,
    app_owner: String,
    privileges: Vec<String>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("warn")),
        )
        .init();

    println!("🔍 ArcGIS API Key Permission Discovery\n");
    println!("Querying /community/self for each configured key...\n");
    println!("{}", "=".repeat(80));

    let config = EnvConfig::global();
    let http = reqwest::Client::new();

    let keys = [
        ("ARCGIS_PUBLIC_KEY", &config.arcgis_public_key),
        ("ARCGIS_LOCATION_KEY", &config.arcgis_location_key),
        ("ARCGIS_SPATIAL_KEY", &config.arcgis_spatial_key),
        ("ARCGIS_GENERAL_KEY", &config.arcgis_general_key),
        ("ARCGIS_ADMIN_KEY", &config.arcgis_admin_key),
        ("ARCGIS_API_KEY", &config.arcgis_api_key),
        ("ARCGIS_ENTERPRISE_KEY", &config.arcgis_enterprise_key),
    ];

    let mut all_privileges: HashSet<String> = HashSet::new();
    let mut successful_queries = 0;

    for (name, key_opt) in keys {
        if let Some(key) = key_opt {
            println!("\n🔑 {}", name);
            println!("   Calling /portals/self...");

            match query_key_privileges(&http, key).await {
                Ok(response) => {
                    successful_queries += 1;
                    println!("   ✅ Success");

                    let (name_or_id, privileges) = match response {
                        SelfResponse::User(user) => {
                            println!("   Type: User");
                            println!("   Username: {}", user.username);
                            println!("   Role: {}", user.role);
                            (user.username, user.privileges)
                        }
                        SelfResponse::App(app) => {
                            println!("   Type: API Key");
                            println!("   App: {}", app.app_info.app_title);
                            println!("   Owner: {}", app.app_info.app_owner);
                            (app.app_info.app_id, app.app_info.privileges)
                        }
                    };

                    println!("   Privileges ({})", privileges.len());

                    if privileges.is_empty() {
                        println!("      (No privileges - public access only)");
                    } else {
                        for privilege in &privileges {
                            println!("      - {}", privilege);
                            all_privileges.insert(privilege.clone());
                        }
                    }
                }
                Err(e) => {
                    println!("   ❌ Error: {}", e);
                }
            }
        } else {
            println!("\n⚠️  {} not configured (skipped)", name);
        }
    }

    println!("\n{}", "=".repeat(80));
    println!("\n📊 Summary:");
    println!("   Keys queried: {}", successful_queries);
    println!("   Unique privileges discovered: {}", all_privileges.len());

    if !all_privileges.is_empty() {
        println!("\n📋 All Unique Privileges:\n");
        let mut sorted: Vec<_> = all_privileges.iter().collect();
        sorted.sort();

        // Group by category
        let mut categories: std::collections::HashMap<String, Vec<&String>> =
            std::collections::HashMap::new();

        for priv_str in sorted {
            let category = priv_str
                .split(':')
                .next()
                .unwrap_or("unknown")
                .to_string();
            categories.entry(category).or_default().push(priv_str);
        }

        for (category, privileges_list) in categories.iter() {
            println!("  {}:", category);
            for privilege in privileges_list {
                println!("    - {}", privilege);
            }
            println!();
        }

        println!("\n💡 Next Steps:");
        println!("   1. Use this list to build the Permission enum in src/auth/permissions.rs");
        println!("   2. Map each privilege string to an enum variant");
        println!("   3. Implement from_esri_string() and to_esri_string() conversions");
    } else {
        println!("\n⚠️  No privileges discovered. Configure at least one API key in .env");
        println!("   Example: ARCGIS_GENERAL_KEY=your_api_key_here");
    }

    Ok(())
}

async fn query_key_privileges(
    http: &reqwest::Client,
    key: &secrecy::SecretString,
) -> anyhow::Result<SelfResponse> {
    use secrecy::ExposeSecret;

    let response = http
        .get("https://www.arcgis.com/sharing/rest/community/self")
        .query(&[("f", "json"), ("token", key.expose_secret())])
        .send()
        .await?;

    let status = response.status();
    if !status.is_success() {
        let error_text = response.text().await?;
        anyhow::bail!("HTTP {}: {}", status, error_text);
    }

    // Get raw text first for debugging
    let text = response.text().await?;

    // /community/self returns either User or AppInfo depending on auth type
    match serde_json::from_str::<SelfResponse>(&text) {
        Ok(response) => Ok(response),
        Err(e) => {
            // Print first 500 chars of response for debugging
            eprintln!("Debug - Response text (first 500 chars):\n{}", &text[..text.len().min(500)]);
            anyhow::bail!("Failed to decode response: {}", e)
        }
    }
}
