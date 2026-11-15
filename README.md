# Hertzline / helper

Minimal per-OS privileged broker for [Hertzline](https://github.com/Hertzline).

Named operations only. No arbitrary shell. The desktop app never runs fully elevated.

## Layout

| Path | Role |
|---|---|
| `crates/hertzline-helper` | Allow-list broker library + `hertzline-helper` binary |
| `docs/THREAT_MODEL.md` | HZ-055 threat model (helper/core/app) |
| `docs/HZ055_AUDIT.md` | HZ-055 audit evidence |
| `scripts/check_hz055.py` | Automated security audit gate |
| `policy/linux/` | Draft polkit action file for packaging |
| `spikes/D2.md` | HZ-005 privileged helper decision evidence |

## Build

```bash
python scripts/check_hz055.py
cargo test --locked
cargo build --release --locked
```

## CLI

```bash
hertzline-helper ping
hertzline-helper probe_elevate_path --json
hertzline-helper restore_all --reason uninstall --json
hertzline-helper restore_all --reason crash --store /path/to/snapshots.json --json
```

`restore_all` loads the Hertzline snapshot store (`HERTZLINE_SNAPSHOT_STORE` or the OS user-data default from `hertzline-core`) and runs headless restore. Unknown ops exit with code 2 and never prompt for elevation.

**Build note:** this crate path-depends on sibling `../core` (`Hertzline/core`). CI checks out that repo next to `helper`.

## Privilege model

| OS | Mechanism |
|---|---|
| Windows | Short-lived elevated process (`ShellExecuteEx` `runas`) |
| Linux | polkit named actions + `pkexec` of this helper |
| macOS | `SMAppService` LaunchDaemon (register on first need) |

See `spikes/D2.md` and the organization `ARCHITECTURE.md` decision D2.

## License

[MIT](LICENSE)

Third-party notices: [NOTICE](NOTICE)
