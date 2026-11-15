# Hertzline threat model (HZ-055)

Scope: `helper`, `core`, and `app` as shipped for v1. Website and packaging
signing are out of this document except where they touch privilege or claims.

Date: 2026-11-15. Status: recorded for HZ-055.

## Assets

| Asset | Location | Why it matters |
|---|---|---|
| User OS settings (power, HAGS, Game Mode, display, startup) | Host OS | Wrong apply leaves the machine worse; must be reversible |
| Snapshot / change log store | User data dir (`hertzline-core` paths) | Rollback integrity; crash/uninstall restore |
| Privileged helper binary | Installed path | Elevation surface; compromise = named OS mutation |
| Game detection paths | Documented launcher files only | Must not become scrape/inject |
| Measurement sessions | Local only | Privacy; no exfiltration |
| Overlay decision state | Local settings | Anti-cheat safety (AC-6.2) |

## Actors

| Actor | Intent |
|---|---|
| Benign user | Measure and apply reversible tweaks |
| Local malware / other local user | Abuse helper elevation or coerce restore |
| Compromised game / anti-cheat | Detect inject/hooks; ban risk if we inject |
| Supply-chain (deps / CI) | Introduce telemetry, shell exec, or inject APIs |
| Remote attacker | No default network surface in v1 app/core/helper |

## Trust boundaries

```
[unprivileged app UI]
        | named IPC / CLI args only
        v
[core (in-process)] ---- snapshots before mutate ---->
        | HelperOp token + reason
        v
[helper allow-list] ---- per-op elevate (runas/pkexec/SMAppService) ---->
        v
[OS APIs]
```

Rules (ARCHITECTURE + RULES.md):

1. App never runs fully elevated.
2. Elevation is per named helper op with a human-readable reason.
3. Helper rejects unknown tokens before elevation (exit 2).
4. No arbitrary shell / generic "run command".
5. No injection, memory reading of games, or game-file rewrite.
6. Telemetry off by default (v1: no product telemetry SDK).

## Threats and controls

| ID | Threat | Control | Residual risk |
|---|---|---|---|
| T1 | Helper accepts arbitrary command | Exhaustive `HelperOp` enum; `parse_op_token` reject; CI smoke unknown-op exit 2 | New op added without review -- code review + HZ-055 check |
| T2 | Whole-app elevation | App `ShellInfo.unprivileged = true`; no `requireAdministrator` manifest; elevate only via helper | User manually "Run as admin" -- document; do not require it |
| T3 | Game/process injection (anti-cheat ban) | Overlay methods non-injection only; `injects_into_game` hard false; PresentMon ETW / MangoHud user wrap / macOS partial; games detect from files only | Vendor policy can still change; AC markers force overlay off by default |
| T4 | Silent destructive OS change | Snapshot before apply; restore_all on crash/uninstall; System Restore offer (Win); cleanup needs confirm | User confirms destructive cleanup; irreversible by nature |
| T5 | Telemetry / data exfil | No reqwest/ureq/sentry product deps in core/helper; app UI has no fetch to remote analytics; CSP default-src self | Tauri/Astro toolchain may ship unused transitive packages -- gate on direct deps + UI |
| T6 | Privilege escalation via helper args | Args are structured (`reason` string); OS backends validate PID/GUID/profile; no shell interpolation of reason into `cmd`/`sh -c` | Bugs in a single setter -- unit tests per action |
| T7 | Rollback failure after crash | Headless `restore_all`; active profile recovery (HZ-018/028) | Missing snapshot if process killed mid-write -- documented residual |
| T8 | Forbidden placebo / harmful tweaks | Product forbid list; actions are explicit AC-4 catalog only | Future feature pressure -- policy freeze HZ-058 |

## Out of scope (v1)

- Network-facing services, cloud sync, accounts
- Kernel drivers authored by Hertzline
- Store / winget submission integrity (owner-gated; HZ-054)
- Physical attacker with disk access to snapshot store

## References

- `ARCHITECTURE.md` privilege boundaries and D2
- `RULES.md` forbidden list
- `docs/HZ055_AUDIT.md` evidence checklist
- Intel PresentMon (MIT) -- ETW, no game DLL inject (see `core` D3)
- MangoHud (MIT) -- user-opted wrap only
