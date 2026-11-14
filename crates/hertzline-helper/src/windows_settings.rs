//! Windows settings helper ops (HZ-020 / HZ-021 / HZ-022).
//!
//! Power plan, Game Mode, priority, background apps, startup, Game Bar,
//! cleanup, storage, drivers, and display are unprivileged (current user /
//! same-session). HAGS write needs elevation (HKLM HwSchMode).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SettingReport {
    pub os: String,
    pub setting: String,
    pub status: String,
    pub value: String,
    pub detail: String,
    pub source_url: String,
    pub requires_elevation: bool,
}

#[cfg(windows)]
mod win {
    use super::*;
    use hertzline_core::actions::windows::game_mode::{
        GameModeBackend, ACTION_ID as GAME_MODE_ID, SOURCE_URL as GAME_MODE_SRC,
    };
    use hertzline_core::actions::windows::hags::{
        HagsBackend, ACTION_ID as HAGS_ID, HAGS_OFF, HAGS_ON, SOURCE_URL as HAGS_SRC,
    };
    use hertzline_core::actions::windows::power_plan::{
        PowerPlanBackend, ACTION_ID as POWER_ID, GUID_HIGH_PERFORMANCE,
    };
    use hertzline_core::actions::windows::{
        game_mode::os::OsGameModeBackend, hags::os::OsHagsBackend,
        power_plan::os::OsPowerPlanBackend,
    };

    const POWER_SRC: &str =
        "https://learn.microsoft.com/en-us/windows/win32/api/powersetting/nf-powersetting-powergetactivescheme";

    pub fn get_power_plan() -> SettingReport {
        match OsPowerPlanBackend.get_active() {
            Ok(value) => SettingReport {
                os: "windows".into(),
                setting: POWER_ID.into(),
                status: "ok".into(),
                value,
                detail: "PowerGetActiveScheme".into(),
                source_url: POWER_SRC.into(),
                requires_elevation: false,
            },
            Err(err) => SettingReport {
                os: "windows".into(),
                setting: POWER_ID.into(),
                status: "failed".into(),
                value: String::new(),
                detail: err.to_string(),
                source_url: POWER_SRC.into(),
                requires_elevation: false,
            },
        }
    }

    pub fn set_power_plan(guid: &str) -> SettingReport {
        let target = if guid.trim().is_empty() {
            GUID_HIGH_PERFORMANCE
        } else {
            guid.trim()
        };
        match OsPowerPlanBackend.set_active(target) {
            Ok(()) => match OsPowerPlanBackend.get_active() {
                Ok(value) => SettingReport {
                    os: "windows".into(),
                    setting: POWER_ID.into(),
                    status: "ok".into(),
                    value,
                    detail: format!("PowerSetActiveScheme {target}"),
                    source_url: POWER_SRC.into(),
                    requires_elevation: false,
                },
                Err(err) => SettingReport {
                    os: "windows".into(),
                    setting: POWER_ID.into(),
                    status: "partial".into(),
                    value: String::new(),
                    detail: format!("set ok but verify read failed: {err}"),
                    source_url: POWER_SRC.into(),
                    requires_elevation: false,
                },
            },
            Err(err) => SettingReport {
                os: "windows".into(),
                setting: POWER_ID.into(),
                status: "failed".into(),
                value: String::new(),
                detail: err.to_string(),
                source_url: POWER_SRC.into(),
                requires_elevation: false,
            },
        }
    }

    pub fn get_game_mode() -> SettingReport {
        match OsGameModeBackend.get() {
            Ok(value) => SettingReport {
                os: "windows".into(),
                setting: GAME_MODE_ID.into(),
                status: "ok".into(),
                value,
                detail: "AutoGameModeEnabled".into(),
                source_url: GAME_MODE_SRC.into(),
                requires_elevation: false,
            },
            Err(err) => SettingReport {
                os: "windows".into(),
                setting: GAME_MODE_ID.into(),
                status: "failed".into(),
                value: String::new(),
                detail: err.to_string(),
                source_url: GAME_MODE_SRC.into(),
                requires_elevation: false,
            },
        }
    }

    pub fn set_game_mode(enabled: bool) -> SettingReport {
        match OsGameModeBackend.set(enabled) {
            Ok(()) => match OsGameModeBackend.get() {
                Ok(value) => SettingReport {
                    os: "windows".into(),
                    setting: GAME_MODE_ID.into(),
                    status: "ok".into(),
                    value,
                    detail: format!("AutoGameModeEnabled={}", if enabled { 1 } else { 0 }),
                    source_url: GAME_MODE_SRC.into(),
                    requires_elevation: false,
                },
                Err(err) => SettingReport {
                    os: "windows".into(),
                    setting: GAME_MODE_ID.into(),
                    status: "partial".into(),
                    value: String::new(),
                    detail: format!("set ok but verify read failed: {err}"),
                    source_url: GAME_MODE_SRC.into(),
                    requires_elevation: false,
                },
            },
            Err(err) => SettingReport {
                os: "windows".into(),
                setting: GAME_MODE_ID.into(),
                status: "failed".into(),
                value: String::new(),
                detail: err.to_string(),
                source_url: GAME_MODE_SRC.into(),
                requires_elevation: false,
            },
        }
    }

