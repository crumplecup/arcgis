//! Attachment operations for Feature Services.
//!
//! Feature attachments allow you to associate files (images, PDFs, etc.)
//! with individual features. Each feature can have multiple attachments.
//!
//! # Operations
//!
//! - [`query_attachments`](super::FeatureServiceClient::query_attachments) - List attachments for a feature
//! - [`add_attachment`](super::FeatureServiceClient::add_attachment) - Add a new attachment
//! - [`update_attachment`](super::FeatureServiceClient::update_attachment) - Update an existing attachment
//! - [`delete_attachments`](super::FeatureServiceClient::delete_attachments) - Delete one or more attachments
//! - [`download_attachment`](super::FeatureServiceClient::download_attachment) - Download attachment data
//!
//! # Example
//!
//! ```no_run
//! use arcgis::{ArcGISClient, ApiKeyAuth, FeatureServiceClient, LayerId, ObjectId, AttachmentSource, DownloadTarget};
//!
//! # async fn example() -> arcgis::Result<()> {
//! let auth = ApiKeyAuth::new("YOUR_API_KEY");
//! let client = ArcGISClient::new(auth);
//! let service = FeatureServiceClient::new("https://example.com/FeatureServer", &client);
//!
//! // Query attachments for a feature
//! let attachments = service
//!     .query_attachments(LayerId::new(0), ObjectId::new(123))
//!     .await?;
//!
//! println!("Feature has {} attachments", attachments.len());
//!
//! // Add attachment from file
//! service
//!     .add_attachment(
//!         LayerId::new(0),
//!         ObjectId::new(123),
//!         AttachmentSource::from_path("photo.jpg"),
//!     )
//!     .await?;
//!
//! // Download attachment
//! if let Some(attachment) = attachments.first() {
//!     service
//!         .download_attachment(
//!             LayerId::new(0),
//!             ObjectId::new(123),
//!             *attachment.id(),
//!             DownloadTarget::Path("downloaded.jpg".into()),
//!         )
//!         .await?;
//! }
//! # Ok(())
//! # }
//! ```

use crate::AttachmentId;
use derive_getters::Getters;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tokio::io::{AsyncRead, AsyncWrite};

/// Information about a feature attachment.
///
/// Returned by [`query_attachments`](super::FeatureServiceClient::query_attachments).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Getters)]
#[serde(rename_all = "camelCase")]
pub struct AttachmentInfo {
    /// Unique identifier for the attachment.
    id: AttachmentId,

    /// Attachment filename.
    name: String,

    /// File size in bytes.
    size: u64,

    /// MIME type of the attachment.
    content_type: String,

    /// Optional keywords/tags for the attachment.
    #[serde(skip_serializing_if = "Option::is_none")]
    keywords: Option<String>,
}

/// Response from querying attachments.
///
/// Contains an array of attachment info objects.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AttachmentInfosResponse {
    /// Array of attachments for the feature.
    pub attachment_infos: Vec<AttachmentInfo>,
}

/// Result of adding an attachment.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Getters)]
#[serde(rename_all = "camelCase")]
pub struct AddAttachmentResult {
    /// ObjectID of the feature the attachment was added to (optional in some API responses).
    #[serde(default)]
    object_id: Option<crate::ObjectId>,

    /// GlobalID of the newly created attachment (if enabled).
    #[serde(skip_serializing_if = "Option::is_none")]
    global_id: Option<String>,

    /// Whether the operation succeeded.
    success: bool,
}

/// Wrapper for addAttachment API response.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct AddAttachmentResponse {
    /// The nested result object.
    pub(crate) add_attachment_result: AddAttachmentResult,
}

/// Result of updating an attachment.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Getters)]
#[serde(rename_all = "camelCase")]
pub struct UpdateAttachmentResult {
    /// ObjectID of the feature.
    object_id: crate::ObjectId,

    /// Whether the operation succeeded.
    success: bool,
}

/// Wrapper for updateAttachment API response.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct UpdateAttachmentResponse {
    /// The nested result object.
    pub(crate) update_attachment_result: UpdateAttachmentResult,
}

/// Response from deleting attachments.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeleteAttachmentsResponse {
    /// Array of delete results (one per attachment).
    pub delete_attachment_results: Vec<DeleteAttachmentResult>,
}

/// Individual result for a single attachment deletion.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Getters)]
#[serde(rename_all = "camelCase")]
pub struct DeleteAttachmentResult {
    /// ObjectID of the feature.
    object_id: crate::ObjectId,

    /// GlobalID of the attachment (if enabled).
    #[serde(skip_serializing_if = "Option::is_none")]
    global_id: Option<String>,

    /// Whether the operation succeeded.
    success: bool,
}

/// Source for attachment file data.
///
/// Supports loading from a file path, in-memory bytes, or async stream.
pub enum AttachmentSource {
    /// Stream from file path (efficient for large files).
    ///
    /// The file will be opened asynchronously and streamed to the server
    /// without loading the entire file into memory.
    Path(PathBuf),

    /// Use in-memory bytes (convenient for small files or generated data).
    ///
    /// The entire file content is held in memory during upload.
    Bytes {
        /// Filename to use for the attachment.
        filename: String,
        /// File data.
        data: Vec<u8>,
        /// Optional explicit content type. If None, will be guessed from filename.
        content_type: Option<String>,
    },

    /// Stream from async reader (advanced use cases).
    ///
    /// Allows streaming from any AsyncRead source. Useful for piping data
    /// from other sources without intermediate buffering.
    Stream {
        /// Filename to use for the attachment.
        filename: String,
        /// Async reader providing the file data.
        reader: Box<dyn AsyncRead + Send + Sync + Unpin>,
        /// MIME content type.
        content_type: String,
        /// Optional known size (helps with progress tracking).
        size: Option<u64>,
    },
}

