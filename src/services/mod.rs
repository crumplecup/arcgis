//! ArcGIS service modules.

mod feature;
mod geocode;
mod map;
mod version_management;

pub use feature::{
    AddAttachmentResult, AttachmentInfo, AttachmentInfosResponse, AttachmentSource,
    DeleteAttachmentResult, DeleteAttachmentsResponse, DownloadResult, DownloadTarget, EditError,
    EditOptions, EditResult, EditResultItem, Feature, FeatureQueryParams,
    FeatureQueryParamsBuilder, FeatureServiceClient, FeatureSet, FeatureStatisticsResponse,
    QueryBuilder, RelatedRecordGroup, RelatedRecordsParams, RelatedRecordsParamsBuilder,
    RelatedRecordsResponse, ResponseFormat, StatisticDefinition, StatisticType,
    UpdateAttachmentResult,
};
pub use geocode::{
    AddressCandidate, Category, Extent, GeocodeAddress, GeocodeResponse, GeocodeServiceClient,
    LocationType, ReverseGeocodeResponse, SuggestResponse, Suggestion,
};
pub use map::{
    ExportExtent, ExportMapBuilder, ExportMapParams, ExportMapParamsBuilder, ExportMapResponse,
    ExportResult, ExportTarget, IdentifyParams, IdentifyParamsBuilder, IdentifyResponse,
    IdentifyResult, ImageFormat, LayerLegend, LayerOperation, LayerSelection, LegendResponse,
    LegendSymbol, LevelOfDetail, MapServiceClient, MapServiceMetadata, ServiceLayer,
    SpatialReference, TileCoordinate, TileInfo, TimeRelation,
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
