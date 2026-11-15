#!/usr/bin/env python3
"""HZ-055 local check: threat model + helper/core/app security audit gates."""

from __future__ import annotations

import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
WORKSPACE = ROOT.parent
CORE = WORKSPACE / "core"
APP = WORKSPACE / "app"

THREAT = ROOT / "docs" / "THREAT_MODEL.md"
AUDIT = ROOT / "docs" / "HZ055_AUDIT.md"
SECURITY = ROOT / "SECURITY.md"
PROTOCOL = ROOT / "crates" / "hertzline-helper" / "src" / "protocol.rs"

REQUIRED_DOC_MARKERS = {
    THREAT: [
        "Assets",
        "Actors",
        "Trust boundaries",
        "Threats and controls",
        "T1",
        "T3",
        "T5",
        "allow-list",
        "injection",
        "telemetry",
        "elevation",
    ],
    AUDIT: [
        "No injection",
        "No full-app elevation",
        "Allow-list helper",
        "Telemetry off by default",
        "PASS",
        "HelperOp::ALL",
        "AC-6.2",
        "AC-7",
    ],
    SECURITY: [
        "allow-list",
        "HelperOp",
        "THREAT_MODEL.md",
        "HZ055_AUDIT.md",
        "no arbitrary shell",
    ],
}

# Injection / memory-write APIs that must not appear as code identifiers.
FORBIDDEN_API = re.compile(
    r"\b("
    r"CreateRemoteThread|WriteProcessMemory|ReadProcessMemory|VirtualAllocEx|"
    r"NtWriteVirtualMemory|NtReadVirtualMemory|process_vm_writev|process_vm_readv|"
    r"SetWindowsHookEx[AW]?|QueueUserAPC"
    r")\b"
)

# Product telemetry SDKs that must not be direct dependencies.
FORBIDDEN_DEP = re.compile(
    r"(?im)^\s*(sentry|sentry-tracing|opentelemetry|posthog|mixpanel|"
    r"amplitude|segment|reqwest|ureq|isahc)\s*="
)

FORBIDDEN_TOKENS = [
    "run_shell",
    "exec",
    "cmd",
    "powershell",
    "bash",
    "bcdedit",
    "disable_defender",
    "inject",
    "write_process_memory",
]


def _read(path: Path) -> str:
    return path.read_text(encoding="utf-8")


def _rs_files(root: Path) -> list[Path]:
    if not root.is_dir():
        return []
    out: list[Path] = []
    for path in root.rglob("*.rs"):
        if "target" in path.parts:
            continue
        out.append(path)
    return out


def _scan_forbidden_apis(roots: list[Path]) -> list[str]:
    hits: list[str] = []
    for root in roots:
        for path in _rs_files(root):
            text = _read(path)
            for i, line in enumerate(text.splitlines(), 1):
                stripped = line.lstrip()
                if stripped.startswith("//") or stripped.startswith("///"):
                    continue
                if FORBIDDEN_API.search(line):
                    hits.append(f"{path}:{i}:{line.strip()}")
    return hits


def _scan_cargo_deps(roots: list[Path]) -> list[str]:
    hits: list[str] = []
    for root in roots:
        for path in root.rglob("Cargo.toml"):
            if "target" in path.parts:
                continue
            text = _read(path)
            for i, line in enumerate(text.splitlines(), 1):
                if FORBIDDEN_DEP.search(line):
                    hits.append(f"{path}:{i}:{line.strip()}")
    return hits


def _extract_helper_tokens(protocol: str) -> set[str]:
    tokens = set(re.findall(r'"([a-z0-9_]+)"\s*=>\s*Some\(HelperOp::', protocol))
    return tokens


def main() -> int:
    missing = [str(p.relative_to(ROOT)) for p in (THREAT, AUDIT, SECURITY, PROTOCOL) if not p.is_file()]
    if missing:
        print("FAIL missing files:", ", ".join(missing))
        return 1

    for path, markers in REQUIRED_DOC_MARKERS.items():
        text = _read(path)
        lower = text.lower()
        absent = [m for m in markers if m.lower() not in lower]
        if absent:
            print(f"FAIL {path.relative_to(ROOT)} missing markers:", ", ".join(absent))
            return 1

    protocol = _read(PROTOCOL)
    if "pub const ALL" not in protocol and "pub const ALL:" not in protocol:
        print("FAIL protocol.rs missing HelperOp::ALL")
        return 1

    tokens = _extract_helper_tokens(protocol)
    if len(tokens) < 30:
        print(f"FAIL expected >=30 allow-list tokens, got {len(tokens)}")
        return 1

    for bad in FORBIDDEN_TOKENS:
        if bad in tokens:
            print(f"FAIL forbidden token on allow-list: {bad}")
            return 1
        # ensure from_token rejects via absence
        if re.search(rf'"{re.escape(bad)}"\s*=>\s*Some\(HelperOp::', protocol):
            print(f"FAIL forbidden token mapped in from_token: {bad}")
            return 1

    elev_ops = {"restore_all", "create_restore_point", "set_hags", "set_nice_ionice"}
    if not elev_ops.issubset(tokens):
        print("FAIL missing elevation ops:", ", ".join(sorted(elev_ops - tokens)))
        return 1

    roots = [ROOT]
    if CORE.is_dir():
        roots.append(CORE)
    if APP.is_dir():
        roots.append(APP)

    api_hits = _scan_forbidden_apis(roots)
    if api_hits:
        print("FAIL forbidden inject/memory APIs:")
        for hit in api_hits[:20]:
            print(" ", hit)
        return 1

    dep_hits = _scan_cargo_deps(roots)
    if dep_hits:
        print("FAIL forbidden telemetry/network direct deps:")
        for hit in dep_hits[:20]:
            print(" ", hit)
        return 1

    if APP.is_dir():
        shell = APP / "crates" / "hertzline-app" / "src" / "shell.rs"
        if shell.is_file() and "unprivileged: true" not in _read(shell):
            print("FAIL app shell.rs must set unprivileged: true")
            return 1
        ui = APP / "ui"
        if ui.is_dir():
            for path in list(ui.rglob("*.js")) + list(ui.rglob("*.html")):
                text = _read(path)
                if re.search(r"\bfetch\s*\(|XMLHttpRequest|sendBeacon", text):
                    print(f"FAIL remote call pattern in {path.relative_to(APP)}")
                    return 1
        tauri = APP / "src-tauri" / "tauri.conf.json"
        if tauri.is_file():
            conf = _read(tauri)
            if "requireAdministrator" in conf or "highestAvailable" in conf:
                print("FAIL tauri.conf.json must not request elevated execution level")
                return 1

    if CORE.is_dir():
        method = CORE / "crates" / "hertzline-core" / "src" / "overlay" / "method.rs"
        if method.is_file():
            text = _read(method)
            if "injects_into_game: false" not in text:
                print("FAIL overlay method must hard-code injects_into_game: false")
                return 1

    print(
        "PASS hz055 threat_model+audit "
        f"tokens={len(tokens)} roots={[str(r.name) for r in roots]}"
    )
    return 0


if __name__ == "__main__":
    sys.exit(main())
