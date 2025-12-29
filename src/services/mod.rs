//! ArcGIS service modules.

mod feature;
mod geocode;
mod version_management;

pub use feature::{
    EditError, EditOptions, EditResult, EditResultItem, Feature, FeatureQueryParams,
    FeatureQueryParamsBuilder, FeatureServiceClient, FeatureSet, QueryBuilder, ResponseFormat,
};
pub use geocode::{
    AddressCandidate, Category, Extent, GeocodeAddress, GeocodeResponse, GeocodeServiceClient,
    LocationType, ReverseGeocodeResponse, SuggestResponse, Suggestion,
};
pub use version_management::{
    EditSessionError, SessionId, StartEditingResponse, StopEditingResponse, VersionGuid,
    VersionInfo, VersionManagementClient, VersioningType,
};
