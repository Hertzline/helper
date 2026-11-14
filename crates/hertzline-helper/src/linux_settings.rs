//! Linux settings helper ops (HZ-023 / AC-4.11--AC-4.13).
//!
//! Feral GameMode and energy profile (ppd/tuned) are unprivileged probes/sets.
//! nice/ionice writes require elevation (CAP_SYS_NICE / polkit) when lowering nice.

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

#[cfg(target_os = "linux")]
mod linux {
    use super::*;
    use hertzline_core::actions::linux::gamemode::{
        FeralGameModeBackend, ACTION_ID as GM_ID, SOURCE_URL as GM_SRC,
    };
    use hertzline_core::actions::linux::governor::{
        EnergyProfileBackend, ACTION_ID as ENERGY_ID, PROFILE_PERFORMANCE, SOURCE_URL_PPD,
    };
    use hertzline_core::actions::linux::nice_ionice::{
        validate_ioprio, validate_nice, NiceIoniceBackend, NiceIoniceState, ACTION_ID as NICE_ID,
        DEFAULT_TARGET_IOPRIO_CLASS, DEFAULT_TARGET_IOPRIO_LEVEL, DEFAULT_TARGET_NICE,
        SOURCE_URL_IONICE, SOURCE_URL_NICE,
    };
    use hertzline_core::actions::linux::{
        gamemode::os::OsFeralGameModeBackend, governor::os::OsEnergyProfileBackend,
        nice_ionice::os::OsNiceIoniceBackend,
    };

    pub fn get_feral_gamemode(reason: &str) -> SettingReport {
        let pid = match parse_pid(reason) {
            Ok(p) => p,
            Err(detail) => {
                return failed(GM_ID, GM_SRC, false, detail);
            }
        };
        match OsFeralGameModeBackend.probe() {
            Ok(false) => SettingReport {
                os: "linux".into(),
                setting: GM_ID.into(),
                status: "not_possible".into(),
                value: format!("pid={pid};status=unavailable"),
                detail: "gamemoded not reachable on session bus".into(),
                source_url: GM_SRC.into(),
                requires_elevation: false,
            },
            Ok(true) => match OsFeralGameModeBackend.query_status(pid) {
                Ok(code) => SettingReport {
                    os: "linux".into(),
                    setting: GM_ID.into(),
                    status: "ok".into(),
                    value: format!("pid={pid};status={code}"),
                    detail: "QueryStatus".into(),
                    source_url: GM_SRC.into(),
                    requires_elevation: false,
                },
                Err(err) => failed(GM_ID, GM_SRC, false, err.to_string()),
            },
            Err(err) => SettingReport {
                os: "linux".into(),
                setting: GM_ID.into(),
                status: "not_possible".into(),
                value: String::new(),
                detail: err.to_string(),
                source_url: GM_SRC.into(),
                requires_elevation: false,
            },
        }
    }

    pub fn set_feral_gamemode(reason: &str) -> SettingReport {
        let (pid, op) = match parse_gamemode_arg(reason) {
            Ok(v) => v,
            Err(detail) => return failed(GM_ID, GM_SRC, false, detail),
        };
        let result = match op.as_str() {
            "register" => OsFeralGameModeBackend.register(pid),
            "unregister" => OsFeralGameModeBackend.unregister(pid),
            other => {
                return failed(
                    GM_ID,
                    GM_SRC,
                    false,
                    format!("op must be register|unregister, got {other}"),
                );
            }
        };
        match result {
            Ok(code) if code >= 0 => SettingReport {
                os: "linux".into(),
                setting: GM_ID.into(),
                status: "ok".into(),
                value: format!("pid={pid};op={op};result={code}"),
                detail: format!("{op} via D-Bus"),
                source_url: GM_SRC.into(),
                requires_elevation: false,
            },
            Ok(code) => failed(GM_ID, GM_SRC, false, format!("{op} returned {code}")),
            Err(err) => {
                let status = if err.to_string().contains("not possible") {
                    "not_possible"
                } else {
                    "failed"
                };
                SettingReport {
                    os: "linux".into(),
                    setting: GM_ID.into(),
                    status: status.into(),
                    value: String::new(),
                    detail: err.to_string(),
                    source_url: GM_SRC.into(),
                    requires_elevation: false,
                }
            }
        }
    }

