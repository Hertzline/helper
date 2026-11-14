//! Hertzline privileged helper -- allow-listed broker only.
//!
//! No arbitrary shell. Elevation is requested per named operation.

pub mod elevate;
pub mod protocol;
pub mod restore;
pub mod system_restore;

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
    }
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
}
