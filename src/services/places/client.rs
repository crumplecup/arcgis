//! Places service client implementation.

use crate::{ArcGISClient, Result};
use tracing::instrument;

use super::types::{
    CategoriesResult, PlaceDetailsResult, PlaceSearchParameters, PlaceSearchResult,
};

/// Client for interacting with ArcGIS Places Service.
///
/// The Places Service provides access to points of interest (POI) data,
/// allowing you to search for businesses, landmarks, and other locations.
///
/// # Example
///
/// ```no_run
/// use arcgis::{ApiKeyAuth, ArcGISClient, PlacesClient, PlaceSearchParametersBuilder};
///
/// # async fn example() -> arcgis::Result<()> {
/// let auth = ApiKeyAuth::new("YOUR_API_KEY");
/// let client = ArcGISClient::new(auth);
/// let places = PlacesClient::new(&client);
///
/// // Search for restaurants near a point
/// let params = PlaceSearchParametersBuilder::default()
///     .x(-118.2437)
///     .y(34.0522)
///     .radius(1000.0)
///     .search_text("restaurant")
///     .build()
///     .expect("Valid parameters");
///
/// let results = places.find_places_near_point(params).await?;
/// # Ok(())
/// # }
/// ```
#[derive(Clone)]
pub struct PlacesClient<'a> {
    /// Base URL of the places service.
    url: String,

    /// Reference to the ArcGIS client.
    client: &'a ArcGISClient,
}

impl<'a> PlacesClient<'a> {
    /// Creates a new places service client.
    ///
    /// # Arguments
    ///
    /// * `client` - Reference to an [`ArcGISClient`] for making requests
    ///
    /// Uses the default ArcGIS Online Places Service URL.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use arcgis::{ApiKeyAuth, ArcGISClient, PlacesClient};
    ///
    /// let auth = ApiKeyAuth::new("YOUR_API_KEY");
    /// let client = ArcGISClient::new(auth);
    /// let places = PlacesClient::new(&client);
    /// ```
    pub fn new(client: &'a ArcGISClient) -> Self {
        PlacesClient {
            url: "https://places-api.arcgis.com/arcgis/rest/services/places-service/v1"
                .to_string(),
            client,
        }
    }

    /// Creates a new places service client with a custom URL.
    ///
    /// # Arguments
    ///
    /// * `url` - Base URL of the places service
    /// * `client` - Reference to an [`ArcGISClient`] for making requests
    ///
    /// # Example
    ///
    /// ```no_run
    /// use arcgis::{ApiKeyAuth, ArcGISClient, PlacesClient};
    ///
    /// let auth = ApiKeyAuth::new("YOUR_API_KEY");
    /// let client = ArcGISClient::new(auth);
    /// let places = PlacesClient::with_url(
    ///     "https://custom-places-api.example.com/v1",
    ///     &client
    /// );
    /// ```
    pub fn with_url(url: impl Into<String>, client: &'a ArcGISClient) -> Self {
        PlacesClient {
            url: url.into(),
            client,
        }
    }

