"""REL-001: install an OLD published Hub installer, then the NEW one over it, the way a user updates.

    python common/release/tools/acceptance/accept_update.py --old OLD.exe --old-version 0.1.0-beta.1 \
        --new NEW.exe --new-version 0.1.0-beta.2 --new-exe STAGED/respected-hub.exe

Checks: the old installer installs; the new one installs silently over it (the Hub's own update path); one uninstall entry,
the new version, the new executable byte for byte, the user's data untouched; then the uninstaller removes it all.
Real install into HKCU/%LOCALAPPDATA%: refuses to start when a Hub is already installed (run it on a runner or a clean VM).
"""
from __future__ import annotations

import argparse
import hashlib
import os
import subprocess
import sys
import time
import winreg
from pathlib import Path

KEY = r"Software\Microsoft\Windows\CurrentVersion\Uninstall\{5C0F6A1E-3B7D-4E2A-9D55-7A1C2F0E8B31}_is1"
SILENT = ["/VERYSILENT", "/SUPPRESSMSGBOXES", "/NORESTART", "/NOCANCEL"]
DATA = Path(os.environ["LOCALAPPDATA"]) / "Respected" / "RespectedHub"


def entry(name: str) -> str | None:
    try:
        with winreg.OpenKey(winreg.HKEY_CURRENT_USER, KEY) as key:
            return winreg.QueryValueEx(key, name)[0]
    except OSError:
        return None


def sha(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def run(command: list[str]) -> None:
    done = subprocess.run(command, timeout=600)
    if done.returncode:
        sys.exit(f"FAIL exit {done.returncode}: {' '.join(command)}")


def main() -> None:
    p = argparse.ArgumentParser()
    p.add_argument("--old", type=Path, required=True)
    p.add_argument("--old-version", required=True)
    p.add_argument("--new", type=Path, required=True)
    p.add_argument("--new-version", required=True)
    p.add_argument("--new-exe", type=Path, required=True, help="the executable the new installer carries")
    a = p.parse_args()

    if entry("DisplayVersion") is not None:
        sys.exit("REFUSED: a Hub is already installed here; this test installs and removes a real one")

    checks: list[str] = []

    def check(ok: bool, what: str) -> None:
        print(("PASS " if ok else "FAIL ") + what)
        if not ok:
            sys.exit(1)
        checks.append(what)

    try:
        run([str(a.old), *SILENT])
        check(entry("DisplayVersion") == a.old_version, f"old installer installs v{a.old_version}")
        app = Path(entry("InstallLocation").rstrip("\\"))
        sentinel = DATA / "update-sentinel.txt"
        sentinel.parent.mkdir(parents=True, exist_ok=True)
        sentinel.write_text("user data", encoding="utf-8")

        run([str(a.new), *SILENT, "/RELAUNCH=0"])  # same switches the Hub's own self-update uses
        check(entry("DisplayVersion") == a.new_version, f"new installer updates to v{a.new_version}")
        check(sha(app / "respected-hub.exe") == sha(a.new_exe), "installed executable is the new one, byte for byte")
        check(sentinel.read_text(encoding="utf-8") == "user data", "user data untouched by the update")
        check(Path(entry("InstallLocation").rstrip("\\")) == app, "same install folder")
    finally:
        uninstall = entry("UninstallString")
        if uninstall:
            run([uninstall.strip('"'), *SILENT])
    check(entry("DisplayVersion") is None, "uninstaller removes the entry")
    for _ in range(30):  # Inno's uninstaller finishes in a temp copy after the started process exits
        if not DATA.exists():
            break
        time.sleep(1)
    left = [str(f) for f in DATA.rglob("*")] if DATA.exists() else []
    check(not DATA.exists(), f"uninstaller removes the Hub's own data (left: {left})")
    print(f"ALL PASSED ({len(checks)} checks)")


if __name__ == "__main__":
    main()