    pub fn get_hags() -> SettingReport {
        match OsHagsBackend.get() {
            Ok(value) => SettingReport {
                os: "windows".into(),
                setting: HAGS_ID.into(),
                status: if value == "unsupported" {
                    "not_possible".into()
                } else {
                    "ok".into()
                },
                value,
                detail: "HwSchMode (reboot required after set)".into(),
                source_url: HAGS_SRC.into(),
                requires_elevation: false,
            },
            Err(err) => SettingReport {
                os: "windows".into(),
                setting: HAGS_ID.into(),
                status: "failed".into(),
                value: String::new(),
                detail: err.to_string(),
                source_url: HAGS_SRC.into(),
                requires_elevation: false,
            },
        }
    }

    pub fn set_hags(mode: &str) -> SettingReport {
        let mode = mode.trim();
        if mode != HAGS_OFF && mode != HAGS_ON {
            return SettingReport {
                os: "windows".into(),
                setting: HAGS_ID.into(),
                status: "failed".into(),
                value: String::new(),
                detail: format!("invalid mode {mode}; expected {HAGS_OFF} or {HAGS_ON}"),
                source_url: HAGS_SRC.into(),
                requires_elevation: true,
            };
        }
        match OsHagsBackend.set(mode) {
            Ok(()) => match OsHagsBackend.get() {
                Ok(value) => SettingReport {
                    os: "windows".into(),
                    setting: HAGS_ID.into(),
                    status: "ok".into(),
                    value,
                    detail: format!("HwSchMode set to {mode}; reboot required for effect"),
                    source_url: HAGS_SRC.into(),
                    requires_elevation: true,
                },
                Err(err) => SettingReport {
                    os: "windows".into(),
                    setting: HAGS_ID.into(),
                    status: "partial".into(),
                    value: String::new(),
                    detail: format!("set ok but verify read failed: {err}"),
                    source_url: HAGS_SRC.into(),
                    requires_elevation: true,
                },
            },
            Err(err) => {
                let status = if err.to_string().contains("not possible")
                    || err.to_string().contains("not exposed")
                {
                    "not_possible"
                } else {
                    "failed"
                };
                SettingReport {
                    os: "windows".into(),
                    setting: HAGS_ID.into(),
                    status: status.into(),
                    value: String::new(),
                    detail: err.to_string(),
                    source_url: HAGS_SRC.into(),
                    requires_elevation: true,
                }
            }
        }
    }

    use hertzline_core::actions::windows::background::{
        BackgroundAppsBackend, ACTION_ID as BG_ID, SOURCE_URL as BG_SRC,
    };
    use hertzline_core::actions::windows::priority::{
        ProcessPriorityBackend, ACTION_ID as PRI_ID, HIGH_PRIORITY_CLASS, SOURCE_URL as PRI_SRC,
    };
    use hertzline_core::actions::windows::startup::{
        StartupAppsBackend, StartupEntry, ACTION_ID as START_ID, SOURCE_URL as START_SRC,
    };
    use hertzline_core::actions::windows::{
        background::os::OsBackgroundAppsBackend, priority::os::OsProcessPriorityBackend,
        startup::os::OsStartupAppsBackend,
    };

    fn parse_pid(reason: &str) -> Result<u32, String> {
        let first = reason.split(';').next().unwrap_or("").trim();
        let token = first.strip_prefix("pid=").unwrap_or(first).trim();
        token
            .parse::<u32>()
            .map_err(|_| format!("expected pid, got {reason}"))
    }

    pub fn get_process_priority(reason: &str) -> SettingReport {
        let pid = match parse_pid(reason) {
            Ok(pid) => pid,
            Err(detail) => {
                return SettingReport {
                    os: "windows".into(),
                    setting: PRI_ID.into(),
                    status: "failed".into(),
                    value: String::new(),
                    detail,
                    source_url: PRI_SRC.into(),
                    requires_elevation: false,
                };
            }
        };
        match OsProcessPriorityBackend.read(pid) {
            Ok(state) => SettingReport {
                os: "windows".into(),
                setting: PRI_ID.into(),
                status: "ok".into(),
                value: state.encode(),
                detail: "GetPriorityClass/GetProcessAffinityMask".into(),
                source_url: PRI_SRC.into(),
                requires_elevation: false,
            },
            Err(err) => SettingReport {
                os: "windows".into(),
                setting: PRI_ID.into(),
                status: "failed".into(),
                value: String::new(),
                detail: err.to_string(),
                source_url: PRI_SRC.into(),
                requires_elevation: false,
            },
        }
    }

