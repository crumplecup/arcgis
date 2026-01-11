//! Image service client implementation.

use crate::{ArcGISClient, ArcGISGeometry, Result};
use tracing::instrument;

use super::types::{
    ExportImageParameters, ExportImageResult, HistogramParameters, HistogramResult,
    IdentifyParameters, ImageIdentifyResult, RasterInfo, SampleParameters, SampleResult,
};

/// Client for interacting with ArcGIS Image Services (ImageServer).
///
/// Image services provide access to raster datasets with dynamic rendering,
/// analysis, and processing capabilities.
///
/// # Example
///
/// ```no_run
/// use arcgis::{ApiKeyAuth, ArcGISClient, ImageServiceClient, ArcGISGeometry};
/// use arcgis::geo_types::{Geometry, Point};
///
/// # async fn example() -> arcgis::Result<()> {
/// let auth = ApiKeyAuth::new("YOUR_API_KEY");
/// let client = ArcGISClient::new(auth);
/// let image_service = ImageServiceClient::new(
///     "https://sampleserver6.arcgisonline.com/arcgis/rest/services/NLCDLandCover2001/ImageServer",
///     &client
/// );
///
/// // Identify pixel value at a point
/// let point = Point::new(-120.0, 40.0);
/// let geom: Geometry = point.into();
/// let geometry = ArcGISGeometry::from_geo_types(&geom)?;
/// let result = image_service.identify(&geometry).await?;
/// # Ok(())
/// # }
/// ```
#[derive(Clone)]
pub struct ImageServiceClient<'a> {
    /// Base URL of the image service.
    url: String,

    /// Reference to the ArcGIS client.
    client: &'a ArcGISClient,
}

impl<'a> ImageServiceClient<'a> {
    /// Creates a new image service client.
    ///
    /// # Arguments
    ///
    /// * `url` - Base URL of the image service (e.g., `https://server/arcgis/rest/services/ImageService/ImageServer`)
    /// * `client` - Reference to an [`ArcGISClient`] for making requests
    ///
    /// # Example
    ///
    /// ```no_run
    /// use arcgis::{ApiKeyAuth, ArcGISClient, ImageServiceClient};
    ///
    /// let auth = ApiKeyAuth::new("YOUR_API_KEY");
    /// let client = ArcGISClient::new(auth);
    /// let image_service = ImageServiceClient::new(
    ///     "https://sampleserver6.arcgisonline.com/arcgis/rest/services/NLCDLandCover2001/ImageServer",
    ///     &client
    /// );
    /// ```
    pub fn new(url: impl Into<String>, client: &'a ArcGISClient) -> Self {
        ImageServiceClient {
            url: url.into(),
            client,
        }
    }

    /// Exports an image from the image service.
    ///
    /// Returns a URL to the exported image with the specified parameters.
    ///
    /// # Arguments
    ///
    /// * `params` - Export parameters (bounding box, size, format, etc.)
    ///
    /// # Returns
    ///
    /// Export result containing the image URL and metadata.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use arcgis::{ApiKeyAuth, ArcGISClient, ImageServiceClient, ExportImageParametersBuilder};
    ///
    /// # async fn example() -> arcgis::Result<()> {
    /// let auth = ApiKeyAuth::new("YOUR_API_KEY");
    /// let client = ArcGISClient::new(auth);
    /// let image_service = ImageServiceClient::new(
    ///     "https://sampleserver6.arcgisonline.com/arcgis/rest/services/NLCDLandCover2001/ImageServer",
    ///     &client
    /// );
    ///
    /// let params = ExportImageParametersBuilder::default()
    ///     .bbox("-120,40,-119,41")
    ///     .size("400,400")
    ///     .format("png")
    ///     .build()
    ///     .expect("Valid parameters");
    ///
    /// let result = image_service.export_image(params).await?;
    /// tracing::info!(url = %result.href(), "Image exported");
    /// # Ok(())
    /// # }
    /// ```
    #[instrument(skip(self, params))]
    pub async fn export_image(&self, params: ExportImageParameters) -> Result<ExportImageResult> {
        tracing::debug!("Exporting image");

        let export_url = format!("{}/exportImage", self.url);

        let params_json = serde_json::to_string(&params)?;

        let response = self
            .client
            .http()
            .get(&export_url)
            .query(&[("f", "json"), ("params", &params_json)])
            .send()
            .await?;

        let result: ExportImageResult = response.json().await?;

        tracing::debug!(url = %result.href(), "Image exported");

        Ok(result)
    }

