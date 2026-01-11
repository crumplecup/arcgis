//! Query builder for Feature Service queries.

use crate::{
    ArcGISGeometry, FeatureQueryParams, FeatureServiceClient, FeatureSet, GeometryType, LayerId,
    ObjectId, ResponseFormat, Result, SpatialRel,
};
use tracing::instrument;

/// A fluent builder for constructing and executing feature queries.
///
/// This provides an ergonomic API for building complex queries without
/// manually constructing [`FeatureQueryParams`].
///
/// # Example
///
/// ```no_run
/// use arcgis::{ArcGISClient, ApiKeyAuth, FeatureServiceClient, LayerId};
///
/// # async fn example() -> arcgis::Result<()> {
/// let auth = ApiKeyAuth::new("YOUR_API_KEY");
/// let client = ArcGISClient::new(auth);
/// let service = FeatureServiceClient::new("https://example.com/FeatureServer", &client);
///
/// // Simple query
/// let features = service
///     .query(LayerId::new(0))
///     .where_clause("POPULATION > 100000")
///     .out_fields(&["NAME", "POPULATION"])
///     .execute()
///     .await?;
///
/// // Auto-paginated query
/// let all_features = service
///     .query(LayerId::new(0))
///     .where_clause("STATE = 'CA'")
///     .execute_all()
///     .await?;
/// # Ok(())
/// # }
/// ```
pub struct QueryBuilder<'a> {
    client: &'a FeatureServiceClient<'a>,
    layer_id: LayerId,
    params: FeatureQueryParams,
}

impl<'a> QueryBuilder<'a> {
    /// Creates a new query builder.
    ///
    /// Typically you don't call this directly - use [`FeatureServiceClient::query`] instead.
    #[instrument(skip(client))]
    pub(crate) fn new(client: &'a FeatureServiceClient<'a>, layer_id: LayerId) -> Self {
        tracing::debug!(layer_id = %layer_id, "Creating QueryBuilder");
        Self {
            client,
            layer_id,
            params: FeatureQueryParams::default(),
        }
    }

    /// Sets the WHERE clause for the query.
    ///
    /// # Example
    /// ```no_run
    /// # use arcgis::{ArcGISClient, ApiKeyAuth, FeatureServiceClient, LayerId};
    /// # async fn example(service: &FeatureServiceClient<'_>) -> arcgis::Result<()> {
    /// let features = service
    ///     .query(LayerId::new(0))
    ///     .where_clause("POPULATION > 1000000")
    ///     .execute()
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn where_clause(mut self, clause: impl Into<String>) -> Self {
        self.params.set_where_clause(clause.into());
        self
    }

    /// Sets the fields to return in the response.
    ///
    /// Pass `&["*"]` to return all fields.
    ///
    /// # Example
    /// ```no_run
    /// # use arcgis::{ArcGISClient, ApiKeyAuth, FeatureServiceClient, LayerId};
    /// # async fn example(service: &FeatureServiceClient<'_>) -> arcgis::Result<()> {
    /// let features = service
    ///     .query(LayerId::new(0))
    ///     .out_fields(&["NAME", "POPULATION", "CITY_ID"])
    ///     .execute()
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn out_fields(mut self, fields: &[&str]) -> Self {
        self.params
            .set_out_fields(Some(fields.iter().map(|s| s.to_string()).collect()));
        self
    }

    /// Sets whether to return geometry with features.
    ///
    /// Default is `true`.
    pub fn return_geometry(mut self, return_geom: bool) -> Self {
        self.params.set_return_geometry(return_geom);
        self
    }

    /// Sets the response format.
    ///
    /// Default is [`ResponseFormat::Json`].
    pub fn format(mut self, format: ResponseFormat) -> Self {
        self.params.set_format(format);
        self
    }

    /// Requests Protocol Buffer (PBF) format for 3-5x performance improvement.
    ///
    /// PBF is a binary format that's more efficient than JSON for large datasets.
    /// Supported by ArcGIS Enterprise 10.7+ and ArcGIS Online.
    ///
    /// # Example
    /// ```no_run
    /// # use arcgis::{ArcGISClient, ApiKeyAuth, FeatureServiceClient, LayerId};
    /// # async fn example(service: &FeatureServiceClient<'_>) -> arcgis::Result<()> {
    /// // Get large dataset efficiently with PBF
    /// let features = service
    ///     .query(LayerId::new(0))
    ///     .where_clause("1=1")
    ///     .pbf()
    ///     .execute()
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn pbf(self) -> Self {
        self.format(ResponseFormat::Pbf)
    }

    /// Requests JSON format (default).
    ///
    /// This is the standard Esri JSON format.
    pub fn json(self) -> Self {
        self.format(ResponseFormat::Json)
    }

    /// Requests GeoJSON format.
    ///
    /// GeoJSON is an open standard format for geographic data.
    pub fn geojson(self) -> Self {
        self.format(ResponseFormat::GeoJson)
    }

