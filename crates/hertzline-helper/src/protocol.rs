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
}

impl HelperOp {
    pub const ALL: &'static [HelperOp] = &[
        HelperOp::Ping,
        HelperOp::ProbeElevatePath,
        HelperOp::RestoreAll,
    ];

    pub fn as_token(self) -> &'static str {
        match self {
            HelperOp::Ping => "ping",
            HelperOp::ProbeElevatePath => "probe_elevate_path",
            HelperOp::RestoreAll => "restore_all",
        }
    }

    pub fn from_token(token: &str) -> Option<Self> {
        match token {
            "ping" => Some(HelperOp::Ping),
            "probe_elevate_path" => Some(HelperOp::ProbeElevatePath),
            "restore_all" => Some(HelperOp::RestoreAll),
            _ => None,
        }
    }

    /// Human-readable reason shown in elevation prompts (AC-8.1).
    pub fn elevation_reason(self) -> &'static str {
        match self {
            HelperOp::Ping => "Hertzline helper connectivity check (no elevation)",
            HelperOp::ProbeElevatePath => "Hertzline helper elevation path probe (no elevation)",
            HelperOp::RestoreAll => "Hertzline restore all previous system changes",
        }
    }

    pub fn requires_elevation(self) -> bool {
        matches!(self, HelperOp::RestoreAll)
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
}
