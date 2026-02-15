//! Geocoding service client.

use crate::{
    ArcGISClient, ArcGISPoint, GeocodeResponse, LocationType, Result, ReverseGeocodeResponse,
    SuggestResponse,
};
use tracing::instrument;

/// Client for interacting with an ArcGIS Geocoding Service.
///
/// The geocoding service converts addresses to geographic coordinates (geocoding)
/// and coordinates to addresses (reverse geocoding). It also provides autocomplete
/// suggestions for partial addresses.
///
/// # Example
///
/// ```no_run
/// use arcgis::{ApiKeyAuth, ArcGISClient, GeocodeServiceClient};
///
/// # async fn example() -> arcgis::Result<()> {
/// let auth = ApiKeyAuth::new("YOUR_API_KEY");
/// let client = ArcGISClient::new(auth);
///
/// let geocoder = GeocodeServiceClient::new(
///     "https://geocode.arcgis.com/arcgis/rest/services/World/GeocodeServer",
///     &client,
/// );
///
/// // Geocode an address
/// let candidates = geocoder
///     .find_address_candidates("380 New York St, Redlands, CA 92373")
///     .await?;
///
/// for candidate in candidates.candidates() {
///     println!("{}: {}, {} (score: {})",
///         candidate.address(),
///         candidate.location().x(),
///         candidate.location().y(),
///         candidate.score()
///     );
/// }
/// # Ok(())
/// # }
/// ```
pub struct GeocodeServiceClient<'a> {
    /// Base URL of the geocoding service
    base_url: String,
    /// Reference to the ArcGIS client for HTTP operations
    client: &'a ArcGISClient,
}

impl<'a> GeocodeServiceClient<'a> {
    /// Creates a new Geocoding Service client.
    ///
    /// # Arguments
    ///
    /// * `base_url` - The base URL of the geocoding service
    /// * `client` - Reference to an authenticated ArcGIS client
    ///
    /// # Example
    ///
    /// ```no_run
    /// use arcgis::{ApiKeyAuth, ArcGISClient, GeocodeServiceClient};
    ///
    /// let auth = ApiKeyAuth::new("YOUR_API_KEY");
    /// let client = ArcGISClient::new(auth);
    ///
    /// // World Geocoding Service
    /// let geocoder = GeocodeServiceClient::new(
    ///     "https://geocode.arcgis.com/arcgis/rest/services/World/GeocodeServer",
    ///     &client,
    /// );
    /// ```
    #[instrument(skip(base_url, client))]
    pub fn new(base_url: impl Into<String>, client: &'a ArcGISClient) -> Self {
        let base_url = base_url.into();
        tracing::debug!(base_url = %base_url, "Creating GeocodeServiceClient");
        Self { base_url, client }
    }

    /// Finds address candidates for a given address string.
    ///
    /// This performs forward geocoding - converting an address to coordinates.
    ///
    /// # Arguments
    ///
    /// * `address` - The address to geocode (single-line format)
    ///
    /// # Example
    ///
    /// ```no_run
    /// use arcgis::{ApiKeyAuth, ArcGISClient, GeocodeServiceClient};
    ///
    /// # async fn example() -> arcgis::Result<()> {
    /// # let auth = ApiKeyAuth::new("YOUR_API_KEY");
    /// # let client = ArcGISClient::new(auth);
    /// # let geocoder = GeocodeServiceClient::new("https://geocode.arcgis.com/arcgis/rest/services/World/GeocodeServer", &client);
    /// let response = geocoder
    ///     .find_address_candidates("1600 Pennsylvania Ave NW, Washington, DC")
    ///     .await?;
    ///
    /// if let Some(best_match) = response.candidates().first() {
    ///     println!("Location: {}, {}",
    ///         best_match.location().x(),
    ///         best_match.location().y()
    ///     );
    /// }
    /// # Ok(())
    /// # }
    /// ```
    #[instrument(skip(self, address), fields(base_url = %self.base_url))]
    pub async fn find_address_candidates(
        &self,
        address: impl Into<String>,
    ) -> Result<GeocodeResponse> {
        let address = address.into();
        tracing::debug!(address = %address, "Finding address candidates");

        let url = format!("{}/findAddressCandidates", self.base_url);

        tracing::debug!(url = %url, "Sending findAddressCandidates request");

        let mut request = self.client.http().get(&url).query(&[
            ("SingleLine", address.as_str()),
            ("f", "json"),
            ("outFields", "*"),
            ("maxLocations", "50"),
        ]);

        if let Some(token) = self.client.get_token_if_required().await? {
            request = request.query(&[("token", token)]);
        }

        let response = request.send().await?;

        let status = response.status();
        if !status.is_success() {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|e| format!("Failed to read error: {}", e));
            tracing::error!(status = %status, error = %error_text, "findAddressCandidates failed");
            return Err(crate::Error::from(crate::ErrorKind::Api {
                code: status.as_u16() as i32,
                message: format!("HTTP {}: {}", status, error_text),
            }));
        }