    pub fn get_energy_profile() -> SettingReport {
        match OsEnergyProfileBackend.backend_name() {
            Ok(name) => match OsEnergyProfileBackend.get_profile() {
                Ok(profile) => SettingReport {
                    os: "linux".into(),
                    setting: ENERGY_ID.into(),
                    status: "ok".into(),
                    value: format!("backend={name};profile={profile}"),
                    detail: name.into(),
                    source_url: SOURCE_URL_PPD.into(),
                    requires_elevation: false,
                },
                Err(err) => failed(ENERGY_ID, SOURCE_URL_PPD, false, err.to_string()),
            },
            Err(err) => SettingReport {
                os: "linux".into(),
                setting: ENERGY_ID.into(),
                status: "not_possible".into(),
                value: "backend=unavailable;profile=unavailable".into(),
                detail: err.to_string(),
                source_url: SOURCE_URL_PPD.into(),
                requires_elevation: false,
            },
        }
    }

    pub fn set_energy_profile(reason: &str) -> SettingReport {
        let target = if reason.trim().is_empty() {
            PROFILE_PERFORMANCE
        } else {
            reason.trim()
        };
        match OsEnergyProfileBackend.set_profile(target) {
            Ok(()) => match OsEnergyProfileBackend.get_profile() {
                Ok(profile) => {
                    let backend = OsEnergyProfileBackend.backend_name().unwrap_or("unknown");
                    SettingReport {
                        os: "linux".into(),
                        setting: ENERGY_ID.into(),
                        status: "ok".into(),
                        value: format!("backend={backend};profile={profile}"),
                        detail: format!("set {target}"),
                        source_url: SOURCE_URL_PPD.into(),
                        requires_elevation: false,
                    }
                }
                Err(err) => SettingReport {
                    os: "linux".into(),
                    setting: ENERGY_ID.into(),
                    status: "partial".into(),
                    value: String::new(),
                    detail: format!("set ok but verify read failed: {err}"),
                    source_url: SOURCE_URL_PPD.into(),
                    requires_elevation: false,
                },
            },
            Err(err) => {
                let msg = err.to_string();
                let status = if msg.contains("not possible") {
                    "not_possible"
                } else {
                    "failed"
                };
                SettingReport {
                    os: "linux".into(),
                    setting: ENERGY_ID.into(),
                    status: status.into(),
                    value: String::new(),
                    detail: msg,
                    source_url: SOURCE_URL_PPD.into(),
                    requires_elevation: false,
                }
            }
        }
    }

    pub fn get_nice_ionice(reason: &str) -> SettingReport {
        let pid = match parse_pid(reason) {
            Ok(p) => p,
            Err(detail) => return failed(NICE_ID, SOURCE_URL_NICE, false, detail),
        };
        match OsNiceIoniceBackend.read(pid) {
            Ok(state) => SettingReport {
                os: "linux".into(),
                setting: NICE_ID.into(),
                status: "ok".into(),
                value: state.encode(),
                detail: "getpriority + ionice -p".into(),
                source_url: SOURCE_URL_NICE.into(),
                requires_elevation: false,
            },
            Err(err) => failed(NICE_ID, SOURCE_URL_NICE, false, err.to_string()),
        }
    }

    pub fn set_nice_ionice(reason: &str) -> SettingReport {
        let target = match parse_nice_arg(reason) {
            Ok(v) => v,
            Err(detail) => {
                return failed(NICE_ID, SOURCE_URL_NICE, true, detail);
            }
        };
        if let Err(err) = validate_nice(target.nice) {
            return failed(NICE_ID, SOURCE_URL_NICE, true, err.to_string());
        }
        if let Err(err) = validate_ioprio(target.ioprio_class, target.ioprio_level) {
            return failed(NICE_ID, SOURCE_URL_IONICE, true, err.to_string());
        }
        if let Err(err) = OsNiceIoniceBackend.set_nice(target.pid, target.nice) {
            return failed(NICE_ID, SOURCE_URL_NICE, true, err.to_string());
        }
        if let Err(err) =
            OsNiceIoniceBackend.set_ioprio(target.pid, target.ioprio_class, target.ioprio_level)
        {
            return failed(NICE_ID, SOURCE_URL_IONICE, true, err.to_string());
        }
        match OsNiceIoniceBackend.read(target.pid) {
            Ok(state) => SettingReport {
                os: "linux".into(),
                setting: NICE_ID.into(),
                status: "ok".into(),
                value: state.encode(),
                detail: "setpriority + ionice".into(),
                source_url: SOURCE_URL_NICE.into(),
                requires_elevation: true,
            },
            Err(err) => SettingReport {
                os: "linux".into(),
                setting: NICE_ID.into(),
                status: "partial".into(),
                value: String::new(),
                detail: format!("set ok but verify read failed: {err}"),
                source_url: SOURCE_URL_NICE.into(),
                requires_elevation: true,
            },
        }
    }

