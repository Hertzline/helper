//! macOS settings helper ops (HZ-026 / AC-4.19--AC-4.22).
//!
//! Game Mode status, Low Power Mode probe, background signals, and display
//! refresh. Non-macOS hosts report `not_possible`.

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

fn not_macos(setting: &str, source_url: &str, elev: bool) -> SettingReport {
    SettingReport {
        os: std::env::consts::OS.into(),
        setting: setting.into(),
        status: "not_possible".into(),
        value: String::new(),
        detail: "macOS-only setting".into(),
        source_url: source_url.into(),
        requires_elevation: elev,
    }
}

#[cfg(target_os = "macos")]
mod macos {
    use super::*;
    use hertzline_core::actions::macos::background::{
        MacOsBackgroundAppsBackend, ACTION_ID as BG_ID, SOURCE_URL as BG_SRC,
    };
    use hertzline_core::actions::macos::game_mode::{
        MacOsGameModeBackend, ACTION_ID as GM_ID, SOURCE_URL as GM_SRC, STATUS_NOT_POSSIBLE,
    };
    use hertzline_core::actions::macos::low_power::{
        LowPowerBackend, ACTION_ID as LPM_ID, SOURCE_URL as LPM_SRC, WARNING_PERF_WHILE_LPM,
    };
    use hertzline_core::actions::macos::refresh::{
        MacOsRefreshBackend, ACTION_ID as REF_ID, SOURCE_URL as REF_SRC,
    };
    use hertzline_core::actions::macos::{
        background::os::OsMacOsBackgroundAppsBackend, game_mode::host::HostMacOsGameModeBackend,
        low_power::os::OsLowPowerBackend, refresh::os::OsMacOsRefreshBackend,
    };

    fn parse_pid(reason: &str) -> Result<u32, String> {
        let token = reason
            .split(';')
            .next()
            .unwrap_or("")
            .trim()
            .strip_prefix("pid=")
            .unwrap_or(reason.trim());
        token
            .parse::<u32>()
            .map_err(|_| format!("pid required, got {reason}"))
    }

    pub fn get_macos_game_mode() -> SettingReport {
        match HostMacOsGameModeBackend.status() {
            Ok(report) => SettingReport {
                os: "macos".into(),
                setting: GM_ID.into(),
                status: if report.status == STATUS_NOT_POSSIBLE {
                    "not_possible".into()
                } else {
                    "ok".into()
                },
                value: report.encode(),
                detail: report.detail,
                source_url: GM_SRC.into(),
                requires_elevation: false,
            },
            Err(err) => SettingReport {
                os: "macos".into(),
                setting: GM_ID.into(),
                status: "failed".into(),
                value: String::new(),
                detail: err.to_string(),
                source_url: GM_SRC.into(),
                requires_elevation: false,
            },
        }
    }

    pub fn get_macos_lpm(reason: &str) -> SettingReport {
        let perf = reason.contains("performance=1")
            || reason.contains("perf=1")
            || reason.trim().is_empty();
        match OsLowPowerBackend.read() {
            Ok(mut state) => {
                if perf && state.available && state.active {
                    state.warning = Some(WARNING_PERF_WHILE_LPM.into());
                }
                SettingReport {
                    os: "macos".into(),
                    setting: LPM_ID.into(),
                    status: if state.available {
                        "ok".into()
                    } else {
                        "unavailable".into()
                    },
                    value: state.encode(),
                    detail: state.detail,
                    source_url: LPM_SRC.into(),
                    requires_elevation: false,
                }
            }
            Err(err) => SettingReport {
                os: "macos".into(),
                setting: LPM_ID.into(),
                status: "failed".into(),
                value: String::new(),
                detail: err.to_string(),
                source_url: LPM_SRC.into(),
                requires_elevation: false,
            },
        }
    }

    pub fn macos_close_background_app(reason: &str) -> SettingReport {
        let pid = match parse_pid(reason) {
            Ok(p) => p,
            Err(detail) => {
                return SettingReport {
                    os: "macos".into(),
                    setting: BG_ID.into(),
                    status: "failed".into(),
                    value: String::new(),
                    detail,
                    source_url: BG_SRC.into(),
                    requires_elevation: false,
                };
            }
        };
        match OsMacOsBackgroundAppsBackend.close(pid) {
            Ok(()) => SettingReport {
                os: "macos".into(),
                setting: BG_ID.into(),
                status: "ok".into(),
                value: format!("pid={pid};status=closed"),
                detail: "SIGTERM (user-chosen PID only)".into(),
                source_url: BG_SRC.into(),
                requires_elevation: false,
            },
            Err(err) => SettingReport {
                os: "macos".into(),
                setting: BG_ID.into(),
                status: "failed".into(),
                value: String::new(),
                detail: err.to_string(),
                source_url: BG_SRC.into(),
                requires_elevation: false,
            },
        }
    }