        let geocode_response: GeocodeResponse = response.json().await?;

        tracing::info!(
            candidate_count = geocode_response.candidates().len(),
            "findAddressCandidates completed"
        );

        Ok(geocode_response)
    }

    /// Finds address candidates with advanced options.
    ///
    /// # Arguments
    ///
    /// * `address` - The address to geocode
    /// * `max_locations` - Maximum number of candidates to return (default: 50)
    /// * `location_type` - Type of location to return (rooftop or street)
    ///
    /// # Example
    ///
    /// ```no_run
    /// use arcgis::{ApiKeyAuth, ArcGISClient, GeocodeServiceClient, LocationType};
    ///
    /// # async fn example() -> arcgis::Result<()> {
    /// # let auth = ApiKeyAuth::new("YOUR_API_KEY");
    /// # let client = ArcGISClient::new(auth);
    /// # let geocoder = GeocodeServiceClient::new("https://geocode.arcgis.com/arcgis/rest/services/World/GeocodeServer", &client);
    /// let response = geocoder
    ///     .find_address_candidates_with_options(
    ///         "Starbucks, Seattle",
    ///         Some(10),
    ///         Some(LocationType::Street),
    ///     )
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    #[instrument(skip(self, address), fields(base_url = %self.base_url))]
    pub async fn find_address_candidates_with_options(
        &self,
        address: impl Into<String>,
        max_locations: Option<u32>,
        location_type: Option<LocationType>,
    ) -> Result<GeocodeResponse> {
        let address = address.into();
        tracing::debug!(address = %address, "Finding address candidates with options");

        let url = format!("{}/findAddressCandidates", self.base_url);

        let max_locs = max_locations.unwrap_or(50).to_string();
        let mut params = vec![
            ("SingleLine", address.as_str()),
            ("f", "json"),
            ("outFields", "*"),
            ("maxLocations", max_locs.as_str()),
        ];

        let loc_type_str;
        if let Some(lt) = location_type {
            loc_type_str = match lt {
                LocationType::Rooftop => "rooftop",
                LocationType::Street => "street",
            };
            params.push(("locationType", loc_type_str));
        }

        tracing::debug!(url = %url, "Sending findAddressCandidates request");

        let mut request = self.client.http().get(&url).query(&params);

        if let Some(token) = self.client.get_token_if_required().await? {
            request = request.query(&[("token", token)]);
        }

        let response = request.send().await?;

        let status = response.status();
        if !status.is_success() {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|e| format!("Failed to read error: {}", e));
            tracing::error!(status = %status, error = %error_text, "findAddressCandidates failed");
            return Err(crate::Error::from(crate::ErrorKind::Api {
                code: status.as_u16() as i32,
                message: format!("HTTP {}: {}", status, error_text),
            }));
        }

        let geocode_response: GeocodeResponse = response.json().await?;

        tracing::info!(
            candidate_count = geocode_response.candidates().len(),
            "findAddressCandidates completed"
        );

        Ok(geocode_response)
    }

    /// Finds address candidates with custom spatial reference.
    ///
    /// This extends the basic `find_address_candidates` operation by allowing you to
    /// specify the output spatial reference for the returned coordinates.
    ///
    /// # Arguments
    ///
    /// * `address` - The address to geocode
    /// * `out_sr` - The WKID of the desired output spatial reference
    ///
    /// # Example
    ///
    /// ```no_run
    /// use arcgis::{ApiKeyAuth, ArcGISClient, GeocodeServiceClient};
    ///
    /// # async fn example() -> arcgis::Result<()> {
    /// let auth = ApiKeyAuth::new("YOUR_API_KEY");
    /// let client = ArcGISClient::new(auth);
    /// let geocoder = GeocodeServiceClient::new("https://geocode.arcgis.com/arcgis/rest/services/World/GeocodeServer", &client);
    ///
    /// // Geocode with Web Mercator output (WKID: 3857)
    /// let result = geocoder
    ///     .find_address_candidates_with_sr("380 New York St, Redlands, CA", 3857)
    ///     .await?;
    ///
    /// // Geocode with State Plane California Zone 6 (WKID: 2230)
    /// let result_sp = geocoder
    ///     .find_address_candidates_with_sr("380 New York St, Redlands, CA", 2230)
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    #[instrument(skip(self, address), fields(out_sr = out_sr))]
    pub async fn find_address_candidates_with_sr(
        &self,
        address: impl Into<String>,
        out_sr: i32,
    ) -> Result<GeocodeResponse> {
        let address = address.into();
        tracing::debug!(address = %address, out_sr = out_sr, "Finding address candidates with custom SR");

        let url = format!("{}/findAddressCandidates", self.base_url);

        tracing::debug!(url = %url, "Sending findAddressCandidates request");

        let mut request = self.client.http().get(&url).query(&[
            ("SingleLine", address.as_str()),
            ("outSR", out_sr.to_string().as_str()),
            ("f", "json"),
            ("outFields", "*"),
            ("maxLocations", "50"),
        ]);

        if let Some(token) = self.client.get_token_if_required().await? {
            request = request.query(&[("token", token)]);
        }

        let response = request.send().await?;

        let status = response.status();
        if !status.is_success() {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|e| format!("Failed to read error: {}", e));
            tracing::error!(status = %status, error = %error_text, "findAddressCandidates with SR failed");
            return Err(crate::Error::from(crate::ErrorKind::Api {
                code: status.as_u16() as i32,
                message: format!("HTTP {}: {}", status, error_text),
            }));
        }

        let geocode_response: GeocodeResponse = response.json().await?;

        tracing::info!(
            candidate_count = geocode_response.candidates().len(),
            out_sr = out_sr,
            "findAddressCandidates with SR completed"
        );

        Ok(geocode_response)
    }

    /// Converts a location (coordinates) to an address.
    ///
    /// This performs reverse geocoding - converting coordinates to an address.
    ///
    /// # Arguments
    ///
    /// * `location` - The point to reverse geocode
    ///
    /// # Example
    ///
    /// ```no_run
    /// use arcgis::{ApiKeyAuth, ArcGISClient, GeocodeServiceClient, ArcGISPoint};
    ///
    /// # async fn example() -> arcgis::Result<()> {
    /// # let auth = ApiKeyAuth::new("YOUR_API_KEY");
    /// # let client = ArcGISClient::new(auth);
    /// # let geocoder = GeocodeServiceClient::new("https://geocode.arcgis.com/arcgis/rest/services/World/GeocodeServer", &client);
    /// let point = ArcGISPoint::new(-117.196, 34.056);
    ///
    /// let response = geocoder.reverse_geocode(&point).await?;
    ///
    /// if let Some(addr) = response.address().match_addr() {
    ///     println!("Address: {}", addr);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    #[instrument(skip(self, location), fields(base_url = %self.base_url, x = *location.x(), y = *location.y()))]
    pub async fn reverse_geocode(&self, location: &ArcGISPoint) -> Result<ReverseGeocodeResponse> {
        tracing::debug!(
            x = *location.x(),
            y = *location.y(),
            "Reverse geocoding location"
        );

        let url = format!("{}/reverseGeocode", self.base_url);

        let location_str = format!("{},{}", location.x(), location.y());

        tracing::debug!(url = %url, location = %location_str, "Sending reverseGeocode request");

        let mut request = self
            .client
            .http()
            .get(&url)
            .query(&[("location", location_str.as_str()), ("f", "json")]);

        if let Some(token) = self.client.get_token_if_required().await? {
            request = request.query(&[("token", token)]);
        }

        let response = request.send().await?;

        let status = response.status();
        if !status.is_success() {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|e| format!("Failed to read error: {}", e));
            tracing::error!(status = %status, error = %error_text, "reverseGeocode failed");
            return Err(crate::Error::from(crate::ErrorKind::Api {
                code: status.as_u16() as i32,
                message: format!("HTTP {}: {}", status, error_text),
            }));
        }

        let reverse_response: ReverseGeocodeResponse = response.json().await?;

        tracing::info!(
            address = ?reverse_response.address().match_addr(),
            "reverseGeocode completed"
        );

        Ok(reverse_response)
    }

    /// Reverse geocodes a location with custom spatial reference.
    ///
    /// This extends the basic `reverse_geocode` operation by allowing you to specify
    /// the output spatial reference for the returned coordinates.
    ///
    /// # Arguments
    ///
    /// * `location` - The point to reverse geocode
    /// * `out_sr` - The WKID of the desired output spatial reference
    ///
    /// # Example
    ///
    /// ```no_run
    /// use arcgis::{ApiKeyAuth, ArcGISClient, GeocodeServiceClient, ArcGISPoint};
    ///
    /// # async fn example() -> arcgis::Result<()> {
    /// let auth = ApiKeyAuth::new("YOUR_API_KEY");
    /// let client = ArcGISClient::new(auth);
    /// let geocoder = GeocodeServiceClient::new("https://geocode.arcgis.com/arcgis/rest/services/World/GeocodeServer", &client);
    ///
    /// // Reverse geocode with Web Mercator output
    /// let point = ArcGISPoint::new(-122.4194, 37.7749);
    /// let result = geocoder
    ///     .reverse_geocode_with_sr(&point, 3857)  // Web Mercator output
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    #[instrument(skip(self, location), fields(x = *location.x(), y = *location.y(), out_sr = out_sr))]
    pub async fn reverse_geocode_with_sr(
        &self,
        location: &ArcGISPoint,
        out_sr: i32,
    ) -> Result<ReverseGeocodeResponse> {
        tracing::debug!(
            x = *location.x(),
            y = *location.y(),
            out_sr = out_sr,
            "Reverse geocoding location with custom SR"
        );

        let url = format!("{}/reverseGeocode", self.base_url);

        let location_str = format!("{},{}", location.x(), location.y());

        tracing::debug!(url = %url, location = %location_str, "Sending reverseGeocode request");

        let mut request = self.client.http().get(&url).query(&[
            ("location", location_str.as_str()),
            ("outSR", out_sr.to_string().as_str()),
            ("f", "json"),
        ]);

        if let Some(token) = self.client.get_token_if_required().await? {
            request = request.query(&[("token", token)]);
        }

        let response = request.send().await?;

        let status = response.status();
        if !status.is_success() {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|e| format!("Failed to read error: {}", e));
            tracing::error!(status = %status, error = %error_text, "reverseGeocode with SR failed");
            return Err(crate::Error::from(crate::ErrorKind::Api {
                code: status.as_u16() as i32,
                message: format!("HTTP {}: {}", status, error_text),
            }));
        }

        let reverse_response: ReverseGeocodeResponse = response.json().await?;

        tracing::info!(
            address = ?reverse_response.address().match_addr(),
            out_sr = out_sr,
            "reverseGeocode with SR completed"
        );

        Ok(reverse_response)
    }

    /// Gets autocomplete suggestions for partial address input.
    ///
    /// This is useful for implementing search-as-you-type functionality.
    /// The magic key from suggestions can be used with `find_address_candidates`
    /// for faster geocoding.
    ///
    /// # Arguments
    ///
    /// * `text` - Partial address or place name
    ///
    /// # Example
    ///
    /// ```no_run
    /// use arcgis::{ApiKeyAuth, ArcGISClient, GeocodeServiceClient};
    ///
    /// # async fn example() -> arcgis::Result<()> {
    /// # let auth = ApiKeyAuth::new("YOUR_API_KEY");
    /// # let client = ArcGISClient::new(auth);
    /// # let geocoder = GeocodeServiceClient::new("https://geocode.arcgis.com/arcgis/rest/services/World/GeocodeServer", &client);
    /// let suggestions = geocoder.suggest("380 New York").await?;
    ///
    /// for suggestion in suggestions.suggestions() {
    ///     println!("{} (key: {})", suggestion.text(), suggestion.magic_key());
    /// }
    /// # Ok(())
    /// # }
    /// ```
    #[instrument(skip(self, text), fields(base_url = %self.base_url))]
    pub async fn suggest(&self, text: impl Into<String>) -> Result<SuggestResponse> {
        let text = text.into();
        tracing::debug!(text = %text, "Getting autocomplete suggestions");

        let url = format!("{}/suggest", self.base_url);

        tracing::debug!(url = %url, "Sending suggest request");

        let mut request = self
            .client
            .http()
            .get(&url)
            .query(&[("text", text.as_str()), ("f", "json")]);

        if let Some(token) = self.client.get_token_if_required().await? {
            request = request.query(&[("token", token)]);
        }

        let response = request.send().await?;

        let status = response.status();
        if !status.is_success() {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|e| format!("Failed to read error: {}", e));
            tracing::error!(status = %status, error = %error_text, "suggest failed");
            return Err(crate::Error::from(crate::ErrorKind::Api {
                code: status.as_u16() as i32,
                message: format!("HTTP {}: {}", status, error_text),
            }));
        }

        let suggest_response: SuggestResponse = response.json().await?;

        tracing::info!(
            suggestion_count = suggest_response.suggestions().len(),
            "suggest completed"
        );

        Ok(suggest_response)
    }

    /// Gets autocomplete suggestions filtered by category.
    ///
    /// This method extends the basic `suggest` operation by allowing you to filter
    /// results by location category (e.g., "Address", "City", "POI").
    ///
    /// # Arguments
    ///
    /// * `text` - The partial address text to get suggestions for
    /// * `category` - The category to filter by (e.g., "Address", "City", "POI")
    ///
    /// # Example
    ///
    /// ```no_run
    /// use arcgis::{ApiKeyAuth, ArcGISClient, GeocodeServiceClient, Category};
    ///
    /// # async fn example() -> arcgis::Result<()> {
    /// let auth = ApiKeyAuth::new("YOUR_API_KEY");
    /// let client = ArcGISClient::new(auth);
    /// let geocoder = GeocodeServiceClient::new("https://geocode.arcgis.com/arcgis/rest/services/World/GeocodeServer", &client);
    ///
    /// // Get suggestions for POIs only
    /// let poi_suggestions = geocoder
    ///     .suggest_with_category("Starbucks", Category::Poi)
    ///     .await?;
    ///
    /// // Get suggestions for food establishments only
    /// let food_suggestions = geocoder
    ///     .suggest_with_category("Pizza", Category::Food)
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    #[instrument(skip(self, text, category), fields(text_len = text.as_ref().len()))]
    pub async fn suggest_with_category(
        &self,
        text: impl Into<String> + AsRef<str>,
        category: crate::Category,
    ) -> Result<SuggestResponse> {
        let text = text.into();
        tracing::debug!(text = %text, category = ?category, "Getting category-filtered suggestions");

        let url = format!("{}/suggest", self.base_url);

        tracing::debug!(url = %url, "Sending suggest request with category filter");

        let mut request = self.client.http().get(&url).query(&[
            ("text", text.as_str()),
            ("category", category.as_str()),
            ("f", "json"),
        ]);

        if let Some(token) = self.client.get_token_if_required().await? {
            request = request.query(&[("token", token)]);
        }

        let response = request.send().await?;

        let status = response.status();
        if !status.is_success() {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|e| format!("Failed to read error: {}", e));
            tracing::error!(status = %status, error = %error_text, "suggest with category failed");
            return Err(crate::Error::from(crate::ErrorKind::Api {
                code: status.as_u16() as i32,
                message: format!("HTTP {}: {}", status, error_text),
            }));
        }

        let suggest_response: SuggestResponse = response.json().await?;

        tracing::info!(
            suggestion_count = suggest_response.suggestions().len(),
            category = ?category,
            "suggest with category completed"
        );

        Ok(suggest_response)
    }

    /// Geocodes multiple addresses in a single batch request.
    ///
    /// This is more efficient than geocoding addresses individually when you have
    /// many addresses to process. The batch size is typically limited to 1000 addresses.
    ///
    /// # Arguments
    ///
    /// * `addresses` - Vector of addresses to geocode
    ///
    /// # Example
    ///
    /// ```no_run
    /// use arcgis::{ApiKeyAuth, ArcGISClient, GeocodeServiceClient, BatchGeocodeRecord};
    ///
    /// # async fn example() -> arcgis::Result<()> {
    /// # let auth = ApiKeyAuth::new("YOUR_API_KEY");
    /// # let client = ArcGISClient::new(auth);
    /// # let geocoder = GeocodeServiceClient::new("https://geocode.arcgis.com/arcgis/rest/services/World/GeocodeServer", &client);
    /// let addresses = vec![
    ///     BatchGeocodeRecord::with_single_line(1, "380 New York St, Redlands, CA 92373"),
    ///     BatchGeocodeRecord::with_single_line(2, "1 World Way, Los Angeles, CA 90045"),
    /// ];
    ///
    /// let results = geocoder.geocode_addresses(addresses).await?;
    /// # Ok(())
    /// # }
    /// ```
    #[instrument(skip(self, addresses), fields(count = addresses.len()))]
    pub async fn geocode_addresses(
        &self,
        addresses: Vec<crate::BatchGeocodeRecord>,
    ) -> Result<crate::BatchGeocodeResponse> {
        tracing::debug!("Batch geocoding addresses");

        let url = format!("{}/geocodeAddresses", self.base_url);

        // Format records with attributes wrapper as required by API
        let records: Vec<serde_json::Value> = addresses
            .iter()
            .map(|addr| {
                serde_json::json!({
                    "attributes": addr
                })
            })
            .collect();

        let addresses_json = serde_json::json!({
            "records": records
        });

        tracing::debug!(url = %url, count = addresses.len(), "Sending geocodeAddresses request");

        let addresses_str = addresses_json.to_string();
        let mut form = vec![("addresses", addresses_str.as_str()), ("f", "json")];

        // Add token if required by auth provider
        let token_opt = self.client.get_token_if_required().await?;
        let token_str;
        if let Some(token) = token_opt {
            token_str = token;
            form.push(("token", token_str.as_str()));
        }

        let response = self.client.http().post(&url).form(&form).send().await?;

        let status = response.status();
        if !status.is_success() {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|e| format!("Failed to read error: {}", e));
            tracing::error!(status = %status, error = %error_text, "geocodeAddresses failed");
            return Err(crate::Error::from(crate::ErrorKind::Api {
                code: status.as_u16() as i32,
                message: format!("HTTP {}: {}", status, error_text),
            }));
        }

        let batch_response: crate::BatchGeocodeResponse = response.json().await?;

        tracing::info!(
            location_count = batch_response.locations().len(),
            "geocodeAddresses completed"
        );

        Ok(batch_response)
    }
}
