//! Hertzline privileged helper -- allow-listed broker only.
//!
//! No arbitrary shell. Elevation is requested per named operation.

pub mod elevate;
pub mod linux_settings;
pub mod macos_settings;
pub mod protocol;
pub mod restore;
pub mod system_restore;
pub mod windows_settings;

pub const CRATE_VERSION: &str = env!("CARGO_PKG_VERSION");

use protocol::{HelperError, HelperOp, HelperRequest, HelperResponse};
use serde::Serialize;

/// Dispatch a named request. Unknown ops never reach OS elevation.
pub fn dispatch(request: &HelperRequest) -> HelperResponse {
    match request.op {
        HelperOp::Ping => HelperResponse::ok("helper_pong"),
        HelperOp::ProbeElevatePath => {
            let plan = elevate::plan_for_host();
            HelperResponse::ok_json("elevate_path", &plan)
        }
        HelperOp::RestoreAll => {
            let report = restore::restore_all(request.reason.as_deref().unwrap_or("explicit"));
            HelperResponse::ok_json("restore_all", &report)
        }
        HelperOp::CreateRestorePoint => {
            let report = system_restore::create_restore_point(request.reason.as_deref());
            let mut response = HelperResponse::ok_json("create_restore_point", &report);
            if report.status == "failed" || report.status == "unavailable" {
                response.accepted = false;
            }
            response
        }
        HelperOp::GetPowerPlan => {
            let report = windows_settings::get_power_plan();
            setting_response("get_power_plan", &report)
        }
        HelperOp::SetPowerPlan => {
            let report = windows_settings::set_power_plan(request.reason.as_deref());
            setting_response("set_power_plan", &report)
        }
        HelperOp::GetGameMode => {
            let report = windows_settings::get_game_mode();
            setting_response("get_game_mode", &report)
        }
        HelperOp::SetGameMode => {
            // Default when reason omitted: enable (action target).
            let enabled = match request.reason.as_deref().map(str::trim) {
                None | Some("1") | Some("on") | Some("true") => true,
                Some("0") | Some("off") | Some("false") => false,
                Some(other) => {
                    return HelperResponse::rejected(format!(
                        "set_game_mode reason must be 0 or 1, got {other}"
                    ));
                }
            };
            let report = windows_settings::set_game_mode(enabled);
            setting_response("set_game_mode", &report)
        }
        HelperOp::GetHags => {
            let report = windows_settings::get_hags();
            setting_response("get_hags", &report)
        }
        HelperOp::SetHags => {
            let report = windows_settings::set_hags(request.reason.as_deref());
            setting_response("set_hags", &report)
        }
        HelperOp::GetProcessPriority => {
            let report = windows_settings::get_process_priority(request.reason.as_deref());
            setting_response("get_process_priority", &report)
        }
        HelperOp::SetProcessPriority => {
            let report = windows_settings::set_process_priority(request.reason.as_deref());
            setting_response("set_process_priority", &report)
        }
        HelperOp::CloseBackgroundApp => {
            let report = windows_settings::close_background_app(request.reason.as_deref());
            setting_response("close_background_app", &report)
        }
        HelperOp::PauseBackgroundApp => {
            let report = windows_settings::pause_background_app(request.reason.as_deref());
            setting_response("pause_background_app", &report)
        }
        HelperOp::ResumeBackgroundApp => {
            let report = windows_settings::resume_background_app(request.reason.as_deref());
            setting_response("resume_background_app", &report)
        }
        HelperOp::ListStartupApps => {
            let report = windows_settings::list_startup_apps();
            setting_response("list_startup_apps", &report)
        }
        HelperOp::SetStartupApp => {
            let report = windows_settings::set_startup_app(request.reason.as_deref());
            setting_response("set_startup_app", &report)
        }
        HelperOp::GetGameBar => {
            let report = windows_settings::get_game_bar();
            setting_response("get_game_bar", &report)
        }
        HelperOp::SetGameBar => {
            let enabled = match request.reason.as_deref().map(str::trim) {
                None | Some("0") | Some("off") | Some("false") => false,
                Some("1") | Some("on") | Some("true") => true,
                Some(other) => {
                    return HelperResponse::rejected(format!(
                        "set_game_bar reason must be 0 or 1, got {other}"
                    ));
                }
            };
            let report = windows_settings::set_game_bar(enabled);
            setting_response("set_game_bar", &report)
        }
        HelperOp::PreviewCleanup => {
            let report = windows_settings::preview_cleanup();
            setting_response("preview_cleanup", &report)
        }
        HelperOp::ApplyCleanup => {
            let report = windows_settings::apply_cleanup(request.reason.as_deref());
            setting_response("apply_cleanup", &report)
        }
        HelperOp::GetStorageHealth => {
            let report = windows_settings::get_storage_health();
            setting_response("get_storage_health", &report)
        }
        HelperOp::GetDriverInfo => {
            let report = windows_settings::get_driver_info();
            setting_response("get_driver_info", &report)
        }
        HelperOp::GetDisplay => {
            let report = windows_settings::get_display();
            setting_response("get_display", &report)
        }
        HelperOp::SetDisplayRefresh => {
            let report = windows_settings::set_display_refresh(request.reason.as_deref());
            setting_response("set_display_refresh", &report)
        }
        HelperOp::GetFeralGameMode => {
            let report = linux_settings::get_feral_gamemode(request.reason.as_deref());
            linux_setting_response("get_feral_gamemode", &report)
        }
        HelperOp::SetFeralGameMode => {
            let report = linux_settings::set_feral_gamemode(request.reason.as_deref());
            linux_setting_response("set_feral_gamemode", &report)
        }
        HelperOp::GetEnergyProfile => {
            let report = linux_settings::get_energy_profile();
            linux_setting_response("get_energy_profile", &report)
        }
        HelperOp::SetEnergyProfile => {
            let report = linux_settings::set_energy_profile(request.reason.as_deref());
            linux_setting_response("set_energy_profile", &report)
        }
        HelperOp::GetNiceIonice => {
            let report = linux_settings::get_nice_ionice(request.reason.as_deref());
            linux_setting_response("get_nice_ionice", &report)
        }
        HelperOp::SetNiceIonice => {
            let report = linux_settings::set_nice_ionice(request.reason.as_deref());
            linux_setting_response("set_nice_ionice", &report)
        }
        HelperOp::GetMacOsGameMode => {
            let report = macos_settings::get_macos_game_mode();
            macos_setting_response("get_macos_game_mode", &report)
        }
        HelperOp::GetMacOsLpm => {
            let report = macos_settings::get_macos_lpm(request.reason.as_deref());
            macos_setting_response("get_macos_lpm", &report)
        }
        HelperOp::MacOsCloseBackgroundApp => {
            let report = macos_settings::macos_close_background_app(request.reason.as_deref());
            macos_setting_response("macos_close_background_app", &report)
        }
        HelperOp::MacOsPauseBackgroundApp => {
            let report = macos_settings::macos_pause_background_app(request.reason.as_deref());
            macos_setting_response("macos_pause_background_app", &report)
        }
        HelperOp::MacOsResumeBackgroundApp => {
            let report = macos_settings::macos_resume_background_app(request.reason.as_deref());
            macos_setting_response("macos_resume_background_app", &report)
        }
        HelperOp::GetMacOsDisplay => {
            let report = macos_settings::get_macos_display();
            macos_setting_response("get_macos_display", &report)
        }
        HelperOp::SetMacOsDisplayRefresh => {
            let report = macos_settings::set_macos_display_refresh(request.reason.as_deref());
            macos_setting_response("set_macos_display_refresh", &report)
        }
    }
}

