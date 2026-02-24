//! Example run tracking and accountability framework.
//!
//! This module provides automated tracking of example execution to create
//! a verifiable audit trail. It prevents false claims about test coverage
//! by requiring actual successful runs to be logged.
//!
//! # Purpose
//!
//! Creates accountability by:
//! - Logging every example run with timestamp
//! - Recording success/failure status
//! - Tracking which methods were actually tested
//! - Capturing error details on failure
//! - Creating a CSV audit trail
//!
//! # Usage
//!
//! Add to the beginning of your example's `main()`:
//!
//! ```no_run
//! use arcgis::example_tracker::ExampleTracker;
//!
//! #[tokio::main]
//! async fn main() -> anyhow::Result<()> {
//!     let tracker = ExampleTracker::new("my_example")
//!         .methods(&["method1", "method2", "method3"])
//!         .service_type("FeatureServiceClient")
//!         .start();
//!
//!     // Your example code here...
//!
//!     // IMPORTANT: Mark success explicitly at the end
//!     tracker.success();
//!     Ok(())
//! }
//! ```
//!
//! The tracker uses RAII pattern - it automatically logs on drop.
//! **You must call `.success()` to mark completion**, otherwise it
//! will be logged as failed (handles early returns via `?` or panics).

use std::fs::OpenOptions;
use std::io::Write;
use std::panic;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::Mutex;
use std::time::Instant;

/// Tracks example execution and logs results to CSV.
///
/// Uses RAII pattern - automatically logs on drop.
/// Logs success if no panic, failure if panic occurred.
pub struct ExampleTracker {
    example_name: String,
    methods_tested: Vec<String>,
    service_type: Option<String>,
    start_time: Instant,
    status: Arc<Mutex<TrackingStatus>>,
}

#[derive(Debug, Clone)]
enum TrackingStatus {
    Running,
    Success,
    Failed(String),
}

impl ExampleTracker {
    /// Creates a new example tracker.
    ///
    /// # Arguments
    ///
    /// * `example_name` - Name of the example (e.g., "feature_service_batch_editing")
    pub fn new(example_name: impl Into<String>) -> Self {
        Self {
            example_name: example_name.into(),
            methods_tested: Vec::new(),
            service_type: None,
            start_time: Instant::now(),
            status: Arc::new(Mutex::new(TrackingStatus::Running)),
        }
    }

    /// Sets the methods tested by this example.
    ///
    /// # Arguments
    ///
    /// * `methods` - Slice of method names tested
    pub fn methods(mut self, methods: &[&str]) -> Self {
        self.methods_tested = methods.iter().map(|s| s.to_string()).collect();
        self
    }

    /// Sets the service type being tested.
    ///
    /// # Arguments
    ///
    /// * `service_type` - Name of the service client (e.g., "FeatureServiceClient")
    pub fn service_type(mut self, service_type: impl Into<String>) -> Self {
        self.service_type = Some(service_type.into());
        self
    }

    /// Starts tracking and sets up panic handler.
    ///
    /// Returns self to allow method chaining and RAII pattern.
    pub fn start(self) -> Self {
        // Set up panic hook to capture panics
        let status = Arc::clone(&self.status);
        let example_name = self.example_name.clone();

        let old_hook = panic::take_hook();
        panic::set_hook(Box::new(move |panic_info| {
            let msg = if let Some(s) = panic_info.payload().downcast_ref::<&str>() {
                s.to_string()
            } else if let Some(s) = panic_info.payload().downcast_ref::<String>() {
                s.clone()
            } else {
                "Unknown panic".to_string()
            };

            let location = if let Some(loc) = panic_info.location() {
                format!(" at {}:{}", loc.file(), loc.line())
            } else {
                String::new()
            };

            let error_msg = format!("Panic: {}{}", msg, location);

            if let Ok(mut status) = status.lock() {
                *status = TrackingStatus::Failed(error_msg.clone());
            }

            tracing::error!(example = %example_name, error = %error_msg, "Example panicked");

            // Call the old hook
            old_hook(panic_info);
        }));

        tracing::info!(example = %self.example_name, "Example tracking started");
        self
    }

