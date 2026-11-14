//! Windows elevate path: short-lived elevated process via ShellExecuteEx runas.
//!
//! Official sources:
//! - ShellExecuteExW: https://learn.microsoft.com/en-us/windows/win32/api/shellapi/nf-shellapi-shellexecuteexw
//! - runas verb: https://learn.microsoft.com/en-us/windows/win32/shell/launch
//!
//! Rejected for v1: always-on Windows service (larger attack surface, no need
//! proven by this spike).

use super::ElevatePlan;

pub fn plan() -> ElevatePlan {
    ElevatePlan {
        os: "windows".into(),
        mechanism: "short_lived_elevated_process".into(),
        api: "ShellExecuteExW + lpVerb=runas".into(),
        source_url: "https://learn.microsoft.com/en-us/windows/win32/shell/launch".into(),
        always_on_service: false,
        notes: vec![
            "Desktop app stays unelevated; only hertzline-helper.exe is relaunched elevated."
                .into(),
            "UAC consent shows the helper path and the human-readable reason for the op.".into(),
            "Prefer batching privileged restores into one elevated helper invocation.".into(),
            "Windows service rejected for v1: always-on privilege without spike justification."
                .into(),
        ],
    }
}