fn setting_response(detail: &str, report: &windows_settings::SettingReport) -> HelperResponse {
    finish_setting_response(detail, &report.status, report)
}

fn linux_setting_response(detail: &str, report: &linux_settings::SettingReport) -> HelperResponse {
    finish_setting_response(detail, &report.status, report)
}

fn macos_setting_response(detail: &str, report: &macos_settings::SettingReport) -> HelperResponse {
    finish_setting_response(detail, &report.status, report)
}

fn finish_setting_response(detail: &str, status: &str, report: &impl Serialize) -> HelperResponse {
    let mut response = HelperResponse::ok_json(detail, report);
    if status == "failed" || status == "unavailable" {
        response.accepted = false;
    } else if status == "not_possible" {
        // Get probes of unsupported settings still succeed; set/apply paths reject.
        response.accepted = detail.starts_with("get_");
    }
    response
}

/// Parse a CLI op token into the allow-list. Anything else is rejected.
pub fn parse_op_token(token: &str) -> Result<HelperOp, HelperError> {
    HelperOp::from_token(token).ok_or_else(|| HelperError::Rejected {
        detail: format!("op not on allow-list: {token}"),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use protocol::HelperOp;

    #[test]
    fn allow_list_ping() {
        let res = dispatch(&HelperRequest {
            op: HelperOp::Ping,
            reason: None,
        });
        assert!(res.accepted);
        assert_eq!(res.detail, "helper_pong");
    }

    #[test]
    fn reject_unknown_token() {
        let err = parse_op_token("rm_rf_root").expect_err("must reject");
        match err {
            HelperError::Rejected { detail } => {
                assert!(detail.contains("allow-list"));
            }
        }
    }

    #[test]
    fn probe_returns_host_plan() {
        let res = dispatch(&HelperRequest {
            op: HelperOp::ProbeElevatePath,
            reason: Some("d2 spike".into()),
        });
        assert!(res.accepted);
        assert!(res.detail.contains("elevate_path") || res.payload.is_some());
        let plan = elevate::plan_for_host();
        assert!(!plan.api.is_empty());
        assert!(!plan.mechanism.is_empty());
    }

    #[test]
    fn restore_all_reports_trigger_headless() {
        let res = dispatch(&HelperRequest {
            op: HelperOp::RestoreAll,
            reason: Some("crash".into()),
        });
        assert!(res.accepted);
        let report = restore::restore_all("crash");
        assert_eq!(report.trigger, "crash");
        assert!(!report.requires_ui);
        assert!(report.status == "ok" || report.status == "partial" || report.status == "error");
    }

    #[test]
    fn create_restore_point_is_on_allow_list() {
        assert_eq!(
            parse_op_token("create_restore_point").unwrap(),
            HelperOp::CreateRestorePoint
        );
        assert!(HelperOp::CreateRestorePoint.requires_elevation());
        let report = system_restore::create_restore_point(Some("Hertzline test restore point"));
        assert!(!report.description.is_empty());
        assert!(!report.source_url.is_empty());
        let res = dispatch(&HelperRequest {
            op: HelperOp::CreateRestorePoint,
            reason: Some("Hertzline test restore point".into()),
        });
        assert!(res.payload.is_some());
        assert_eq!(res.detail, "create_restore_point");
    }

    #[test]
    fn hz020_ops_on_allow_list() {
        for token in [
            "get_power_plan",
            "set_power_plan",
            "get_game_mode",
            "set_game_mode",
            "get_hags",
            "set_hags",
        ] {
            assert!(parse_op_token(token).is_ok(), "{token}");
        }
        let power = dispatch(&HelperRequest {
            op: HelperOp::GetPowerPlan,
            reason: None,
        });
        assert!(power.payload.is_some());
        let game = dispatch(&HelperRequest {
            op: HelperOp::GetGameMode,
            reason: None,
        });
        assert!(game.payload.is_some());
        let hags = dispatch(&HelperRequest {
            op: HelperOp::GetHags,
            reason: None,
        });
        assert!(hags.payload.is_some());
        assert!(HelperOp::SetHags.requires_elevation());
    }

    #[test]
    fn hz021_ops_on_allow_list() {
        for token in [
            "get_process_priority",
            "set_process_priority",
            "close_background_app",
            "pause_background_app",
            "resume_background_app",
            "list_startup_apps",
            "set_startup_app",
        ] {
            assert!(parse_op_token(token).is_ok(), "{token}");
        }
        let list = dispatch(&HelperRequest {
            op: HelperOp::ListStartupApps,
            reason: None,
        });
        assert!(list.payload.is_some());
        let pri = dispatch(&HelperRequest {
            op: HelperOp::GetProcessPriority,
            reason: Some(std::process::id().to_string()),
        });
        assert!(pri.payload.is_some());
    }

    #[test]
    fn hz022_ops_on_allow_list() {
        for token in [
            "get_game_bar",
            "set_game_bar",
            "preview_cleanup",
            "apply_cleanup",
            "get_storage_health",
            "get_driver_info",
            "get_display",
            "set_display_refresh",
        ] {
            assert!(parse_op_token(token).is_ok(), "{token}");
        }
        let bar = dispatch(&HelperRequest {
            op: HelperOp::GetGameBar,
            reason: None,
        });
        assert!(bar.payload.is_some());
        let preview = dispatch(&HelperRequest {
            op: HelperOp::PreviewCleanup,
            reason: None,
        });
        assert!(preview.payload.is_some());
        let storage = dispatch(&HelperRequest {
            op: HelperOp::GetStorageHealth,
            reason: None,
        });
        assert!(storage.payload.is_some());
        let drivers = dispatch(&HelperRequest {
            op: HelperOp::GetDriverInfo,
            reason: None,
        });
        assert!(drivers.payload.is_some());
        let display = dispatch(&HelperRequest {
            op: HelperOp::GetDisplay,
            reason: None,
        });
        assert!(display.payload.is_some());
    }

    #[test]
    fn hz023_ops_on_allow_list() {
        for token in [
            "get_feral_gamemode",
            "set_feral_gamemode",
            "get_energy_profile",
            "set_energy_profile",
            "get_nice_ionice",
            "set_nice_ionice",
        ] {
            assert!(parse_op_token(token).is_ok(), "{token}");
        }
        assert!(HelperOp::SetNiceIonice.requires_elevation());
        let gm = dispatch(&HelperRequest {
            op: HelperOp::GetFeralGameMode,
            reason: Some(std::process::id().to_string()),
        });
        assert!(gm.payload.is_some());
        let energy = dispatch(&HelperRequest {
            op: HelperOp::GetEnergyProfile,
            reason: None,
        });
        assert!(energy.payload.is_some());
        let nice = dispatch(&HelperRequest {
            op: HelperOp::GetNiceIonice,
            reason: Some(std::process::id().to_string()),
        });
        assert!(nice.payload.is_some());
    }

    #[test]
    fn hz026_ops_on_allow_list() {
        for token in [
            "get_macos_game_mode",
            "get_macos_lpm",
            "macos_close_background_app",
            "macos_pause_background_app",
            "macos_resume_background_app",
            "get_macos_display",
            "set_macos_display_refresh",
        ] {
            assert!(parse_op_token(token).is_ok(), "{token}");
        }
        let gm = dispatch(&HelperRequest {
            op: HelperOp::GetMacOsGameMode,
            reason: None,
        });
        assert!(gm.payload.is_some());
        let lpm = dispatch(&HelperRequest {
            op: HelperOp::GetMacOsLpm,
            reason: Some("performance=1".into()),
        });
        assert!(lpm.payload.is_some());
        let display = dispatch(&HelperRequest {
            op: HelperOp::GetMacOsDisplay,
            reason: None,
        });
        assert!(display.payload.is_some());
        #[cfg(not(target_os = "macos"))]
        {
            assert!(gm.accepted);
            assert!(lpm.accepted);
            assert!(display.accepted);
            let set = dispatch(&HelperRequest {
                op: HelperOp::SetMacOsDisplayRefresh,
                reason: Some("60".into()),
            });
            assert!(!set.accepted);
        }
    }

    #[test]
    fn hz055_allow_list_exhaustive_and_rejects_shell_ops() {
        assert!(HelperOp::ALL.len() >= 30);
        for op in HelperOp::ALL {
            assert_eq!(HelperOp::from_token(op.as_token()), Some(*op));
        }
        for bad in [
            "run_shell",
            "exec",
            "cmd",
            "powershell",
            "bash",
            "bcdedit",
            "disable_defender",
            "inject",
            "write_process_memory",
            "rm_rf_root",
        ] {
            assert!(parse_op_token(bad).is_err(), "{bad}");
        }
        let elev: Vec<_> = HelperOp::ALL
            .iter()
            .copied()
            .filter(|op| op.requires_elevation())
            .collect();
        assert_eq!(
            elev,
            vec![
                HelperOp::RestoreAll,
                HelperOp::CreateRestorePoint,
                HelperOp::SetHags,
                HelperOp::SetNiceIonice,
            ]
        );
    }
}
