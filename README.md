# Hertzline / helper

Minimal per-OS privileged broker for [Hertzline](https://github.com/Hertzline).

Named operations only. No arbitrary shell. The desktop app never runs fully elevated.

## Layout

| Path | Role |
|---|---|
| `crates/hertzline-helper` | Allow-list broker library + `hertzline-helper` binary |
| `policy/linux/` | Draft polkit action file for packaging |
| `spikes/D2.md` | HZ-005 privileged helper decision evidence |

## Build

```bash
cargo test --locked
cargo build --release --locked
```

## CLI

```bash
hertzline-helper ping
hertzline-helper probe_elevate_path --json
hertzline-helper restore_all --reason uninstall --json
```

Unknown ops exit with code 2 and never prompt for elevation.

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
