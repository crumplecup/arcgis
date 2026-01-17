//! Search operations for the Portal client.

use super::super::{SearchParameters, SearchResult, SortOrder};
use super::PortalClient;
use crate::Result;
use serde::Serialize;
use tracing::instrument;

impl<'a> PortalClient<'a> {
    /// Searches for portal items.
    ///
    /// Supports Lucene query syntax for flexible item discovery.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use arcgis::{ArcGISClient, ApiKeyAuth, PortalClient, SearchParameters, SortOrder};
    /// # async fn example(portal: &PortalClient<'_>) -> arcgis::Result<()> {
    /// // Search for feature services
    /// let results = portal
    ///     .search(SearchParameters::new("type:\"Feature Service\""))
    ///     .await?;
    ///
    /// for item in results.results() {
    ///     println!("{}: {}", item.title(), item.id());
    /// }
    ///
    /// // Search with filters
    /// let results = portal
    ///     .search(
    ///         SearchParameters::new("tags:transportation")
    ///             .with_pagination(1, 20)
    ///             .with_sort("modified", SortOrder::Desc)
    ///     )
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    #[instrument(skip(self))]
    pub async fn search(&self, params: SearchParameters) -> Result<SearchResult> {
        tracing::debug!(query = %params.query(), "Searching portal items");

        let url = format!("{}/search", self.base_url);

        // Get authentication token
        tracing::debug!(url = %url, "Sending search request");

        // Build query parameters
        #[derive(Serialize)]
        struct SearchQuery<'a> {
            q: &'a str,
            #[serde(skip_serializing_if = "Option::is_none")]
            bbox: Option<String>,
            #[serde(skip_serializing_if = "Option::is_none")]
            categories: Option<&'a str>,
            #[serde(rename = "sortField", skip_serializing_if = "Option::is_none")]
            sort_field: Option<&'a str>,
            #[serde(rename = "sortOrder", skip_serializing_if = "Option::is_none")]
            sort_order: Option<&'static str>,
            #[serde(skip_serializing_if = "Option::is_none")]
            start: Option<u32>,
            #[serde(skip_serializing_if = "Option::is_none")]
            num: Option<u32>,
            f: &'static str,
            #[serde(skip_serializing_if = "Option::is_none")]
            token: Option<String>,
        }

        let token_opt = self.client.get_token_if_required().await?;

        let query = SearchQuery {
            q: params.query(),
            bbox: params.bbox().as_ref().map(|b| {
                b.iter()
                    .map(|v| v.to_string())
                    .collect::<Vec<_>>()
                    .join(",")
            }),
            categories: params.categories().as_deref(),
            sort_field: params.sort_field().as_deref(),
            sort_order: params.sort_order().map(|o| match o {
                SortOrder::Asc => "asc",
                SortOrder::Desc => "desc",
            }),
            start: *params.start(),
            num: *params.num(),
            f: "json",
            token: token_opt,
        };

        // Build request
        let response = self.client.http().get(&url).query(&query).send().await?;

        // Check for HTTP errors
        let status = response.status();
        if !status.is_success() {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|e| format!("Failed to read error response: {}", e));
            tracing::error!(status = %status, error = %error_text, "Search request failed");
            return Err(crate::Error::from(crate::ErrorKind::Api {
                code: status.as_u16() as i32,
                message: format!("HTTP {}: {}", status, error_text),
            }));
        }

        // Parse response
        let result: SearchResult = response.json().await?;

        tracing::debug!(
            total = result.total(),
            count = result.results().len(),
            "Search completed"
        );

        Ok(result)
    }
}
