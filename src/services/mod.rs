//! ArcGIS service modules.

mod feature;
mod geocode;

pub use feature::{
    EditError, EditOptions, EditResult, EditResultItem, Feature, FeatureQueryParams,
    FeatureQueryParamsBuilder, FeatureServiceClient, FeatureSet, QueryBuilder, ResponseFormat,
};
pub use geocode::{
    AddressCandidate, Category, Extent, GeocodeAddress, GeocodeResponse, GeocodeServiceClient,
    LocationType, ReverseGeocodeResponse, Suggestion, SuggestResponse,
};
