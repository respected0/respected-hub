"""Ed25519 signing of catalog bundles. The Hub verifies this; nothing here ships a private key.

    python common/release/tools/signing/bundle_signing.py init                 # create the DEVELOPMENT key pair if missing (local, never committed)
    python common/release/tools/signing/bundle_signing.py sign BUNDLE_DIR      # (re)sign a bundle.json with the dev key, or --key FILE for release
    python common/release/tools/signing/bundle_signing.py verify BUNDLE_DIR    # offline check against the dev public key

Trust is separated by key, not by flag: the DEVELOPMENT private key lives in ``.build-cache/signing`` (ignored by
git, created on first use) and only development Hub builds trust its public half. The PRODUCTION key is generated
and kept by the release owner and passed with ``--key`` / ``RESPECTED_SIGNING_KEY``; its public half is compiled into
production Hub builds (see respected-hub/src-tauri/src/trust.rs).

What is signed: a domain-separated, canonical statement of product, kind, version, schema and the bundle digest (the
digest already binds every file path, size and sha256), so a signature cannot be moved to another product or version.
"""
import argparse, base64, hashlib, json, os, sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))

from cryptography.hazmat.primitives import serialization
from cryptography.hazmat.primitives.asymmetric.ed25519 import Ed25519PrivateKey, Ed25519PublicKey

from common.ecosystem_root import ECO as ROOT
SIGNING_DIR = ROOT / ".build-cache" / "signing"
DEV_KEY = SIGNING_DIR / "dev-ed25519.key"
DEV_PUBLIC = SIGNING_DIR / "dev-ed25519.pub"
DOMAIN = b"respected-bundle-v1\n"


def statement(manifest: dict) -> bytes:
    body = {k: manifest.get(k) for k in ("schema", "product", "kind", "version", "bundle_digest")}
    return DOMAIN + json.dumps(body, sort_keys=True, separators=(",", ":"), ensure_ascii=False).encode("utf-8")


def public_bytes(private: Ed25519PrivateKey) -> bytes:
    return private.public_key().public_bytes(serialization.Encoding.Raw, serialization.PublicFormat.Raw)


def key_id(public: bytes) -> str:
    return hashlib.sha256(public).hexdigest()[:16]


def load_private(path: Path) -> Ed25519PrivateKey:
    from signing import release_vault                               # passphrase-encrypted release key (no plaintext on disk)
    if release_vault.is_vault(path):
        return release_vault.load(path)
    return Ed25519PrivateKey.from_private_bytes(base64.b64decode(path.read_text("ascii").strip()))


def ensure_dev_key() -> Ed25519PrivateKey:
    if DEV_KEY.is_file():
        private = load_private(DEV_KEY)
    else:
        SIGNING_DIR.mkdir(parents=True, exist_ok=True)
        private = Ed25519PrivateKey.generate()
        raw = private.private_bytes(serialization.Encoding.Raw, serialization.PrivateFormat.Raw, serialization.NoEncryption())
        DEV_KEY.write_text(base64.b64encode(raw).decode("ascii"), encoding="ascii")
        try:
            os.chmod(DEV_KEY, 0o600)
        except OSError:
            pass
    public = public_bytes(private)
    line = f"{key_id(public)} {base64.b64encode(public).decode('ascii')}" + chr(10)
    if not DEV_PUBLIC.is_file() or DEV_PUBLIC.read_text("ascii") != line:   # concurrent builds/tests read this file: never rewrite it needlessly
        temporary = DEV_PUBLIC.with_name(DEV_PUBLIC.name + f".{os.getpid()}.tmp")
        temporary.write_text(line, encoding="ascii")
        os.replace(temporary, DEV_PUBLIC)
    return private


def sign_manifest(manifest: dict, key_file: Path | None = None) -> dict:
    """Return manifest with a fresh signature: production key when given (--key / RESPECTED_SIGNING_KEY), else the dev key."""
    key_file = key_file or (Path(os.environ["RESPECTED_SIGNING_KEY"]) if os.environ.get("RESPECTED_SIGNING_KEY") else None)
    private = load_private(key_file) if key_file else ensure_dev_key()
    signature = private.sign(statement(manifest))
    return dict(manifest, signature={"alg": "ed25519", "key_id": key_id(public_bytes(private)),
                                     "value": base64.b64encode(signature).decode("ascii")})


def sign_bundle_dir(bundle: Path, key_file: Path | None = None) -> None:
    path = bundle / "bundle.json"
    manifest = json.loads(path.read_text("utf-8-sig"))
    path.write_text(json.dumps(sign_manifest(manifest, key_file), indent=1), encoding="utf-8")


def verify_manifest(manifest: dict, public: bytes) -> bool:
    signature = manifest.get("signature")
    if not isinstance(signature, dict) or signature.get("alg") != "ed25519":
        return False
    try:
        Ed25519PublicKey.from_public_bytes(public).verify(base64.b64decode(signature["value"]), statement(manifest))
        return True
    except Exception:
        return False


def main(argv=None) -> int:
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument("action", choices=("init", "sign", "verify"))
    parser.add_argument("bundle", nargs="?", type=Path)
    parser.add_argument("--key", type=Path, help="release private key file (base64 raw Ed25519); default: the development key")
    args = parser.parse_args(argv)
    if args.action == "init":
        ensure_dev_key()
        print(DEV_PUBLIC.read_text("ascii").strip())
        return 0
    if args.bundle is None:
        parser.error("bundle directory required")
    if args.action == "sign":
        sign_bundle_dir(args.bundle, args.key)
        print("signed", args.bundle)
        return 0
    manifest = json.loads((args.bundle / "bundle.json").read_text("utf-8-sig"))
    public = base64.b64decode(DEV_PUBLIC.read_text("ascii").split()[1])
    ok = verify_manifest(manifest, public)
    print("valid" if ok else "INVALID")
    return 0 if ok else 1


if __name__ == "__main__":
    sys.exit(main())
