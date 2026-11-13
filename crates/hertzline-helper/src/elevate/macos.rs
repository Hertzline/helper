//! macOS elevate path: SMAppService LaunchDaemon (macOS 13+) with tiny surface.
//!
//! Official sources:
//! - SMAppService: https://developer.apple.com/documentation/servicemanagement/smappservice
//! - Updating helpers: https://developer.apple.com/documentation/servicemanagement/updating-helper-executables-from-earlier-versions-of-macos
//!
//! macOS exposes few tunables (ARCHITECTURE matrix). Prefer keeping the daemon
//! unregistered until the first privileged action; unregister on uninstall.
//! Always-on without need is avoided; SMAppService still bootstraps after admin
//! approval on subsequent boots once registered -- surface stays allow-listed.

use super::ElevatePlan;

pub fn plan() -> ElevatePlan {
    ElevatePlan {
        os: "macos".into(),
        mechanism: "smappservice_launch_daemon".into(),
        api: "SMAppService.daemon(plistName:) + register()/unregister()".into(),
        source_url: "https://developer.apple.com/documentation/servicemanagement/smappservice"
            .into(),
        always_on_service: true,
        notes: vec![
            "macOS 13+: place helper under Contents/Resources and plist under Contents/Library/LaunchDaemons.".into(),
            "Use BundleProgram relative to the app bundle; admin approval required in System Settings.".into(),
            "Register only when a privileged op is first required; unregister on uninstall.".into(),
            "always_on_service=true after approval is an OS constraint; keep allow-list minimal.".into(),
            "Fewer v1 tunables than Windows/Linux; many actions remain not-possible without elevation.".into(),
        ],
    }
}
