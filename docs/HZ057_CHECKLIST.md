# HZ-057 -- Helper integration checklist

Companion to `core/docs/HZ057_CHECKLIST.md`.

| OS | Check | Notes |
|---|---|---|
| Windows / Linux / macOS | `cargo test -p hertzline-helper --test hz057_integration --locked` | ping, elevate probe, headless restore_all, unknown-op reject |
| All | Allow-list only | No arbitrary shell |

Sign-off is recorded when CI `helper-test` passes on windows/ubuntu/macos and
local `scripts/check_hz057.py` is green.
