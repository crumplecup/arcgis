//! Geocoding Service operations.
//!
//! The Geocoding Service converts addresses to geographic coordinates (geocoding),
//! coordinates to addresses (reverse geocoding), and provides autocomplete suggestions
//! for partial addresses.

mod client;
mod types;

pub use client::GeocodeServiceClient;
pub use types::{
    AddressCandidate, Category, Extent, GeocodeAddress, GeocodeResponse, LocationType,
    ReverseGeocodeResponse, Suggestion, SuggestResponse,
};
