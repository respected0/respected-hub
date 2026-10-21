"""Local acceptance gate for the Hub: cargo tests, TypeScript check and unit tests, guarded against real-environment drift.

    python release/tools/accept_hub.py        # from respected-hub; serial, a few minutes

The Rust end-to-end tests drive the sibling products on disposable roots; real_env_guard proves afterwards that no
real AI-client configuration, shortcut or scheduled task was touched. CI runs the same steps without the guard."""
import os, shutil, subprocess, sys
from pathlib import Path

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import real_env_guard  # noqa: E402
from ecosystem_root import ECO  # noqa: E402

hub = ECO / "respected-hub"
before = real_env_guard.snapshot()
env = dict(os.environ, CARGO_TARGET_DIR=str(ECO / ".cargo-target-hub"), RESPECTED_ECOSYSTEM_ROOT=str(ECO), PYTHONDONTWRITEBYTECODE="1")
npm = shutil.which("npm") or "npm"
steps = [(["cargo", "test", "--manifest-path", "src-tauri/Cargo.toml", "--no-fail-fast"], 3600),
         ([npm, "run", "check"], 600)]
code = 0
for command, limit in steps:
    done = subprocess.run(command, cwd=hub, env=env, capture_output=True, text=True, encoding="utf-8", errors="replace", timeout=limit)
    print("$", " ".join(command), "->", done.returncode)
    if done.returncode:
        print((done.stdout + done.stderr)[-4000:])
        code = done.returncode
drift = real_env_guard.diff(before, real_env_guard.snapshot())
print("REAL ENVIRONMENT DRIFT:" if drift else "real environment unchanged", *drift, sep="\n")
sys.exit(code or (1 if drift else 0))