    pub fn macos_pause_background_app(reason: &str) -> SettingReport {
        let pid = match parse_pid(reason) {
            Ok(p) => p,
            Err(detail) => {
                return SettingReport {
                    os: "macos".into(),
                    setting: BG_ID.into(),
                    status: "failed".into(),
                    value: String::new(),
                    detail,
                    source_url: BG_SRC.into(),
                    requires_elevation: false,
                };
            }
        };
        match OsMacOsBackgroundAppsBackend.pause(pid) {
            Ok(()) => SettingReport {
                os: "macos".into(),
                setting: BG_ID.into(),
                status: "ok".into(),
                value: format!("pid={pid};status=paused"),
                detail: "SIGSTOP (user-chosen PID only)".into(),
                source_url: BG_SRC.into(),
                requires_elevation: false,
            },
            Err(err) => SettingReport {
                os: "macos".into(),
                setting: BG_ID.into(),
                status: "failed".into(),
                value: String::new(),
                detail: err.to_string(),
                source_url: BG_SRC.into(),
                requires_elevation: false,
            },
        }
    }

    pub fn macos_resume_background_app(reason: &str) -> SettingReport {
        let pid = match parse_pid(reason) {
            Ok(p) => p,
            Err(detail) => {
                return SettingReport {
                    os: "macos".into(),
                    setting: BG_ID.into(),
                    status: "failed".into(),
                    value: String::new(),
                    detail,
                    source_url: BG_SRC.into(),
                    requires_elevation: false,
                };
            }
        };
        match OsMacOsBackgroundAppsBackend.resume(pid) {
            Ok(()) => SettingReport {
                os: "macos".into(),
                setting: BG_ID.into(),
                status: "ok".into(),
                value: format!("pid={pid};status=running"),
                detail: "SIGCONT (user-chosen PID only)".into(),
                source_url: BG_SRC.into(),
                requires_elevation: false,
            },
            Err(err) => SettingReport {
                os: "macos".into(),
                setting: BG_ID.into(),
                status: "failed".into(),
                value: String::new(),
                detail: err.to_string(),
                source_url: BG_SRC.into(),
                requires_elevation: false,
            },
        }
    }

    pub fn get_macos_display() -> SettingReport {
        match OsMacOsRefreshBackend.get() {
            Ok(state) => SettingReport {
                os: "macos".into(),
                setting: REF_ID.into(),
                status: "ok".into(),
                value: state.encode(),
                detail: "CGDisplayModeGetRefreshRate".into(),
                source_url: REF_SRC.into(),
                requires_elevation: false,
            },
            Err(err) => SettingReport {
                os: "macos".into(),
                setting: REF_ID.into(),
                status: "failed".into(),
                value: String::new(),
                detail: err.to_string(),
                source_url: REF_SRC.into(),
                requires_elevation: false,
            },
        }
    }

    pub fn set_macos_display_refresh(reason: &str) -> SettingReport {
        let hz = match reason.trim().parse::<u32>() {
            Ok(h) if h > 0 => h,
            _ => {
                return SettingReport {
                    os: "macos".into(),
                    setting: REF_ID.into(),
                    status: "failed".into(),
                    value: String::new(),
                    detail: format!("Hz required, got {reason}"),
                    source_url: REF_SRC.into(),
                    requires_elevation: false,
                };
            }
        };
        match OsMacOsRefreshBackend.set_refresh_hz(hz) {
            Ok(()) => SettingReport {
                os: "macos".into(),
                setting: REF_ID.into(),
                status: "ok".into(),
                value: format!("refresh_hz={hz}"),
                detail: "CGDisplaySetDisplayMode".into(),
                source_url: REF_SRC.into(),
                requires_elevation: false,
            },
            Err(err) => {
                let status = if err.to_string().contains("not possible") {
                    "not_possible"
                } else {
                    "failed"
                };
                SettingReport {
                    os: "macos".into(),
                    setting: REF_ID.into(),
                    status: status.into(),
                    value: String::new(),
                    detail: err.to_string(),
                    source_url: REF_SRC.into(),
                    requires_elevation: false,
                }
            }
        }
    }
}

pub fn get_macos_game_mode() -> SettingReport {
    #[cfg(target_os = "macos")]
    {
        macos::get_macos_game_mode()
    }
    #[cfg(not(target_os = "macos"))]
    {
        not_macos(
            "macos.game_mode_status",
            "https://developer.apple.com/videos/play/wwdc2025/209/",
            false,
        )
    }
}

pub fn get_macos_lpm(reason: Option<&str>) -> SettingReport {
    #[cfg(target_os = "macos")]
    {
        macos::get_macos_lpm(reason.unwrap_or(""))
    }
    #[cfg(not(target_os = "macos"))]
    {
        let _ = reason;
        not_macos(
            "macos.low_power_mode",
            "https://developer.apple.com/library/archive/documentation/Performance/Conceptual/EnergyGuide-iOS/LowPowerMode.html",
            false,
        )
    }
}

