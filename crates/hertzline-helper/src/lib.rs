//! Hertzline privileged helper -- allow-listed broker only.
//!
//! No arbitrary shell. Elevation is requested per named operation.

pub mod elevate;
pub mod protocol;
pub mod restore;
pub mod system_restore;
pub mod windows_settings;

pub const CRATE_VERSION: &str = env!("CARGO_PKG_VERSION");

use protocol::{HelperError, HelperOp, HelperRequest, HelperResponse};

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
    }
}

fn setting_response(detail: &str, report: &windows_settings::SettingReport) -> HelperResponse {
    let mut response = HelperResponse::ok_json(detail, report);
    if report.status == "failed" || report.status == "unavailable" {
        response.accepted = false;
    } else if report.status == "not_possible" {
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
}
