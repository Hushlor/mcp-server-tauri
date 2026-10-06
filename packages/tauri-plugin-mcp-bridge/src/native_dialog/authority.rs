//! Platform-neutral authorization boundary for native dialog automation.
//!
//! A native dialog is automatable only when it is owned, directly or through a
//! bounded chain of owner windows, by the targeted Tauri window, and when every
//! window in that chain belongs to an authorized process:
//!
//! - the Tauri host process itself, or
//! - the WebView2 browser process reported by the targeted window's own
//!   `ICoreWebView2::BrowserProcessId`.
//!
//! The WebView2 process is never identified by executable name or window title.
//! Its PID is re-read from the live WebView2 instance on every request, so the
//! authority is always derived from the targeted window rather than cached.

use std::collections::HashSet;
use std::time::Instant;

/// Maximum number of owner hops followed from a dialog to the targeted window.
pub(crate) const MAX_OWNER_CHAIN_DEPTH: usize = 8;

/// Read-only view of the native window tree used by the authorization checks.
///
/// Window handles are represented as `usize` so the rules can be tested
/// without a desktop session.
pub(crate) trait WindowTopology {
    /// Returns whether the handle still refers to a visible window.
    fn is_live_visible(&self, window: usize) -> bool;
    /// Returns the owner window (`GW_OWNER`), if any.
    fn owner(&self, window: usize) -> Option<usize>;
    /// Returns the PID of the process that created the window, if known.
    fn process_id(&self, window: usize) -> Option<u32>;
}

/// The identity a request is authorized to automate.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct DialogAuthority {
    pub host_process_id: u32,
    pub webview_process_id: Option<u32>,
    pub owner_window: usize,
}

/// A dialog window accepted by [`DialogAuthority::owned_dialog`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct OwnedDialog {
    pub window: usize,
    pub immediate_owner: usize,
    pub owner_depth: usize,
    pub process_id: u32,
}

impl DialogAuthority {
    /// Returns whether a window or UI Automation element from `process_id` may
    /// take part in a dialog owned by the targeted window.
    pub fn allows_process(&self, process_id: u32) -> bool {
        process_id != 0
            && (process_id == self.host_process_id || self.webview_process_id == Some(process_id))
    }

    /// Accepts `window` only when it is a live, visible window from an
    /// authorized process whose owner chain reaches the targeted window
    /// through authorized processes only.
    pub fn owned_dialog(
        &self,
        topology: &impl WindowTopology,
        window: usize,
    ) -> Option<OwnedDialog> {
        if window == 0 || window == self.owner_window || !topology.is_live_visible(window) {
            return None;
        }
        let process_id = topology.process_id(window)?;
        if !self.allows_process(process_id) {
            return None;
        }

        let mut current = window;
        let mut immediate_owner = None;
        let mut visited = HashSet::with_capacity(MAX_OWNER_CHAIN_DEPTH + 1);
        visited.insert(window);

        for depth in 1..=MAX_OWNER_CHAIN_DEPTH {
            let owner = topology.owner(current)?;
            if owner == 0 || !visited.insert(owner) {
                return None;
            }
            immediate_owner.get_or_insert(owner);
            if !self.allows_process(topology.process_id(owner)?) {
                return None;
            }
            if owner == self.owner_window {
                return Some(OwnedDialog {
                    window,
                    immediate_owner: immediate_owner?,
                    owner_depth: depth,
                    process_id,
                });
            }
            current = owner;
        }
        None
    }
}

/// Identity captured when an element reference is issued by a snapshot.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ElementIdentity {
    pub scope_id: String,
    pub host_process_id: u32,
    pub owner_window: usize,
    pub dialog_window: usize,
    pub dialog_process_id: u32,
    pub expires_at: Instant,
}

/// Why a cached element reference was rejected. Every reason is reported to
/// callers with the same stale-reference message.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ReferenceRejection {
    Expired,
    OtherSession,
    OtherHost,
    OtherWindow,
    DialogUnavailable,
    DialogProcessChanged,
    ElementProcessChanged,
}