    /// Marks the example as successful.
    ///
    /// Call this explicitly at the end of your example if you want to
    /// mark success before the tracker is dropped.
    pub fn success(&self) {
        if let Ok(mut status) = self.status.lock() {
            *status = TrackingStatus::Success;
        }
    }

    /// Marks the example as failed with an error message.
    ///
    /// # Arguments
    ///
    /// * `error` - Error message describing the failure
    pub fn fail(&self, error: impl Into<String>) {
        if let Ok(mut status) = self.status.lock() {
            *status = TrackingStatus::Failed(error.into());
        }
    }

    /// Gets the CSV file path.
    fn csv_path() -> PathBuf {
        // Look for workspace root (where Cargo.toml is)
        let mut path = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));

        // If we're in target/debug/examples, go up to workspace root
        if path.ends_with("examples") {
            path = path
                .parent()
                .and_then(|p| p.parent())
                .and_then(|p| p.parent())
                .unwrap_or(&path)
                .to_path_buf();
        }

        path.join("example_runs.csv")
    }

    /// Logs the example run to CSV.
    fn log_to_csv(&self) {
        let duration_ms = self.start_time.elapsed().as_millis();

        let status = self.status.lock().ok();
        let (status_str, error_msg) = match status.as_deref() {
            Some(TrackingStatus::Success) => ("success", String::new()),
            Some(TrackingStatus::Failed(msg)) => ("failed", msg.clone()),
            Some(TrackingStatus::Running) => {
                // If still running at drop, assume success (no panic occurred)
                ("success", String::new())
            }
            None => ("unknown", "Failed to acquire status lock".to_string()),
        };

        let timestamp = chrono::Utc::now().to_rfc3339();
        let methods = self.methods_tested.join(";");
        let service_type = self.service_type.as_deref().unwrap_or("");

        // Get git commit if available
        let git_commit = std::process::Command::new("git")
            .args(["rev-parse", "--short", "HEAD"])
            .output()
            .ok()
            .and_then(|o| String::from_utf8(o.stdout).ok())
            .map(|s| s.trim().to_string())
            .unwrap_or_default();

        // Escape CSV fields
        let error_escaped = error_msg.replace('"', "\"\"");

        let csv_line = format!(
            "{},{},{},{},{},{},\"{}\",{}\n",
            self.example_name,
            timestamp,
            status_str,
            duration_ms,
            methods,
            service_type,
            error_escaped,
            git_commit
        );

        // Append to CSV file
        let csv_path = Self::csv_path();

        match OpenOptions::new().create(true).append(true).open(&csv_path) {
            Ok(mut file) => {
                if let Err(e) = file.write_all(csv_line.as_bytes()) {
                    eprintln!("Failed to write to tracking CSV: {}", e);
                } else {
                    tracing::info!(
                        example = %self.example_name,
                        status = %status_str,
                        duration_ms = %duration_ms,
                        csv_path = %csv_path.display(),
                        "Example run logged"
                    );
                }
            }
            Err(e) => {
                eprintln!("Failed to open tracking CSV at {:?}: {}", csv_path, e);
            }
        }
    }
}

impl Drop for ExampleTracker {
    fn drop(&mut self) {
        // If status is still Running, mark as FAILED
        // Success must be explicitly called with .success() method
        if let Ok(mut status) = self.status.lock() {
            if matches!(*status, TrackingStatus::Running) {
                *status = TrackingStatus::Failed(
                    "Example exited without explicit success() - likely returned early via Result::Err".to_string()
                );
            }
        }

        // Log to CSV
        self.log_to_csv();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tracker_creation() {
        let tracker = ExampleTracker::new("test_example")
            .methods(&["method1", "method2"])
            .service_type("TestClient");

        assert_eq!(tracker.example_name, "test_example");
        assert_eq!(tracker.methods_tested, vec!["method1", "method2"]);
        assert_eq!(tracker.service_type, Some("TestClient".to_string()));
    }

    #[test]
    fn test_csv_path() {
        let path = ExampleTracker::csv_path();
        assert!(path.ends_with("example_runs.csv"));
    }
}
