//! Crash / uninstall restore (AC-7.2, AC-7.3 / HZ-018).
//!
//! Loads the Hertzline snapshot store and runs headless restore-all via core.
//! Action handlers for real OS tweaks register here as they ship (HZ-020+).
//! Until then an empty registry still aborts pending snapshots and reports
//! outstanding rows that need handlers -- never silently marks them restored.

use hertzline_core::paths::{snapshot_store_path, SNAPSHOT_STORE_ENV};
use hertzline_core::restore::{
    restore_all as core_restore_all, restore_all_at_default_path, MapActionRegistry,
    RestoreAllReport, RestoreTrigger,
};
use hertzline_core::snapshot::SnapshotStore;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RestoreReport {
    pub trigger: String,
    pub requires_ui: bool,
    pub steps: Vec<String>,
    pub status: String,
    pub store_path: String,
    pub item_count: usize,
    pub ok: bool,
}

/// Headless restore-all for crash recovery and uninstall hooks.
pub fn restore_all_for_trigger(trigger: &str) -> RestoreReport {
    let parsed = RestoreTrigger::parse(trigger).unwrap_or(RestoreTrigger::Explicit);
    let path = snapshot_store_path();
    let mut registry = MapActionRegistry::new();
    // Real OS action handlers register here when implemented (HZ-020+).
    match restore_all_at_default_path(&mut registry, parsed) {
        Ok(report) => to_report(&path, report),
        Err(err) => RestoreReport {
            trigger: parsed.as_str().into(),
            requires_ui: false,
            steps: vec![format!("restore failed: {err}")],
            status: "error".into(),
            store_path: path.display().to_string(),
            item_count: 0,
            ok: false,
        },
    }
}

/// Restore using an explicit store path (tests / packaging).
pub fn restore_all_at(store_path: &Path, trigger: &str) -> RestoreReport {
    let parsed = RestoreTrigger::parse(trigger).unwrap_or(RestoreTrigger::Explicit);
    let mut registry = MapActionRegistry::new();
    if !store_path.exists() {
        return RestoreReport {
            trigger: parsed.as_str().into(),
            requires_ui: false,
            steps: vec!["no snapshot store; nothing to restore".into()],
            status: "ok".into(),
            store_path: store_path.display().to_string(),
            item_count: 0,
            ok: true,
        };
    }
    match SnapshotStore::load_from_path(store_path) {
        Ok(mut store) => {
            let report = core_restore_all(&mut store, &mut registry, parsed);
            to_report(store_path, report)
        }
        Err(err) => RestoreReport {
            trigger: parsed.as_str().into(),
            requires_ui: false,
            steps: vec![format!("failed to load store: {err}")],
            status: "error".into(),
            store_path: store_path.display().to_string(),
            item_count: 0,
            ok: false,
        },
    }
}

/// Documented restore-all path used by CLI allow-list op `restore_all`.
pub fn restore_all(trigger: &str) -> RestoreReport {
    restore_all_for_trigger(trigger)
}

/// Triggers that must work without the desktop UI.
pub fn headless_triggers() -> &'static [&'static str] {
    &["crash", "uninstall", "watchdog", "explicit"]
}

pub fn env_store_override() -> &'static str {
    SNAPSHOT_STORE_ENV
}

fn to_report(path: &Path, report: RestoreAllReport) -> RestoreReport {
    let mut steps: Vec<String> = report
        .items
        .iter()
        .map(|item| {
            let id = item
                .entry_id
                .map(|n| n.to_string())
                .unwrap_or_else(|| "-".into());
            format!(
                "{} entry={} action={} ({})",
                item.status.as_str(),
                id,
                item.action,
                item.detail
            )
        })
        .collect();
    if steps.is_empty() {
        steps.push("no outstanding Hertzline changes".into());
    }
    RestoreReport {
        trigger: report.trigger.as_str().into(),
        requires_ui: report.requires_ui,
        steps,
        status: report.status_label().into(),
        store_path: path.display().to_string(),
        item_count: report.items.len(),
        ok: report.ok,
    }
}

/// Resolve store path for callers that need the default location.
pub fn default_store_path() -> PathBuf {
    snapshot_store_path()
}

#[cfg(test)]
mod tests {
    use super::*;
    use hertzline_core::action::{apply_with_snapshot, MockToggleAction};
    use hertzline_core::restore::{restore_all as core_restore_all, MapActionRegistry};
    use std::sync::atomic::{AtomicU64, Ordering};

    fn scratch_path(label: &str) -> PathBuf {
        static N: AtomicU64 = AtomicU64::new(0);
        let n = N.fetch_add(1, Ordering::Relaxed);
        let dir = std::env::temp_dir().join(format!(
            "hertzline-helper-restore-{}-{label}-{n}",
            std::process::id()
        ));
        std::fs::create_dir_all(&dir).expect("temp dir");
        dir.join("snapshots.json")
    }

    struct TempFile(PathBuf);

    impl Drop for TempFile {
        fn drop(&mut self) {
            if let Some(parent) = self.0.parent() {
                let _ = std::fs::remove_dir_all(parent);
            }
        }
    }

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
        assert!(report.status == "ok" || report.status == "partial" || report.status == "error");
    }

    #[test]
    fn missing_store_is_ok() {
        let path = scratch_path("missing");
        let _keep = TempFile(path.clone());
        let _ = std::fs::remove_file(&path);
        let report = restore_all_at(&path, "crash");
        assert!(report.ok);
        assert_eq!(report.status, "ok");
        assert!(!report.requires_ui);
    }

    #[test]
    fn outstanding_without_handlers_is_partial_not_silent() {
        // mock: durable outstanding row, empty helper registry
        let path = scratch_path("outstanding");
        let _keep = TempFile(path.clone());
        {
            let mut store = SnapshotStore::open(&path).unwrap();
            let mut action = MockToggleAction::new("mock_toggle", "off", "on");
            apply_with_snapshot(&mut action, &mut store, 1).unwrap();
        }
        let report = restore_all_at(&path, "uninstall");
        assert!(!report.ok);
        assert_eq!(report.status, "partial");
        assert!(report.steps.iter().any(|s| s.contains("missing_action")));
        let loaded = SnapshotStore::load_from_path(&path).unwrap();
        assert_eq!(
            loaded.change_log()[0].restore_status,
            hertzline_core::snapshot::RestoreStatus::Outstanding
        );
    }

    #[test]
    fn helper_path_matches_core_with_registered_mock() {
        // mock crash-rollback through core registry (what app/helper share)
        let path = scratch_path("mock-ok");
        let _keep = TempFile(path.clone());
        {
            let mut store = SnapshotStore::open(&path).unwrap();
            let mut action = MockToggleAction::new("mock_toggle", "a", "b");
            apply_with_snapshot(&mut action, &mut store, 5).unwrap();
        }
        let mut registry = MapActionRegistry::new();
        registry.insert(Box::new(MockToggleAction::new("mock_toggle", "b", "b")));
        let mut store = SnapshotStore::load_from_path(&path).unwrap();
        let report = core_restore_all(&mut store, &mut registry, RestoreTrigger::Crash);
        assert!(report.ok);
        assert_eq!(
            registry.get("mock_toggle").unwrap().read_state().unwrap(),
            "a"
        );
    }
}