    pub fn set_process_priority(reason: &str) -> SettingReport {
        let mut pid = None;
        let mut priority = HIGH_PRIORITY_CLASS;
        let mut affinity: Option<u64> = None;
        for part in reason.split(';') {
            let part = part.trim();
            if part.is_empty() {
                continue;
            }
            if let Some(v) = part.strip_prefix("pid=") {
                pid = v.parse::<u32>().ok();
            } else if let Some(v) = part.strip_prefix("priority=") {
                if let Some(hex) = v.strip_prefix("0x").or_else(|| v.strip_prefix("0X")) {
                    priority = u32::from_str_radix(hex, 16).unwrap_or(priority);
                } else if let Ok(n) = v.parse::<u32>() {
                    priority = n;
                }
            } else if let Some(v) = part.strip_prefix("affinity=") {
                if let Some(hex) = v.strip_prefix("0x").or_else(|| v.strip_prefix("0X")) {
                    affinity = u64::from_str_radix(hex, 16).ok();
                } else {
                    affinity = v.parse::<u64>().ok();
                }
            } else if pid.is_none() {
                pid = part.parse::<u32>().ok();
            }
        }
        let Some(pid) = pid else {
            return SettingReport {
                os: "windows".into(),
                setting: PRI_ID.into(),
                status: "failed".into(),
                value: String::new(),
                detail: format!("expected pid[;priority=..][;affinity=..], got {reason}"),
                source_url: PRI_SRC.into(),
                requires_elevation: false,
            };
        };
        if let Err(err) = OsProcessPriorityBackend.set_priority(pid, priority) {
            return SettingReport {
                os: "windows".into(),
                setting: PRI_ID.into(),
                status: "failed".into(),
                value: String::new(),
                detail: err.to_string(),
                source_url: PRI_SRC.into(),
                requires_elevation: false,
            };
        }
        if let Some(mask) = affinity {
            if let Err(err) = OsProcessPriorityBackend.set_affinity(pid, mask) {
                return SettingReport {
                    os: "windows".into(),
                    setting: PRI_ID.into(),
                    status: "failed".into(),
                    value: String::new(),
                    detail: err.to_string(),
                    source_url: PRI_SRC.into(),
                    requires_elevation: false,
                };
            }
        }
        match OsProcessPriorityBackend.read(pid) {
            Ok(state) => SettingReport {
                os: "windows".into(),
                setting: PRI_ID.into(),
                status: "ok".into(),
                value: state.encode(),
                detail: format!("SetPriorityClass pid={pid} priority=0x{priority:x}"),
                source_url: PRI_SRC.into(),
                requires_elevation: false,
            },
            Err(err) => SettingReport {
                os: "windows".into(),
                setting: PRI_ID.into(),
                status: "partial".into(),
                value: String::new(),
                detail: format!("set ok but verify read failed: {err}"),
                source_url: PRI_SRC.into(),
                requires_elevation: false,
            },
        }
    }

    pub fn close_background_app(reason: &str) -> SettingReport {
        let pid = match parse_pid(reason) {
            Ok(pid) => pid,
            Err(detail) => {
                return SettingReport {
                    os: "windows".into(),
                    setting: BG_ID.into(),
                    status: "failed".into(),
                    value: String::new(),
                    detail,
                    source_url: BG_SRC.into(),
                    requires_elevation: false,
                };
            }
        };
        match OsBackgroundAppsBackend.close(pid) {
            Ok(()) => SettingReport {
                os: "windows".into(),
                setting: BG_ID.into(),
                status: "ok".into(),
                value: format!("pid={pid};status=closed"),
                detail: "TerminateProcess (user-chosen PID only)".into(),
                source_url: BG_SRC.into(),
                requires_elevation: false,
            },
            Err(err) => SettingReport {
                os: "windows".into(),
                setting: BG_ID.into(),
                status: "failed".into(),
                value: String::new(),
                detail: err.to_string(),
                source_url: BG_SRC.into(),
                requires_elevation: false,
            },
        }
    }

    pub fn pause_background_app(reason: &str) -> SettingReport {
        let pid = match parse_pid(reason) {
            Ok(pid) => pid,
            Err(detail) => {
                return SettingReport {
                    os: "windows".into(),
                    setting: BG_ID.into(),
                    status: "failed".into(),
                    value: String::new(),
                    detail,
                    source_url: BG_SRC.into(),
                    requires_elevation: false,
                };
            }
        };
        match OsBackgroundAppsBackend.pause(pid) {
            Ok(()) => SettingReport {
                os: "windows".into(),
                setting: BG_ID.into(),
                status: "ok".into(),
                value: format!("pid={pid};status=paused"),
                detail: "SuspendThread (user-chosen PID only)".into(),
                source_url: BG_SRC.into(),
                requires_elevation: false,
            },
            Err(err) => SettingReport {
                os: "windows".into(),
                setting: BG_ID.into(),
                status: "failed".into(),
                value: String::new(),
                detail: err.to_string(),
                source_url: BG_SRC.into(),
                requires_elevation: false,
            },
        }
    }

