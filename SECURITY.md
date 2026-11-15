# Security Policy

Report vulnerabilities privately as described in the organization
[SECURITY.md](https://github.com/Hertzline/.github/blob/main/SECURITY.md).

Do not open public issues for undisclosed vulnerabilities.

## Helper surface

This repository owns the privileged broker. It must accept only a fixed
allow-list of named operations. No arbitrary shell, no generic "run command".

Threat model: [docs/THREAT_MODEL.md](docs/THREAT_MODEL.md)  
HZ-055 audit: [docs/HZ055_AUDIT.md](docs/HZ055_AUDIT.md)

Allow-list is `HelperOp::ALL` in `crates/hertzline-helper/src/protocol.rs`.
Unknown tokens are rejected (CLI exit 2) before any elevation prompt.

Elevation-required ops only: `restore_all`, `create_restore_point`, `set_hags`,
`set_nice_ionice`. All other ops run without elevation or are read-only probes.

### Allow-list tokens

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

## Product rules (enforced with core/app)

- No injection into games; overlay methods are non-injection only.
- Desktop app never runs fully elevated; elevation is per action via this helper.
- Telemetry off by default (no product telemetry SDK in this broker).
