//! Native Windows dialog discovery and interaction.
//!
//! UI Automation objects are apartment-bound. The platform implementation keeps
//! every COM object on a dedicated MTA thread and exposes only serializable data
//! and opaque element references to the WebSocket layer.

use serde::Serialize;
use std::time::Duration;

// The authorization rules are platform-neutral so they are unit tested on every
// platform; only the Windows backend consumes them at runtime.
#[cfg_attr(not(target_os = "windows"), allow(dead_code))]
mod authority;
#[cfg(not(target_os = "windows"))]
mod unsupported;
#[cfg(target_os = "windows")]
mod windows;

#[cfg(not(target_os = "windows"))]
pub use unsupported::NativeDialogAutomation;
#[cfg(target_os = "windows")]
pub(crate) use windows::process_parent_ids;
#[cfg(target_os = "windows")]
pub use windows::NativeDialogAutomation;

#[cfg_attr(not(target_os = "windows"), allow(unused_imports))]
pub(crate) use authority::{verified_utility_children, MAX_WEBVIEW_PROCESS_INFOS};

/// Maximum time a caller may ask the UI Automation worker to wait.
pub const MAX_TIMEOUT: Duration = Duration::from_secs(10);

/// A request to discover dialogs in the ownership chain of a Tauri window.
///
/// `process_id` is the Tauri host process. `webview_process_id` is the WebView2
/// browser process and `webview_utility_process_ids` are the utility processes
/// reported by the targeted window's own WebView2 instance and environment.
/// Dialogs created by those processes (for example by `<input type="file">`)
/// are accepted when their owner chain reaches `owner_window`.
#[derive(Debug, Clone)]
pub struct SnapshotRequest {
    pub process_id: u32,
    pub webview_process_id: Option<u32>,
    pub webview_utility_process_ids: Vec<u32>,
    pub owner_window: usize,
    pub scope_id: String,
    pub min_owner_depth: usize,
    pub timeout: Duration,
}

/// A semantic action supported by a UI Automation control pattern.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum NativeDialogAction {
    Invoke,
    SetValue,
    SetPaths,
    Select,
}

impl NativeDialogAction {
    /// Parses the public WebSocket action name without including command data in errors.
    pub fn parse(value: &str) -> Result<Self, String> {
        match value {
            "invoke" => Ok(Self::Invoke),
            "setValue" => Ok(Self::SetValue),
            "setPaths" => Ok(Self::SetPaths),
            "select" => Ok(Self::Select),
            _ => Err("Unsupported native dialog action".to_string()),
        }
    }
}

/// A request to interact with an element from the latest snapshot in a session.
#[derive(Debug, Clone)]
pub struct InteractRequest {
    pub process_id: u32,
    pub webview_process_id: Option<u32>,
    pub webview_utility_process_ids: Vec<u32>,
    pub owner_window: usize,
    pub scope_id: String,
    pub element_ref: String,
    pub action: NativeDialogAction,
    pub value: Option<String>,
    pub paths: Option<Vec<String>>,
    pub timeout: Duration,
}

/// Semantic metadata for a native dialog control.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NativeDialogControl {
    pub element_ref: Option<String>,
    pub control_type: String,
    pub name: String,
    pub automation_id: String,
    pub semantic_role: Option<String>,
    pub enabled: bool,
    pub offscreen: bool,
    pub depth: usize,
    pub supported_actions: Vec<NativeDialogAction>,
}

/// A bounded semantic snapshot of one native dialog.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NativeDialog {
    pub dialog_ref: String,
    pub parent_dialog_ref: Option<String>,
    pub owner_depth: usize,
    pub kind: String,
    pub title: String,
    pub automation_id: String,
    pub controls: Vec<NativeDialogControl>,
    pub truncated: bool,
}

/// Snapshot returned to the MCP server.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NativeDialogSnapshot {
    pub platform: &'static str,
    pub interactive_desktop_required: bool,
    pub dialogs: Vec<NativeDialog>,
    pub dialog_count: usize,
}

/// Result of applying a UI Automation control pattern.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NativeDialogInteractionResult {
    pub action: NativeDialogAction,
    pub element_ref: String,
    pub references_invalidated: bool,
}