    /// Sets a spatial filter for the query.
    ///
    /// # Example
    /// ```no_run
    /// # use arcgis::{ArcGISClient, ApiKeyAuth, FeatureServiceClient, LayerId, ArcGISPoint, ArcGISGeometry, GeometryType, SpatialRel};
    /// # async fn example(service: &FeatureServiceClient<'_>) -> arcgis::Result<()> {
    /// let point = ArcGISPoint { x: -118.0, y: 34.0, z: None, m: None, spatial_reference: None };
    /// let features = service
    ///     .query(LayerId::new(0))
    ///     .spatial_filter(ArcGISGeometry::Point(point), GeometryType::Point, SpatialRel::Intersects)
    ///     .execute()
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn spatial_filter(
        mut self,
        geometry: ArcGISGeometry,
        geometry_type: GeometryType,
        spatial_rel: SpatialRel,
    ) -> Self {
        self.params.set_geometry(Some(geometry));
        self.params.set_geometry_type(Some(geometry_type));
        self.params.set_spatial_rel(Some(spatial_rel));
        self
    }

    /// Sets the maximum number of features to return.
    ///
    /// Used for pagination. If not set, the service default is used.
    pub fn limit(mut self, count: u32) -> Self {
        self.params.set_result_record_count(Some(count));
        self
    }

    /// Sets the offset for pagination.
    ///
    /// Skips the first `offset` features in the result set.
    pub fn offset(mut self, offset: u32) -> Self {
        self.params.set_result_offset(Some(offset));
        self
    }

    /// Queries specific features by object IDs.
    ///
    /// # Example
    /// ```no_run
    /// # use arcgis::{ArcGISClient, ApiKeyAuth, FeatureServiceClient, LayerId, ObjectId};
    /// # async fn example(service: &FeatureServiceClient<'_>) -> arcgis::Result<()> {
    /// let features = service
    ///     .query(LayerId::new(0))
    ///     .object_ids(&[ObjectId::new(1), ObjectId::new(2), ObjectId::new(3)])
    ///     .execute()
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn object_ids(mut self, ids: &[ObjectId]) -> Self {
        self.params.set_object_ids(Some(ids.to_vec()));
        self
    }

    /// Returns only distinct values.
    pub fn distinct(mut self, distinct: bool) -> Self {
        self.params.set_return_distinct_values(Some(distinct));
        self
    }

    /// Returns only object IDs (no attributes or geometry).
    pub fn ids_only(mut self, ids_only: bool) -> Self {
        self.params.set_return_ids_only(Some(ids_only));
        self
    }

    /// Returns only a count of features (no features).
    pub fn count_only(mut self, count_only: bool) -> Self {
        self.params.set_return_count_only(Some(count_only));
        self
    }

    /// Sets the ORDER BY clause.
    ///
    /// # Example
    /// ```no_run
    /// # use arcgis::{ArcGISClient, ApiKeyAuth, FeatureServiceClient, LayerId};
    /// # async fn example(service: &FeatureServiceClient<'_>) -> arcgis::Result<()> {
    /// let features = service
    ///     .query(LayerId::new(0))
    ///     .order_by(&["POPULATION DESC", "NAME ASC"])
    ///     .execute()
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn order_by(mut self, fields: &[&str]) -> Self {
        self.params
            .set_order_by_fields(Some(fields.iter().map(|s| s.to_string()).collect()));
        self
    }

    /// Sets the GROUP BY clause for statistics.
    ///
    /// # Example
    /// ```no_run
    /// # use arcgis::{ArcGISClient, ApiKeyAuth, FeatureServiceClient, LayerId, StatisticDefinition, StatisticType};
    /// # async fn example(service: &FeatureServiceClient<'_>) -> arcgis::Result<()> {
    /// let stats = service
    ///     .query(LayerId::new(0))
    ///     .statistics(vec![
    ///         StatisticDefinition::new(StatisticType::Avg, "POPULATION".to_string(), "avg_pop".to_string())
    ///     ])
    ///     .group_by(&["STATE"])
    ///     .execute()
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn group_by(mut self, fields: &[&str]) -> Self {
        self.params
            .set_group_by_fields(Some(fields.iter().map(|s| s.to_string()).collect()));
        self
    }

    /// Sets statistics to calculate (aggregate query).
    ///
    /// When using statistics, the query can only include these additional parameters:
    /// `group_by`, `order_by`, `where_clause`, `time`, and `return_distinct_values`.
    ///
    /// # Example
    /// ```no_run
    /// # use arcgis::{ArcGISClient, ApiKeyAuth, FeatureServiceClient, LayerId, StatisticDefinition, StatisticType};
    /// # async fn example(service: &FeatureServiceClient<'_>) -> arcgis::Result<()> {
    /// let stats = service
    ///     .query(LayerId::new(0))
    ///     .statistics(vec![
    ///         StatisticDefinition::new(StatisticType::Count, "OBJECTID".to_string(), "total_count".to_string()),
    ///         StatisticDefinition::new(StatisticType::Sum, "POPULATION".to_string(), "total_population".to_string()),
    ///         StatisticDefinition::new(StatisticType::Avg, "AREA".to_string(), "avg_area".to_string()),
    ///     ])
    ///     .group_by(&["STATE"])
    ///     .execute()
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn statistics(mut self, stats: Vec<crate::StatisticDefinition>) -> Self {
        self.params.set_out_statistics(Some(stats));
        self
    }

