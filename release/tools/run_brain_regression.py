import os, re, shutil, subprocess, sys, unittest

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import real_env_guard  # noqa: E402

# The whole suite runs with an empty sandbox profile (USERPROFILE/HOME/APPDATA), so no test can reach the real AI-client
# configuration by accident; the guard additionally proves the real files are unchanged afterwards.
_before = real_env_guard.snapshot()
_inno = os.path.join(os.environ.get("LOCALAPPDATA", ""), "Programs", "Inno Setup 6")  # installer tests find ISCC.exe through the real profile
if os.path.isdir(_inno):
    os.environ["PATH"] = os.environ["PATH"] + os.pathsep + _inno
_sandbox = real_env_guard.sandbox_home()
os.environ.update(real_env_guard.sandbox_env(_sandbox))

sys.path.insert(0, "src")
# subprocess tests launch `python -m respectedbrain`; the only installed metadata points at a moved checkout, so export src
os.environ["PYTHONPATH"] = os.path.abspath("src") + (os.pathsep + os.environ["PYTHONPATH"] if os.environ.get("PYTHONPATH") else "")
os.environ["PYTHONDONTWRITEBYTECODE"] = "1"
initial_env = os.environ.copy()


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
    command = [sys.executable, "-B", os.path.abspath(__file__), "--passes-alone", name]
    try:
        completed = subprocess.run(command, cwd=os.getcwd(), env=initial_env, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL, check=False)
    except OSError:
        return False
    return completed.returncode == 0


if len(sys.argv) == 3 and sys.argv[1] == "--passes-alone":
    sys.exit(run_single_test(sys.argv[2]))

suite = unittest.defaultTestLoader.discover("tests", pattern="*test*.py", top_level_dir=".")
result = unittest.TextTestRunner(verbosity=1).run(suite)
failing = sorted({test.id() for test, _ in result.failures + result.errors})
discovery_import_errors = [
    test.id()
    for test, _ in result.errors
    if test.__class__.__module__ == "unittest.loader"
    and test.__class__.__qualname__ == "_FailedTest"
]
# an import failure must fail the gate even if the import succeeds during an isolated rerun
real = [name for name in failing if name in discovery_import_errors or not passes_alone(name)]
for name in failing:
    print("FLAKY (passed on serial rerun):" if name not in real else "FAILED:", name)
_drift = real_env_guard.diff(_before, real_env_guard.snapshot())
print("REAL ENVIRONMENT DRIFT:" if _drift else "real environment unchanged", *_drift, sep=chr(10))
shutil.rmtree(_sandbox, ignore_errors=True)
sys.exit(0 if not real and result.testsRun >= 900 and not _drift else 1)
