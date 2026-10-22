"""Build a signed catalog bundle of a product's own Windows installer.

    python release/tools/build/build_bundle.py orchestrator|secondbrain [--out DIR] [--key FILE] [--force]

The product's own tool (`packaging/build_installer.py` in its repository) builds `dist/<Product>-Windows-Setup.exe`; it is
reused while no source or packaging file is newer (``--force`` always rebuilds). The bundle is
``<out>/<product>-<version>/bundle.json`` + ``payload/<Product>-Windows-Setup.exe``: one file with its size and sha256, a digest
over that list and an Ed25519 signature (bundle_signing.py), with the local DEVELOPMENT key by default and the release key when
``--key`` / ``RESPECTED_SIGNING_KEY`` is given. The Hub accepts only signatures from keys it trusts and runs the installer.
"""
import argparse, hashlib, json, os, shutil, subprocess, sys
from contextlib import contextmanager
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from signing.bundle_signing import sign_manifest
from common.ecosystem_root import ECO

# product -> (repository, installer file, distribution folder its build leaves beside it)
PRODUCTS = {
    "orchestrator": ("respected-orchestrator", "RespectedOrchestrator-Windows-Setup.exe", "RespectedOrchestrator"),
    "secondbrain": ("respected-secondbrain", "RespectedSecondBrain-Windows-Setup.exe", "RespectedBrain"),
}


def sha256_file(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as handle:
        for chunk in iter(lambda: handle.read(1 << 20), b""):
            digest.update(chunk)
    return digest.hexdigest()


@contextmanager
def build_lock(folder: Path):
    """One build per product at a time: parallel test threads may all ask for the same installer."""
    folder.mkdir(parents=True, exist_ok=True)
    with open(folder / ".build.lock", "a+b") as handle:
        if os.name == "nt":
            import msvcrt
            while True:
                try:
                    msvcrt.locking(handle.fileno(), msvcrt.LK_LOCK, 1)  # retries for ~10 s, then raises
                    break
                except OSError:
                    continue
        else:
            import fcntl
            fcntl.flock(handle.fileno(), fcntl.LOCK_EX)
        try:
            yield
        finally:
            if os.name == "nt":
                handle.seek(0)
                msvcrt.locking(handle.fileno(), msvcrt.LK_UNLCK, 1)


def installer(product: str, force: bool = False) -> tuple[Path, Path]:
    """The product's installer and the distribution it was built from, rebuilt with its own tool when missing or stale."""
    repo, name, folder = PRODUCTS[product]
    source = ECO / repo
    setup, dist = source / "dist" / name, source / "dist" / folder
    with build_lock(ECO / ".build-cache" / "installers" / product):
        inputs = [p for part in ("src", "packaging") for p in (source / part).rglob("*") if p.is_file() and "__pycache__" not in p.parts]
        stale = force or not setup.is_file() or any(p.stat().st_mtime > setup.stat().st_mtime for p in inputs)
        if stale:
            subprocess.run([sys.executable, "-B", "packaging/build_installer.py"], cwd=source, check=True)
    return setup, dist


def build(product: str, setup: Path, dist: Path, out: Path, key: Path | None = None) -> Path:
    if not setup.name.endswith("-Windows-Setup.exe"):
        raise ValueError("the installer file must be named ...-Windows-Setup.exe")
    version = json.loads((dist / "distribution.json").read_text("utf-8"))["version"]
    target = out / f"{product}-{version}"
    if target.exists():
        shutil.rmtree(target)
    (target / "payload").mkdir(parents=True)
    shutil.copyfile(setup, target / "payload" / setup.name)
    files = [{"path": setup.name, "size": setup.stat().st_size, "sha256": sha256_file(setup)}]
    digest = hashlib.sha256(json.dumps(files, sort_keys=True, separators=(",", ":"), ensure_ascii=False).encode("utf-8")).hexdigest()
    manifest = {"schema": 1, "product": product, "kind": "installer", "version": version, "files": files, "bundle_digest": digest, "signature": None}
    (target / "bundle.json").write_text(json.dumps(sign_manifest(manifest, key), indent=1), encoding="utf-8")
    return target


if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument("product", choices=sorted(PRODUCTS))
    parser.add_argument("--out", type=Path, default=ECO / "catalog")
    parser.add_argument("--key", type=Path, help="release signing key (default: local development key)")
    parser.add_argument("--force", action="store_true", help="rebuild the installer even when it is current")
    args = parser.parse_args()
    print(build(args.product, *installer(args.product, args.force), args.out, args.key))
