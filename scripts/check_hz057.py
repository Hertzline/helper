#!/usr/bin/env python3
"""HZ-057 local check: helper integration suite gates."""

from __future__ import annotations

import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
TEST = ROOT / "crates" / "hertzline-helper" / "tests" / "hz057_integration.rs"
CHECKLIST = ROOT / "docs" / "HZ057_CHECKLIST.md"
CI = ROOT / ".github" / "workflows" / "ci.yml"

REQUIRED = [
    "ac7_allow_list_ping_probe_restore_empty",
    "ac7_unknown_op_token_rejected",
    "HelperOp::Ping",
    "RestoreAll",
    "definitely_not_an_op",
]


def main() -> int:
    for path in (TEST, CHECKLIST, CI):
        if not path.is_file():
            print("FAIL missing", path.relative_to(ROOT))
            return 1

    text = TEST.read_text(encoding="utf-8")
    missing = [m for m in REQUIRED if m not in text]
    if missing:
        print("FAIL hz057_integration.rs missing:", ", ".join(missing))
        return 1

    ci = CI.read_text(encoding="utf-8")
    for marker in ("check_hz057.py", "hz057_integration"):
        if marker not in ci:
            print("FAIL ci.yml missing", marker)
            return 1

    print("check_hz057: PASS")
    return 0


if __name__ == "__main__":
    sys.exit(main())
