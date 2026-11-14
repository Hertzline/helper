//! Per-OS elevation plans (D2).
//!
//! Plans document the official API path. They do not prompt for elevation
//! unless an explicit future elevate executor is invoked interactively.

use serde::{Deserialize, Serialize};

#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "macos")]
mod macos;
#[cfg(target_os = "windows")]
mod windows;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ElevatePlan {
    pub os: String,
    pub mechanism: String,
    pub api: String,
    pub source_url: String,
    pub always_on_service: bool,
    pub notes: Vec<String>,
}

/// Build the elevate plan for the compile-time host OS.
pub fn plan_for_host() -> ElevatePlan {
    #[cfg(target_os = "windows")]
    {
        windows::plan()
    }
    #[cfg(target_os = "linux")]
    {
        linux::plan()
    }
    #[cfg(target_os = "macos")]
    {
        macos::plan()
    }
    #[cfg(not(any(target_os = "windows", target_os = "linux", target_os = "macos")))]
    {
        ElevatePlan {
            os: std::env::consts::OS.into(),
            mechanism: "unsupported".into(),
            api: "n/a".into(),
            source_url: "".into(),
            always_on_service: false,
            notes: vec!["Host OS is outside Hertzline v1 targets.".into()],
        }
    }
}

/// Resolve how a client would launch this helper elevated for `op`.
/// Dry-run only: never spawns an elevated process from unit tests or CI smoke.
pub fn resolve_launch(op: crate::protocol::HelperOp) -> LaunchSpec {
    let plan = plan_for_host();
    LaunchSpec {
        op: op.as_token().into(),
        requires_elevation: op.requires_elevation(),
        elevation_reason: op.elevation_reason().into(),
        plan,
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LaunchSpec {
    pub op: String,
    pub requires_elevation: bool,
    pub elevation_reason: String,
    pub plan: ElevatePlan,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::protocol::HelperOp;

    #[test]
    fn ping_does_not_require_elevation() {
        let spec = resolve_launch(HelperOp::Ping);
        assert!(!spec.requires_elevation);
    }

    #[test]
    fn restore_all_marks_elevation() {
        let spec = resolve_launch(HelperOp::RestoreAll);
        assert!(spec.requires_elevation);
        assert!(!spec.plan.mechanism.is_empty());
    }

    #[test]
    fn create_restore_point_marks_elevation() {
        let spec = resolve_launch(HelperOp::CreateRestorePoint);
        assert!(spec.requires_elevation);
        assert!(spec.elevation_reason.contains("System Restore"));
    }

    #[test]
    fn set_hags_marks_elevation() {
        let spec = resolve_launch(HelperOp::SetHags);
        assert!(spec.requires_elevation);
        assert!(spec.elevation_reason.contains("HwSchMode"));
        assert!(!resolve_launch(HelperOp::GetHags).requires_elevation);
        assert!(!resolve_launch(HelperOp::SetPowerPlan).requires_elevation);
        assert!(!resolve_launch(HelperOp::SetGameMode).requires_elevation);
    }
}
