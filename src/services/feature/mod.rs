//! Feature Service operations.
//!
//! The Feature Service provides access to feature data, allowing you to query
//! and edit features with full CRUD support.

mod client;
mod types;

pub use client::FeatureServiceClient;
pub use types::{
    Feature, FeatureQueryParams, FeatureQueryParamsBuilder, FeatureSet, ResponseFormat,
};