pub fn macos_close_background_app(reason: Option<&str>) -> SettingReport {
    #[cfg(target_os = "macos")]
    {
        macos::macos_close_background_app(reason.unwrap_or(""))
    }
    #[cfg(not(target_os = "macos"))]
    {
        let _ = reason;
        not_macos(
            "macos.background_apps",
            "https://developer.apple.com/library/archive/documentation/System/Conceptual/ManPages_iPhoneOS/man2/kill.2.html",
            false,
        )
    }
}

pub fn macos_pause_background_app(reason: Option<&str>) -> SettingReport {
    #[cfg(target_os = "macos")]
    {
        macos::macos_pause_background_app(reason.unwrap_or(""))
    }
    #[cfg(not(target_os = "macos"))]
    {
        let _ = reason;
        not_macos(
            "macos.background_apps",
            "https://developer.apple.com/library/archive/documentation/System/Conceptual/ManPages_iPhoneOS/man2/kill.2.html",
            false,
        )
    }
}

pub fn macos_resume_background_app(reason: Option<&str>) -> SettingReport {
    #[cfg(target_os = "macos")]
    {
        macos::macos_resume_background_app(reason.unwrap_or(""))
    }
    #[cfg(not(target_os = "macos"))]
    {
        let _ = reason;
        not_macos(
            "macos.background_apps",
            "https://developer.apple.com/library/archive/documentation/System/Conceptual/ManPages_iPhoneOS/man2/kill.2.html",
            false,
        )
    }
}

pub fn get_macos_display() -> SettingReport {
    #[cfg(target_os = "macos")]
    {
        macos::get_macos_display()
    }
    #[cfg(not(target_os = "macos"))]
    {
        not_macos(
            "macos.refresh_rate",
            "https://developer.apple.com/documentation/coregraphics/cgdisplaymode/refreshrate",
            false,
        )
    }
}

pub fn set_macos_display_refresh(reason: Option<&str>) -> SettingReport {
    #[cfg(target_os = "macos")]
    {
        macos::set_macos_display_refresh(reason.unwrap_or(""))
    }
    #[cfg(not(target_os = "macos"))]
    {
        let _ = reason;
        not_macos(
            "macos.refresh_rate",
            "https://developer.apple.com/documentation/coregraphics/cgdisplaymode/refreshrate",
            false,
        )
    }
}

/// Register macOS action handlers for headless restore-all (HZ-026).
pub fn register_macos_actions(registry: &mut hertzline_core::restore::MapActionRegistry) {
    #[cfg(target_os = "macos")]
    {
        use hertzline_core::actions::macos::{
            background::os::OsMacOsBackgroundAppsBackend,
            game_mode::host::HostMacOsGameModeBackend, low_power::os::OsLowPowerBackend,
            refresh::os::OsMacOsRefreshBackend, LowPowerModeWarningAction,
            MacOsBackgroundAppsAction, MacOsGameModeStatusAction, MacOsRefreshAction,
        };
        registry.insert(Box::new(MacOsGameModeStatusAction::new(
            HostMacOsGameModeBackend,
        )));
        registry.insert(Box::new(LowPowerModeWarningAction::for_performance_work(
            OsLowPowerBackend,
        )));
        registry.insert(Box::new(MacOsBackgroundAppsAction::pause(
            OsMacOsBackgroundAppsBackend,
            0,
        )));
        registry.insert(Box::new(MacOsRefreshAction::report_only(
            OsMacOsRefreshBackend,
        )));
    }
    #[cfg(not(target_os = "macos"))]
    {
        let _ = registry;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hz026_ops_shape() {
        let gm = get_macos_game_mode();
        assert_eq!(gm.setting, "macos.game_mode_status");
        let lpm = get_macos_lpm(None);
        assert_eq!(lpm.setting, "macos.low_power_mode");
        let display = get_macos_display();
        assert_eq!(display.setting, "macos.refresh_rate");
        #[cfg(not(target_os = "macos"))]
        {
            assert_eq!(gm.status, "not_possible");
            assert_eq!(lpm.status, "not_possible");
            assert_eq!(display.status, "not_possible");
            let close = macos_close_background_app(Some("1"));
            assert_eq!(close.status, "not_possible");
            let set = set_macos_display_refresh(Some("60"));
            assert_eq!(set.status, "not_possible");
        }
        #[cfg(target_os = "macos")]
        {
            assert!(gm.status == "not_possible" || gm.status == "ok");
            assert!(lpm.status == "ok" || lpm.status == "unavailable" || lpm.status == "failed");
            assert!(display.status == "ok" || display.status == "failed");
        }
    }
}