    pub fn resume_background_app(reason: &str) -> SettingReport {
        let pid = match parse_pid(reason) {
            Ok(pid) => pid,
            Err(detail) => {
                return SettingReport {
                    os: "windows".into(),
                    setting: BG_ID.into(),
                    status: "failed".into(),
                    value: String::new(),
                    detail,
                    source_url: BG_SRC.into(),
                    requires_elevation: false,
                };
            }
        };
        match OsBackgroundAppsBackend.resume(pid) {
            Ok(()) => SettingReport {
                os: "windows".into(),
                setting: BG_ID.into(),
                status: "ok".into(),
                value: format!("pid={pid};status=running"),
                detail: "ResumeThread (user-chosen PID only)".into(),
                source_url: BG_SRC.into(),
                requires_elevation: false,
            },
            Err(err) => SettingReport {
                os: "windows".into(),
                setting: BG_ID.into(),
                status: "failed".into(),
                value: String::new(),
                detail: err.to_string(),
                source_url: BG_SRC.into(),
                requires_elevation: false,
            },
        }
    }

    pub fn list_startup_apps() -> SettingReport {
        match OsStartupAppsBackend.list() {
            Ok(entries) => {
                let value = entries
                    .iter()
                    .map(StartupEntry::encode)
                    .collect::<Vec<_>>()
                    .join("|");
                SettingReport {
                    os: "windows".into(),
                    setting: START_ID.into(),
                    status: "ok".into(),
                    value,
                    detail: format!("{} HKCU Run entries", entries.len()),
                    source_url: START_SRC.into(),
                    requires_elevation: false,
                }
            }
            Err(err) => SettingReport {
                os: "windows".into(),
                setting: START_ID.into(),
                status: "failed".into(),
                value: String::new(),
                detail: err.to_string(),
                source_url: START_SRC.into(),
                requires_elevation: false,
            },
        }
    }

    pub fn set_startup_app(reason: &str) -> SettingReport {
        let mut name = None;
        let mut enabled = None;
        for part in reason.split(';') {
            let part = part.trim();
            if let Some(v) = part.strip_prefix("name=") {
                name = Some(v.to_string());
            } else if let Some(v) = part.strip_prefix("enabled=") {
                enabled = Some(matches!(v, "1" | "true" | "on"));
            } else if name.is_none() && !part.is_empty() && !part.contains('=') {
                name = Some(part.to_string());
            }
        }
        let (Some(name), Some(enabled)) = (name, enabled) else {
            return SettingReport {
                os: "windows".into(),
                setting: START_ID.into(),
                status: "failed".into(),
                value: String::new(),
                detail: format!("expected name=...;enabled=0|1, got {reason}"),
                source_url: START_SRC.into(),
                requires_elevation: false,
            };
        };
        match OsStartupAppsBackend.set_enabled(&name, enabled) {
            Ok(()) => match OsStartupAppsBackend.get(&name) {
                Ok(entry) => SettingReport {
                    os: "windows".into(),
                    setting: START_ID.into(),
                    status: "ok".into(),
                    value: entry.encode(),
                    detail: format!("StartupApproved {} => enabled={enabled}", entry.name),
                    source_url: START_SRC.into(),
                    requires_elevation: false,
                },
                Err(err) => SettingReport {
                    os: "windows".into(),
                    setting: START_ID.into(),
                    status: "partial".into(),
                    value: String::new(),
                    detail: format!("set ok but verify read failed: {err}"),
                    source_url: START_SRC.into(),
                    requires_elevation: false,
                },
            },
            Err(err) => SettingReport {
                os: "windows".into(),
                setting: START_ID.into(),
                status: "failed".into(),
                value: String::new(),
                detail: err.to_string(),
                source_url: START_SRC.into(),
                requires_elevation: false,
            },
        }
    }

    use hertzline_core::action::Action;
    use hertzline_core::actions::windows::cleanup::{
        CleanupBackend, ACTION_ID as CLEAN_ID, SOURCE_URL as CLEAN_SRC,
    };
    use hertzline_core::actions::windows::display::{
        DisplayBackend, ACTION_ID as DISP_ID, SOURCE_URL as DISP_SRC,
    };
    use hertzline_core::actions::windows::drivers::{ACTION_ID as DRV_ID, SOURCE_URL as DRV_SRC};
    use hertzline_core::actions::windows::game_bar::{
        GameBarBackend, ACTION_ID as BAR_ID, SOURCE_URL as BAR_SRC,
    };
    use hertzline_core::actions::windows::storage::{ACTION_ID as STOR_ID, SOURCE_URL as STOR_SRC};
    use hertzline_core::actions::windows::{
        cleanup::os::OsCleanupBackend, display::os::OsDisplayBackend,
        drivers::os::OsDriversBackend, game_bar::os::OsGameBarBackend,
        storage::os::OsStorageBackend, CleanupAction, DriverCheckAction, StorageHealthAction,
    };

