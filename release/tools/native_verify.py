"""Native Hub smoke driver: builds frontend + release exe from a throwaway copy, then runs the Hub's tools/native_smoke.py.

Always writes --report (build/smoke failures are recorded, not raised); exits 0 unless this script itself crashes.
Run with cwd = the respected-hub source directory."""
import argparse, os, shutil, subprocess, sys, tempfile
from pathlib import Path

from ecosystem_root import ECO
parser = argparse.ArgumentParser()
parser.add_argument("--report", required=True)
report = Path(parser.parse_args().report)
report.parent.mkdir(parents=True, exist_ok=True)
lines = []
work = Path(tempfile.mkdtemp(prefix="hubnative-"))
hub = work / "hub"
env = dict(os.environ, CARGO_TARGET_DIR=str(ECO / ".cargo-target-hub"), PYTHONDONTWRITEBYTECODE="1",
           PATH=os.environ["PATH"] + os.pathsep + str(Path.home() / ".cargo" / "bin"))


def step(name, cmd, cwd, limit):
    done = subprocess.run(cmd, cwd=cwd, env=env, capture_output=True, text=True, timeout=limit)
    lines.append(f"- {name}: exit {done.returncode}")
    if done.returncode:
        lines.append("```\n" + (done.stdout + done.stderr)[-2500:] + "\n```")
    return done.returncode == 0


try:
    shutil.copytree(Path.cwd(), hub, ignore=shutil.ignore_patterns("node_modules", "target", ".git", "gen", "dist"))
    subprocess.run(["cmd", "/c", "mklink", "/J", str(hub / "node_modules"), str(ECO / "respected-hub" / "node_modules")], check=True, capture_output=True)
    ok = step("frontend build", ["cmd", "/c", "npm", "run", "build"], hub, 600)
    ok = ok and step("release build", [str(Path.home() / ".cargo" / "bin" / "cargo.exe"), "build", "--release", "--manifest-path", "src-tauri/Cargo.toml"], hub, 3000)
    exe = next(iter((ECO / ".cargo-target-hub" / "release").glob("respected*hub*.exe")), None) if ok else None
    smoke = hub / "tools" / "native_smoke.py"
    if ok and exe and smoke.is_file():
        step("native smoke", [sys.executable, "-B", str(smoke), "--exe", str(exe), "--ecosystem", str(ECO), "--seconds", "20", "--report", str(work / "smoke.md")], hub, 600)
        if (work / "smoke.md").is_file():
            lines.append((work / "smoke.md").read_text(encoding="utf8")[-4000:])
    elif ok:
        lines.append("- native smoke: NOT RUN (exe or tools/native_smoke.py missing)")
finally:
    subprocess.run(["cmd", "/c", "rmdir", str(hub / "node_modules")], capture_output=True)
    shutil.rmtree(work, ignore_errors=True)
    report.write_text("# Native Hub smoke (unsigned local build)\n" + "\n".join(lines) + "\n", encoding="utf8")
