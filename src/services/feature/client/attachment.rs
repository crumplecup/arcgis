//! Attachment operations for the Feature Service client.

use super::super::{
    AddAttachmentResult, AttachmentInfo, AttachmentInfosResponse, AttachmentSource,
    DeleteAttachmentsResponse, DownloadResult, DownloadTarget,
};
use super::FeatureServiceClient;
use crate::{AttachmentId, LayerId, ObjectId, Result};
use futures::StreamExt;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tracing::instrument;

impl<'a> FeatureServiceClient<'a> {
    /// Queries attachments for a specific feature.
    ///
    /// Returns metadata about all attachments associated with the feature.
    /// To download attachment data, use [`download_attachment`](Self::download_attachment).
    ///
    /// # Arguments
    ///
    /// * `layer_id` - The layer containing the feature
    /// * `object_id` - The feature to query attachments for
    ///
    /// # Example
    ///
    /// ```no_run
    /// use arcgis::{ArcGISClient, ApiKeyAuth, FeatureServiceClient, LayerId, ObjectId};
    ///
    /// # async fn example() -> arcgis::Result<()> {
    /// let auth = ApiKeyAuth::new("YOUR_API_KEY");
    /// let client = ArcGISClient::new(auth);
    /// let service = FeatureServiceClient::new("https://example.com/FeatureServer", &client);
    ///
    /// let attachments = service
    ///     .query_attachments(LayerId::new(0), ObjectId::new(123))
    ///     .await?;
    ///
    /// for attachment in &attachments {
    ///     println!("Attachment: {} ({} bytes)", attachment.name(), attachment.size());
    /// }
    /// # Ok(())
    /// # }
    /// ```
    #[instrument(skip(self), fields(layer_id = %layer_id, object_id = %object_id))]
    pub async fn query_attachments(
        &self,
        layer_id: LayerId,
        object_id: ObjectId,
    ) -> Result<Vec<AttachmentInfo>> {
        tracing::debug!("Querying attachments for feature");

        let url = format!("{}/{}/{}/attachments", self.base_url, layer_id, object_id);

        tracing::debug!(url = %url, "Sending queryAttachments request");

        let mut request = self.client.http().get(&url).query(&[("f", "json")]);
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
            tracing::error!(status = %status, error = %error_text, "queryAttachments failed");
            return Err(crate::Error::from(crate::ErrorKind::Api {
                code: status.as_u16() as i32,
                message: format!("HTTP {}: {}", status, error_text),
            }));
        }

        let result: AttachmentInfosResponse = response.json().await?;

        tracing::info!(
            attachment_count = result.attachment_infos.len(),
            "queryAttachments completed"
        );