    pub fn get_game_bar() -> SettingReport {
        match OsGameBarBackend.get() {
            Ok(value) => SettingReport {
                os: "windows".into(),
                setting: BAR_ID.into(),
                status: "ok".into(),
                value,
                detail: "VKMToggleRecording".into(),
                source_url: BAR_SRC.into(),
                requires_elevation: false,
            },
            Err(err) => SettingReport {
                os: "windows".into(),
                setting: BAR_ID.into(),
                status: "failed".into(),
                value: String::new(),
                detail: err.to_string(),
                source_url: BAR_SRC.into(),
                requires_elevation: false,
            },
        }
    }

    pub fn set_game_bar(enabled: bool) -> SettingReport {
        match OsGameBarBackend.set(enabled) {
            Ok(()) => match OsGameBarBackend.get() {
                Ok(value) => SettingReport {
                    os: "windows".into(),
                    setting: BAR_ID.into(),
                    status: "ok".into(),
                    value,
                    detail: format!("VKMToggleRecording={}", if enabled { 1 } else { 0 }),
                    source_url: BAR_SRC.into(),
                    requires_elevation: false,
                },
                Err(err) => SettingReport {
                    os: "windows".into(),
                    setting: BAR_ID.into(),
                    status: "partial".into(),
                    value: String::new(),
                    detail: format!("set ok but verify read failed: {err}"),
                    source_url: BAR_SRC.into(),
                    requires_elevation: false,
                },
            },
            Err(err) => SettingReport {
                os: "windows".into(),
                setting: BAR_ID.into(),
                status: "failed".into(),
                value: String::new(),
                detail: err.to_string(),
                source_url: BAR_SRC.into(),
                requires_elevation: false,
            },
        }
    }

    pub fn preview_cleanup() -> SettingReport {
        match OsCleanupBackend.preview() {
            Ok(preview) => SettingReport {
                os: "windows".into(),
                setting: CLEAN_ID.into(),
                status: "ok".into(),
                value: preview.encode(),
                detail: format!(
                    "preview entries={} total_bytes={}",
                    preview.entries.len(),
                    preview.total_bytes
                ),
                source_url: CLEAN_SRC.into(),
                requires_elevation: false,
            },
            Err(err) => SettingReport {
                os: "windows".into(),
                setting: CLEAN_ID.into(),
                status: "failed".into(),
                value: String::new(),
                detail: err.to_string(),
                source_url: CLEAN_SRC.into(),
                requires_elevation: false,
            },
        }
    }

    pub fn apply_cleanup(reason: &str) -> SettingReport {
        let confirmed = reason.trim() == "confirm=1"
            || reason.trim() == "confirm=true"
            || reason.trim().eq_ignore_ascii_case("confirm");
        if !confirmed {
            return SettingReport {
                os: "windows".into(),
                setting: CLEAN_ID.into(),
                status: "failed".into(),
                value: String::new(),
                detail: "apply_cleanup requires --reason confirm=1 after preview (AC-4.7)".into(),
                source_url: CLEAN_SRC.into(),
                requires_elevation: false,
            };
        }
        let mut action = CleanupAction::confirmed(OsCleanupBackend);
        match action.apply() {
            Ok(()) => match OsCleanupBackend.preview() {
                Ok(preview) => SettingReport {
                    os: "windows".into(),
                    setting: CLEAN_ID.into(),
                    status: "ok".into(),
                    value: preview.encode(),
                    detail: "cleanup applied (irreversible)".into(),
                    source_url: CLEAN_SRC.into(),
                    requires_elevation: false,
                },
                Err(err) => SettingReport {
                    os: "windows".into(),
                    setting: CLEAN_ID.into(),
                    status: "partial".into(),
                    value: String::new(),
                    detail: format!("apply ok but preview failed: {err}"),
                    source_url: CLEAN_SRC.into(),
                    requires_elevation: false,
                },
            },
            Err(err) => SettingReport {
                os: "windows".into(),
                setting: CLEAN_ID.into(),
                status: "failed".into(),
                value: String::new(),
                detail: err.to_string(),
                source_url: CLEAN_SRC.into(),
                requires_elevation: false,
            },
        }
    }

    pub fn get_storage_health() -> SettingReport {
        let action = StorageHealthAction::new(OsStorageBackend);
        match action.read_state() {
            Ok(value) => SettingReport {
                os: "windows".into(),
                setting: STOR_ID.into(),
                status: "ok".into(),
                value,
                detail: "GetDiskFreeSpaceExW; SMART unavailable".into(),
                source_url: STOR_SRC.into(),
                requires_elevation: false,
            },
            Err(err) => SettingReport {
                os: "windows".into(),
                setting: STOR_ID.into(),
                status: "failed".into(),
                value: String::new(),
                detail: err.to_string(),
                source_url: STOR_SRC.into(),
                requires_elevation: false,
            },
        }
    }

