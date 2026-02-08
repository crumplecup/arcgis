//! Elevation Service.
//!
//! The Elevation Service provides terrain analysis operations including
//! elevation profiles, elevation statistics, and viewshed analysis.
//!
//! # Operations
//!
//! - **Profile**: Generate elevation profiles along lines or points
//! - **Summarize Elevation**: Compute elevation statistics within polygons
//! - **Viewshed**: Perform viewshed analysis from observer points
//!
//! # Example
//!
//! ```no_run
//! use arcgis::{ApiKeyAuth, ArcGISClient, ElevationClient, ProfileParametersBuilder};
//!
//! # async fn example() -> arcgis::Result<()> {
//! let auth = ApiKeyAuth::new("YOUR_API_KEY");
//! let client = ArcGISClient::new(auth);
//! let elevation = ElevationClient::new(&client);
//!
//! // Generate elevation profile along a line (FeatureSet JSON with a polyline)
//! let line_features = r#"{"geometryType":"esriGeometryPolyline","features":[{"geometry":{"paths":[[[-120.5,38.5],[-120.0,39.0]]]}}],"spatialReference":{"wkid":4326}}"#;
//! let params = ProfileParametersBuilder::default()
//!     .input_line_features(line_features)
//!     .dem_resolution("30m")
//!     .build()
//!     .expect("Valid parameters");
//!
//! let result = elevation.profile(params).await?;
//!
//! tracing::info!(
//!     point_count = result.output_profile().features().len(),
//!     "Elevation profile generated"
//! );
//! # Ok(())
//! # }
//! ```

mod client;
mod types;

pub use client::ElevationClient;
pub use types::{
    DemResolution, ElevationPoint, ProfileParameters, ProfileParametersBuilder, ProfileResult,
    SummarizeElevationParameters, SummarizeElevationParametersBuilder, SummarizeElevationResult,
    ViewshedParameters, ViewshedParametersBuilder, ViewshedResult,
};
