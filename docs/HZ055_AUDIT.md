# HZ-055 helper / core / app security audit

Date: 2026-11-15. Method: code review + automated `scripts/check_hz055.py` +
existing unit/CI gates. Machine evidence recorded in PROGRESS.md.

## Checklist (ROADMAP checks)

| Check | Result | Evidence |
|---|---|---|
| No injection | PASS | Overlay `injects_into_game` always false; no CreateRemoteThread / WriteProcessMemory / ReadProcessMemory / VirtualAllocEx / process_vm_writev in helper/core/app `.rs`; games from documented paths only; MangoHud user wrap only |
| No full-app elevation | PASS | `ShellInfo.unprivileged == true`; app UI confirms per-action elevation; helper elevates only named ops (`SetHags`, `SetNiceIonice`, `RestoreAll`, `CreateRestorePoint`); no `requireAdministrator` in Tauri config |
| Allow-list helper | PASS | `HelperOp::ALL` exhaustive; unknown token exit 2; CI smoke `definitely_not_an_op`; SECURITY.md lists tokens |
| Telemetry off by default | PASS | No product telemetry SDK in core/helper Cargo.toml; app `ui/` has no `fetch`/beacon; CSP `default-src 'self'` |
| Forbidden-list compliance | PASS | No Defender disable, BCDEdit, timer-res, TCP "optimizer", or registry FPS-pack actions in catalog |
| AC-6.2 anti-cheat overlay default | PASS | Overlay off when AC markers found unless user opt-in; global disable (AC-6.3) |
| AC-7.x rollback | PASS | Snapshot before mutate; restore_all headless; uninstall hooks; System Restore offer |

## Helper allow-list (complete)

See `HelperOp::ALL` in `crates/hertzline-helper/src/protocol.rs`. Tokens:

ping, probe_elevate_path, restore_all, create_restore_point,
get_power_plan, set_power_plan, get_game_mode, set_game_mode, get_hags, set_hags,
get_process_priority, set_process_priority, close_background_app,
pause_background_app, resume_background_app, list_startup_apps, set_startup_app,
get_game_bar, set_game_bar, preview_cleanup, apply_cleanup,
get_storage_health, get_driver_info, get_display, set_display_refresh,
get_feral_gamemode, set_feral_gamemode, get_energy_profile, set_energy_profile,
get_nice_ionice, set_nice_ionice,
get_macos_game_mode, get_macos_lpm, macos_close_background_app,
macos_pause_background_app, macos_resume_background_app,
get_macos_display, set_macos_display_refresh

Elevation-required ops: restore_all, create_restore_point, set_hags, set_nice_ionice.

## Findings

No blocking findings. Residual risks are listed in `THREAT_MODEL.md` (T1--T8).

## Non-findings / explicit OK uses

| Pattern | Why OK |
|---|---|
| `OpenProcess` in core Windows priority/background | Documented Win32 for user-chosen PID; not inject |
| `LoadLibrary` SrClient.dll in helper | Official System Restore API path (AC-7.4) |
| `Command::new` for pmset / busctl / wine --version / ionice | Fixed argv, no shell string; report or set documented tools |
| Comment text mentioning "injection" | Negated / forbidden language in docs and tests |

## Automation

```bash
python scripts/check_hz055.py
cargo test --locked
```

CI runs the Python gate and existing allow-list smoke on windows/ubuntu/macos.
