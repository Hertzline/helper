<div align="center">

<a href="https://github.com/Hertzline/helper"><img src="https://capsule-render.vercel.app/api?type=waving&height=180&color=0:0f1412,55:12352c,100:0f6b4c&text=helper&fontColor=e8eeeb&fontSize=64&fontAlignY=45&animation=twinkling" alt="Hertzline helper" width="100%" /></a>

<img src="https://readme-typing-svg.demolab.com?font=IBM+Plex+Mono&weight=500&size=16&color=2A9B7A&center=true&vCenter=true&width=560&height=34&duration=2800&pause=1200&lines=Minimal+privileged+broker;Named+operations+only;No+arbitrary+shell;App+never+runs+fully+elevated" alt="Hertzline helper" />

<p>
  <a href="https://github.com/Hertzline"><img src="https://skillicons.dev/icons?i=github&theme=dark" width="44" height="44" alt="GitHub" /></a>&nbsp;
  <a href="https://www.rust-lang.org"><img src="https://skillicons.dev/icons?i=rust&theme=dark" width="44" height="44" alt="Rust" /></a>&nbsp;
  <a href="https://www.kernel.org"><img src="https://skillicons.dev/icons?i=linux&theme=dark" width="44" height="44" alt="Linux" /></a>
</p>

</div>

<br/>

<img src="https://capsule-render.vercel.app/api?type=rect&height=3&color=0:0f1412,35:2a9b7a,70:c9a227,100:0f1412" width="100%" alt="" />

<br/>

<p align="center"><img src="https://readme-typing-svg.demolab.com?font=IBM+Plex+Mono&weight=700&size=18&color=2A9B7A&center=true&vCenter=true&width=420&height=40&duration=1&pause=1000000&letterSpacing=6px&lines=WHAT+THIS+REPO+IS" alt="What this repo is" /></p>

Minimal per-OS privileged broker for [Hertzline](https://github.com/Hertzline). Named operations only. No arbitrary shell. The desktop app never runs fully elevated.

Path-depends on sibling `../core` (`Hertzline/core`).

<img src="https://capsule-render.vercel.app/api?type=rect&height=3&color=0:0f1412,35:2a9b7a,70:c9a227,100:0f1412" width="100%" alt="" />

<br/>

<p align="center"><img src="https://readme-typing-svg.demolab.com?font=IBM+Plex+Mono&weight=700&size=18&color=2A9B7A&center=true&vCenter=true&width=420&height=40&duration=1&pause=1000000&letterSpacing=6px&lines=LAYOUT" alt="Layout" /></p>

| Path | Role |
|---|---|
| `crates/hertzline-helper` | Allow-list broker + `hertzline-helper` binary |
| `docs/THREAT_MODEL.md` | HZ-055 threat model |
| `docs/HZ055_AUDIT.md` | HZ-055 audit evidence |
| `scripts/check_hz055.py` | Security audit gate |
| `policy/linux/` | Draft polkit policy |
| `spikes/D2.md` | Privilege model decision |

<img src="https://capsule-render.vercel.app/api?type=rect&height=3&color=0:0f1412,35:2a9b7a,70:c9a227,100:0f1412" width="100%" alt="" />

<br/>

<p align="center"><img src="https://readme-typing-svg.demolab.com?font=IBM+Plex+Mono&weight=700&size=18&color=2A9B7A&center=true&vCenter=true&width=420&height=40&duration=1&pause=1000000&letterSpacing=6px&lines=BUILD+%2F+CLI" alt="Build / CLI" /></p>

```bash
python scripts/check_hz055.py
cargo test --locked
cargo build --release --locked
```

```bash
hertzline-helper ping
hertzline-helper probe_elevate_path --json
hertzline-helper restore_all --reason uninstall --json
```

Unknown ops exit code 2 and never prompt for elevation.

<img src="https://capsule-render.vercel.app/api?type=rect&height=3&color=0:0f1412,35:2a9b7a,70:c9a227,100:0f1412" width="100%" alt="" />

<br/>

<p align="center"><img src="https://readme-typing-svg.demolab.com?font=IBM+Plex+Mono&weight=700&size=18&color=2A9B7A&center=true&vCenter=true&width=420&height=40&duration=1&pause=1000000&letterSpacing=6px&lines=PRIVILEGE+MODEL" alt="Privilege model" /></p>

| OS | Mechanism |
|---|---|
| Windows | Short-lived elevated process (`ShellExecuteEx` `runas`) |
| Linux | polkit named actions + `pkexec` |
| macOS | `SMAppService` LaunchDaemon (register on first need) |

See `spikes/D2.md`.

<p align="center"><sub><i>Clean measurements are like a well-refereed match: when everything works, nobody invents the score.</i></sub></p>

## License

[MIT](LICENSE) · [NOTICE](NOTICE)

<img src="https://capsule-render.vercel.app/api?type=waving&section=footer&height=110&color=0:0f1412,55:12352c,100:0f6b4c" alt="" width="100%" />
