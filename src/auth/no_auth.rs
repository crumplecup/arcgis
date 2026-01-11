//! No authentication provider for public ArcGIS services.

use super::AuthProvider;
use crate::{Error, ErrorKind, Result};
use async_trait::async_trait;

/// No authentication provider for accessing public ArcGIS services.
///
/// Many ArcGIS services are publicly accessible without authentication.
/// This provider allows querying public feature services, map services, etc.
///
/// # Example
///
/// ```
/// use arcgis::{ArcGISClient, NoAuth, FeatureServiceClient, LayerId};
///
/// # async fn example() -> arcgis::Result<()> {
/// // Create client without authentication
/// let client = ArcGISClient::new(NoAuth);
///
/// // Query public World Cities service
/// let service = FeatureServiceClient::new(
///     "https://services.arcgis.com/P3ePLMYs2RVChkJx/arcgis/rest/services/World_Cities/FeatureServer",
///     &client,
/// );
///
/// let features = service
///     .query(LayerId::new(0))
///     .where_clause("POP > 5000000")
///     .limit(10)
///     .execute()
///     .await?;
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone, Copy, Default)]
pub struct NoAuth;

#[async_trait]
impl AuthProvider for NoAuth {
    async fn get_token(&self) -> Result<String> {
        Err(Error::from(ErrorKind::Validation(
            "NoAuth provider does not provide tokens. Use for public services only.".to_string(),
        )))
    }

    fn requires_token_param(&self) -> bool {
        false
    }
}
