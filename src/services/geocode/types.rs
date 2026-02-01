//! Geocoding service types.

use crate::{ArcGISPointV2 as ArcGISPoint, SpatialReferenceV2 as SpatialReference};
use derive_getters::Getters;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A candidate address returned from geocoding.
///
/// Contains the matched address, location, score, and detailed attributes.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Getters)]
pub struct AddressCandidate {
    /// The matched address string
    address: String,

    /// Geographic location of the address
    location: ArcGISPoint,

    /// Match score (0-100, where 100 is a perfect match)
    score: f64,

    /// Detailed attributes about the address
    #[serde(default)]
    attributes: HashMap<String, serde_json::Value>,

    /// Bounding box extent of the address
    #[serde(skip_serializing_if = "Option::is_none")]
    extent: Option<Extent>,
}

/// Bounding box extent.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, Getters)]
pub struct Extent {
    /// Minimum X coordinate
    xmin: f64,
    /// Minimum Y coordinate
    ymin: f64,
    /// Maximum X coordinate
    xmax: f64,
    /// Maximum Y coordinate
    ymax: f64,
}

/// Response from findAddressCandidates operation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Getters)]
#[serde(rename_all = "camelCase")]
pub struct GeocodeResponse {
    /// List of address candidates
    candidates: Vec<AddressCandidate>,

    /// Spatial reference of results
    #[serde(skip_serializing_if = "Option::is_none")]
    spatial_reference: Option<SpatialReference>,
}

/// Response from reverseGeocode operation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Getters)]
pub struct ReverseGeocodeResponse {
    /// The address found at the location
    address: GeocodeAddress,

    /// The location (may differ from input if snapped to address)
    location: ArcGISPoint,
}

/// Address information from reverse geocoding.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Getters)]
#[serde(rename_all = "PascalCase")]
pub struct GeocodeAddress {
    /// Complete matched address
    #[serde(skip_serializing_if = "Option::is_none")]
    match_addr: Option<String>,

    /// Long descriptive label
    #[serde(skip_serializing_if = "Option::is_none")]
    long_label: Option<String>,

    /// Short label
    #[serde(skip_serializing_if = "Option::is_none")]
    short_label: Option<String>,

    /// Address type (e.g., "StreetAddress", "PointAddress")
    #[serde(skip_serializing_if = "Option::is_none")]
    addr_type: Option<String>,

    /// Feature type
    #[serde(rename = "Type", skip_serializing_if = "Option::is_none")]
    feature_type: Option<String>,

    /// Place name
    #[serde(skip_serializing_if = "Option::is_none")]
    place_name: Option<String>,

    /// Street number
    #[serde(skip_serializing_if = "Option::is_none")]
    add_num: Option<String>,

    /// Street name
    #[serde(skip_serializing_if = "Option::is_none")]
    st_name: Option<String>,

    /// Street type (e.g., "St", "Ave")
    #[serde(skip_serializing_if = "Option::is_none")]
    st_type: Option<String>,

    /// City name
    #[serde(skip_serializing_if = "Option::is_none")]
    city: Option<String>,

    /// Region/state name
    #[serde(skip_serializing_if = "Option::is_none")]
    region: Option<String>,

    /// Region abbreviation
    #[serde(skip_serializing_if = "Option::is_none")]
    region_abbr: Option<String>,

    /// Postal code
    #[serde(skip_serializing_if = "Option::is_none")]
    postal: Option<String>,

    /// Postal code extension
    #[serde(skip_serializing_if = "Option::is_none")]
    postal_ext: Option<String>,

    /// Country code
    #[serde(skip_serializing_if = "Option::is_none")]
    country_code: Option<String>,

    /// Country name
    #[serde(skip_serializing_if = "Option::is_none")]
    cntry_name: Option<String>,
}

/// Suggestion from autocomplete/suggest operation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Getters)]
#[serde(rename_all = "camelCase")]
pub struct Suggestion {
    /// Suggested text
    text: String,

    /// Magic key for use in findAddressCandidates
    magic_key: String,

    /// Whether this is a collection (area) or single location
    is_collection: bool,
}

/// Response from suggest operation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Getters)]
pub struct SuggestResponse {
    /// List of suggestions
    suggestions: Vec<Suggestion>,
}

/// Location type for geocoding results.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum LocationType {
    /// Rooftop location (most precise)
    #[default]
    Rooftop,
    /// Street entrance location
    Street,
}

/// Category for filtering geocoding results.
///
/// Categories allow you to filter results by place type.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Category {
    /// Point of Interest
    Poi,
    /// Education facilities
    Education,
    /// Food establishments
    Food,
    /// Shops and stores
    Shops,
    /// Custom category
    Custom(String),
}

impl Category {
    /// Returns the string representation for the ArcGIS API.
    pub fn as_str(&self) -> &str {
        match self {
            Self::Poi => "POI",
            Self::Education => "Education",
            Self::Food => "Food",
            Self::Shops => "Shops and Service",
            Self::Custom(s) => s.as_str(),
        }
    }
}

impl From<&str> for Category {
    fn from(s: &str) -> Self {
        match s {
            "POI" => Self::Poi,
            "Education" => Self::Education,
            "Food" => Self::Food,
            "Shops and Service" => Self::Shops,
            _ => Self::Custom(s.to_string()),
        }
    }
}

impl GeocodeAddress {
    /// Creates a new GeocodeAddress with just the address string.
    ///
    /// This is useful for batch geocoding where you just have address strings.
    pub fn new(address: impl Into<String>) -> Self {
        Self {
            match_addr: Some(address.into()),
            long_label: None,
            short_label: None,
            addr_type: None,
            feature_type: None,
            place_name: None,
            add_num: None,
            st_name: None,
            st_type: None,
            city: None,
            region: None,
            region_abbr: None,
            postal: None,
            postal_ext: None,
            country_code: None,
            cntry_name: None,
        }
    }
}

/// Response from batch geocoding (geocodeAddresses).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Getters)]
pub struct BatchGeocodeResponse {
    /// Array of geocoded locations.
    locations: Vec<BatchLocation>,

    /// Spatial reference of results.
    #[serde(skip_serializing_if = "Option::is_none")]
    spatial_reference: Option<SpatialReference>,
}

/// A single result from batch geocoding.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Getters)]
#[serde(rename_all = "camelCase")]
pub struct BatchLocation {
    /// Address that was geocoded.
    address: String,

    /// Geographic location.
    location: ArcGISPoint,

    /// Match score (0-100).
    score: f64,

    /// Detailed attributes.
    #[serde(default)]
    attributes: HashMap<String, serde_json::Value>,
}

/// Response from batch findAddressCandidates.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Getters)]
pub struct BatchCandidatesResponse {
    /// Array of candidate results for each input address.
    candidates: Vec<BatchCandidateResult>,

    /// Spatial reference of results.
    #[serde(skip_serializing_if = "Option::is_none")]
    spatial_reference: Option<SpatialReference>,
}

/// Candidates for a single address in batch processing.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Getters)]
pub struct BatchCandidateResult {
    /// The input address.
    address: String,

    /// All candidates found for this address.
    candidates: Vec<AddressCandidate>,
}
