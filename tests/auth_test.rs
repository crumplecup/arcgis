//! Tests for authentication providers.

mod common;

use arcgis::ApiKeyAuth;

#[tokio::test]
async fn test_api_key_auth() -> anyhow::Result<()> {
    common::init_tracing();
    tracing::info!("test_api_key_auth: Starting");

    use arcgis::AuthProvider;

    tracing::info!("test_api_key_auth: Creating API key auth");
    let auth = ApiKeyAuth::new("test_api_key");

    tracing::info!("test_api_key_auth: Getting token");
    let token = auth.get_token().await?;

    tracing::info!(token = %token, "test_api_key_auth: Received token");
    assert_eq!(token, "test_api_key");

    tracing::info!("test_api_key_auth: Completed");
    Ok(())
}
