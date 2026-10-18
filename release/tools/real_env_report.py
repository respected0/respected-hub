"""Read-only report of leftovers that Respected development tests may have left in the REAL user environment.

    python tools/real_env_report.py            # prints a report, changes nothing

Looks at AI-client configuration, AI-client session folders, scheduled tasks, shortcuts, uninstall entries, app folders and temp
directories. It never writes or deletes; cleaning is a separate, authorised step.
"""
from __future__ import annotations

import json
import os
import re
import subprocess
import sys
from pathlib import Path

HOME = Path.home()
TEST_PATH = re.compile(r"(\\ov\\|/ov/|hub-native|hub-installed|cc-real|\.build-cache|\.campaign-data|idx-fresh|rpp-[a-z]+-|respected-maf-smoke|hubaccept|local\\temp\\tmp|respected-activation)", re.I)
RESPECTED = re.compile(r"respected|respectedbrain", re.I)


def size(path: Path) -> int:
    total = 0
    for item in path.rglob("*") if path.is_dir() else [path]:
        try:
            total += item.stat().st_size if item.is_file() else 0
        except OSError:
            pass
    return total


def mb(value: int) -> str:
    return f"{value / 1e6:.1f} MB"


def section(title: str) -> None:
    print(f"\n## {title}")


def claude_json() -> None:
    section("~/.claude.json (read only)")
    path = HOME / ".claude.json"
    if not path.is_file():
        print("absent")
        return
    document = json.loads(path.read_text(encoding="utf-8"))
    servers = document.get("mcpServers", {})
    print("user-scope MCP servers:", sorted(servers) or "none")
    bad = [name for name, entry in servers.items() if TEST_PATH.search(json.dumps(entry))]
    print("MCP entries pointing into test/temp locations:", bad or "none")
    projects = [p for p in document.get("projects", {}) if TEST_PATH.search(p)]
    print(f"project records for test/temp folders: {len(projects)}")
    for p in projects[:12]:
        print("  ", p)


def claude_dirs() -> None:
    section("~/.claude/projects (session transcripts of test folders)")
    root = HOME / ".claude" / "projects"
    if not root.is_dir():
        print("absent")
        return
    hits = [d for d in root.iterdir() if d.is_dir() and re.search(r"(cc-real|standalone|build-cache|hub-native|\-ov-|Temp-tmp|idx-fresh|rpp-|respected-maf)", d.name, re.I)]
    print(f"{len(hits)} folders, {mb(sum(size(d) for d in hits))}")
    for d in sorted(hits)[:15]:
        print(f"   {d.name}  ({len(list(d.glob('*.jsonl')))} sessions, {mb(size(d))})")
    settings = HOME / ".claude" / "settings.json"
    if settings.is_file():
        text = settings.read_text(encoding="utf-8", errors="replace")
        print("~/.claude/settings.json mentions respectedbrain hooks:", bool(re.search(r"respectedbrain|--global-hook", text)), "| test paths:", bool(TEST_PATH.search(text)))


def codex() -> None:
    section("Codex (~/.codex)")
    config = HOME / ".codex" / "config.toml"
    if config.is_file():
        text = config.read_text(encoding="utf-8", errors="replace")
        trusted = re.findall(r"^\[projects\.(?:'|\")(.+?)(?:'|\")\]", text, re.M)
        hits = [t for t in trusted if TEST_PATH.search(t)]
        print(f"trusted-project entries for test/temp folders: {len(hits)} of {len(trusted)}")
        for t in hits[:12]:
            print("  ", t)
        print("respected MCP/notify entries:", bool(re.search(r"respected", text, re.I)))
    sessions = HOME / ".codex" / "sessions"
    count = 0
    if sessions.is_dir():
        for f in sessions.rglob("rollout-*.jsonl"):
            try:
                with f.open("r", encoding="utf-8", errors="replace") as handle:
                    if TEST_PATH.search(handle.read(4000)):
                        count += 1
            except OSError:
                pass
    print("session files whose working folder is a test/temp folder:", count)


def gemini_cursor() -> None:
    section("Gemini / Antigravity / Cursor")
    for rel in (".gemini", ".cursor", ".codeium"):
        base = HOME / rel
        if not base.is_dir():
            print(rel, "absent")
            continue
        hits = []
        for f in list(base.rglob("*.json"))[:4000] + list(base.rglob("*.toml"))[:500] + list(base.rglob("*.md"))[:500]:
            try:
                if f.stat().st_size < 400_000 and TEST_PATH.search(f.read_text(encoding="utf-8", errors="replace")):
                    hits.append(f)
            except OSError:
                pass
        print(rel, "files mentioning test/temp paths:", len(hits), [str(h.relative_to(HOME)) for h in hits[:5]])


def system() -> None:
    section("Scheduled tasks, shortcuts, uninstall entries")
    out = subprocess.run(["schtasks", "/query", "/fo", "csv", "/nh"], capture_output=True, text=True, encoding="utf-8", errors="replace").stdout
    print("scheduled tasks named like Respected:", sorted({line.split(",")[0].strip('"') for line in out.splitlines() if RESPECTED.search(line)}) or "none")
    desktop = HOME / "Desktop"
    print("desktop shortcuts:", [p.name for p in desktop.glob("*") if RESPECTED.search(p.name)] if desktop.is_dir() else "no desktop folder")
    reg = subprocess.run(["reg", "query", r"HKCU\Software\Microsoft\Windows\CurrentVersion\Uninstall", "/s", "/f", "Respected", "/d"], capture_output=True, text=True, errors="replace").stdout
    print("uninstall entries:", [l.strip() for l in reg.splitlines() if "DisplayName" in l] or "none")


def folders() -> None:
    section("App/data folders and temp leftovers")
    for env in ("LOCALAPPDATA", "APPDATA"):
        base = Path(os.environ.get(env, ""))
        if base.is_dir():
            print(env, [f"{p.name} ({mb(size(p))})" for p in base.iterdir() if RESPECTED.search(p.name) or "respected" in p.name.lower()] or "none")
    temp = Path(os.environ.get("TEMP", ""))
    left = [p for p in temp.iterdir() if re.match(r"(rpp-|idx-fresh|hub-native|hub-installed|hubaccept|respected|cc-iso)", p.name, re.I)] if temp.is_dir() else []
    print(f"TEMP leftovers: {len(left)} folders, {mb(sum(size(p) for p in left))}")
    groups: dict[str, list[Path]] = {}
    for p in left:
        groups.setdefault(re.sub(r"[-_ ][A-Za-z0-9_]{5,9}$", "", p.name)[:28], []).append(p)
    for name, items in sorted(groups.items(), key=lambda kv: -sum(size(i) for i in kv[1]))[:8]:
        print(f"   {name}*: {len(items)} folders, {mb(sum(size(i) for i in items))}")
    ov = Path(r"C:\ov")
    print(r"C:\ov working roots (orchestrator mirrors/data):", mb(size(ov)) if ov.is_dir() else "absent")


if __name__ == "__main__":
    print("Real-environment leftover report (read only)")
    for step in (claude_json, claude_dirs, codex, gemini_cursor, system, folders):
        try:
            step()
        except Exception as error:  # a broken probe must not hide the others
            print(f"[{step.__name__}] probe failed: {type(error).__name__}: {error}")
    sys.exit(0)
