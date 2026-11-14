//! Named helper operations and wire shapes.
//!
//! Allow-list is exhaustive. Adding an op requires a code change and review.

use serde::{Deserialize, Serialize};

/// Fixed set of privileged (or broker) operations for Hertzline.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HelperOp {
    /// Connectivity / version check. Never elevates.
    Ping,
    /// Report the OS elevate plan without prompting (D2 probe).
    ProbeElevatePath,
    /// Roll back every outstanding Hertzline change (AC-7.2 / AC-7.3).
    RestoreAll,
    /// Create a Windows System Restore point before system-level changes (AC-7.4).
    CreateRestorePoint,
    /// Read active Windows power plan GUID (AC-4.1). No elevation.
    GetPowerPlan,
    /// Set active Windows power plan GUID (AC-4.1). No elevation. Arg = GUID.
    SetPowerPlan,
    /// Read Windows Game Mode (AC-4.2). No elevation.
    GetGameMode,
    /// Set Windows Game Mode (AC-4.2). No elevation. Arg = "0"|"1".
    SetGameMode,
    /// Read HAGS HwSchMode (AC-4.2). No elevation.
    GetHags,
    /// Set HAGS HwSchMode (AC-4.2). Elevation required. Arg = "1"|"2".
    SetHags,
    /// Read process priority/affinity (AC-4.3). Arg = pid. No elevation.
    GetProcessPriority,
    /// Set process priority/affinity (AC-4.3). Arg = pid;priority[=hex][;affinity=hex].
    SetProcessPriority,
    /// Close a user-chosen background PID (AC-4.4). Arg = pid[;restart=0|1].
    CloseBackgroundApp,
    /// Pause a user-chosen background PID (AC-4.4). Arg = pid.
    PauseBackgroundApp,
    /// Resume a paused user-chosen background PID (AC-4.4). Arg = pid.
    ResumeBackgroundApp,
    /// List current-user startup Run entries (AC-4.5). No elevation.
    ListStartupApps,
    /// Enable/disable a user-chosen startup entry (AC-4.5). Arg = name;enabled=0|1.
    SetStartupApp,
    /// Read Xbox Game Bar capture toggle (AC-4.6). No elevation.
    GetGameBar,
    /// Set Xbox Game Bar capture toggle (AC-4.6). Arg = "0"|"1". No elevation.
    SetGameBar,
    /// Preview temp/shader cleanup sizes (AC-4.7). No elevation.
    PreviewCleanup,
    /// Apply opt-in cleanup after preview (AC-4.7). Arg must be "confirm=1". No elevation.
    ApplyCleanup,
    /// Storage free-space report (AC-4.8). No elevation. No destructive changes.
    GetStorageHealth,
    /// GPU driver versions + official vendor links (AC-4.9). No elevation.
    GetDriverInfo,
    /// Active display refresh + VRR honesty (AC-4.10). No elevation.
    GetDisplay,
    /// Set display refresh Hz (AC-4.10). Arg = Hz. No elevation.
    SetDisplayRefresh,
    /// Probe/query Feral GameMode status for a PID (AC-4.11). Arg = pid. No elevation.
    GetFeralGameMode,
    /// Register or unregister a PID with Feral GameMode (AC-4.11). Arg = pid;op=register|unregister.
    SetFeralGameMode,
    /// Read energy profile via ppd or tuned (AC-4.12). No elevation.
    GetEnergyProfile,
    /// Set energy profile (AC-4.12). Arg = profile name (ppd) or tuned profile. No elevation for ppd.
    SetEnergyProfile,
    /// Read nice + ionice for a PID (AC-4.13). Arg = pid. No elevation.
    GetNiceIonice,
    /// Set nice + ionice for a PID (AC-4.13). Arg = pid;nice=N;ioprio_class=C;ioprio_level=L. Elevation when nice < 0.
    SetNiceIonice,
    /// Read macOS Game Mode status (AC-4.19). Always not_possible via public APIs.
    GetMacOsGameMode,
    /// Read macOS Low Power Mode + optional perf warning (AC-4.21). Arg = performance=1.
    GetMacOsLpm,
    /// Close a user-chosen macOS background PID (AC-4.20). Arg = pid.
    MacOsCloseBackgroundApp,
    /// Pause a user-chosen macOS background PID (AC-4.20). Arg = pid.
    MacOsPauseBackgroundApp,
    /// Resume a paused macOS background PID (AC-4.20). Arg = pid.
    MacOsResumeBackgroundApp,
    /// Read macOS main display refresh (AC-4.22). No elevation.
    GetMacOsDisplay,
    /// Set macOS display refresh Hz (AC-4.22). Arg = Hz.
    SetMacOsDisplayRefresh,
}

