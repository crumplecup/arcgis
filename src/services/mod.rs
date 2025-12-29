//! ArcGIS service modules.

mod feature;

pub use feature::{
    EditError, EditOptions, EditResult, EditResultItem, Feature, FeatureQueryParams,
    FeatureQueryParamsBuilder, FeatureServiceClient, FeatureSet, QueryBuilder, ResponseFormat,
};
