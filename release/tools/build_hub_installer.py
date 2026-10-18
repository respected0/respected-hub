"""Build the unsigned Windows installer for the Hub: dist/RespectedHub-Windows-Setup.exe.

    python tools/build_hub_installer.py [--skip-app]

Stages the release executable plus a catalog (Orchestrator source bundle, Python runtime, frozen SecondBrain
distribution), then compiles packaging/windows/hub_setup.iss with Inno Setup (the same compiler SecondBrain uses).
Nothing is downloaded; the installer is local and unsigned.
"""
import argparse, json, os, shutil, subprocess, sys
from pathlib import Path

from ecosystem_root import ECO
TOOLS = Path(__file__).resolve().parent
HUB = ECO / "respected-hub"
STAGE = ECO / ".build-cache" / "hub-stage"
TARGET = ECO / ".cargo-target-hub"


def run(command, **kwargs):
    done = subprocess.run(command, **kwargs)
    if done.returncode:
        sys.exit(f"failed ({done.returncode}): {' '.join(map(str, command))}")


def inno() -> str:
    for candidate in (os.environ.get("INNO_COMPILER"), shutil.which("ISCC.exe"),
                      str(Path(os.environ.get("LOCALAPPDATA", "")) / "Programs/Inno Setup 6/ISCC.exe")):
        if candidate and Path(candidate).is_file():
            return candidate
    sys.exit("Inno Setup compiler (ISCC.exe) not found; set INNO_COMPILER")


if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument("--skip-app", action="store_true", help="reuse the existing release build")
    parser.add_argument("--release-key", type=Path, help="PRODUCTION build: sign bundles with this private key and build the Hub WITHOUT development trust; default is a DEVELOPMENT installer")
    args = parser.parse_args()
    py = [sys.executable, "-B"]
    release = args.release_key is not None
    sign = ["--key", str(args.release_key)] if release else []
    print("PRODUCTION build (no development trust)" if release else "DEVELOPMENT build (trusts the local dev signing key; not for distribution)")
    if not release:
        run(py + [str(TOOLS / "bundle_signing.py"), "init"])
    else:
        # a release key that production Hubs do not trust, or one kept in the repo, would ship an installer that is useless or unsafe
        run(py + [str(TOOLS / "release_keys.py"), "check", "--key", str(args.release_key)])
        if ECO in args.release_key.resolve().parents:
            sys.exit("the release private key must be outside the repository")
        os.environ["RESPECTED_SIGNING_KEY"] = str(args.release_key)
    if not args.skip_app:
        env = dict(os.environ, CARGO_TARGET_DIR=str(TARGET), PYTHONDONTWRITEBYTECODE="1")
        run(["cmd", "/c", "npm", "run", "tauri", "--", "build", "--no-bundle", *([] if release else ["--features", "dev-trust"])], cwd=HUB, env=env)
    exe = TARGET / "release" / "respected-hub.exe"
    if not exe.is_file():
        sys.exit("release executable missing; run without --skip-app")
    if STAGE.exists():
        shutil.rmtree(STAGE)
    (STAGE / "catalog").mkdir(parents=True)
    shutil.copyfile(exe, STAGE / "respected-hub.exe")
    if not release:  # public half only; production installers carry no development trust
        shutil.copyfile(ECO / ".build-cache" / "signing" / "dev-ed25519.pub", STAGE / "dev-trust.pub")
    shutil.copyfile(TOOLS.parent / "BETA_NOTICE.txt", STAGE / "BETA_NOTICE.txt")
    if release:  # a production artifact must not carry the development override surface
        data = (STAGE / "respected-hub.exe").read_bytes()
        leaked = [s for s in (b"RESPECTED_HUB_ALLOW_UNSIGNED", b"RESPECTED_HUB_DEV_TRUST", b"RESPECTED_HUB_TEST_HOME") if s in data]
        if leaked:
            sys.exit(f"production executable still contains development overrides: {leaked}")
    brain = subprocess.run(py + [str(TOOLS / "build_brain_dist.py")], capture_output=True, text=True)
    if brain.returncode:
        sys.exit(brain.stderr)
    brain_dist = brain.stdout.strip().splitlines()[-1]
    run(py + [str(TOOLS / "build_bundle.py"), "orchestrator", "--out", str(STAGE / "catalog"), *sign])
    run(py + [str(TOOLS / "build_bundle.py"), "secondbrain-dist", "--dist", brain_dist, "--out", str(STAGE / "catalog"), *sign])
    run(py + [str(TOOLS / "build_runtime.py"), "--out", str(STAGE / "catalog")])
    version = json.loads((HUB / "src-tauri" / "tauri.conf.json").read_text("utf-8-sig"))["version"]
    (ECO / "dist").mkdir(exist_ok=True)
    # One installer per download: the product installers are the same Hub + catalog; they only open Hub on that product's install page.
    for product, name in (("", "RespectedHub"), ("--install=orchestrator", "RespectedOrchestrator"), ("--install=secondbrain", "RespectedSecondBrain")):
        run([inno(), f"/DAppVersion={version}", f"/DStageDir={STAGE}", f"/DOutputDir={ECO / 'dist'}", f"/DProduct={product}", f"/DOutputBase={name}-Windows-Setup", str(HUB / "packaging" / "windows" / "hub_setup.iss")])
    print(ECO / "dist" / "RespectedHub-Windows-Setup.exe")