    fn parse_pid(reason: &str) -> Result<u32, String> {
        let trimmed = reason.trim();
        if trimmed.is_empty() {
            return Ok(std::process::id());
        }
        if let Ok(pid) = trimmed.parse::<u32>() {
            return Ok(pid);
        }
        for part in trimmed.split(';') {
            if let Some(v) = part.trim().strip_prefix("pid=") {
                return v.parse::<u32>().map_err(|_| format!("invalid pid={v}"));
            }
        }
        Err(format!("expected pid or pid=N, got {trimmed}"))
    }

    fn parse_gamemode_arg(reason: &str) -> Result<(u32, String), String> {
        let trimmed = reason.trim();
        if trimmed.is_empty() {
            return Ok((std::process::id(), "register".into()));
        }
        let mut pid = None;
        let mut op = None;
        if let Ok(p) = trimmed.parse::<u32>() {
            return Ok((p, "register".into()));
        }
        for part in trimmed.split(';') {
            let part = part.trim();
            if let Some(v) = part.strip_prefix("pid=") {
                pid = Some(v.parse::<u32>().map_err(|_| format!("invalid pid={v}"))?);
            } else if let Some(v) = part.strip_prefix("op=") {
                op = Some(v.to_string());
            }
        }
        Ok((
            pid.unwrap_or_else(std::process::id),
            op.unwrap_or_else(|| "register".into()),
        ))
    }

    fn parse_nice_arg(reason: &str) -> Result<NiceIoniceState, String> {
        let trimmed = reason.trim();
        if trimmed.is_empty() {
            return Ok(NiceIoniceState {
                pid: std::process::id(),
                nice: DEFAULT_TARGET_NICE,
                ioprio_class: DEFAULT_TARGET_IOPRIO_CLASS,
                ioprio_level: DEFAULT_TARGET_IOPRIO_LEVEL,
            });
        }
        if let Ok(state) = NiceIoniceState::decode(trimmed) {
            return Ok(state);
        }
        // Allow pid-only -> defaults.
        let pid = parse_pid(trimmed)?;
        Ok(NiceIoniceState {
            pid,
            nice: DEFAULT_TARGET_NICE,
            ioprio_class: DEFAULT_TARGET_IOPRIO_CLASS,
            ioprio_level: DEFAULT_TARGET_IOPRIO_LEVEL,
        })
    }

    fn failed(setting: &str, source: &str, elev: bool, detail: impl Into<String>) -> SettingReport {
        SettingReport {
            os: "linux".into(),
            setting: setting.into(),
            status: "failed".into(),
            value: String::new(),
            detail: detail.into(),
            source_url: source.into(),
            requires_elevation: elev,
        }
    }
}

fn not_linux(setting: &str, source: &str, elev: bool) -> SettingReport {
    SettingReport {
        os: std::env::consts::OS.into(),
        setting: setting.into(),
        status: "not_possible".into(),
        value: String::new(),
        detail: "Linux-only setting".into(),
        source_url: source.into(),
        requires_elevation: elev,
    }
}

pub fn get_feral_gamemode(reason: Option<&str>) -> SettingReport {
    #[cfg(target_os = "linux")]
    {
        linux::get_feral_gamemode(reason.unwrap_or(""))
    }
    #[cfg(not(target_os = "linux"))]
    {
        let _ = reason;
        not_linux(
            "linux.feral_gamemode",
            "https://github.com/FeralInteractive/gamemode",
            false,
        )
    }
}

