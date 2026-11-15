//! HZ-057 helper integration smoke on the host OS.
//!
//! Allow-list broker only: ping, elevate-path probe, headless restore_all.
//! No arbitrary shell. Unknown ops must exit the protocol reject path.

use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

use hertzline_core::paths::SNAPSHOT_STORE_ENV;
use hertzline_helper::protocol::{HelperOp, HelperRequest};
use hertzline_helper::{dispatch, parse_op_token};

fn temp_dir(label: &str) -> PathBuf {
    static N: AtomicU64 = AtomicU64::new(0);
    let n = N.fetch_add(1, Ordering::Relaxed);
    let dir = std::env::temp_dir().join(format!(
        "hertzline-helper-hz057-{label}-{}-{n}",
        std::process::id()
    ));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).expect("temp dir");
    dir
}

#[test]
fn hz057_host_banner() {
    println!(
        "hz057_helper_host os={} arch={} crate={}",
        std::env::consts::OS,
        std::env::consts::ARCH,
        hertzline_helper::CRATE_VERSION
    );
}

#[test]
fn ac7_allow_list_ping_probe_restore_empty() {
    let ping = dispatch(&HelperRequest {
        op: HelperOp::Ping,
        reason: None,
    });
    assert!(ping.accepted);
    assert_eq!(ping.detail, "helper_pong");

    let probe = dispatch(&HelperRequest {
        op: HelperOp::ProbeElevatePath,
        reason: None,
    });
    assert!(probe.accepted);
    assert!(
        probe.detail.contains("elevate_path") || probe.payload.is_some(),
        "probe detail={} payload={:?}",
        probe.detail,
        probe.payload
    );

    let dir = temp_dir("restore");
    let store = dir.join("snapshots.json");
    std::env::set_var(SNAPSHOT_STORE_ENV, store.as_os_str());
    let restore = dispatch(&HelperRequest {
        op: HelperOp::RestoreAll,
        reason: Some("uninstall".into()),
    });
    assert!(restore.accepted);
    let payload = restore.payload.expect("restore payload");
    let text = payload.to_string();
    assert!(text.contains("requires_ui"));
    assert!(text.contains("\"ok\"") || text.contains("ok"));
    std::env::remove_var(SNAPSHOT_STORE_ENV);
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn ac7_unknown_op_token_rejected() {
    let err = parse_op_token("definitely_not_an_op").expect_err("must reject");
    let detail = format!("{err:?}");
    assert!(detail.contains("allow-list") || detail.contains("Rejected"));
    assert!(!HelperOp::ALL.is_empty());
}
