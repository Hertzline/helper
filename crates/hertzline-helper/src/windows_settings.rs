//! Windows settings helper ops (HZ-020 / HZ-021).
//!
//! Power plan, Game Mode, priority, background apps, and startup are
//! unprivileged (current user / same-session process). HAGS write needs
//! elevation (HKLM HwSchMode).

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

/// Register Windows action handlers for headless restore-all (HZ-020 / HZ-021).
pub fn register_windows_actions(registry: &mut hertzline_core::restore::MapActionRegistry) {
    #[cfg(windows)]
    {
        use hertzline_core::actions::windows::{
            background::os::OsBackgroundAppsBackend, game_mode::os::OsGameModeBackend,
            hags::os::OsHagsBackend, power_plan::os::OsPowerPlanBackend,
            priority::os::OsProcessPriorityBackend, startup::os::OsStartupAppsBackend,
            BackgroundAppsAction, GameModeAction, HagsAction, PowerPlanAction,
            ProcessPriorityAction, StartupAppsAction,
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
}