    pub fn get_driver_info() -> SettingReport {
        let action = DriverCheckAction::new(OsDriversBackend);
        match action.read_state() {
            Ok(value) => SettingReport {
                os: "windows".into(),
                setting: DRV_ID.into(),
                status: "ok".into(),
                value,
                detail: "display class DriverVersion + official vendor URLs only".into(),
                source_url: DRV_SRC.into(),
                requires_elevation: false,
            },
            Err(err) => SettingReport {
                os: "windows".into(),
                setting: DRV_ID.into(),
                status: "failed".into(),
                value: String::new(),
                detail: err.to_string(),
                source_url: DRV_SRC.into(),
                requires_elevation: false,
            },
        }
    }

    pub fn get_display() -> SettingReport {
        match OsDisplayBackend.get() {
            Ok(state) => SettingReport {
                os: "windows".into(),
                setting: DISP_ID.into(),
                status: "ok".into(),
                value: state.encode(),
                detail: "EnumDisplaySettingsW; VRR unavailable via public Win32".into(),
                source_url: DISP_SRC.into(),
                requires_elevation: false,
            },
            Err(err) => SettingReport {
                os: "windows".into(),
                setting: DISP_ID.into(),
                status: "failed".into(),
                value: String::new(),
                detail: err.to_string(),
                source_url: DISP_SRC.into(),
                requires_elevation: false,
            },
        }
    }

    pub fn set_display_refresh(hz: &str) -> SettingReport {
        let Ok(hz) = hz.trim().parse::<u32>() else {
            return SettingReport {
                os: "windows".into(),
                setting: DISP_ID.into(),
                status: "failed".into(),
                value: String::new(),
                detail: format!("set_display_refresh reason must be Hz integer, got {hz}"),
                source_url: DISP_SRC.into(),
                requires_elevation: false,
            };
        };
        match OsDisplayBackend.set_refresh_hz(hz) {
            Ok(()) => match OsDisplayBackend.get() {
                Ok(state) => SettingReport {
                    os: "windows".into(),
                    setting: DISP_ID.into(),
                    status: "ok".into(),
                    value: state.encode(),
                    detail: format!("ChangeDisplaySettingsExW refresh={hz}"),
                    source_url: DISP_SRC.into(),
                    requires_elevation: false,
                },
                Err(err) => SettingReport {
                    os: "windows".into(),
                    setting: DISP_ID.into(),
                    status: "partial".into(),
                    value: String::new(),
                    detail: format!("set ok but verify read failed: {err}"),
                    source_url: DISP_SRC.into(),
                    requires_elevation: false,
                },
            },
            Err(err) => SettingReport {
                os: "windows".into(),
                setting: DISP_ID.into(),
                status: "failed".into(),
                value: String::new(),
                detail: err.to_string(),
                source_url: DISP_SRC.into(),
                requires_elevation: false,
            },
        }
    }
}

#[cfg(not(windows))]
fn not_windows(setting: &str, elev: bool) -> SettingReport {
    SettingReport {
        os: std::env::consts::OS.into(),
        setting: setting.into(),
        status: "not_possible".into(),
        value: String::new(),
        detail: "Windows-only setting (AC-4.1 / AC-4.2)".into(),
        source_url: String::new(),
        requires_elevation: elev,
    }
}

pub fn get_power_plan() -> SettingReport {
    #[cfg(windows)]
    {
        win::get_power_plan()
    }
    #[cfg(not(windows))]
    {
        not_windows("windows.power_plan", false)
    }
}

pub fn set_power_plan(guid: Option<&str>) -> SettingReport {
    #[cfg(windows)]
    {
        win::set_power_plan(guid.unwrap_or(""))
    }
    #[cfg(not(windows))]
    {
        let _ = guid;
        not_windows("windows.power_plan", false)
    }
}

pub fn get_game_mode() -> SettingReport {
    #[cfg(windows)]
    {
        win::get_game_mode()
    }
    #[cfg(not(windows))]
    {
        not_windows("windows.game_mode", false)
    }
}

pub fn set_game_mode(enabled: bool) -> SettingReport {
    #[cfg(windows)]
    {
        win::set_game_mode(enabled)
    }
    #[cfg(not(windows))]
    {
        let _ = enabled;
        not_windows("windows.game_mode", false)
    }
}

pub fn get_hags() -> SettingReport {
    #[cfg(windows)]
    {
        win::get_hags()
    }
    #[cfg(not(windows))]
    {
        not_windows("windows.hags", false)
    }
}

pub fn set_hags(mode: Option<&str>) -> SettingReport {
    #[cfg(windows)]
    {
        win::set_hags(mode.unwrap_or(""))
    }
    #[cfg(not(windows))]
    {
        let _ = mode;
        not_windows("windows.hags", true)
    }
}

