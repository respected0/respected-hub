"""Build a verifiable local product bundle for the Hub catalog.

    python tools/build_bundle.py orchestrator|secondbrain [--out DIR]

Writes ``<out>/<product>-<version>/bundle.json`` plus ``payload/src/...`` (a copy of the
product's ``src`` tree without caches). ``bundle.json`` lists every payload file with its
size and sha256 and carries a digest over that list. Bundles are UNSIGNED local builds:
Every bundle is signed (Ed25519, tools/bundle_signing.py): with the local DEVELOPMENT key by default, with the release key
when ``--key``/``RESPECTED_SIGNING_KEY`` is given. The Hub accepts only signatures from keys it trusts.
"""
import argparse, hashlib, json, re, shutil, sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
from bundle_signing import sign_manifest

from ecosystem_root import ECO as ROOT
PRODUCTS = {"orchestrator": "respected-orchestrator", "secondbrain": "respected-secondbrain"}
SKIP_DIRS = {"__pycache__"}
SKIP_SUFFIXES = {".pyc", ".pyo"}
SIGNING_KEY = None  # set from --key for release builds


def version_of(source: Path) -> str:
    match = re.search(r'(?m)^version\s*=\s*"([^"]+)"', (source / "pyproject.toml").read_text("utf-8"))
    if not match:
        raise SystemExit("no version in pyproject.toml")
    return match.group(1)


def sha256_file(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as handle:
        for chunk in iter(lambda: handle.read(1 << 20), b""):
            digest.update(chunk)
    return digest.hexdigest()


def build_distribution(dist: Path, out: Path) -> Path:
    """Wrap a frozen SecondBrain distribution folder (distribution.json + launcher + app/) as a catalog bundle."""
    version = json.loads((dist / "distribution.json").read_text("utf-8"))["version"]
    target = out / f"secondbrain-dist-{version}"
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
    manifest = {"schema": 1, "product": "secondbrain", "kind": "distribution", "version": version, "files": files,
                "bundle_digest": digest, "signature": None}
    (target / "bundle.json").write_text(json.dumps(sign_manifest(manifest, SIGNING_KEY), indent=1), encoding="utf-8")
    return target


def build(product: str, out: Path) -> Path:
    source = ROOT / PRODUCTS[product]
    version = version_of(source)
    target = out / f"{product}-{version}"
    if target.exists():
        shutil.rmtree(target)
    files = []
    for path in sorted((source / "src").rglob("*")):
        if not path.is_file() or path.is_symlink() or SKIP_DIRS & set(path.parts) or path.suffix in SKIP_SUFFIXES:
            continue
        relative = path.relative_to(source).as_posix()
        destination = target / "payload" / relative
        destination.parent.mkdir(parents=True, exist_ok=True)
        shutil.copyfile(path, destination)
        files.append({"path": relative, "size": destination.stat().st_size, "sha256": sha256_file(destination)})
    digest = hashlib.sha256(json.dumps(files, sort_keys=True, separators=(",", ":"), ensure_ascii=False).encode("utf-8")).hexdigest()
    manifest = {"schema": 1, "product": product, "version": version, "files": files,
                "bundle_digest": digest, "signature": None}
    (target / "bundle.json").write_text(json.dumps(sign_manifest(manifest, SIGNING_KEY), indent=1), encoding="utf-8")
    return target


if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument("product", choices=sorted(PRODUCTS) + ["secondbrain-dist"])
    parser.add_argument("--dist", type=Path, help="frozen SecondBrain distribution folder (secondbrain-dist only)")
    parser.add_argument("--out", type=Path, default=ROOT / "catalog")
    parser.add_argument("--key", type=Path, help="release signing key (default: local development key)")
    args = parser.parse_args()
    SIGNING_KEY = args.key
    built = build_distribution(args.dist, args.out) if args.product == "secondbrain-dist" else build(args.product, args.out)
    print(built)
    sys.exit(0)
