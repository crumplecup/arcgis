//! Utility functions and helpers.

use crate::Result;

/// Check for an ESRI error embedded in a successful HTTP response.
///
/// ESRI's REST API commonly returns application-level errors as HTTP 200 with
/// a JSON error payload:
///
/// ```json
/// {"error": {"code": 400, "message": "Unable to complete operation", "details": []}}
/// ```
///
/// Call this after verifying the HTTP status is successful but before deserializing
/// the response body.
///
/// # Example
///
/// ```no_run
/// use arcgis::check_esri_error;
///
/// # async fn example() -> arcgis::Result<()> {
/// let response_text = r#"{"error": {"code": 400, "message": "Token required"}}"#;
/// check_esri_error(response_text, "myOperation")?;  // Returns Err
/// # Ok(())
/// # }
/// ```
pub fn check_esri_error(response_text: &str, operation: &str) -> Result<()> {
    if !response_text.contains("\"error\"") {
        return Ok(());
    }

    tracing::error!(operation = %operation, response = %response_text, "ESRI returned error in response body");

    // Try format 1: {"error": {"code": 400, "message": "..."}}
    #[derive(serde::Deserialize)]
    struct ErrorResponse1 {
        error: ErrorDetail1,
    }
    #[derive(serde::Deserialize)]
    struct ErrorDetail1 {
        code: i32,
        message: String,
    }

    if let Ok(err) = serde_json::from_str::<ErrorResponse1>(response_text) {
        return Err(crate::Error::from(crate::ErrorKind::Api {
            code: err.error.code,
            message: err.error.message,
        }));
    }

    // Try format 2: {"success": false, "error": {"message": "..."}}
    #[derive(serde::Deserialize)]
    struct ErrorResponse2 {
        success: bool,
        error: ErrorDetail2,
    }
    #[derive(serde::Deserialize)]
    struct ErrorDetail2 {
        message: String,
        #[serde(default)]
        code: Option<i32>,
    }

    if let Ok(err) = serde_json::from_str::<ErrorResponse2>(response_text) {
        if !err.success {
            return Err(crate::Error::from(crate::ErrorKind::Api {
                code: err.error.code.unwrap_or(0),
                message: err.error.message,
            }));
        }
    }

    Err(crate::Error::from(crate::ErrorKind::Api {
        code: -1,
        message: format!("{} returned unexpected error: {}", operation, response_text),
    }))
}
