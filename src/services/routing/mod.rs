//! Routing and Network Analysis Service.
//!
//! The Routing Service (Network Analyst Server) provides operations for:
//! - **Route**: Calculate optimal routes between multiple stops
//! - **Service Area**: Generate drive-time or distance polygons
//! - **Closest Facility**: Find nearest facilities from incidents
//! - **OD Cost Matrix**: Compute origin-destination cost matrices
//!
//! # Example
//!
//! ```no_run
//! use arcgis::{ApiKeyAuth, ArcGISClient, RoutingServiceClient, ArcGISPoint};
//!
//! # async fn example() -> arcgis::Result<()> {
//! let auth = ApiKeyAuth::new("YOUR_API_KEY");
//! let client = ArcGISClient::new(auth);
//! let routing_service = RoutingServiceClient::new(
//!     "https://route.arcgis.com/arcgis/rest/services/World/Route/NAServer/Route_World",
//!     &client
//! );
//!
//! // Calculate a route between two points
//! // (Route implementation coming next)
//! # Ok(())
//! # }
//! ```

mod client;
mod types;

pub use client::RoutingServiceClient;
pub use types::{
    BarrierType, ClosestFacilityParameters, ClosestFacilityParametersBuilder,
    ClosestFacilityResult, CurbApproach, DirectionsLength, DirectionsStyle,
    DirectionsTimeAttribute, ImpedanceAttribute, NALocation, ODCostMatrixParameters,
    ODCostMatrixParametersBuilder, ODCostMatrixResult, OutputLine, RestrictionAttribute,
    RouteParameters, RouteParametersBuilder, RouteResult, RouteShape, ServiceAreaParameters,
    ServiceAreaParametersBuilder, ServiceAreaResult, TravelDirection, TravelMode, UTurnPolicy,
};
