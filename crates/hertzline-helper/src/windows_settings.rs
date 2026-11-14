//! Windows power / Game Mode / HAGS helper ops (HZ-020).
//!
//! Power plan and Game Mode are unprivileged (current user).
//! HAGS write needs elevation (HKLM HwSchMode).

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

/// Register Windows action handlers for headless restore-all (HZ-020).
pub fn register_windows_actions(registry: &mut hertzline_core::restore::MapActionRegistry) {
    #[cfg(windows)]
    {
        use hertzline_core::actions::windows::{
            game_mode::os::OsGameModeBackend, hags::os::OsHagsBackend,
            power_plan::os::OsPowerPlanBackend, GameModeAction, HagsAction, PowerPlanAction,
        };
        registry.insert(Box::new(PowerPlanAction::high_performance(
            OsPowerPlanBackend,
        )));
        registry.insert(Box::new(GameModeAction::enable(OsGameModeBackend)));
        registry.insert(Box::new(HagsAction::enable(OsHagsBackend)));
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
}
