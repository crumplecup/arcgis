//! Tests for authentication providers.

use arcgis::ApiKeyAuth;

#[tokio::test]
async fn test_api_key_auth() {
    use arcgis::AuthProvider;

    let auth = ApiKeyAuth::new("test_api_key");
    let token = auth.get_token().await.unwrap();
    assert_eq!(token, "test_api_key");
}