    /// Identifies pixel values at a location.
    ///
    /// Returns the pixel value(s) and optionally catalog items and geometry.
    ///
    /// # Arguments
    ///
    /// * `geometry` - Point or polygon geometry to identify at
    ///
    /// # Returns
    ///
    /// Identify result containing pixel values and metadata.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use arcgis::{ApiKeyAuth, ArcGISClient, ImageServiceClient, ArcGISGeometry};
    /// use arcgis::geo_types::{Geometry, Point};
    ///
    /// # async fn example() -> arcgis::Result<()> {
    /// let auth = ApiKeyAuth::new("YOUR_API_KEY");
    /// let client = ArcGISClient::new(auth);
    /// let image_service = ImageServiceClient::new(
    ///     "https://sampleserver6.arcgisonline.com/arcgis/rest/services/NLCDLandCover2001/ImageServer",
    ///     &client
    /// );
    ///
    /// let point = Point::new(-120.0, 40.0);
    /// let geom: Geometry = point.into();
    /// let geometry = ArcGISGeometry::from_geo_types(&geom)?;
    /// let result = image_service.identify(&geometry).await?;
    ///
    /// if let Some(value) = result.value() {
    ///     tracing::info!(value = %value, "Pixel value");
    /// }
    /// # Ok(())
    /// # }
    /// ```
    #[instrument(skip(self))]
    pub async fn identify(&self, geometry: &ArcGISGeometry) -> Result<ImageIdentifyResult> {
        tracing::debug!("Identifying pixel value");

        let identify_url = format!("{}/identify", self.url);

        let geometry_json = serde_json::to_string(geometry)?;
        let geometry_type = match geometry {
            ArcGISGeometry::Point(_) => "esriGeometryPoint",
            ArcGISGeometry::Polygon(_) => "esriGeometryPolygon",
            _ => "esriGeometryPoint",
        };

        let response = self
            .client
            .http()
            .get(&identify_url)
            .query(&[
                ("f", "json"),
                ("geometry", &geometry_json),
                ("geometryType", geometry_type),
            ])
            .send()
            .await?;

        let result: ImageIdentifyResult = response.json().await?;

        tracing::debug!("Identification complete");

        Ok(result)
    }

    /// Identifies pixel values with custom parameters.
    ///
    /// Allows full control over mosaic rules, rendering rules, and return options.
    ///
    /// # Arguments
    ///
    /// * `params` - Identify parameters
    ///
    /// # Returns
    ///
    /// Identify result containing pixel values and metadata.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use arcgis::{ApiKeyAuth, ArcGISClient, ImageServiceClient, IdentifyParametersBuilder, ArcGISGeometry};
    /// use arcgis::geo_types::{Geometry, Point};
    ///
    /// # async fn example() -> arcgis::Result<()> {
    /// let auth = ApiKeyAuth::new("YOUR_API_KEY");
    /// let client = ArcGISClient::new(auth);
    /// let image_service = ImageServiceClient::new(
    ///     "https://sampleserver6.arcgisonline.com/arcgis/rest/services/NLCDLandCover2001/ImageServer",
    ///     &client
    /// );
    ///
    /// let point = Point::new(-120.0, 40.0);
    /// let geom: Geometry = point.into();
    /// let geometry = ArcGISGeometry::from_geo_types(&geom)?;
    /// let geometry_json = serde_json::to_string(&geometry)?;
    ///
    /// let params = IdentifyParametersBuilder::default()
    ///     .geometry(geometry_json)
    ///     .geometry_type("esriGeometryPoint")
    ///     .return_catalog_items(true)
    ///     .build()
    ///     .expect("Valid parameters");
    ///
    /// let result = image_service.identify_with_params(params).await?;
    /// # Ok(())
    /// # }
    /// ```
    #[instrument(skip(self, params))]
    pub async fn identify_with_params(
        &self,
        params: IdentifyParameters,
    ) -> Result<ImageIdentifyResult> {
        tracing::debug!("Identifying with custom parameters");

        let identify_url = format!("{}/identify", self.url);

        let params_json = serde_json::to_string(&params)?;

        let response = self
            .client
            .http()
            .get(&identify_url)
            .query(&[("f", "json"), ("params", &params_json)])
            .send()
            .await?;

        let result: ImageIdentifyResult = response.json().await?;

        tracing::debug!("Identification complete");

        Ok(result)
    }