        Ok(result.attachment_infos)
    }

    /// Deletes one or more attachments from a feature.
    ///
    /// # Arguments
    ///
    /// * `layer_id` - The layer containing the feature
    /// * `object_id` - The feature that owns the attachments
    /// * `attachment_ids` - Vector of attachment IDs to delete
    ///
    /// # Example
    ///
    /// ```no_run
    /// use arcgis::{ArcGISClient, ApiKeyAuth, FeatureServiceClient, LayerId, ObjectId, AttachmentId};
    ///
    /// # async fn example() -> arcgis::Result<()> {
    /// let auth = ApiKeyAuth::new("YOUR_API_KEY");
    /// let client = ArcGISClient::new(auth);
    /// let service = FeatureServiceClient::new("https://example.com/FeatureServer", &client);
    ///
    /// let ids_to_delete = vec![AttachmentId::new(1), AttachmentId::new(2)];
    ///
    /// let result = service
    ///     .delete_attachments(LayerId::new(0), ObjectId::new(123), ids_to_delete)
    ///     .await?;
    ///
    /// for item in &result.delete_attachment_results {
    ///     if *item.success() {
    ///         println!("Deleted attachment from feature {}", item.object_id());
    ///     }
    /// }
    /// # Ok(())
    /// # }
    /// ```
    #[instrument(skip(self, attachment_ids), fields(layer_id = %layer_id, object_id = %object_id, count = attachment_ids.len()))]
    pub async fn delete_attachments(
        &self,
        layer_id: LayerId,
        object_id: ObjectId,
        attachment_ids: Vec<AttachmentId>,
    ) -> Result<DeleteAttachmentsResponse> {
        tracing::debug!("Deleting attachments from feature");

        let url = format!(
            "{}/{}/{}/deleteAttachments",
            self.base_url, layer_id, object_id
        );
        // Convert AttachmentIds to comma-separated string
        let attachment_ids_str = attachment_ids
            .iter()
            .map(|id| id.to_string())
            .collect::<Vec<_>>()
            .join(",");

        tracing::debug!(
            url = %url,
            attachment_ids = %attachment_ids_str,
            "Sending deleteAttachments request"
        );

        let mut form = vec![
            ("attachmentIds", attachment_ids_str.as_str()),
            ("f", "json"),
        ];

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
            tracing::error!(status = %status, error = %error_text, "deleteAttachments failed");
            return Err(crate::Error::from(crate::ErrorKind::Api {
                code: status.as_u16() as i32,
                message: format!("HTTP {}: {}", status, error_text),
            }));
        }

        let result: DeleteAttachmentsResponse = response.json().await?;

        let success_count = result
            .delete_attachment_results
            .iter()
            .filter(|r| *r.success())
            .count();

        tracing::info!(
            success_count = success_count,
            total_count = result.delete_attachment_results.len(),
            "deleteAttachments completed"
        );

        Ok(result)
    }

    /// Adds an attachment to a feature.
    ///
    /// Uploads a file and associates it with the specified feature.
    /// Supports streaming for efficient handling of large files.
    ///
    /// # Arguments
    ///
    /// * `layer_id` - The layer containing the feature
    /// * `object_id` - The feature to attach the file to
    /// * `source` - The attachment source (file path, bytes, or stream)
    ///
    /// # Example
    ///
    /// ```no_run
    /// use arcgis::{ArcGISClient, ApiKeyAuth, FeatureServiceClient, LayerId, ObjectId, AttachmentSource};
    ///
    /// # async fn example() -> arcgis::Result<()> {
    /// let auth = ApiKeyAuth::new("YOUR_API_KEY");
    /// let client = ArcGISClient::new(auth);
    /// let service = FeatureServiceClient::new("https://example.com/FeatureServer", &client);
    ///
    /// // From file path (streaming)
    /// let result = service
    ///     .add_attachment(
    ///         LayerId::new(0),
    ///         ObjectId::new(123),
    ///         AttachmentSource::from_path("/path/to/photo.jpg"),
    ///     )
    ///     .await?;
    ///
    /// if *result.success() {
    ///     println!("Attachment added successfully");
    /// }
    ///
    /// // From bytes
    /// let image_data = vec![0xFF, 0xD8, 0xFF]; // JPEG header...
    /// let result = service
    ///     .add_attachment(
    ///         LayerId::new(0),
    ///         ObjectId::new(456),
    ///         AttachmentSource::from_bytes("photo.jpg", image_data),
    ///     )
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    #[instrument(skip(self, source), fields(layer_id = %layer_id, object_id = %object_id))]
    pub async fn add_attachment(
        &self,
        layer_id: LayerId,
        object_id: ObjectId,
        source: AttachmentSource,
    ) -> Result<AddAttachmentResult> {
        tracing::debug!("Adding attachment to feature");

        let url = format!("{}/{}/{}/addAttachment", self.base_url, layer_id, object_id);
        // Build multipart form
        let mut form = reqwest::multipart::Form::new();

        // Add the file part based on source
        match source {
            AttachmentSource::Path(path) => {
                tracing::debug!(path = %path.display(), "Loading attachment from file");

                // Get filename
                let filename = path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("attachment")
                    .to_string();

                // Open file for streaming
                let file = tokio::fs::File::open(&path).await?;
                let metadata = file.metadata().await?;
                let file_size = metadata.len();

                // Detect content type
                let content_type = mime_guess::from_path(&path)
                    .first_or_octet_stream()
                    .to_string();

                tracing::debug!(
                    filename = %filename,
                    content_type = %content_type,
                    size = file_size,
                    "File opened for streaming"
                );

                // Create streaming body
                let part = reqwest::multipart::Part::stream(reqwest::Body::from(file))
                    .file_name(filename)
                    .mime_str(&content_type)?;

                form = form.part("attachment", part);
            }
            AttachmentSource::Bytes {
                filename,
                data,
                content_type,
            } => {
                tracing::debug!(
                    filename = %filename,
                    size = data.len(),
                    "Using in-memory attachment data"
                );

                let content_type = content_type.unwrap_or_else(|| {
                    mime_guess::from_path(&filename)
                        .first_or_octet_stream()
                        .to_string()
                });

                tracing::debug!(content_type = %content_type, "Content type determined");

                let part = reqwest::multipart::Part::bytes(data)
                    .file_name(filename)
                    .mime_str(&content_type)?;

                form = form.part("attachment", part);
            }
            AttachmentSource::Stream {
                filename,
                mut reader,
                content_type,
                size,
            } => {
                tracing::debug!(
                    filename = %filename,
                    size = ?size,
                    "Streaming from async reader"
                );

                // Read from the async reader into a buffer
                let mut buffer = Vec::new();
                reader.read_to_end(&mut buffer).await?;

                tracing::debug!(bytes_read = buffer.len(), "Data read from stream");

                let part = reqwest::multipart::Part::bytes(buffer)
                    .file_name(filename)
                    .mime_str(&content_type)?;

                form = form.part("attachment", part);
            }
        }

        // Add standard parameters
        form = form.text("f", "json");

        // Add token if required by auth provider
        if let Some(token) = self.client.get_token_if_required().await? {
            form = form.text("token", token);
        }

        tracing::debug!(url = %url, "Sending addAttachment request");

        let response = self.client.http().post(&url).multipart(form).send().await?;

        let status = response.status();
        if !status.is_success() {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|e| format!("Failed to read error: {}", e));
            tracing::error!(status = %status, error = %error_text, "addAttachment failed");
            return Err(crate::Error::from(crate::ErrorKind::Api {
                code: status.as_u16() as i32,
                message: format!("HTTP {}: {}", status, error_text),
            }));
        }

        let result: AddAttachmentResult = response.json().await?;

        tracing::info!(
            success = result.success(),
            object_id = %result.object_id(),
            "addAttachment completed"
        );

        Ok(result)
    }

    /// Updates an existing attachment.
    ///
    /// Replaces the attachment file while keeping the same attachment ID.
    /// Supports streaming for efficient handling of large files.
    ///
    /// # Arguments
    ///
    /// * `layer_id` - The layer containing the feature
    /// * `object_id` - The feature that owns the attachment
    /// * `attachment_id` - The attachment to update
    /// * `source` - The new attachment source (file path, bytes, or stream)
    ///
    /// # Example
    ///
    /// ```no_run
    /// use arcgis::{ArcGISClient, ApiKeyAuth, FeatureServiceClient, LayerId, ObjectId, AttachmentId, AttachmentSource};
    ///
    /// # async fn example() -> arcgis::Result<()> {
    /// let auth = ApiKeyAuth::new("YOUR_API_KEY");
    /// let client = ArcGISClient::new(auth);
    /// let service = FeatureServiceClient::new("https://example.com/FeatureServer", &client);
    ///
    /// let result = service
    ///     .update_attachment(
    ///         LayerId::new(0),
    ///         ObjectId::new(123),
    ///         AttachmentId::new(5),
    ///         AttachmentSource::from_path("/path/to/updated_photo.jpg"),
    ///     )
    ///     .await?;
    ///
    /// if *result.success() {
    ///     println!("Attachment updated successfully");
    /// }
    /// # Ok(())
    /// # }
    /// ```
    #[instrument(skip(self, source), fields(layer_id = %layer_id, object_id = %object_id, attachment_id = %attachment_id))]
    pub async fn update_attachment(
        &self,
        layer_id: LayerId,
        object_id: ObjectId,
        attachment_id: AttachmentId,
        source: AttachmentSource,
    ) -> Result<crate::UpdateAttachmentResult> {
        tracing::debug!("Updating attachment");

        let url = format!(
            "{}/{}/{}/updateAttachment",
            self.base_url, layer_id, object_id
        );
        // Build multipart form
        let mut form = reqwest::multipart::Form::new();

        // Add attachment ID
        form = form.text("attachmentId", attachment_id.to_string());

        // Add the file part based on source
        match source {
            AttachmentSource::Path(path) => {
                tracing::debug!(path = %path.display(), "Loading attachment from file");

                let filename = path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("attachment")
                    .to_string();

                let file = tokio::fs::File::open(&path).await?;
                let metadata = file.metadata().await?;
                let file_size = metadata.len();

                let content_type = mime_guess::from_path(&path)
                    .first_or_octet_stream()
                    .to_string();

                tracing::debug!(
                    filename = %filename,
                    content_type = %content_type,
                    size = file_size,
                    "File opened for streaming"
                );

                let part = reqwest::multipart::Part::stream(reqwest::Body::from(file))
                    .file_name(filename)
                    .mime_str(&content_type)?;

                form = form.part("attachment", part);
            }
            AttachmentSource::Bytes {
                filename,
                data,
                content_type,
            } => {
                tracing::debug!(
                    filename = %filename,
                    size = data.len(),
                    "Using in-memory attachment data"
                );

                let content_type = content_type.unwrap_or_else(|| {
                    mime_guess::from_path(&filename)
                        .first_or_octet_stream()
                        .to_string()
                });

                tracing::debug!(content_type = %content_type, "Content type determined");

                let part = reqwest::multipart::Part::bytes(data)
                    .file_name(filename)
                    .mime_str(&content_type)?;

                form = form.part("attachment", part);
            }
            AttachmentSource::Stream {
                filename,
                mut reader,
                content_type,
                size,
            } => {
                tracing::debug!(
                    filename = %filename,
                    size = ?size,
                    "Streaming from async reader"
                );

                let mut buffer = Vec::new();
                reader.read_to_end(&mut buffer).await?;

                tracing::debug!(bytes_read = buffer.len(), "Data read from stream");

                let part = reqwest::multipart::Part::bytes(buffer)
                    .file_name(filename)
                    .mime_str(&content_type)?;

                form = form.part("attachment", part);
            }
        }

        // Add standard parameters
        form = form.text("f", "json");

        // Add token if required by auth provider
        if let Some(token) = self.client.get_token_if_required().await? {
            form = form.text("token", token);
        }

        tracing::debug!(url = %url, "Sending updateAttachment request");

        let response = self.client.http().post(&url).multipart(form).send().await?;

        let status = response.status();
        if !status.is_success() {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|e| format!("Failed to read error: {}", e));
            tracing::error!(status = %status, error = %error_text, "updateAttachment failed");
            return Err(crate::Error::from(crate::ErrorKind::Api {
                code: status.as_u16() as i32,
                message: format!("HTTP {}: {}", status, error_text),
            }));
        }

        let result: crate::UpdateAttachmentResult = response.json().await?;

        tracing::info!(
            success = result.success(),
            object_id = %result.object_id(),
            "updateAttachment completed"
        );

        Ok(result)
    }

    /// Downloads an attachment from a feature.
    ///
    /// Streams the attachment data to the specified target (file, bytes, or writer).
    /// Efficient for large files as it doesn't load the entire file into memory unless
    /// using the Bytes target.
    ///
    /// # Arguments
    ///
    /// * `layer_id` - The layer containing the feature
    /// * `object_id` - The feature that owns the attachment
    /// * `attachment_id` - The attachment to download
    /// * `target` - Where to write the downloaded data
    ///
    /// # Example
    ///
    /// ```no_run
    /// use arcgis::{ArcGISClient, ApiKeyAuth, FeatureServiceClient, LayerId, ObjectId, AttachmentId, DownloadTarget};
    /// use std::path::PathBuf;
    ///
    /// # async fn example() -> arcgis::Result<()> {
    /// let auth = ApiKeyAuth::new("YOUR_API_KEY");
    /// let client = ArcGISClient::new(auth);
    /// let service = FeatureServiceClient::new("https://example.com/FeatureServer", &client);
    ///
    /// // Download to file (streaming)
    /// let result = service
    ///     .download_attachment(
    ///         LayerId::new(0),
    ///         ObjectId::new(123),
    ///         AttachmentId::new(5),
    ///         DownloadTarget::to_path("/path/to/save/photo.jpg"),
    ///     )
    ///     .await?;
    ///
    /// if let Some(path) = result.path() {
    ///     println!("Downloaded to {:?}", path);
    /// }
    ///
    /// // Download to memory
    /// let result = service
    ///     .download_attachment(
    ///         LayerId::new(0),
    ///         ObjectId::new(123),
    ///         AttachmentId::new(6),
    ///         DownloadTarget::to_bytes(),
    ///     )
    ///     .await?;
    ///
    /// if let Some(bytes) = result.bytes() {
    ///     println!("Downloaded {} bytes", bytes.len());
    /// }
    /// # Ok(())
    /// # }
    /// ```
    #[instrument(skip(self, target), fields(layer_id = %layer_id, object_id = %object_id, attachment_id = %attachment_id))]
    pub async fn download_attachment(
        &self,
        layer_id: LayerId,
        object_id: ObjectId,
        attachment_id: AttachmentId,
        target: DownloadTarget,
    ) -> Result<DownloadResult> {
        tracing::debug!("Downloading attachment");

        let url = format!(
            "{}/{}/{}/attachments/{}",
            self.base_url, layer_id, object_id, attachment_id
        );
        tracing::debug!(url = %url, "Sending download request");

        let mut request = self.client.http().get(&url);
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
            tracing::error!(status = %status, error = %error_text, "download attachment failed");
            return Err(crate::Error::from(crate::ErrorKind::Api {
                code: status.as_u16() as i32,
                message: format!("HTTP {}: {}", status, error_text),
            }));
        }

        // Stream the response based on target
        match target {
            DownloadTarget::Path(path) => {
                tracing::debug!(path = %path.display(), "Streaming to file");

                let mut file = tokio::fs::File::create(&path).await?;
                let mut stream = response.bytes_stream();
                let mut total_bytes = 0u64;

                while let Some(chunk_result) = stream.next().await {
                    let chunk = chunk_result?;
                    file.write_all(&chunk).await?;
                    total_bytes += chunk.len() as u64;
                }

                file.flush().await?;

                tracing::info!(
                    path = %path.display(),
                    bytes = total_bytes,
                    "Download to file completed"
                );

                Ok(DownloadResult::Path(path))
            }
            DownloadTarget::Bytes => {
                tracing::debug!("Collecting bytes to memory");

                let mut stream = response.bytes_stream();
                let mut buffer = Vec::new();

                while let Some(chunk_result) = stream.next().await {
                    let chunk = chunk_result?;
                    buffer.extend_from_slice(&chunk);
                }

                tracing::info!(bytes = buffer.len(), "Download to bytes completed");

                Ok(DownloadResult::Bytes(buffer))
            }
            DownloadTarget::Writer(mut writer) => {
                tracing::debug!("Streaming to writer");

                let mut stream = response.bytes_stream();
                let mut total_bytes = 0u64;

                while let Some(chunk_result) = stream.next().await {
                    let chunk = chunk_result?;
                    writer.write_all(&chunk).await?;
                    total_bytes += chunk.len() as u64;
                }

                writer.flush().await?;

                tracing::info!(bytes = total_bytes, "Download to writer completed");

                Ok(DownloadResult::Written(total_bytes))
            }
        }
    }
}
