//! ArcGIS service modules.

mod feature;
mod geocode;
mod geometry;
mod map;
mod version_management;

pub use feature::{
    AddAttachmentResult, AttachmentInfo, AttachmentInfosResponse, AttachmentSource, CodedValue,
    DeleteAttachmentResult, DeleteAttachmentsResponse, Domain, DownloadResult, DownloadTarget,
    EditError, EditOptions, EditResult, EditResultItem, Feature, FeatureQueryParams,
    FeatureQueryParamsBuilder, FeatureServiceClient, FeatureSet, FeatureStatisticsResponse,
    FieldCalculation, LayerDomainInfo, QueryBuilder, QueryDomainsResponse, RelatedRecordGroup,
    RelatedRecordsParams, RelatedRecordsParamsBuilder, RelatedRecordsResponse, ResponseFormat,
    StatisticDefinition, StatisticType, Subtype, TopFeaturesParams, TopFeaturesParamsBuilder,
    TopFilter, TruncateResult, UpdateAttachmentResult,
};
pub use geocode::{
    AddressCandidate, BatchCandidateResult, BatchCandidatesResponse, BatchGeocodeResponse,
    BatchLocation, Category, Extent, GeocodeAddress, GeocodeResponse, GeocodeServiceClient,
    LocationType, ReverseGeocodeResponse, SuggestResponse, Suggestion,
};
pub use geometry::{
    AreaUnit, AreasAndLengthsParameters, AreasAndLengthsParametersBuilder, AreasAndLengthsResult,
    BufferParameters, BufferParametersBuilder, BufferResult, CalculationType, DistanceParameters,
    DistanceParametersBuilder, DistanceResult, GeometryServiceClient, LinearUnit,
    ProjectParameters, ProjectParametersBuilder, ProjectResult, SimplifyParameters,
    SimplifyParametersBuilder, SimplifyResult, Transformation, UnionParameters,
    UnionParametersBuilder, UnionResult,
};
pub use map::{
    ClassBreakInfo, ExportExtent, ExportMapBuilder, ExportMapParams, ExportMapParamsBuilder,
    ExportMapResponse, ExportResult, ExportTarget, FindParams, FindParamsBuilder, FindResponse,
    FindResult, GenerateKmlParams, GenerateKmlParamsBuilder, GenerateRendererParams,
    GenerateRendererParamsBuilder, IdentifyParams, IdentifyParamsBuilder, IdentifyResponse,
    IdentifyResult, ImageFormat, LayerDefinitions, LayerLegend, LayerOperation, LayerSelection,
    LegendResponse, LegendSymbol, LevelOfDetail, MapServiceClient, MapServiceMetadata,
    RendererResponse, ServiceLayer, SpatialReference, TileCoordinate, TileInfo, TimeRelation,
    UniqueValueInfo,
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
