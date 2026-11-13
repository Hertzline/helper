//! Linux elevate path: polkit action allow-list + pkexec of this helper.
//!
//! Official sources:
//! - polkit overview / actions: https://www.freedesktop.org/software/polkit/docs/latest/polkit.8.html
//! - pkexec: https://www.freedesktop.org/software/polkit/docs/latest/pkexec.1.html
//!
//! Policy draft lives at policy/linux/org.hertzline.helper.policy.
//! Flatpak: sandbox cannot own system polkit actions; document host helper
//! or portal limits in packaging (HZ-008 / HZ-050).

use super::ElevatePlan;

pub fn plan() -> ElevatePlan {
    ElevatePlan {
        os: "linux".into(),
        mechanism: "polkit_pkexec_helper".into(),
        api: "polkit action + pkexec PROGRAM argv".into(),
        source_url: "https://www.freedesktop.org/software/polkit/docs/latest/pkexec.1.html"
            .into(),
        always_on_service: false,
        notes: vec![
            "Declare named actions under org.hertzline.helper.* (no generic org.freedesktop.policykit.exec).".into(),
            "Annotate org.freedesktop.policykit.exec.path to the installed helper path; argv1 selects subcommand.".into(),
            "pkexec does not validate ARGUMENTS; helper must re-check allow-list after elevation.".into(),
            "Flatpak: privileged ops need a host-installed helper or documented not-possible; see packaging.".into(),
        ],
    }
}