pub fn set_feral_gamemode(reason: Option<&str>) -> SettingReport {
    #[cfg(target_os = "linux")]
    {
        linux::set_feral_gamemode(reason.unwrap_or(""))
    }
    #[cfg(not(target_os = "linux"))]
    {
        let _ = reason;
        not_linux(
            "linux.feral_gamemode",
            "https://github.com/FeralInteractive/gamemode",
            false,
        )
    }
}

pub fn get_energy_profile() -> SettingReport {
    #[cfg(target_os = "linux")]
    {
        linux::get_energy_profile()
    }
    #[cfg(not(target_os = "linux"))]
    {
        not_linux(
            "linux.energy_profile",
            "https://upower.pages.freedesktop.org/power-profiles-daemon/gdbus-org.freedesktop.UPower.PowerProfiles.html",
            false,
        )
    }
}

pub fn set_energy_profile(reason: Option<&str>) -> SettingReport {
    #[cfg(target_os = "linux")]
    {
        linux::set_energy_profile(reason.unwrap_or(""))
    }
    #[cfg(not(target_os = "linux"))]
    {
        let _ = reason;
        not_linux(
            "linux.energy_profile",
            "https://upower.pages.freedesktop.org/power-profiles-daemon/gdbus-org.freedesktop.UPower.PowerProfiles.html",
            false,
        )
    }
}

pub fn get_nice_ionice(reason: Option<&str>) -> SettingReport {
    #[cfg(target_os = "linux")]
    {
        linux::get_nice_ionice(reason.unwrap_or(""))
    }
    #[cfg(not(target_os = "linux"))]
    {
        let _ = reason;
        not_linux(
            "linux.nice_ionice",
            "https://man7.org/linux/man-pages/man2/setpriority.2.html",
            false,
        )
    }
}

pub fn set_nice_ionice(reason: Option<&str>) -> SettingReport {
    #[cfg(target_os = "linux")]
    {
        linux::set_nice_ionice(reason.unwrap_or(""))
    }
    #[cfg(not(target_os = "linux"))]
    {
        let _ = reason;
        not_linux(
            "linux.nice_ionice",
            "https://man7.org/linux/man-pages/man2/setpriority.2.html",
            true,
        )
    }
}

/// Register Linux action handlers for headless restore-all (HZ-023).
pub fn register_linux_actions(registry: &mut hertzline_core::restore::MapActionRegistry) {
    #[cfg(target_os = "linux")]
    {
        use hertzline_core::actions::linux::{
            gamemode::os::OsFeralGameModeBackend, governor::os::OsEnergyProfileBackend,
            nice_ionice::os::OsNiceIoniceBackend, EnergyProfileAction, FeralGameModeAction,
            NiceIoniceAction,
        };
        registry.insert(Box::new(FeralGameModeAction::for_pid(
            OsFeralGameModeBackend,
            0,
        )));
        registry.insert(Box::new(EnergyProfileAction::performance(
            OsEnergyProfileBackend,
        )));
        registry.insert(Box::new(NiceIoniceAction::default_boost(
            OsNiceIoniceBackend,
            0,
        )));
    }
    #[cfg(not(target_os = "linux"))]
    {
        let _ = registry;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hz023_ops_shape() {
        let gm = get_feral_gamemode(None);
        assert_eq!(gm.setting, "linux.feral_gamemode");
        let energy = get_energy_profile();
        assert_eq!(energy.setting, "linux.energy_profile");
        let nice = get_nice_ionice(None);
        assert_eq!(nice.setting, "linux.nice_ionice");
        #[cfg(not(target_os = "linux"))]
        {
            assert_eq!(gm.status, "not_possible");
            assert_eq!(energy.status, "not_possible");
            assert_eq!(nice.status, "not_possible");
            let set_nice = set_nice_ionice(None);
            assert_eq!(set_nice.status, "not_possible");
            assert!(set_nice.requires_elevation);
        }
        #[cfg(target_os = "linux")]
        {
            assert!(gm.status == "ok" || gm.status == "not_possible" || gm.status == "failed");
            assert!(
                energy.status == "ok"
                    || energy.status == "not_possible"
                    || energy.status == "failed"
            );
            assert!(nice.status == "ok" || nice.status == "failed");
        }
    }
}
