//! Places Service types and parameters.

use derive_getters::Getters;
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Parameters for searching places near a point.
#[derive(Debug, Clone, Serialize, derive_builder::Builder, Getters)]
#[builder(setter(into, strip_option))]
#[serde(rename_all = "camelCase")]
pub struct PlaceSearchParameters {
    /// Center point (x, y).
    #[serde(rename = "x")]
    x: f64,

    /// Center point y coordinate.
    #[serde(rename = "y")]
    y: f64,

    /// Search radius in meters.
    #[builder(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    radius: Option<f64>,

    /// POI category IDs to filter by.
    #[builder(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    category_ids: Option<String>,

    /// Text search query.
    #[builder(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    search_text: Option<String>,

    /// Maximum number of results per page.
    #[builder(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    page_size: Option<u32>,

    /// Pagination token for next page.
    #[builder(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    page_token: Option<String>,

    /// Spatial reference WKID.
    #[builder(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "inSR")]
    in_sr: Option<u32>,

    /// Output spatial reference WKID.
    #[builder(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "outSR")]
    out_sr: Option<u32>,
}

/// Result from place search operation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Getters)]
#[serde(rename_all = "camelCase")]
pub struct PlaceSearchResult {
    /// Array of places found.
    #[serde(default)]
    results: Vec<PlaceInfo>,

    /// Pagination token for next page.
    #[serde(skip_serializing_if = "Option::is_none")]
    next_page_token: Option<String>,
}

/// Information about a place (point of interest).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Getters)]
#[serde(rename_all = "camelCase")]
pub struct PlaceInfo {
    /// Unique place identifier.
    place_id: String,

    /// Place name.
    name: String,

    /// Place categories.
    #[serde(default)]
    categories: Vec<PlaceCategory>,

    /// Location (point geometry).
    #[serde(skip_serializing_if = "Option::is_none")]
    location: Option<Value>,

    /// Address information.
    #[serde(skip_serializing_if = "Option::is_none")]
    address: Option<PlaceAddress>,

    /// Contact information.
    #[serde(skip_serializing_if = "Option::is_none")]
    contact_info: Option<PlaceContactInfo>,

    /// Distance from search point (meters).
    #[serde(skip_serializing_if = "Option::is_none")]
    distance: Option<f64>,

    /// Additional attributes.
    #[serde(skip_serializing_if = "Option::is_none")]
    attributes: Option<Value>,
}

/// Place category information.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Getters)]
#[serde(rename_all = "camelCase")]
pub struct PlaceCategory {
    /// Category ID.
    category_id: String,

    /// Category label/name.
    label: String,
}

/// Place address information.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Getters)]
#[serde(rename_all = "camelCase")]
pub struct PlaceAddress {
    /// Street address.
    #[serde(skip_serializing_if = "Option::is_none")]
    street_address: Option<String>,

    /// Locality/city.
    #[serde(skip_serializing_if = "Option::is_none")]
    locality: Option<String>,

    /// Region/state/province.
    #[serde(skip_serializing_if = "Option::is_none")]
    region: Option<String>,

    /// Postal code.
    #[serde(skip_serializing_if = "Option::is_none")]
    postal_code: Option<String>,

    /// Country.
    #[serde(skip_serializing_if = "Option::is_none")]
    country: Option<String>,

    /// Administrative area.
    #[serde(skip_serializing_if = "Option::is_none")]
    admin_region: Option<String>,

    /// Post office box.
    #[serde(skip_serializing_if = "Option::is_none")]
    po_box: Option<String>,

    /// Neighborhood.
    #[serde(skip_serializing_if = "Option::is_none")]
    neighborhood: Option<String>,
}

/// Place contact information.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Getters)]
#[serde(rename_all = "camelCase")]
pub struct PlaceContactInfo {
    /// Telephone number.
    #[serde(skip_serializing_if = "Option::is_none")]
    telephone: Option<String>,

    /// Website URL.
    #[serde(skip_serializing_if = "Option::is_none")]
    website: Option<String>,

    /// Email address.
    #[serde(skip_serializing_if = "Option::is_none")]
    email: Option<String>,

    /// Fax number.
    #[serde(skip_serializing_if = "Option::is_none")]
    fax_number: Option<String>,
}

/// Detailed place information result.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Getters)]
#[serde(rename_all = "camelCase")]
pub struct PlaceDetailsResult {
    /// Place information.
    place: PlaceInfo,

    /// Operating hours.
    #[serde(skip_serializing_if = "Option::is_none")]
    hours: Option<PlaceHours>,

    /// Ratings.
    #[serde(skip_serializing_if = "Option::is_none")]
    rating: Option<PlaceRating>,

    /// Social media links.
    #[serde(skip_serializing_if = "Option::is_none")]
    social_media: Option<Value>,

    /// Description.
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<String>,
}

/// Place operating hours.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Getters)]
#[serde(rename_all = "camelCase")]
pub struct PlaceHours {
    /// Opening hours by day.
    #[serde(default)]
    opening_hours: Vec<DayHours>,

    /// Popular times (if available).
    #[serde(skip_serializing_if = "Option::is_none")]
    popular_times: Option<Value>,
}

/// Operating hours for a specific day.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Getters)]
#[serde(rename_all = "camelCase")]
pub struct DayHours {
    /// Day of week (Monday, Tuesday, etc.).
    day: String,

    /// Opening time (HH:MM format).
    #[serde(skip_serializing_if = "Option::is_none")]
    open: Option<String>,

    /// Closing time (HH:MM format).
    #[serde(skip_serializing_if = "Option::is_none")]
    close: Option<String>,

    /// Whether closed on this day.
    #[serde(default)]
    is_closed: bool,
}

/// Place rating information.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Getters)]
#[serde(rename_all = "camelCase")]
pub struct PlaceRating {
    /// User rating (0.0 to 5.0).
    #[serde(skip_serializing_if = "Option::is_none")]
    user: Option<f64>,

    /// Price rating (1-4 dollar signs).
    #[serde(skip_serializing_if = "Option::is_none")]
    price: Option<u8>,
}

/// Category listing result.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Getters)]
#[serde(rename_all = "camelCase")]
pub struct CategoriesResult {
    /// Available categories.
    #[serde(default)]
    categories: Vec<CategoryInfo>,
}

/// Detailed category information.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Getters)]
#[serde(rename_all = "camelCase")]
pub struct CategoryInfo {
    /// Category ID.
    category_id: String,

    /// Category name/label.
    name: String,

    /// Full category name.
    #[serde(skip_serializing_if = "Option::is_none")]
    full_label: Option<String>,

    /// Parent category ID.
    #[serde(skip_serializing_if = "Option::is_none")]
    parent_category_id: Option<String>,

    /// Icon reference.
    #[serde(skip_serializing_if = "Option::is_none")]
    icon: Option<String>,
}