    /// Sets the HAVING clause for filtering aggregated results.
    ///
    /// Only valid when `statistics()` has been called. The HAVING clause filters
    /// the results after aggregation, similar to SQL's HAVING.
    ///
    /// # Example
    /// ```no_run
    /// # use arcgis::{ArcGISClient, ApiKeyAuth, FeatureServiceClient, LayerId, StatisticDefinition, StatisticType};
    /// # async fn example(service: &FeatureServiceClient<'_>) -> arcgis::Result<()> {
    /// let stats = service
    ///     .query(LayerId::new(0))
    ///     .statistics(vec![
    ///         StatisticDefinition::new(StatisticType::Count, "OBJECTID".to_string(), "count".to_string())
    ///     ])
    ///     .group_by(&["STATE"])
    ///     .having("count > 1000")
    ///     .execute()
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn having(mut self, clause: impl Into<String>) -> Self {
        self.params.set_having(Some(clause.into()));
        self
    }

    /// Sets the output spatial reference WKID.
    ///
    /// # Example
    /// ```no_run
    /// # use arcgis::{ArcGISClient, ApiKeyAuth, FeatureServiceClient, LayerId};
    /// # async fn example(service: &FeatureServiceClient<'_>) -> arcgis::Result<()> {
    /// let features = service
    ///     .query(LayerId::new(0))
    ///     .out_sr(4326)  // WGS84
    ///     .execute()
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn out_sr(mut self, wkid: i32) -> Self {
        self.params.set_out_sr(Some(wkid));
        self
    }

    /// Executes the query and returns a single page of results.
    ///
    /// This method sends a single request to the server and returns
    /// whatever fits in the response (subject to server limits).
    ///
    /// For queries that may return more results than fit in a single
    /// response, use [`execute_all`](Self::execute_all) instead.
    ///
    /// # Example
    /// ```no_run
    /// # use arcgis::{ArcGISClient, ApiKeyAuth, FeatureServiceClient, LayerId};
    /// # async fn example(service: &FeatureServiceClient<'_>) -> arcgis::Result<()> {
    /// let features = service
    ///     .query(LayerId::new(0))
    ///     .where_clause("STATE = 'CA'")
    ///     .limit(100)
    ///     .execute()
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    #[instrument(skip(self), fields(layer_id = %self.layer_id))]
    pub async fn execute(self) -> Result<FeatureSet> {
        tracing::debug!("Executing single-page query");
        self.client
            .query_with_params(self.layer_id, self.params)
            .await
    }

    /// Executes the query with automatic pagination, returning all results.
    ///
    /// This method automatically handles pagination by making multiple requests
    /// if necessary. It continues fetching until all matching features are retrieved
    /// or the server indicates no more results.
    ///
    /// # Performance Note
    ///
    /// This method may make many requests for large result sets. Consider using
    /// [`execute`](Self::execute) with manual pagination for very large queries.
    ///
    /// # Example
    /// ```no_run
    /// # use arcgis::{ArcGISClient, ApiKeyAuth, FeatureServiceClient, LayerId};
    /// # async fn example(service: &FeatureServiceClient<'_>) -> arcgis::Result<()> {
    /// // Automatically fetches all matching features across multiple requests
    /// let all_features = service
    ///     .query(LayerId::new(0))
    ///     .where_clause("POPULATION > 100000")
    ///     .execute_all()
    ///     .await?;
    ///
    /// println!("Retrieved {} total features", all_features.features.len());
    /// # Ok(())
    /// # }
    /// ```
    #[instrument(skip(self), fields(layer_id = %self.layer_id))]
    pub async fn execute_all(mut self) -> Result<FeatureSet> {
        tracing::debug!("Executing auto-paginated query");

        let mut all_features = Vec::new();
        let mut offset = 0u32;
        let page_size = (*self.params.result_record_count()).unwrap_or(1000);

        // Store geometry type from first response
        let mut geometry_type = None;

        loop {
            // Set pagination parameters
            self.params.set_result_offset(Some(offset));
            self.params.set_result_record_count(Some(page_size));

            tracing::debug!(
                offset = offset,
                page_size = page_size,
                "Fetching page of results"
            );

            // Execute query for this page
            let mut page = self
                .client
                .query_with_params(self.layer_id, self.params.clone())
                .await?;

            // Capture geometry type from first response
            if geometry_type.is_none() {
                geometry_type = *page.geometry_type();
            }

            let feature_count = page.features().len();
            tracing::debug!(
                feature_count = feature_count,
                exceeded_limit = page.exceeded_transfer_limit(),
                "Page retrieved"
            );

            // Add features to our collection
            all_features.append(page.features_mut());

            // Check if we're done
            if feature_count == 0 || !*page.exceeded_transfer_limit() {
                tracing::debug!(
                    total_features = all_features.len(),
                    "Auto-pagination complete"
                );
                break;
            }

            // Move to next page
            offset += page_size;
        }

        Ok(FeatureSet::new(geometry_type, all_features, None, false))
    }
}
