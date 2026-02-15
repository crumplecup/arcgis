//! Geocoding service types.

use crate::{ArcGISPoint, SpatialReference};
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

/// A single address record for batch geocoding requests.
///
/// For geocodeAddresses API, each record must have an OBJECTID and address components.
/// You can use either single-field format (SingleLine) or multi-field format.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, derive_new::new)]
pub struct BatchGeocodeRecord {
    /// Unique identifier for this record (required by API).
    #[serde(rename = "OBJECTID")]
    object_id: i32,

    /// Complete address as single string (use this OR multi-field format).
    #[serde(rename = "SingleLine", skip_serializing_if = "Option::is_none")]
    #[new(default)]
    single_line: Option<String>,

    /// Street address (for multi-field format).
    #[serde(rename = "Address", skip_serializing_if = "Option::is_none")]
    #[new(default)]
    address: Option<String>,

    /// City name (for multi-field format).
    #[serde(rename = "City", skip_serializing_if = "Option::is_none")]
    #[new(default)]
    city: Option<String>,

    /// Region/state (for multi-field format).
    #[serde(rename = "Region", skip_serializing_if = "Option::is_none")]
    #[new(default)]
    region: Option<String>,

    /// Postal/ZIP code (for multi-field format).
    #[serde(rename = "Postal", skip_serializing_if = "Option::is_none")]
    #[new(default)]
    postal: Option<String>,

    /// Country code (optional).
    #[serde(rename = "CountryCode", skip_serializing_if = "Option::is_none")]
    #[new(default)]
    country_code: Option<String>,
}

impl BatchGeocodeRecord {
    /// Creates a record with a single-line address.
    ///
    /// # Example
    ///
    /// ```
    /// use arcgis::BatchGeocodeRecord;
    ///
    /// let record = BatchGeocodeRecord::with_single_line(1, "380 New York St, Redlands, CA 92373");
    /// ```
    pub fn with_single_line(object_id: i32, address: impl Into<String>) -> Self {
        Self {
            object_id,
            single_line: Some(address.into()),
            address: None,
            city: None,
            region: None,
            postal: None,
            country_code: None,
        }
    }

    /// Creates a record with multi-field address components.
    ///
    /// # Example
    ///
    /// ```
    /// use arcgis::BatchGeocodeRecord;
    ///
    /// let record = BatchGeocodeRecord::with_components(
    ///     1,
    ///     "380 New York St",
    ///     Some("Redlands"),
    ///     Some("CA"),
    ///     Some("92373"),
    ///     None,
    /// );
    /// ```
    pub fn with_components(
        object_id: i32,
        address: impl Into<String>,
        city: Option<impl Into<String>>,
        region: Option<impl Into<String>>,
        postal: Option<impl Into<String>>,
        country_code: Option<impl Into<String>>,
    ) -> Self {
        Self {
            object_id,
            single_line: None,
            address: Some(address.into()),
            city: city.map(Into::into),
            region: region.map(Into::into),
            postal: postal.map(Into::into),
            country_code: country_code.map(Into::into),
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

