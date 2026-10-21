"""Production signing key ceremony helper. Run by the release owner, on a machine they trust, OFFLINE.

    python release/tools/signing/release_keys.py generate --out D:\\vault\\respected-release.key   # new key pair; private key stays at --out
    python release/tools/signing/release_keys.py public --key D:\\vault\\respected-release.key      # print "<key_id> <base64 public>"
    python release/tools/signing/release_keys.py check  --key D:\\vault\\respected-release.key      # is this key trusted by respected-hub/src-tauri/release-keys.txt?

`generate` refuses a path inside this repository, refuses to overwrite, and appends only the PUBLIC half to release-keys.txt
(compiled into production Hub builds). Back the private key up in two places; losing it means no further updates can be
signed, leaking it means anyone can sign bundles production Hubs trust (rotate: add a new public line, ship, then remove the old).
"""
import argparse, base64, sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from cryptography.hazmat.primitives import serialization
from cryptography.hazmat.primitives.asymmetric.ed25519 import Ed25519PrivateKey

from signing import bundle_signing as bs

TRUST_FILE = bs.ROOT / "respected-hub" / "src-tauri" / "release-keys.txt"


def public_line(private: Ed25519PrivateKey) -> str:
    public = bs.public_bytes(private)
    return f"{bs.key_id(public)} {base64.b64encode(public).decode('ascii')}"


def trusted_ids() -> set:
    lines = TRUST_FILE.read_text("utf-8").splitlines() if TRUST_FILE.is_file() else []
    return {line.split()[0] for line in lines if line.strip() and not line.lstrip().startswith("#")}


def inside_repo(path: Path) -> bool:
    return bs.ROOT in path.resolve().parents or path.resolve() == bs.ROOT


def generate(out: Path) -> str:
    if inside_repo(out.parent / out.name):
        sys.exit("refusing: the private key must live outside this repository")
    if out.exists():
        sys.exit(f"refusing to overwrite {out}")
    out.parent.mkdir(parents=True, exist_ok=True)
    private = Ed25519PrivateKey.generate()
    raw = private.private_bytes(serialization.Encoding.Raw, serialization.PrivateFormat.Raw, serialization.NoEncryption())
    with open(out, "x", encoding="ascii") as handle:
        handle.write(base64.b64encode(raw).decode("ascii"))
    line = public_line(private)
    if line.split()[0] not in trusted_ids():
        with open(TRUST_FILE, "a", encoding="utf-8") as handle:
            handle.write(line + "\n")
    return line


def main(argv=None) -> int:
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument("action", choices=("generate", "public", "check"))
    parser.add_argument("--out", type=Path)
    parser.add_argument("--key", type=Path)
    args = parser.parse_args(argv)
    if args.action == "generate":
        if not args.out:
            parser.error("--out required")
        print(generate(args.out))
        return 0
    if not args.key:
        parser.error("--key required")
    line = public_line(bs.load_private(args.key))
    if args.action == "public":
        print(line)
        return 0
    trusted = line.split()[0] in trusted_ids()
    print("trusted by release-keys.txt" if trusted else "NOT in release-keys.txt: a production Hub would reject everything signed with it")
    return 0 if trusted else 1


if __name__ == "__main__":
    sys.exit(main())
