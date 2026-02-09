//! Image Service.
//!
//! The Image Service (ImageServer) provides operations for:
//! - **Export**: Export raster images with dynamic rendering
//! - **Identify**: Get pixel values at specific locations
//! - **Sampling**: Sample pixel values along geometries
//! - **Analysis**: Compute histograms and statistics
//! - **Metadata**: Query raster information and catalog
//!
//! # Example
//!
//! ```no_run
//! use arcgis::{ApiKeyAuth, ArcGISClient, ImageServiceClient, ArcGISGeometry, ArcGISPoint};
//!
//! # async fn example() -> arcgis::Result<()> {
//! let auth = ApiKeyAuth::new("YOUR_API_KEY");
//! let client = ArcGISClient::new(auth);
//! let image_service = ImageServiceClient::new(
//!     "https://sampleserver6.arcgisonline.com/arcgis/rest/services/NLCDLandCover2001/ImageServer",
//!     &client
//! );
//!
//! // Get pixel value at a location
//! let geometry = ArcGISGeometry::Point(ArcGISPoint::new(-120.0, 40.0));
//! let value = image_service.identify(&geometry).await?;
//! # Ok(())
//! # }
//! ```

mod client;
mod types;

pub use client::ImageServiceClient;
pub use types::{
    ExportImageParameters, ExportImageParametersBuilder, ExportImageResult, HistogramParameters,
    HistogramParametersBuilder, HistogramResult, IdentifyParameters, IdentifyParametersBuilder,
    ImageIdentifyResult, InterpolationType, MosaicRule, PixelType, RasterInfo, RenderingRule,
    SampleParameters, SampleParametersBuilder, SampleResult,
};