    /// Finds places near a point.
    ///
    /// Searches for points of interest within a specified radius of a location.
    ///
    /// # Arguments
    ///
    /// * `params` - Search parameters (location, radius, filters, etc.)
    ///
    /// # Returns
    ///
    /// Search result containing matching places and pagination token.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use arcgis::{ApiKeyAuth, ArcGISClient, PlacesClient, PlaceSearchParametersBuilder};
    ///
    /// # async fn example() -> arcgis::Result<()> {
    /// let auth = ApiKeyAuth::new("YOUR_API_KEY");
    /// let client = ArcGISClient::new(auth);
    /// let places = PlacesClient::new(&client);
    ///
    /// let params = PlaceSearchParametersBuilder::default()
    ///     .x(-118.2437)
    ///     .y(34.0522)
    ///     .radius(500.0)
    ///     .search_text("coffee")
    ///     .page_size(20u32)
    ///     .build()
    ///     .expect("Valid parameters");
    ///
    /// let results = places.find_places_near_point(params).await?;
    ///
    /// for place in results.results() {
    ///     tracing::info!(name = %place.name(), "Found place");
    /// }
    /// # Ok(())
    /// # }
    /// ```
    #[instrument(skip(self, params))]
    pub async fn find_places_near_point(
        &self,
        params: PlaceSearchParameters,
    ) -> Result<PlaceSearchResult> {
        tracing::debug!(
            x = params.x(),
            y = params.y(),
            radius = ?params.radius(),
            "Searching for places"
        );

        let search_url = format!("{}/places/near-point", self.url);

        let response = self
            .client
            .http()
            .get(&search_url)
            .query(&[
                ("x", params.x().to_string()),
                ("y", params.y().to_string()),
                ("f", "json".to_string()),
            ])
            .query(&params)
            .send()
            .await?;

        let result: PlaceSearchResult = response.json().await?;

        tracing::debug!(count = result.results().len(), "Places found");

        Ok(result)
    }

    /// Gets detailed information about a specific place.
    ///
    /// Retrieves extended details including operating hours, ratings, and contact info.
    ///
    /// # Arguments
    ///
    /// * `place_id` - Unique identifier of the place
    ///
    /// # Returns
    ///
    /// Detailed place information including hours, ratings, and contact details.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use arcgis::{ApiKeyAuth, ArcGISClient, PlacesClient};
    ///
    /// # async fn example() -> arcgis::Result<()> {
    /// let auth = ApiKeyAuth::new("YOUR_API_KEY");
    /// let client = ArcGISClient::new(auth);
    /// let places = PlacesClient::new(&client);
    ///
    /// let details = places.get_place_details("place_12345").await?;
    ///
    /// if let Some(hours) = details.hours() {
    ///     tracing::info!(hours = hours.opening_hours().len(), "Operating hours available");
    /// }
    /// # Ok(())
    /// # }
    /// ```
    #[instrument(skip(self))]
    pub async fn get_place_details(&self, place_id: &str) -> Result<PlaceDetailsResult> {
        tracing::debug!(place_id = %place_id, "Getting place details");

        let details_url = format!("{}/places/{}", self.url, place_id);

        let response = self
            .client
            .http()
            .get(&details_url)
            .query(&[("f", "json")])
            .send()
            .await?;

        let result: PlaceDetailsResult = response.json().await?;

        tracing::debug!(place_id = %place_id, "Place details retrieved");

        Ok(result)
    }

    /// Gets the list of available place categories.
    ///
    /// Retrieves all POI categories that can be used for filtering searches.
    ///
    /// # Returns
    ///
    /// List of all available place categories with IDs and labels.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use arcgis::{ApiKeyAuth, ArcGISClient, PlacesClient};
    ///
    /// # async fn example() -> arcgis::Result<()> {
    /// let auth = ApiKeyAuth::new("YOUR_API_KEY");
    /// let client = ArcGISClient::new(auth);
    /// let places = PlacesClient::new(&client);
    ///
    /// let categories = places.get_categories().await?;
    ///
    /// for category in categories.categories() {
    ///     tracing::info!(
    ///         id = %category.category_id(),
    ///         name = %category.name(),
    ///         "Category"
    ///     );
    /// }
    /// # Ok(())
    /// # }
    /// ```
    #[instrument(skip(self))]
    pub async fn get_categories(&self) -> Result<CategoriesResult> {
        tracing::debug!("Getting place categories");

        let categories_url = format!("{}/categories", self.url);

        let response = self
            .client
            .http()
            .get(&categories_url)
            .query(&[("f", "json")])
            .send()
            .await?;

        let result: CategoriesResult = response.json().await?;

        tracing::debug!(count = result.categories().len(), "Categories retrieved");

        Ok(result)
    }
}
