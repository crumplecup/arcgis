//! Places Service.
//!
//! The Places Service provides access to points of interest (POI) data from ArcGIS.
//! Search for businesses, landmarks, and other locations with detailed information.
//!
//! # Operations
//!
//! - **Search**: Find places near a point with filters
//! - **Details**: Get extended information about specific places
//! - **Categories**: List all available POI categories
//!
//! # Example
//!
//! ```no_run
//! use arcgis::{ApiKeyAuth, ArcGISClient, PlacesClient, PlaceSearchParametersBuilder};
//!
//! # async fn example() -> arcgis::Result<()> {
//! let auth = ApiKeyAuth::new("YOUR_API_KEY");
//! let client = ArcGISClient::new(auth);
//! let places = PlacesClient::new(&client);
//!
//! // Find restaurants within 1km
//! let params = PlaceSearchParametersBuilder::default()
//!     .x(-118.2437)
//!     .y(34.0522)
//!     .radius(1000.0)
//!     .search_text("restaurant")
//!     .page_size(20u32)
//!     .build()
//!     .expect("Valid parameters");
//!
//! let results = places.find_places_near_point(params).await?;
//!
//! for place in results.results() {
//!     tracing::info!(
//!         name = %place.name(),
//!         distance = ?place.distance(),
//!         "Found restaurant"
//!     );
//! }
//! # Ok(())
//! # }
//! ```

mod client;
mod types;

pub use client::PlacesClient;
pub use types::{
    CategoriesResult, CategoryInfo, DayHours, PlaceAddress, PlaceCategory, PlaceContactInfo,
    PlaceDetailsResult, PlaceHours, PlaceInfo, PlaceRating, PlaceSearchParameters,
    PlaceSearchParametersBuilder, PlaceSearchResult,
};