impl AttachmentSource {
    /// Creates an attachment source from a file path.
    ///
    /// The file will be streamed efficiently without loading into memory.
    ///
    /// # Example
    ///
    /// ```
    /// use arcgis::AttachmentSource;
    ///
    /// let source = AttachmentSource::from_path("/path/to/photo.jpg");
    /// ```
    pub fn from_path(path: impl Into<PathBuf>) -> Self {
        Self::Path(path.into())
    }

    /// Creates an attachment source from in-memory bytes.
    ///
    /// Content type will be auto-detected from the filename extension.
    ///
    /// # Example
    ///
    /// ```
    /// use arcgis::AttachmentSource;
    ///
    /// let data = vec![0xFF, 0xD8, 0xFF]; // JPEG data...
    /// let source = AttachmentSource::from_bytes("photo.jpg", data);
    /// ```
    pub fn from_bytes(filename: impl Into<String>, data: Vec<u8>) -> Self {
        Self::Bytes {
            filename: filename.into(),
            data,
            content_type: None,
        }
    }

    /// Creates an attachment source from in-memory bytes with explicit content type.
    ///
    /// # Example
    ///
    /// ```
    /// use arcgis::AttachmentSource;
    ///
    /// let data = vec![0x25, 0x50, 0x44, 0x46]; // PDF data...
    /// let source = AttachmentSource::from_bytes_with_type(
    ///     "document.pdf",
    ///     data,
    ///     "application/pdf",
    /// );
    /// ```
    pub fn from_bytes_with_type(
        filename: impl Into<String>,
        data: Vec<u8>,
        content_type: impl Into<String>,
    ) -> Self {
        Self::Bytes {
            filename: filename.into(),
            data,
            content_type: Some(content_type.into()),
        }
    }

    /// Creates an attachment source from an async reader.
    ///
    /// This is an advanced method for streaming from arbitrary sources.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use arcgis::AttachmentSource;
    /// use tokio::fs::File;
    ///
    /// # async fn example() -> std::io::Result<()> {
    /// let file = File::open("large_file.zip").await?;
    /// let source = AttachmentSource::from_stream(
    ///     "large_file.zip",
    ///     Box::new(file),
    ///     "application/zip",
    ///     Some(1024 * 1024 * 100), // 100 MB
    /// );
    /// # Ok(())
    /// # }
    /// ```
    pub fn from_stream(
        filename: impl Into<String>,
        reader: Box<dyn AsyncRead + Send + Sync + Unpin>,
        content_type: impl Into<String>,
        size: Option<u64>,
    ) -> Self {
        Self::Stream {
            filename: filename.into(),
            reader,
            content_type: content_type.into(),
            size,
        }
    }
}

/// Target for attachment download.
///
/// Specifies where to write the downloaded attachment data.
pub enum DownloadTarget {
    /// Download to file path.
    ///
    /// The file will be created (or overwritten) and data streamed to it.
    Path(PathBuf),

    /// Download to in-memory bytes.
    ///
    /// Returns `Vec<u8>` containing the entire attachment. Use for small files only.
    Bytes,

    /// Stream to async writer.
    ///
    /// Allows writing to any AsyncWrite destination.
    Writer(Box<dyn AsyncWrite + Send + Sync + Unpin>),
}

impl DownloadTarget {
    /// Creates a download target for a file path.
    ///
    /// # Example
    ///
    /// ```
    /// use arcgis::DownloadTarget;
    ///
    /// let target = DownloadTarget::to_path("/path/to/save/photo.jpg");
    /// ```
    pub fn to_path(path: impl Into<PathBuf>) -> Self {
        Self::Path(path.into())
    }

    /// Creates a download target for in-memory bytes.
    ///
    /// # Example
    ///
    /// ```
    /// use arcgis::DownloadTarget;
    ///
    /// let target = DownloadTarget::to_bytes();
    /// ```
    pub fn to_bytes() -> Self {
        Self::Bytes
    }

    /// Creates a download target for an async writer.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use arcgis::DownloadTarget;
    /// use tokio::fs::File;
    ///
    /// # async fn example() -> std::io::Result<()> {
    /// let file = File::create("output.dat").await?;
    /// let target = DownloadTarget::to_writer(Box::new(file));
    /// # Ok(())
    /// # }
    /// ```
    pub fn to_writer(writer: Box<dyn AsyncWrite + Send + Sync + Unpin>) -> Self {
        Self::Writer(writer)
    }
}

/// Result of downloading an attachment.
///
/// The variant matches the `DownloadTarget` used.
#[derive(Debug)]
pub enum DownloadResult {
    /// File written to path.
    ///
    /// Contains the path where the file was written.
    Path(PathBuf),

    /// Bytes loaded into memory.
    ///
    /// Contains the attachment data.
    Bytes(Vec<u8>),

    /// Bytes written to writer.
    ///
    /// Contains the number of bytes written.
    Written(u64),
}

impl DownloadResult {
    /// Returns the path if this is a Path result.
    pub fn path(&self) -> Option<&PathBuf> {
        match self {
            Self::Path(p) => Some(p),
            _ => None,
        }
    }

    /// Returns the bytes if this is a Bytes result.
    pub fn bytes(&self) -> Option<&[u8]> {
        match self {
            Self::Bytes(b) => Some(b),
            _ => None,
        }
    }

    /// Returns the bytes count if this is a Written result.
    pub fn written(&self) -> Option<u64> {
        match self {
            Self::Written(n) => Some(*n),
            _ => None,
        }
    }

    /// Consumes the result and returns the bytes if this is a Bytes result.
    pub fn into_bytes(self) -> Option<Vec<u8>> {
        match self {
            Self::Bytes(b) => Some(b),
            _ => None,
        }
    }
}