/// Revalidates a cached element immediately before an interaction.
///
/// `element_process_id` is the live UI Automation `CurrentProcessId` of the
/// cached element, or `None` when the element can no longer be queried.
pub(crate) fn validate_element(
    identity: &ElementIdentity,
    scope_id: &str,
    authority: &DialogAuthority,
    topology: &impl WindowTopology,
    element_process_id: Option<u32>,
    now: Instant,
) -> Result<(), ReferenceRejection> {
    if identity.expires_at <= now {
        return Err(ReferenceRejection::Expired);
    }
    if identity.scope_id != scope_id {
        return Err(ReferenceRejection::OtherSession);
    }
    if identity.host_process_id != authority.host_process_id {
        return Err(ReferenceRejection::OtherHost);
    }
    if identity.owner_window != authority.owner_window {
        return Err(ReferenceRejection::OtherWindow);
    }
    let owned = authority
        .owned_dialog(topology, identity.dialog_window)
        .ok_or(ReferenceRejection::DialogUnavailable)?;
    if owned.process_id != identity.dialog_process_id {
        return Err(ReferenceRejection::DialogProcessChanged);
    }
    if element_process_id != Some(identity.dialog_process_id) {
        return Err(ReferenceRejection::ElementProcessChanged);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use std::time::Duration;

    const HOST: u32 = 100;
    const WEBVIEW: u32 = 200;
    const OTHER_APP: u32 = 300;
    /// Another `msedgewebview2.exe` instance (e.g. another app's WebView2).
    const UNRELATED_WEBVIEW: u32 = 400;

    const MAIN_WINDOW: usize = 1;
    const SECOND_WINDOW: usize = 2;
    const FOREIGN_WINDOW: usize = 3;

    #[derive(Default)]
    struct FakeTopology {
        windows: HashMap<usize, (u32, Option<usize>)>,
    }

    impl FakeTopology {
        fn with(mut self, window: usize, process_id: u32, owner: Option<usize>) -> Self {
            self.windows.insert(window, (process_id, owner));
            self
        }

        fn base() -> Self {
            Self::default()
                .with(MAIN_WINDOW, HOST, None)
                .with(SECOND_WINDOW, HOST, None)
                .with(FOREIGN_WINDOW, OTHER_APP, None)
        }
    }

    impl WindowTopology for FakeTopology {
        fn is_live_visible(&self, window: usize) -> bool {
            self.windows.contains_key(&window)
        }

        fn owner(&self, window: usize) -> Option<usize> {
            self.windows.get(&window).and_then(|(_, owner)| *owner)
        }

        fn process_id(&self, window: usize) -> Option<u32> {
            self.windows.get(&window).map(|(process_id, _)| *process_id)
        }
    }

    fn authority(owner_window: usize) -> DialogAuthority {
        DialogAuthority {
            host_process_id: HOST,
            webview_process_id: Some(WEBVIEW),
            owner_window,
        }
    }

    fn identity(dialog_window: usize, dialog_process_id: u32) -> ElementIdentity {
        ElementIdentity {
            scope_id: "session-a".to_string(),
            host_process_id: HOST,
            owner_window: MAIN_WINDOW,
            dialog_window,
            dialog_process_id,
            expires_at: Instant::now() + Duration::from_secs(30),
        }
    }

    #[test]
    fn accepts_dialog_from_the_host_process() {
        let topology = FakeTopology::base().with(10, HOST, Some(MAIN_WINDOW));
        let dialog = authority(MAIN_WINDOW).owned_dialog(&topology, 10).unwrap();
        assert_eq!(dialog.process_id, HOST);
        assert_eq!(dialog.owner_depth, 1);
        assert_eq!(dialog.immediate_owner, MAIN_WINDOW);
    }

    #[test]
    fn accepts_dialog_from_the_associated_webview2_process() {
        let topology = FakeTopology::base().with(10, WEBVIEW, Some(MAIN_WINDOW));
        let dialog = authority(MAIN_WINDOW).owned_dialog(&topology, 10).unwrap();
        assert_eq!(dialog.process_id, WEBVIEW);
        assert_eq!(dialog.owner_depth, 1);
    }

    #[test]
    fn accepts_nested_webview2_confirmation_owned_by_a_webview2_dialog() {
        let topology = FakeTopology::base()
            .with(10, WEBVIEW, Some(MAIN_WINDOW))
            .with(11, WEBVIEW, Some(10));
        let dialog = authority(MAIN_WINDOW).owned_dialog(&topology, 11).unwrap();
        assert_eq!(dialog.owner_depth, 2);
        assert_eq!(dialog.immediate_owner, 10);
    }

    #[test]
    fn rejects_webview2_dialog_when_no_webview2_process_is_associated() {
        let topology = FakeTopology::base().with(10, WEBVIEW, Some(MAIN_WINDOW));
        let host_only = DialogAuthority {
            webview_process_id: None,
            ..authority(MAIN_WINDOW)
        };
        assert!(host_only.owned_dialog(&topology, 10).is_none());
    }

    #[test]
    fn rejects_dialog_from_another_application_even_when_owned_by_the_target() {
        let topology = FakeTopology::base().with(10, OTHER_APP, Some(MAIN_WINDOW));
        assert!(authority(MAIN_WINDOW).owned_dialog(&topology, 10).is_none());
    }

    #[test]
    fn rejects_same_named_process_without_a_webview2_association() {
        // Process names are never consulted: an unrelated msedgewebview2.exe
        // has a different PID from the targeted window's BrowserProcessId.
        let topology = FakeTopology::base().with(10, UNRELATED_WEBVIEW, Some(MAIN_WINDOW));
        assert!(authority(MAIN_WINDOW).owned_dialog(&topology, 10).is_none());
    }

    #[test]
    fn rejects_dialog_owned_by_another_window_of_the_same_application() {
        let topology = FakeTopology::base()
            .with(10, WEBVIEW, Some(SECOND_WINDOW))
            .with(11, HOST, Some(SECOND_WINDOW));
        assert!(authority(MAIN_WINDOW).owned_dialog(&topology, 10).is_none());
        assert!(authority(MAIN_WINDOW).owned_dialog(&topology, 11).is_none());
        assert!(authority(SECOND_WINDOW)
            .owned_dialog(&topology, 10)
            .is_some());
    }

    #[test]
    fn rejects_owner_chains_through_unauthorized_processes() {
        let topology = FakeTopology::base()
            .with(10, OTHER_APP, Some(MAIN_WINDOW))
            .with(11, WEBVIEW, Some(10));
        assert!(authority(MAIN_WINDOW).owned_dialog(&topology, 11).is_none());
    }

    #[test]
    fn rejects_unowned_cyclic_and_too_deep_chains() {
        let unowned = FakeTopology::base().with(10, WEBVIEW, None);
        assert!(authority(MAIN_WINDOW).owned_dialog(&unowned, 10).is_none());

        let cyclic = FakeTopology::base()
            .with(10, WEBVIEW, Some(11))
            .with(11, WEBVIEW, Some(10));
        assert!(authority(MAIN_WINDOW).owned_dialog(&cyclic, 10).is_none());

        let mut deep = FakeTopology::base().with(10, WEBVIEW, Some(MAIN_WINDOW));
        for window in 11..(11 + MAX_OWNER_CHAIN_DEPTH) {
            deep = deep.with(window, WEBVIEW, Some(window - 1));
        }
        let deepest = 10 + MAX_OWNER_CHAIN_DEPTH;
        assert!(authority(MAIN_WINDOW)
            .owned_dialog(&deep, deepest)
            .is_none());
        assert!(authority(MAIN_WINDOW)
            .owned_dialog(&deep, deepest - 1)
            .is_some());
    }

    #[test]
    fn never_authorizes_pid_zero_or_the_target_window_itself() {
        let topology = FakeTopology::base().with(10, 0, Some(MAIN_WINDOW));
        let zero_webview = DialogAuthority {
            webview_process_id: Some(0),
            ..authority(MAIN_WINDOW)
        };
        assert!(!zero_webview.allows_process(0));
        assert!(zero_webview.owned_dialog(&topology, 10).is_none());
        assert!(authority(MAIN_WINDOW)
            .owned_dialog(&topology, MAIN_WINDOW)
            .is_none());
    }

    #[test]
    fn validates_live_references_from_both_authorized_processes() {
        let topology = FakeTopology::base().with(10, HOST, Some(MAIN_WINDOW)).with(
            20,
            WEBVIEW,
            Some(MAIN_WINDOW),
        );
        let now = Instant::now();
        for (window, process_id) in [(10, HOST), (20, WEBVIEW)] {
            assert_eq!(
                validate_element(
                    &identity(window, process_id),
                    "session-a",
                    &authority(MAIN_WINDOW),
                    &topology,
                    Some(process_id),
                    now,
                ),
                Ok(())
            );
        }
    }

    #[test]
    fn rejects_expired_references() {
        let topology = FakeTopology::base().with(20, WEBVIEW, Some(MAIN_WINDOW));
        let mut expired = identity(20, WEBVIEW);
        let now = Instant::now();
        expired.expires_at = now;
        assert_eq!(
            validate_element(
                &expired,
                "session-a",
                &authority(MAIN_WINDOW),
                &topology,
                Some(WEBVIEW),
                now,
            ),
            Err(ReferenceRejection::Expired)
        );
    }

    #[test]
    fn rejects_references_from_another_session_host_or_window() {
        let topology = FakeTopology::base().with(20, WEBVIEW, Some(MAIN_WINDOW));
        let reference = identity(20, WEBVIEW);
        let now = Instant::now();
        let check = |scope: &str, authority: DialogAuthority| {
            validate_element(&reference, scope, &authority, &topology, Some(WEBVIEW), now)
        };

        assert_eq!(
            check("session-b", authority(MAIN_WINDOW)),
            Err(ReferenceRejection::OtherSession)
        );
        assert_eq!(
            check(
                "session-a",
                DialogAuthority {
                    host_process_id: OTHER_APP,
                    ..authority(MAIN_WINDOW)
                }
            ),
            Err(ReferenceRejection::OtherHost)
        );
        assert_eq!(
            check("session-a", authority(SECOND_WINDOW)),
            Err(ReferenceRejection::OtherWindow)
        );
    }

    #[test]
    fn rejects_closed_dialogs_and_owner_changes() {
        let reference = identity(20, WEBVIEW);
        let now = Instant::now();

        let closed = FakeTopology::base();
        assert_eq!(
            validate_element(
                &reference,
                "session-a",
                &authority(MAIN_WINDOW),
                &closed,
                Some(WEBVIEW),
                now,
            ),
            Err(ReferenceRejection::DialogUnavailable)
        );

        let reowned = FakeTopology::base().with(20, WEBVIEW, Some(SECOND_WINDOW));
        assert_eq!(
            validate_element(
                &reference,
                "session-a",
                &authority(MAIN_WINDOW),
                &reowned,
                Some(WEBVIEW),
                now,
            ),
            Err(ReferenceRejection::DialogUnavailable)
        );
    }

    #[test]
    fn rejects_reused_handles_and_changed_element_processes() {
        let now = Instant::now();
        // The HWND was destroyed and reused by a host-process window: the
        // dialog identity no longer matches even though ownership is valid.
        let reused = FakeTopology::base().with(20, HOST, Some(MAIN_WINDOW));
        assert_eq!(
            validate_element(
                &identity(20, WEBVIEW),
                "session-a",
                &authority(MAIN_WINDOW),
                &reused,
                Some(HOST),
                now,
            ),
            Err(ReferenceRejection::DialogProcessChanged)
        );

        // The WebView2 browser process restarted with a new PID.
        let topology = FakeTopology::base().with(20, WEBVIEW, Some(MAIN_WINDOW));
        let restarted = DialogAuthority {
            webview_process_id: Some(WEBVIEW + 1),
            ..authority(MAIN_WINDOW)
        };
        assert_eq!(
            validate_element(
                &identity(20, WEBVIEW),
                "session-a",
                &restarted,
                &topology,
                Some(WEBVIEW),
                now,
            ),
            Err(ReferenceRejection::DialogUnavailable)
        );

        for element_process in [None, Some(HOST), Some(OTHER_APP)] {
            assert_eq!(
                validate_element(
                    &identity(20, WEBVIEW),
                    "session-a",
                    &authority(MAIN_WINDOW),
                    &topology,
                    element_process,
                    now,
                ),
                Err(ReferenceRejection::ElementProcessChanged)
            );
        }
    }
}