pub(crate) fn bounded_timeout(timeout: Duration) -> Duration {
    timeout.min(MAX_TIMEOUT)
}

/// Longest single UI Automation attempt within one snapshot request. Between
/// attempts the WebView2 process identities are read again, because the
/// utility process that hosts an `<input type="file">` picker can start after
/// the first read.
pub(crate) const SNAPSHOT_ATTEMPT_SLICE: Duration = Duration::from_secs(1);

/// Minimum remaining budget worth spending on another snapshot attempt.
pub(crate) const MIN_SNAPSHOT_RETRY_BUDGET: Duration = Duration::from_millis(100);

const NO_DIALOG_ERROR_PREFIX: &str =
    "No native dialog owned by the targeted Tauri window appeared within";

/// Error returned when bounded UI Automation traversal runs out of time.
pub(crate) const TRAVERSAL_TIMEOUT_ERROR: &str =
    "Native dialog snapshot timed out during bounded traversal";

/// Error returned when no authorized dialog appears within `timeout`.
pub(crate) fn no_dialog_error(timeout: Duration) -> String {
    format!("{NO_DIALOG_ERROR_PREFIX} {} ms", timeout.as_millis())
}

/// Returns whether a failed snapshot attempt may succeed after re-reading the
/// WebView2 process identities.
pub(crate) fn is_retryable_snapshot_error(error: &str) -> bool {
    error.starts_with(NO_DIALOG_ERROR_PREFIX) || error == TRAVERSAL_TIMEOUT_ERROR
}

/// Returns the timeout for the next snapshot attempt, or `None` when the
/// request deadline leaves no useful budget. The first attempt always runs;
/// later attempts need at least [`MIN_SNAPSHOT_RETRY_BUDGET`]. No attempt ever
/// extends the caller's original deadline.
pub(crate) fn snapshot_attempt_budget(
    remaining: Duration,
    first_attempt: bool,
) -> Option<Duration> {
    if !first_attempt && remaining < MIN_SNAPSHOT_RETRY_BUDGET {
        return None;
    }
    Some(remaining.min(SNAPSHOT_ATTEMPT_SLICE))
}

/// Converts `/` separators to `\` for Windows file-name controls, which
/// reject forward slashes in multi-selection lists.
#[cfg_attr(not(target_os = "windows"), allow(dead_code))]
pub(crate) fn to_windows_separators(path: &str) -> String {
    path.replace('/', "\\")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn snapshot_attempts_stay_within_the_original_deadline() {
        assert_eq!(
            snapshot_attempt_budget(Duration::from_millis(2_500), true),
            Some(SNAPSHOT_ATTEMPT_SLICE)
        );
        assert_eq!(
            snapshot_attempt_budget(Duration::from_millis(400), false),
            Some(Duration::from_millis(400))
        );
        assert_eq!(
            snapshot_attempt_budget(Duration::from_millis(99), false),
            None
        );
        assert_eq!(
            snapshot_attempt_budget(Duration::ZERO, true),
            Some(Duration::ZERO)
        );
    }

    #[test]
    fn only_discovery_timeouts_are_retried() {
        assert!(is_retryable_snapshot_error(&no_dialog_error(
            Duration::from_millis(1_000)
        )));
        assert!(is_retryable_snapshot_error(TRAVERSAL_TIMEOUT_ERROR));
        assert!(!is_retryable_snapshot_error(
            "Native dialog automation is only supported on Windows"
        ));
        assert!(!is_retryable_snapshot_error(
            "UI Automation could not access the native dialog"
        ));
    }

    #[test]
    fn windows_paths_use_backslash_separators() {
        assert_eq!(
            to_windows_separators("C:/Users/test/My File.csv"),
            "C:\\Users\\test\\My File.csv"
        );
        assert_eq!(
            to_windows_separators("C:\\already\\fine.txt"),
            "C:\\already\\fine.txt"
        );
        assert_eq!(
            to_windows_separators("\\\\server/share/a.txt"),
            "\\\\server\\share\\a.txt"
        );
    }
}
