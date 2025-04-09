//! API for IPC use
use serde::{Deserialize, Serialize};

/// The JSON object that gets serialized for IPC usage
///
/// Example:
///
/// ```json
/// {"request": "progress_info", "text": "Hello", "percent": 50}
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "request")]
#[serde(rename_all = "snake_case")]
pub enum Request {
    /// Information message
    Info { text: String },
    /// Informational message with progress bar update
    ProgressInfo {
        text: String,
        // 0 to 100
        percent: u8,
    },
    /// Informational message indicating successful boot up
    ReadyInfo { text: String },
    /// Warning message
    Warning { text: String },
    /// Error message
    Error { text: String },
    /// Output of a command
    CommandOutput { text: String }
}