pub fn get_process_priority(reason: Option<&str>) -> SettingReport {
    #[cfg(windows)]
    {
        win::get_process_priority(reason.unwrap_or(""))
    }
    #[cfg(not(windows))]
    {
        let _ = reason;
        not_windows("windows.process_priority", false)
    }
}

pub fn set_process_priority(reason: Option<&str>) -> SettingReport {
    #[cfg(windows)]
    {
        win::set_process_priority(reason.unwrap_or(""))
    }
    #[cfg(not(windows))]
    {
        let _ = reason;
        not_windows("windows.process_priority", false)
    }
}

pub fn close_background_app(reason: Option<&str>) -> SettingReport {
    #[cfg(windows)]
    {
        win::close_background_app(reason.unwrap_or(""))
    }
    #[cfg(not(windows))]
    {
        let _ = reason;
        not_windows("windows.background_apps", false)
    }
}

pub fn pause_background_app(reason: Option<&str>) -> SettingReport {
    #[cfg(windows)]
    {
        win::pause_background_app(reason.unwrap_or(""))
    }
    #[cfg(not(windows))]
    {
        let _ = reason;
        not_windows("windows.background_apps", false)
    }
}

pub fn resume_background_app(reason: Option<&str>) -> SettingReport {
    #[cfg(windows)]
    {
        win::resume_background_app(reason.unwrap_or(""))
    }
    #[cfg(not(windows))]
    {
        let _ = reason;
        not_windows("windows.background_apps", false)
    }
}

pub fn list_startup_apps() -> SettingReport {
    #[cfg(windows)]
    {
        win::list_startup_apps()
    }
    #[cfg(not(windows))]
    {
        not_windows("windows.startup_apps", false)
    }
}

pub fn set_startup_app(reason: Option<&str>) -> SettingReport {
    #[cfg(windows)]
    {
        win::set_startup_app(reason.unwrap_or(""))
    }
    #[cfg(not(windows))]
    {
        let _ = reason;
        not_windows("windows.startup_apps", false)
    }
}

pub fn get_game_bar() -> SettingReport {
    #[cfg(windows)]
    {
        win::get_game_bar()
    }
    #[cfg(not(windows))]
    {
        not_windows("windows.game_bar_capture", false)
    }
}

pub fn set_game_bar(enabled: bool) -> SettingReport {
    #[cfg(windows)]
    {
        win::set_game_bar(enabled)
    }
    #[cfg(not(windows))]
    {
        let _ = enabled;
        not_windows("windows.game_bar_capture", false)
    }
}

pub fn preview_cleanup() -> SettingReport {
    #[cfg(windows)]
    {
        win::preview_cleanup()
    }
    #[cfg(not(windows))]
    {
        not_windows("windows.temp_shader_cleanup", false)
    }
}

pub fn apply_cleanup(reason: Option<&str>) -> SettingReport {
    #[cfg(windows)]
    {
        win::apply_cleanup(reason.unwrap_or(""))
    }
    #[cfg(not(windows))]
    {
        let _ = reason;
        not_windows("windows.temp_shader_cleanup", false)
    }
}

pub fn get_storage_health() -> SettingReport {
    #[cfg(windows)]
    {
        win::get_storage_health()
    }
    #[cfg(not(windows))]
    {
        not_windows("windows.storage_health", false)
    }
}

pub fn get_driver_info() -> SettingReport {
    #[cfg(windows)]
    {
        win::get_driver_info()
    }
    #[cfg(not(windows))]
    {
        not_windows("windows.driver_check", false)
    }
}

pub fn get_display() -> SettingReport {
    #[cfg(windows)]
    {
        win::get_display()
    }
    #[cfg(not(windows))]
    {
        not_windows("windows.display_refresh", false)
    }
}

pub fn set_display_refresh(reason: Option<&str>) -> SettingReport {
    #[cfg(windows)]
    {
        win::set_display_refresh(reason.unwrap_or(""))
    }
    #[cfg(not(windows))]
    {
        let _ = reason;
        not_windows("windows.display_refresh", false)
    }
}

