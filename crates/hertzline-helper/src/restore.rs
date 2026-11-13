//! Crash / uninstall restore story (AC-7.2, AC-7.3).
//!
//! Snapshots live in user-writable storage owned by `core`. The helper only
//! performs privileged undo steps when elevation is required. This spike
//! returns a structured plan; real restore lands in HZ-018.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RestoreReport {
    pub trigger: String,
    pub requires_ui: bool,
    pub steps: Vec<String>,
    pub status: String,
}

/// Documented restore-all path for crash recovery and uninstall hooks.
pub fn restore_all(trigger: &str) -> RestoreReport {
    RestoreReport {
        trigger: trigger.into(),
        requires_ui: false,
        steps: vec![
            "Load snapshot store from the Hertzline user data directory (core).".into(),
            "Revert unprivileged changes in-process via core action restore.".into(),
            "If privileged undo remains, invoke helper op restore_all with elevation reason.".into(),
            "Write change-log entries with restore status for each action.".into(),
            "On uninstall: packaging runs `hertzline-helper restore_all --reason uninstall` before deleting binaries.".into(),
        ],
        status: "planned_stub".into(),
    }
}

/// Triggers that must work without the desktop UI.
pub fn headless_triggers() -> &'static [&'static str] {
    &["crash", "uninstall", "watchdog", "explicit"]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn headless_triggers_include_crash_and_uninstall() {
        let triggers = headless_triggers();
        assert!(triggers.contains(&"crash"));
        assert!(triggers.contains(&"uninstall"));
    }

    #[test]
    fn restore_does_not_require_ui() {
        let report = restore_all("uninstall");
        assert!(!report.requires_ui);
        assert_eq!(report.status, "planned_stub");
    }
}
