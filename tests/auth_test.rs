//! Tests for authentication providers.

use arcgis::{ApiKeyAuth, Result};

#[tokio::test]
async fn test_api_key_auth() -> Result<()> {
    use arcgis::AuthProvider;

    let auth = ApiKeyAuth::new("test_api_key");
    let token = auth.get_token().await?;
    assert_eq!(token, "test_api_key");
    Ok(())
}
