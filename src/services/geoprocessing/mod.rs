//! Geoprocessing Service.
//!
//! The Geoprocessing Service (GPServer) provides operations for:
//! - **Execute**: Run synchronous geoprocessing tasks
//! - **Submit Job**: Run asynchronous geoprocessing tasks
//! - **Job Management**: Check status, retrieve results, cancel jobs
//!
//! # Example
//!
//! ```no_run
//! use arcgis::{ApiKeyAuth, ArcGISClient, GeoprocessingServiceClient};
//! use std::collections::HashMap;
//!
//! # async fn example() -> arcgis::Result<()> {
//! let auth = ApiKeyAuth::new("YOUR_API_KEY");
//! let client = ArcGISClient::new(auth);
//! let gp_service = GeoprocessingServiceClient::new(
//!     "https://sampleserver6.arcgisonline.com/arcgis/rest/services/Elevation/ESRI_Elevation_World/GPServer/ProfileService",
//!     &client
//! );
//!
//! // Execute a synchronous geoprocessing task
//! let mut params = HashMap::new();
//! params.insert("InputLineFeatures".to_string(), serde_json::json!({
//!     "geometryType": "esriGeometryPolyline",
//!     "features": []
//! }));
//!
//! let result = gp_service.execute(params).await?;
//! # Ok(())
//! # }
//! ```

mod client;
mod parameters;
mod types;

pub use client::GeoprocessingServiceClient;
pub use parameters::{
    GPBoolean, GPDataFile, GPDate, GPDouble, GPFeatureRecordSetLayer, GPLinearUnit, GPLong,
    GPParameter, GPRasterDataLayer, GPString,
};
pub use types::{
    GPExecuteResult, GPJobInfo, GPJobStatus, GPMessage, GPMessageType, GPProgress,
    GPResultParameter,
};
