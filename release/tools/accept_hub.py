"""Acceptance gate for the Hub: builds/tests a throwaway copy so build outputs never pollute the worker workspace."""
import os, shutil, subprocess, sys, tempfile

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import real_env_guard  # noqa: E402

_before = real_env_guard.snapshot()
from pathlib import Path

from ecosystem_root import ECO
src = Path.cwd()
work = Path(tempfile.mkdtemp(prefix="hubaccept-"))
shutil.copytree(src, work / "hub", ignore=shutil.ignore_patterns("node_modules", "target", ".git", "gen", "dist"))
hub = work / "hub"
subprocess.run(["cmd", "/c", "mklink", "/J", str(hub / "node_modules"), str(ECO / "respected-hub" / "node_modules")], check=True, capture_output=True)
env = dict(os.environ, CARGO_TARGET_DIR=str(ECO / ".cargo-target-hub"), RESPECTED_ECOSYSTEM_ROOT=str(ECO),
           PATH=os.environ["PATH"] + os.pathsep + str(Path.home() / ".cargo" / "bin"), PYTHONDONTWRITEBYTECODE="1")
steps = [([str(Path.home() / ".cargo" / "bin" / "cargo.exe"), "test", "--manifest-path", "src-tauri/Cargo.toml", "--lib", "--tests"], 3000),
         ([str(hub / "node_modules" / ".bin" / "tsc.cmd"), "--noEmit"], 300),
         (["node", "--test", *map(str, sorted((hub / "tests").glob("*.test.ts")))], 300)]
code = 0
for cmd, limit in steps:
    done = subprocess.run(cmd, cwd=hub, env=env, capture_output=True, text=True, timeout=limit)
    print("$", " ".join(cmd[:3]), "->", done.returncode)
    if done.returncode:
        print((done.stdout + done.stderr)[-3000:]); code = done.returncode; break
subprocess.run(["cmd", "/c", "rmdir", str(hub / "node_modules")], capture_output=True)
shutil.rmtree(work, ignore_errors=True)
_drift = real_env_guard.diff(_before, real_env_guard.snapshot())
print("REAL ENVIRONMENT DRIFT:" if _drift else "real environment unchanged", *_drift, sep=chr(10))
sys.exit(code or (1 if _drift else 0))
