"""Guard for test runs: fails when a run changed the user's REAL AI-client configuration or left test sessions behind.

    python tools/real_env_guard.py snapshot STATE.json     # before a run
    python tools/real_env_guard.py verify   STATE.json     # after: exit 1 and a list when anything changed
    python tools/real_env_guard.py home                    # prints a fresh sandbox home for the run's USERPROFILE/HOME/APPDATA

What is compared: whole-file hashes of the AI clients' user-level rule/hook/MCP files, the MCP server table of ~/.claude.json
(Claude rewrites the rest of that file all the time), the names of ~/.claude/skills, and Claude session folders that belong to test
paths. Nothing is ever written to or removed from the real environment.
"""
from __future__ import annotations

import hashlib
import json
import os
import re
import sys
import tempfile
from pathlib import Path

HOME = Path.home()
APPDATA = Path(os.environ.get("APPDATA", HOME / "AppData/Roaming"))
WATCHED = [HOME / ".claude/settings.json", HOME / ".claude/CLAUDE.md", HOME / ".codex/config.toml", HOME / ".codex/hooks.json", HOME / ".codex/AGENTS.md",
           HOME / ".gemini/settings.json", HOME / ".gemini/GEMINI.md", HOME / ".gemini/config/hooks.json", HOME / ".gemini/config/mcp_config.json",
           HOME / ".cursor/hooks.json", HOME / ".cursor/mcp.json", HOME / ".codeium/windsurf/mcp_config.json", APPDATA / "Claude/claude_desktop_config.json"]
TEST_SESSION_DIR = re.compile(r"(cc-real|standalone|build-cache|hub-native|hub-installed|idx-fresh|rpp-|respected-maf)", re.I)


def _hash(path: Path) -> str | None:
    try:
        return hashlib.sha256(path.read_bytes()).hexdigest()
    except OSError:
        return None


def snapshot() -> dict:
    state = {"files": {str(p): _hash(p) for p in WATCHED}}
    try:
        servers = json.loads((HOME / ".claude.json").read_text(encoding="utf-8")).get("mcpServers", {})
        state["claude_mcp_servers"] = hashlib.sha256(json.dumps(servers, sort_keys=True).encode("utf-8")).hexdigest()
    except (OSError, ValueError):
        state["claude_mcp_servers"] = None
    skills = HOME / ".claude/skills"
    state["claude_skills"] = sorted(p.name for p in skills.iterdir()) if skills.is_dir() else []
    projects = HOME / ".claude/projects"
    state["claude_test_sessions"] = sorted(p.name for p in projects.iterdir() if TEST_SESSION_DIR.search(p.name)) if projects.is_dir() else []
    return state


def diff(before: dict, after: dict) -> list[str]:
    problems = [f"changed: {path}" for path, digest in before["files"].items() if after["files"].get(path) != digest]
    if before["claude_mcp_servers"] != after["claude_mcp_servers"]:
        problems.append("changed: ~/.claude.json mcpServers")
    for key, label in (("claude_skills", "~/.claude/skills"), ("claude_test_sessions", "Claude session folders of test paths")):
        added, removed = set(after[key]) - set(before[key]), set(before[key]) - set(after[key])
        if added or removed:
            problems.append(f"{label}: added {sorted(added)} removed {sorted(removed)}")
    return problems


def sandbox_home() -> Path:
    """A fresh empty home for a test run, so nothing under the real profile can be reached by accident."""
    home = Path(tempfile.mkdtemp(prefix="respected-test-home-"))
    for sub in ("AppData/Roaming", "AppData/Local"):
        (home / sub).mkdir(parents=True)
    return home


def sandbox_env(home: Path) -> dict[str, str]:
    return {"USERPROFILE": str(home), "HOME": str(home), "APPDATA": str(home / "AppData/Roaming"), "LOCALAPPDATA": str(home / "AppData/Local")}


if __name__ == "__main__":
    command = sys.argv[1] if len(sys.argv) > 1 else ""
    if command == "snapshot":
        Path(sys.argv[2]).write_text(json.dumps(snapshot()), encoding="utf-8")
    elif command == "verify":
        problems = diff(json.loads(Path(sys.argv[2]).read_text(encoding="utf-8")), snapshot())
        print("\n".join(problems) if problems else "real environment unchanged")
        sys.exit(1 if problems else 0)
    elif command == "home":
        print(sandbox_home())
    else:
        sys.exit("usage: real_env_guard.py snapshot|verify STATE.json | home")