impl HelperOp {
    pub const ALL: &'static [HelperOp] = &[
        HelperOp::Ping,
        HelperOp::ProbeElevatePath,
        HelperOp::RestoreAll,
        HelperOp::CreateRestorePoint,
        HelperOp::GetPowerPlan,
        HelperOp::SetPowerPlan,
        HelperOp::GetGameMode,
        HelperOp::SetGameMode,
        HelperOp::GetHags,
        HelperOp::SetHags,
        HelperOp::GetProcessPriority,
        HelperOp::SetProcessPriority,
        HelperOp::CloseBackgroundApp,
        HelperOp::PauseBackgroundApp,
        HelperOp::ResumeBackgroundApp,
        HelperOp::ListStartupApps,
        HelperOp::SetStartupApp,
        HelperOp::GetGameBar,
        HelperOp::SetGameBar,
        HelperOp::PreviewCleanup,
        HelperOp::ApplyCleanup,
        HelperOp::GetStorageHealth,
        HelperOp::GetDriverInfo,
        HelperOp::GetDisplay,
        HelperOp::SetDisplayRefresh,
        HelperOp::GetFeralGameMode,
        HelperOp::SetFeralGameMode,
        HelperOp::GetEnergyProfile,
        HelperOp::SetEnergyProfile,
        HelperOp::GetNiceIonice,
        HelperOp::SetNiceIonice,
        HelperOp::GetMacOsGameMode,
        HelperOp::GetMacOsLpm,
        HelperOp::MacOsCloseBackgroundApp,
        HelperOp::MacOsPauseBackgroundApp,
        HelperOp::MacOsResumeBackgroundApp,
        HelperOp::GetMacOsDisplay,
        HelperOp::SetMacOsDisplayRefresh,
    ];

    pub fn as_token(self) -> &'static str {
        match self {
            HelperOp::Ping => "ping",
            HelperOp::ProbeElevatePath => "probe_elevate_path",
            HelperOp::RestoreAll => "restore_all",
            HelperOp::CreateRestorePoint => "create_restore_point",
            HelperOp::GetPowerPlan => "get_power_plan",
            HelperOp::SetPowerPlan => "set_power_plan",
            HelperOp::GetGameMode => "get_game_mode",
            HelperOp::SetGameMode => "set_game_mode",
            HelperOp::GetHags => "get_hags",
            HelperOp::SetHags => "set_hags",
            HelperOp::GetProcessPriority => "get_process_priority",
            HelperOp::SetProcessPriority => "set_process_priority",
            HelperOp::CloseBackgroundApp => "close_background_app",
            HelperOp::PauseBackgroundApp => "pause_background_app",
            HelperOp::ResumeBackgroundApp => "resume_background_app",
            HelperOp::ListStartupApps => "list_startup_apps",
            HelperOp::SetStartupApp => "set_startup_app",
            HelperOp::GetGameBar => "get_game_bar",
            HelperOp::SetGameBar => "set_game_bar",
            HelperOp::PreviewCleanup => "preview_cleanup",
            HelperOp::ApplyCleanup => "apply_cleanup",
            HelperOp::GetStorageHealth => "get_storage_health",
            HelperOp::GetDriverInfo => "get_driver_info",
            HelperOp::GetDisplay => "get_display",
            HelperOp::SetDisplayRefresh => "set_display_refresh",
            HelperOp::GetFeralGameMode => "get_feral_gamemode",
            HelperOp::SetFeralGameMode => "set_feral_gamemode",
            HelperOp::GetEnergyProfile => "get_energy_profile",
            HelperOp::SetEnergyProfile => "set_energy_profile",
            HelperOp::GetNiceIonice => "get_nice_ionice",
            HelperOp::SetNiceIonice => "set_nice_ionice",
            HelperOp::GetMacOsGameMode => "get_macos_game_mode",
            HelperOp::GetMacOsLpm => "get_macos_lpm",
            HelperOp::MacOsCloseBackgroundApp => "macos_close_background_app",
            HelperOp::MacOsPauseBackgroundApp => "macos_pause_background_app",
            HelperOp::MacOsResumeBackgroundApp => "macos_resume_background_app",
            HelperOp::GetMacOsDisplay => "get_macos_display",
            HelperOp::SetMacOsDisplayRefresh => "set_macos_display_refresh",
        }
    }

    pub fn from_token(token: &str) -> Option<Self> {
        match token {
            "ping" => Some(HelperOp::Ping),
            "probe_elevate_path" => Some(HelperOp::ProbeElevatePath),
            "restore_all" => Some(HelperOp::RestoreAll),
            "create_restore_point" => Some(HelperOp::CreateRestorePoint),
            "get_power_plan" => Some(HelperOp::GetPowerPlan),
            "set_power_plan" => Some(HelperOp::SetPowerPlan),
            "get_game_mode" => Some(HelperOp::GetGameMode),
            "set_game_mode" => Some(HelperOp::SetGameMode),
            "get_hags" => Some(HelperOp::GetHags),
            "set_hags" => Some(HelperOp::SetHags),
            "get_process_priority" => Some(HelperOp::GetProcessPriority),
            "set_process_priority" => Some(HelperOp::SetProcessPriority),
            "close_background_app" => Some(HelperOp::CloseBackgroundApp),
            "pause_background_app" => Some(HelperOp::PauseBackgroundApp),
            "resume_background_app" => Some(HelperOp::ResumeBackgroundApp),
            "list_startup_apps" => Some(HelperOp::ListStartupApps),
            "set_startup_app" => Some(HelperOp::SetStartupApp),
            "get_game_bar" => Some(HelperOp::GetGameBar),
            "set_game_bar" => Some(HelperOp::SetGameBar),
            "preview_cleanup" => Some(HelperOp::PreviewCleanup),
            "apply_cleanup" => Some(HelperOp::ApplyCleanup),
            "get_storage_health" => Some(HelperOp::GetStorageHealth),
            "get_driver_info" => Some(HelperOp::GetDriverInfo),
            "get_display" => Some(HelperOp::GetDisplay),
            "set_display_refresh" => Some(HelperOp::SetDisplayRefresh),
            "get_feral_gamemode" => Some(HelperOp::GetFeralGameMode),
            "set_feral_gamemode" => Some(HelperOp::SetFeralGameMode),
            "get_energy_profile" => Some(HelperOp::GetEnergyProfile),
            "set_energy_profile" => Some(HelperOp::SetEnergyProfile),
            "get_nice_ionice" => Some(HelperOp::GetNiceIonice),
            "set_nice_ionice" => Some(HelperOp::SetNiceIonice),
            "get_macos_game_mode" => Some(HelperOp::GetMacOsGameMode),
            "get_macos_lpm" => Some(HelperOp::GetMacOsLpm),
            "macos_close_background_app" => Some(HelperOp::MacOsCloseBackgroundApp),
            "macos_pause_background_app" => Some(HelperOp::MacOsPauseBackgroundApp),
            "macos_resume_background_app" => Some(HelperOp::MacOsResumeBackgroundApp),
            "get_macos_display" => Some(HelperOp::GetMacOsDisplay),
            "set_macos_display_refresh" => Some(HelperOp::SetMacOsDisplayRefresh),
            _ => None,
        }
    }

    /// Human-readable reason shown in elevation prompts (AC-8.1).
    pub fn elevation_reason(self) -> &'static str {
        match self {
            HelperOp::Ping => "Hertzline helper connectivity check (no elevation)",
            HelperOp::ProbeElevatePath => "Hertzline helper elevation path probe (no elevation)",
            HelperOp::RestoreAll => "Hertzline restore all previous system changes",
            HelperOp::CreateRestorePoint => {
                "Hertzline create a Windows System Restore point before system changes"
            }
            HelperOp::GetPowerPlan => "Hertzline read active Windows power plan (no elevation)",
            HelperOp::SetPowerPlan => "Hertzline set active Windows power plan (no elevation)",
            HelperOp::GetGameMode => "Hertzline read Windows Game Mode (no elevation)",
            HelperOp::SetGameMode => "Hertzline set Windows Game Mode (no elevation)",
            HelperOp::GetHags => "Hertzline read HAGS status (no elevation)",
            HelperOp::SetHags => "Hertzline change Hardware-accelerated GPU scheduling (HwSchMode)",
            HelperOp::GetProcessPriority => {
                "Hertzline read process priority/affinity (no elevation)"
            }
            HelperOp::SetProcessPriority => {
                "Hertzline set process priority/affinity for a user-chosen PID (no elevation)"
            }
            HelperOp::CloseBackgroundApp => {
                "Hertzline close a user-chosen background app (no elevation)"
            }
            HelperOp::PauseBackgroundApp => {
                "Hertzline pause a user-chosen background app (no elevation)"
            }
            HelperOp::ResumeBackgroundApp => {
                "Hertzline resume a user-chosen background app (no elevation)"
            }
            HelperOp::ListStartupApps => "Hertzline list current-user startup apps (no elevation)",
            HelperOp::SetStartupApp => {
                "Hertzline enable/disable a user-chosen startup app (no elevation)"
            }
            HelperOp::GetGameBar => "Hertzline read Xbox Game Bar capture toggle (no elevation)",
            HelperOp::SetGameBar => "Hertzline set Xbox Game Bar capture toggle (no elevation)",
            HelperOp::PreviewCleanup => {
                "Hertzline preview temp/shader cleanup sizes (no elevation)"
            }
            HelperOp::ApplyCleanup => {
                "Hertzline apply opt-in temp/shader cleanup after preview (no elevation)"
            }
            HelperOp::GetStorageHealth => {
                "Hertzline report volume free space warnings (no elevation)"
            }
            HelperOp::GetDriverInfo => {
                "Hertzline report GPU driver versions and official vendor links (no elevation)"
            }
            HelperOp::GetDisplay => {
                "Hertzline report display refresh rate and VRR availability (no elevation)"
            }
            HelperOp::SetDisplayRefresh => "Hertzline set display refresh rate (no elevation)",
            HelperOp::GetFeralGameMode => {
                "Hertzline query Feral GameMode status for a PID (no elevation)"
            }
            HelperOp::SetFeralGameMode => {
                "Hertzline register/unregister a PID with Feral GameMode (no elevation)"
            }
            HelperOp::GetEnergyProfile => {
                "Hertzline read Linux energy profile via ppd or tuned (no elevation)"
            }
            HelperOp::SetEnergyProfile => {
                "Hertzline set Linux energy profile via ppd or tuned (no elevation)"
            }
            HelperOp::GetNiceIonice => {
                "Hertzline read nice/ionice for a user-chosen PID (no elevation)"
            }
            HelperOp::SetNiceIonice => {
                "Hertzline set nice/ionice for a user-chosen PID (may need CAP_SYS_NICE)"
            }
            HelperOp::GetMacOsGameMode => {
                "Hertzline report macOS Game Mode status (no public API; no elevation)"
            }
            HelperOp::GetMacOsLpm => {
                "Hertzline report macOS Low Power Mode and warn if performance work requested (no elevation)"
            }
            HelperOp::MacOsCloseBackgroundApp => {
                "Hertzline close a user-chosen macOS background app (no elevation)"
            }
            HelperOp::MacOsPauseBackgroundApp => {
                "Hertzline pause a user-chosen macOS background app (no elevation)"
            }
            HelperOp::MacOsResumeBackgroundApp => {
                "Hertzline resume a user-chosen macOS background app (no elevation)"
            }
            HelperOp::GetMacOsDisplay => {
                "Hertzline report macOS display refresh rate (no elevation)"
            }
            HelperOp::SetMacOsDisplayRefresh => {
                "Hertzline set macOS display refresh rate (no elevation)"
            }
        }
    }

    pub fn requires_elevation(self) -> bool {
        matches!(
            self,
            HelperOp::RestoreAll
                | HelperOp::CreateRestorePoint
                | HelperOp::SetHags
                | HelperOp::SetNiceIonice
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HelperRequest {
    pub op: HelperOp,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HelperResponse {
    pub accepted: bool,
    pub detail: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub payload: Option<serde_json::Value>,
}

impl HelperResponse {
    pub fn ok(detail: impl Into<String>) -> Self {
        Self {
            accepted: true,
            detail: detail.into(),
            payload: None,
        }
    }

    pub fn ok_json(detail: impl Into<String>, value: &impl Serialize) -> Self {
        Self {
            accepted: true,
            detail: detail.into(),
            payload: Some(serde_json::to_value(value).unwrap_or(serde_json::Value::Null)),
        }
    }

    pub fn rejected(detail: impl Into<String>) -> Self {
        Self {
            accepted: false,
            detail: detail.into(),
            payload: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HelperError {
    Rejected { detail: String },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tokens_round_trip() {
        for op in HelperOp::ALL {
            let token = op.as_token();
            assert_eq!(HelperOp::from_token(token), Some(*op));
        }
    }

    #[test]
    fn request_json_snake_case() {
        let req = HelperRequest {
            op: HelperOp::RestoreAll,
            reason: Some("uninstall".into()),
        };
        let encoded = serde_json::to_string(&req).expect("serialize");
        assert!(encoded.contains("restore_all"));
        let decoded: HelperRequest = serde_json::from_str(&encoded).expect("deserialize");
        assert_eq!(decoded, req);
    }

    #[test]
    fn hz020_ops_elevation_flags() {
        assert!(!HelperOp::GetPowerPlan.requires_elevation());
        assert!(!HelperOp::SetPowerPlan.requires_elevation());
        assert!(!HelperOp::GetGameMode.requires_elevation());
        assert!(!HelperOp::SetGameMode.requires_elevation());
        assert!(!HelperOp::GetHags.requires_elevation());
        assert!(HelperOp::SetHags.requires_elevation());
    }

    #[test]
    fn hz021_ops_no_elevation() {
        assert!(!HelperOp::GetProcessPriority.requires_elevation());
        assert!(!HelperOp::SetProcessPriority.requires_elevation());
        assert!(!HelperOp::CloseBackgroundApp.requires_elevation());
        assert!(!HelperOp::PauseBackgroundApp.requires_elevation());
        assert!(!HelperOp::ResumeBackgroundApp.requires_elevation());
        assert!(!HelperOp::ListStartupApps.requires_elevation());
        assert!(!HelperOp::SetStartupApp.requires_elevation());
    }

    #[test]
    fn hz022_ops_no_elevation() {
        assert!(!HelperOp::GetGameBar.requires_elevation());
        assert!(!HelperOp::SetGameBar.requires_elevation());
        assert!(!HelperOp::PreviewCleanup.requires_elevation());
        assert!(!HelperOp::ApplyCleanup.requires_elevation());
        assert!(!HelperOp::GetStorageHealth.requires_elevation());
        assert!(!HelperOp::GetDriverInfo.requires_elevation());
        assert!(!HelperOp::GetDisplay.requires_elevation());
        assert!(!HelperOp::SetDisplayRefresh.requires_elevation());
    }

    #[test]
    fn hz023_ops_elevation_flags() {
        assert!(!HelperOp::GetFeralGameMode.requires_elevation());
        assert!(!HelperOp::SetFeralGameMode.requires_elevation());
        assert!(!HelperOp::GetEnergyProfile.requires_elevation());
        assert!(!HelperOp::SetEnergyProfile.requires_elevation());
        assert!(!HelperOp::GetNiceIonice.requires_elevation());
        assert!(HelperOp::SetNiceIonice.requires_elevation());
    }

    #[test]
    fn hz026_ops_no_elevation() {
        assert!(!HelperOp::GetMacOsGameMode.requires_elevation());
        assert!(!HelperOp::GetMacOsLpm.requires_elevation());
        assert!(!HelperOp::MacOsCloseBackgroundApp.requires_elevation());
        assert!(!HelperOp::MacOsPauseBackgroundApp.requires_elevation());
        assert!(!HelperOp::MacOsResumeBackgroundApp.requires_elevation());
        assert!(!HelperOp::GetMacOsDisplay.requires_elevation());
        assert!(!HelperOp::SetMacOsDisplayRefresh.requires_elevation());
    }
}
