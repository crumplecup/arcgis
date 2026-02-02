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
//! // Generate elevation profile along a line
//! let params = ProfileParametersBuilder::default()
//!     .input_geometry("{\"paths\":[[[-120.5,38.5],[-120.0,39.0]]]}")
//!     .geometry_type("esriGeometryPolyline")
//!     .dem_resolution("30m")
//!     .return_first_point(true)
//!     .return_last_point(true)
//!     .build()
//!     .expect("Valid parameters");
//!
//! let result = elevation.profile(params).await?;
//!
//! if let Some(first_z) = result.first_point_z() {
//!     tracing::info!(elevation = first_z, "First point elevation");
//! }
//! if let Some(last_z) = result.last_point_z() {
//!     tracing::info!(elevation = last_z, "Last point elevation");
//! }
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
