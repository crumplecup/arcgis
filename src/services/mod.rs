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
    AlterResponse, AlterVersionParams, ConflictDetection, ConflictEntry, ConflictFeature,
    ConflictsResponse, CreateVersionParams, CreateVersionResponse, DeleteForwardEditsResponse,
    DeleteResponse, DifferenceFeature, DifferenceResultType, DifferencesResponse, EditSessionError,
    InspectConflictFeature, InspectConflictLayer, InspectConflictsResponse, LayerConflicts,
    LayerFeatureDifferences, LayerObjectIdDifferences, PartialPostRow, PostResponse,
    ReconcileResponse, RestoreRowsLayer, RestoreRowsResponse, SessionId, StartEditingResponse,
    StartReadingResponse, StopEditingResponse, StopReadingResponse, VersionGuid, VersionInfo,
    VersionInfosResponse, VersionManagementClient, VersionPermission, VersioningType,
};
