"""Build a signed catalog bundle of a product's packaged (frozen) distribution.

    python release/tools/build_bundle.py orchestrator|secondbrain [--dist DIR] [--out DIR] [--key FILE] [--force]

Without --dist the product's own build tool freezes the current source into <ecosystem>/.build-cache/dist/<product>
(reused while no source file is newer). The bundle is ``<out>/<product>-<version>/bundle.json`` + ``payload/<distribution>``;
bundle.json lists every payload file with its size and sha256 and carries a digest over that list. Every bundle is signed
(Ed25519, bundle_signing.py): with the local DEVELOPMENT key by default, with the release key when ``--key`` /
``RESPECTED_SIGNING_KEY`` is given. The Hub accepts only signatures from keys it trusts.
"""
import argparse, hashlib, json, os, shutil, subprocess, sys
from contextlib import contextmanager
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
from bundle_signing import sign_manifest
from ecosystem_root import ECO

# product -> (repository, build command relative to it, distribution folder name, launcher)
PRODUCTS = {
    "orchestrator": ("respected-orchestrator", ["tools/build_dist.py"], "RespectedOrchestrator", "respected-orchestrator.exe"),
    "secondbrain": ("respected-secondbrain", ["tools/build_dist.py", "--platform", "windows"], "RespectedBrain", "respectedbrain.exe"),
}


def sha256_file(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as handle:
        for chunk in iter(lambda: handle.read(1 << 20), b""):
            digest.update(chunk)
    return digest.hexdigest()


@contextmanager
def build_lock(cache: Path):
    """One build per cache at a time: parallel test threads may all ask for the same product."""
    cache.mkdir(parents=True, exist_ok=True)
    with open(cache / ".build.lock", "a+b") as handle:
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


def frozen(product: str, force: bool = False) -> Path:
    """The product's packaged distribution, rebuilt with its own tool when missing or older than its source."""
    repo, command, name, launcher = PRODUCTS[product]
    source = ECO / repo
    cache = ECO / ".build-cache" / "dist" / product
    dist = cache / name
    exe = dist / launcher
    with build_lock(cache):
        stale = force or not exe.is_file() or any(p.stat().st_mtime > exe.stat().st_mtime for p in (source / "src").rglob("*")
                                                 if p.is_file() and "__pycache__" not in p.parts)
        if stale:
            subprocess.run([sys.executable, *command, "--output", str(cache)], cwd=source, check=True)
    return dist


def build(product: str, dist: Path, out: Path, key: Path | None = None) -> Path:
    version = json.loads((dist / "distribution.json").read_text("utf-8"))["version"]
    target = out / f"{product}-{version}"
    if target.exists():
        shutil.rmtree(target)
    files = []
    for path in sorted(dist.rglob("*")):
        if not path.is_file() or path.is_symlink():
            continue
        relative = path.relative_to(dist).as_posix()
        destination = target / "payload" / relative
        destination.parent.mkdir(parents=True, exist_ok=True)
        shutil.copyfile(path, destination)
        files.append({"path": relative, "size": destination.stat().st_size, "sha256": sha256_file(destination)})
    digest = hashlib.sha256(json.dumps(files, sort_keys=True, separators=(",", ":"), ensure_ascii=False).encode("utf-8")).hexdigest()
    manifest = {"schema": 1, "product": product, "kind": "distribution", "version": version, "files": files,
                "bundle_digest": digest, "signature": None}
    (target / "bundle.json").write_text(json.dumps(sign_manifest(manifest, key), indent=1), encoding="utf-8")
    return target


if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument("product", choices=sorted(PRODUCTS))
    parser.add_argument("--dist", type=Path, help="an already built distribution folder (default: build or reuse the cached one)")
    parser.add_argument("--out", type=Path, default=ECO / "catalog")
    parser.add_argument("--key", type=Path, help="release signing key (default: local development key)")
    parser.add_argument("--force", action="store_true", help="rebuild the distribution even when the cache is current")
    args = parser.parse_args()
    print(build(args.product, args.dist or frozen(args.product, args.force), args.out, args.key))
