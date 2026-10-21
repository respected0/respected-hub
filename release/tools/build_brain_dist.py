"""Build (or reuse) the frozen SecondBrain distribution used by the Hub's engine-lifecycle tests.

    python tools/build_brain_dist.py [--force]   ->   <ecosystem>/.build-cache/brain-dist/RespectedBrain

The distribution is built from the current SecondBrain source with its own build tool (PyInstaller);
it is unattested and local. The cache is rebuilt when any file under respected-secondbrain/src is newer.
"""
import argparse, subprocess, sys
from pathlib import Path

from ecosystem_root import ECO
CACHE = ECO / ".build-cache" / "brain-dist"
DIST = CACHE / "RespectedBrain"


def stale() -> bool:
    if not (DIST / "respectedbrain.exe").is_file():
        return True
    built = (DIST / "respectedbrain.exe").stat().st_mtime
    return any(p.stat().st_mtime > built for p in (ECO / "respected-secondbrain" / "src").rglob("*") if p.is_file() and "__pycache__" not in p.parts)


if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument("--force", action="store_true")
    if parser.parse_args().force or stale():
        done = subprocess.run([sys.executable, str(ECO / "respected-secondbrain" / "tools" / "build_installer.py"), "--platform", "windows", "--output", str(CACHE)], cwd=ECO / "respected-secondbrain")
        if done.returncode:
            sys.exit(done.returncode)
    print(DIST)