    /// Samples pixel values along a geometry.
    ///
    /// Useful for creating elevation profiles or extracting values along transects.
    ///
    /// # Arguments
    ///
    /// * `params` - Sample parameters (geometry, sample count/distance, etc.)
    ///
    /// # Returns
    ///
    /// Sample result containing sample points and values.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use arcgis::{ApiKeyAuth, ArcGISClient, ImageServiceClient, SampleParametersBuilder};
    ///
    /// # async fn example() -> arcgis::Result<()> {
    /// let auth = ApiKeyAuth::new("YOUR_API_KEY");
    /// let client = ArcGISClient::new(auth);
    /// let image_service = ImageServiceClient::new(
    ///     "https://sampleserver6.arcgisonline.com/arcgis/rest/services/NLCDLandCover2001/ImageServer",
    ///     &client
    /// );
    ///
    /// let params = SampleParametersBuilder::default()
    ///     .geometry("{\"paths\":[[[-120,40],[-119,41]]]}")
    ///     .geometry_type("esriGeometryPolyline")
    ///     .sample_count(100u32)
    ///     .build()
    ///     .expect("Valid parameters");
    ///
    /// let result = image_service.get_samples(params).await?;
    /// tracing::info!(count = result.samples().len(), "Samples retrieved");
    /// # Ok(())
    /// # }
    /// ```
    #[instrument(skip(self, params))]
    pub async fn get_samples(&self, params: SampleParameters) -> Result<SampleResult> {
        tracing::debug!("Getting samples");

        let samples_url = format!("{}/getSamples", self.url);

        let params_json = serde_json::to_string(&params)?;

        let response = self
            .client
            .http()
            .get(&samples_url)
            .query(&[("f", "json"), ("params", &params_json)])
            .send()
            .await?;

        let result: SampleResult = response.json().await?;

        tracing::debug!(count = result.samples().len(), "Samples retrieved");

        Ok(result)
    }

    /// Computes histograms for the image.
    ///
    /// Returns histogram data and statistics for each band.
    ///
    /// # Arguments
    ///
    /// * `params` - Histogram parameters (geometry, mosaic rule, etc.)
    ///
    /// # Returns
    ///
    /// Histogram result containing per-band histograms and statistics.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use arcgis::{ApiKeyAuth, ArcGISClient, ImageServiceClient, HistogramParametersBuilder};
    ///
    /// # async fn example() -> arcgis::Result<()> {
    /// let auth = ApiKeyAuth::new("YOUR_API_KEY");
    /// let client = ArcGISClient::new(auth);
    /// let image_service = ImageServiceClient::new(
    ///     "https://sampleserver6.arcgisonline.com/arcgis/rest/services/NLCDLandCover2001/ImageServer",
    ///     &client
    /// );
    ///
    /// let params = HistogramParametersBuilder::default()
    ///     .build()
    ///     .expect("Valid parameters");
    ///
    /// let result = image_service.compute_histograms(params).await?;
    /// tracing::info!(bands = result.histograms().len(), "Histograms computed");
    /// # Ok(())
    /// # }
    /// ```
    #[instrument(skip(self, params))]
    pub async fn compute_histograms(&self, params: HistogramParameters) -> Result<HistogramResult> {
        tracing::debug!("Computing histograms");

        let histogram_url = format!("{}/computeHistograms", self.url);

        let params_json = serde_json::to_string(&params)?;

        let response = self
            .client
            .http()
            .get(&histogram_url)
            .query(&[("f", "json"), ("params", &params_json)])
            .send()
            .await?;

        let result: HistogramResult = response.json().await?;

        tracing::debug!(bands = result.histograms().len(), "Histograms computed");

        Ok(result)
    }

    /// Retrieves raster information and metadata.
    ///
    /// Returns details about the raster dataset including bands, extent, and pixel type.
    ///
    /// # Returns
    ///
    /// Raster information metadata.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use arcgis::{ApiKeyAuth, ArcGISClient, ImageServiceClient};
    ///
    /// # async fn example() -> arcgis::Result<()> {
    /// let auth = ApiKeyAuth::new("YOUR_API_KEY");
    /// let client = ArcGISClient::new(auth);
    /// let image_service = ImageServiceClient::new(
    ///     "https://sampleserver6.arcgisonline.com/arcgis/rest/services/NLCDLandCover2001/ImageServer",
    ///     &client
    /// );
    ///
    /// let info = image_service.get_raster_info().await?;
    /// if let Some(band_count) = info.band_count() {
    ///     tracing::info!(bands = band_count, "Raster info");
    /// }
    /// # Ok(())
    /// # }
    /// ```
    #[instrument(skip(self))]
    pub async fn get_raster_info(&self) -> Result<RasterInfo> {
        tracing::debug!("Getting raster info");

        let response = self
            .client
            .http()
            .get(&self.url)
            .query(&[("f", "json")])
            .send()
            .await?;

        let info: RasterInfo = response.json().await?;

        tracing::debug!("Raster info retrieved");

        Ok(info)
    }
}
