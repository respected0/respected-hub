"""Assemble the downloadable beta folder: installers + SHA256SUMS.txt + SHA256SUMS.sig + BETA_NOTICE.txt.

    python common/release/tools/build/release_package.py build --key D:\\vault\\respected-release.key    # writes dist/SHA256SUMS.txt/.sig
    python common/release/tools/build/release_package.py verify [--dir dist]                             # re-check hashes and the Ed25519 signature

The checksum file is signed with the same release key whose public half is compiled into the Hub, so a person with
Python can prove the list is ours without a paid certificate (the installers themselves have no verified publisher).
Verification uses release-keys.txt, i.e. the keys a production Hub trusts.
"""
import argparse, base64, hashlib, shutil, sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from cryptography.hazmat.primitives.asymmetric.ed25519 import Ed25519PublicKey

from signing import bundle_signing as bs
from signing.release_keys import TRUST_FILE

DIST = bs.ROOT / "dist"
PRODUCTS = ("Hub", "Orchestrator", "Brain")
DOMAIN = b"respected-release-v1\n"


def sums(directory: Path) -> str:
    files = []
    for product in PRODUCTS:
        found = sorted(directory.glob(f"Respected-{product}-*-Windows-x64-Setup.exe"))
        if len(found) != 1:
            sys.exit(f"expected one Respected-{product}-<version>-Windows-x64-Setup.exe in {directory}, found {len(found)}; run publish_release.py build first")
        files.append(found[0])
    return "\n".join(f"{hashlib.sha256(file.read_bytes()).hexdigest()} *{file.name}" for file in files) + "\n"


def build(key: Path, directory: Path) -> None:
    private = bs.load_private(key)
    text = sums(directory)
    (directory / "SHA256SUMS.txt").write_text(text, encoding="ascii", newline="\n")
    signature = {"key_id": bs.key_id(bs.public_bytes(private)), "value": base64.b64encode(private.sign(DOMAIN + text.encode("ascii"))).decode("ascii")}
    (directory / "SHA256SUMS.sig").write_text(f"{signature['key_id']} {signature['value']}\n", encoding="ascii")
    shutil.copyfile(bs.ROOT / "respected-hub" / "packaging" / "windows" / "BETA_NOTICE.txt", directory / "BETA_NOTICE.txt")
    print(text, end="")


def verify(directory: Path, trusted: dict | None = None) -> bool:
    text = (directory / "SHA256SUMS.txt").read_text("ascii")
    key_id, value = (directory / "SHA256SUMS.sig").read_text("ascii").split()
    if trusted is None:
        lines = [l.split() for l in TRUST_FILE.read_text("utf-8").splitlines() if l.strip() and not l.lstrip().startswith("#")]
        trusted = {k: base64.b64decode(v) for k, v in lines}
    if key_id not in trusted:
        return False
    try:
        Ed25519PublicKey.from_public_bytes(trusted[key_id]).verify(base64.b64decode(value), DOMAIN + text.encode("ascii"))
    except Exception:
        return False
    return text == sums(directory)


def main(argv=None) -> int:
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument("action", choices=("build", "verify"))
    parser.add_argument("--key", type=Path)
    parser.add_argument("--dir", type=Path, default=DIST)
    args = parser.parse_args(argv)
    if args.action == "build":
        if not args.key:
            parser.error("--key required")
        build(args.key, args.dir)
        return 0 if verify(args.dir) else sys.exit("built checksums do not verify against release-keys.txt")
    ok = verify(args.dir)
    print("valid" if ok else "INVALID")
    return 0 if ok else 1


if __name__ == "__main__":
    sys.exit(main())
