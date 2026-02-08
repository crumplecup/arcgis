//! Geometry Service for geometric operations.
//!
//! The Geometry Service provides operations for:
//! - **Projection & Transformation**: Convert geometries between spatial reference systems
//! - **Geometric Operations**: Buffer, union, intersect, difference, simplify, etc.
//! - **Measurements**: Calculate areas, lengths, distances
//! - **Spatial Analysis**: Label points, convex hull, spatial relationships
//!
//! # Example
//!
//! ```no_run
//! use arcgis::{ApiKeyAuth, ArcGISClient, GeometryServiceClient, ArcGISPoint, ArcGISGeometry};
//!
//! # async fn example() -> arcgis::Result<()> {
//! let auth = ApiKeyAuth::new("YOUR_API_KEY");
//! let client = ArcGISClient::new(auth);
//! let geometry_service = GeometryServiceClient::new(
//!     "https://utility.arcgisonline.com/arcgis/rest/services/Geometry/GeometryServer",
//!     &client
//! );
//!
//! // Project a point from WGS84 to Web Mercator
//! let point = ArcGISPoint::new(-122.4194, 37.7749);
//! let projected = geometry_service.project(vec![ArcGISGeometry::Point(point)], 4326, 3857).await?;
//! # Ok(())
//! # }
//! ```

mod client;
mod types;

pub use client::GeometryServiceClient;
pub use types::{
    AreaUnit, AreasAndLengthsParameters, AreasAndLengthsParametersBuilder, AreasAndLengthsResult,
    BufferParameters, BufferParametersBuilder, BufferResult, CalculationType, DistanceParameters,
    DistanceParametersBuilder, DistanceResult, LinearUnit, ProjectParameters,
    ProjectParametersBuilder, ProjectResult, SimplifyParameters, SimplifyParametersBuilder,
    SimplifyResult, Transformation, UnionParameters, UnionParametersBuilder, UnionResult,
};