/// Register Windows action handlers for headless restore-all (HZ-020--HZ-022).
pub fn register_windows_actions(registry: &mut hertzline_core::restore::MapActionRegistry) {
    #[cfg(windows)]
    {
        use hertzline_core::actions::windows::{
            background::os::OsBackgroundAppsBackend, cleanup::os::OsCleanupBackend,
            display::os::OsDisplayBackend, drivers::os::OsDriversBackend,
            game_bar::os::OsGameBarBackend, game_mode::os::OsGameModeBackend,
            hags::os::OsHagsBackend, power_plan::os::OsPowerPlanBackend,
            priority::os::OsProcessPriorityBackend, startup::os::OsStartupAppsBackend,
            storage::os::OsStorageBackend, BackgroundAppsAction, CleanupAction,
            DisplayRefreshAction, DriverCheckAction, GameBarAction, GameModeAction, HagsAction,
            PowerPlanAction, ProcessPriorityAction, StartupAppsAction, StorageHealthAction,
        };
        registry.insert(Box::new(PowerPlanAction::high_performance(
            OsPowerPlanBackend,
        )));
        registry.insert(Box::new(GameModeAction::enable(OsGameModeBackend)));
        registry.insert(Box::new(HagsAction::enable(OsHagsBackend)));
        // Placeholder pid/name; restore adopts identity from snapshot (AC-4.3--4.5).
        registry.insert(Box::new(ProcessPriorityAction::high_priority(
            OsProcessPriorityBackend,
            0,
        )));
        registry.insert(Box::new(BackgroundAppsAction::pause(
            OsBackgroundAppsBackend,
            0,
        )));
        registry.insert(Box::new(StartupAppsAction::disable(
            OsStartupAppsBackend,
            "",
        )));
        registry.insert(Box::new(GameBarAction::disable(OsGameBarBackend)));
        registry.insert(Box::new(CleanupAction::confirmed(OsCleanupBackend)));
        registry.insert(Box::new(StorageHealthAction::new(OsStorageBackend)));
        registry.insert(Box::new(DriverCheckAction::new(OsDriversBackend)));
        registry.insert(Box::new(DisplayRefreshAction::report_only(
            OsDisplayBackend,
        )));
    }
    #[cfg(not(windows))]
    {
        let _ = registry;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_include_setting_id() {
        let power = get_power_plan();
        assert_eq!(power.setting, "windows.power_plan");
        let game = get_game_mode();
        assert_eq!(game.setting, "windows.game_mode");
        let hags = get_hags();
        assert_eq!(hags.setting, "windows.hags");
        #[cfg(windows)]
        {
            assert_eq!(power.os, "windows");
            assert!(power.status == "ok" || power.status == "failed");
            assert!(game.status == "ok" || game.status == "failed");
            assert!(
                hags.status == "ok" || hags.status == "not_possible" || hags.status == "failed"
            );
        }
        #[cfg(not(windows))]
        {
            assert_eq!(power.status, "not_possible");
            assert_eq!(game.status, "not_possible");
            assert_eq!(hags.status, "not_possible");
        }
    }

    #[test]
    fn set_hags_rejects_bad_mode_on_windows_shape() {
        let report = set_hags(Some("9"));
        #[cfg(windows)]
        {
            assert_eq!(report.status, "failed");
            assert!(report.detail.contains("invalid"));
        }
        #[cfg(not(windows))]
        {
            assert_eq!(report.status, "not_possible");
        }
    }

    #[test]
    fn hz021_ops_shape() {
        let pri = get_process_priority(Some(""));
        assert_eq!(pri.setting, "windows.process_priority");
        let startup = list_startup_apps();
        assert_eq!(startup.setting, "windows.startup_apps");
        #[cfg(windows)]
        {
            assert!(pri.status == "failed" || pri.status == "ok");
            assert!(startup.status == "ok" || startup.status == "failed");
            let self_pri = get_process_priority(Some(&std::process::id().to_string()));
            assert_eq!(self_pri.status, "ok");
            assert!(self_pri.value.contains("pid="));
        }
        #[cfg(not(windows))]
        {
            assert_eq!(pri.status, "not_possible");
            assert_eq!(startup.status, "not_possible");
        }
    }

    #[test]
    fn hz022_ops_shape() {
        let bar = get_game_bar();
        assert_eq!(bar.setting, "windows.game_bar_capture");
        let preview = preview_cleanup();
        assert_eq!(preview.setting, "windows.temp_shader_cleanup");
        let storage = get_storage_health();
        assert_eq!(storage.setting, "windows.storage_health");
        let drivers = get_driver_info();
        assert_eq!(drivers.setting, "windows.driver_check");
        let display = get_display();
        assert_eq!(display.setting, "windows.display_refresh");
        let denied = apply_cleanup(Some("no"));
        assert_eq!(denied.setting, "windows.temp_shader_cleanup");
        #[cfg(windows)]
        {
            assert!(bar.status == "ok" || bar.status == "failed");
            assert!(preview.status == "ok" || preview.status == "failed");
            assert!(preview.value.contains("total=") || preview.status == "failed");
            assert!(storage.status == "ok" || storage.status == "failed");
            assert!(storage.value.contains("smart=unavailable") || storage.status == "failed");
            assert!(drivers.status == "ok" || drivers.status == "failed");
            assert!(display.status == "ok" || display.status == "failed");
            assert!(display.value.contains("vrr=unavailable") || display.status == "failed");
            assert_eq!(denied.status, "failed");
            assert!(denied.detail.contains("confirm"));
        }
        #[cfg(not(windows))]
        {
            assert_eq!(bar.status, "not_possible");
            assert_eq!(preview.status, "not_possible");
            assert_eq!(storage.status, "not_possible");
            assert_eq!(drivers.status, "not_possible");
            assert_eq!(display.status, "not_possible");
        }
    }
}
