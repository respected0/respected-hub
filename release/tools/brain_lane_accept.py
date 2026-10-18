"""Brain acceptance for lane worktrees: full discovery, but only failures OUTSIDE the recorded mirror baseline count.

A lane runs in an isolated mirror (no .git/.venv/build/dist/.local) where environment-dependent tests fail even on
untouched source (tools/brain_mirror_baseline.txt: one exact test id per line, subTest suffix included). Phase `verify`
still runs the unmodified full suite on the real source (tools/run_brain_regression.py), which must be fully green.

`src` is exported through PYTHONPATH so tests that launch `python -m respectedbrain` as a subprocess can import the package."""
import os, re, shutil, subprocess, sys, unittest
from pathlib import Path

root = Path.cwd().resolve()
src_dir = root / "src"
build_dir = root / "build"
src = str(src_dir)
sys.path.insert(0, src)
os.environ["PYTHONPATH"] = src + (os.pathsep + os.environ["PYTHONPATH"] if os.environ.get("PYTHONPATH") else "")
known = set(Path(__file__).with_name("brain_mirror_baseline.txt").read_text(encoding="utf8").splitlines())
os.environ["PYTHONDONTWRITEBYTECODE"] = "1"  # subprocess tests must not leave __pycache__ outside the lane scope
initial_env = os.environ.copy()


def snapshot_build_artifacts():
    files = {f: f.read_bytes() for d in src_dir.glob("*.egg-info") for f in d.rglob("*") if f.is_file()}
    paths = set(src_dir.glob("*.egg-info"))
    for directory in paths:
        paths.update(directory.rglob("*"))
    return files, paths


def remove_path(path):
    if path.is_symlink() or path.is_file():
        path.unlink()
    else:
        shutil.rmtree(path, ignore_errors=True)


def restore_build_artifacts(files, paths, had_build):
    if not had_build:
        remove_path(build_dir)
    for directory in src_dir.glob("*.egg-info"):
        if directory not in paths:
            remove_path(directory)
            continue
        for path in sorted(directory.rglob("*"), reverse=True):
            if path in paths:
                continue
            if path.is_symlink() or path.is_file():
                path.unlink()
            elif path.is_dir():
                path.rmdir()
    for path, data in files.items():
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_bytes(data)


def run_single_test(name):
    try:
        suite = unittest.defaultTestLoader.loadTestsFromName(re.sub(r" \(.*\)$", "", name))
    except Exception:
        return 1
    with open(os.devnull, "w") as sink:
        result = unittest.TextTestRunner(stream=sink, verbosity=0).run(suite)
    return 0 if result.testsRun and result.wasSuccessful() and not result.skipped else 1


def passes_alone(name):
    # shortcut: a child process keeps full-suite global state and test mutations out of the serial rerun
    command = [sys.executable, "-B", str(Path(__file__).resolve()), "--passes-alone", name]
    try:
        completed = subprocess.run(command, cwd=root, env=initial_env, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL, check=False)
    except OSError:
        return False
    return completed.returncode == 0


# tests/package_contract_test.py builds a wheel in the checkout (build/, src/*.egg-info): the lane scope check would
# count those side effects as the worker's changes, so put the tree back exactly as it was after the run.
egg_files, egg_paths = snapshot_build_artifacts()
had_build = build_dir.exists()
try:
    if len(sys.argv) == 3 and sys.argv[1] == "--passes-alone":
        sys.exit(run_single_test(sys.argv[2]))
    suite = unittest.defaultTestLoader.discover("tests", pattern="*test*.py", top_level_dir=".")
    result = unittest.TextTestRunner(verbosity=0).run(suite)
finally:
    restore_build_artifacts(egg_files, egg_paths, had_build)
failing = sorted({test.id() for test, _ in result.failures + result.errors})
bad = [name for name in failing if name not in known]
discovery_import_errors = [
    test.id()
    for test, _ in result.errors
    if test.__class__.__module__ == "unittest.loader"
    and test.__class__.__qualname__ == "_FailedTest"
]
bad.extend(name for name in discovery_import_errors if name not in bad)
if result.testsRun < 900:
    bad.append(f"expected at least 900 tests, ran {result.testsRun}")
flaky = [
    name
    for name in failing
    if name not in known and name not in discovery_import_errors and passes_alone(name)
]
bad = [name for name in bad if name not in flaky]
print(f"ran={result.testsRun} failures={len(result.failures)} errors={len(result.errors)} new_failures={len(bad)}")
for name in failing:
    print("FAILING:", name)
for name in flaky:
    print("FLAKY (passed on serial rerun):", name)
for name in bad:
    print("NEW FAILURE:", name)
sys.exit(1 if bad else 0)
