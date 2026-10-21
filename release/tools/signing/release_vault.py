"""Passphrase-encrypted custody for the production Ed25519 release key. The plaintext key never touches disk.

    python release/tools/signing/release_vault.py create --out %USERPROFILE%\\.respected-release\\respected-release.vault   # run in YOUR terminal
    python release/tools/signing/release_vault.py verify --vault FILE      # decrypt, compare public key, sign + verify a test message

Format: JSON; scrypt (n=2^17, r=8, p=1, 128 MiB) -> AES-256-GCM. Public key line, KDF parameters and version are
authenticated as AAD, so tampering with any of them fails decryption. `bundle_signing.load_private` accepts a vault
file wherever it accepts a raw key (--key FILE / RESPECTED_SIGNING_KEY) and asks for the passphrase without echo
(or reads RESPECTED_VAULT_PASSPHRASE if the owner set it in that process).
"""
import argparse, base64, getpass, json, os, sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from cryptography.exceptions import InvalidTag
from cryptography.hazmat.primitives import serialization
from cryptography.hazmat.primitives.asymmetric.ed25519 import Ed25519PrivateKey
from cryptography.hazmat.primitives.ciphers.aead import AESGCM
from cryptography.hazmat.primitives.kdf.scrypt import Scrypt

from signing import bundle_signing as bs

MAGIC = "respected-vault-v1"
N, R, P = 2 ** 17, 8, 1
MIN_LEN = 14
_cache = {}


def is_vault(path: Path) -> bool:
    try:
        return path.read_text("utf-8")[:80].lstrip().startswith("{") and MAGIC in path.read_text("utf-8")[:200]
    except (OSError, UnicodeDecodeError):
        return False


def _kdf(passphrase: str, salt: bytes) -> bytes:
    return Scrypt(salt=salt, length=32, n=N, r=R, p=P).derive(passphrase.encode("utf-8"))


def _aad(public_line: str) -> bytes:
    return json.dumps({"magic": MAGIC, "n": N, "r": R, "p": P, "public": public_line}, sort_keys=True).encode()


def public_line(private: Ed25519PrivateKey) -> str:
    public = bs.public_bytes(private)
    return f"{bs.key_id(public)} {base64.b64encode(public).decode('ascii')}"


def seal(private: Ed25519PrivateKey, passphrase: str) -> str:
    raw = private.private_bytes(serialization.Encoding.Raw, serialization.PrivateFormat.Raw, serialization.NoEncryption())
    line, salt, nonce = public_line(private), os.urandom(16), os.urandom(12)
    ciphertext = AESGCM(_kdf(passphrase, salt)).encrypt(nonce, raw, _aad(line))
    b64 = lambda b: base64.b64encode(b).decode("ascii")
    return json.dumps({"magic": MAGIC, "n": N, "r": R, "p": P, "public": line, "salt": b64(salt), "nonce": b64(nonce), "ciphertext": b64(ciphertext)}, indent=1)


def unseal(text: str, passphrase: str) -> Ed25519PrivateKey:
    doc = json.loads(text)
    if (doc.get("magic"), doc.get("n"), doc.get("r"), doc.get("p")) != (MAGIC, N, R, P):
        raise ValueError("unsupported vault parameters")
    line = doc["public"]
    try:
        raw = AESGCM(_kdf(passphrase, base64.b64decode(doc["salt"]))).decrypt(base64.b64decode(doc["nonce"]), base64.b64decode(doc["ciphertext"]), _aad(line))
    except InvalidTag:
        raise ValueError("wrong passphrase or damaged vault") from None
    private = Ed25519PrivateKey.from_private_bytes(raw)
    if public_line(private) != line:
        raise ValueError("vault public key does not match its private key")
    return private


def load(path: Path) -> Ed25519PrivateKey:
    text = path.read_text("utf-8")
    key = str(path.resolve())
    if key not in _cache:
        _cache[key] = os.environ.get("RESPECTED_VAULT_PASSPHRASE") or getpass.getpass(f"Passphrase for {path.name}: ")
    return unseal(text, _cache[key])


def selftest(private: Ed25519PrivateKey, expected_line: str) -> None:
    message = b"respected-vault-selftest\n" + os.urandom(16)
    assert public_line(private) == expected_line, "public key mismatch"
    private.public_key().verify(private.sign(message), message)


def create(out: Path) -> None:
    if bs.ROOT in out.resolve().parents:
        sys.exit("refusing: the vault must live outside this repository")
    if out.exists():
        sys.exit(f"refusing to overwrite {out}")
    pw = getpass.getpass(f"New passphrase (min {MIN_LEN} chars, not shown): ")
    if len(pw) < MIN_LEN or pw != getpass.getpass("Repeat passphrase: "):
        sys.exit("passphrase too short or did not match; nothing written")
    private = Ed25519PrivateKey.generate()
    text = seal(private, pw)
    selftest(unseal(text, pw), public_line(private))          # recover from the sealed text before writing anything
    out.parent.mkdir(parents=True, exist_ok=True)
    with open(out, "x", encoding="utf-8") as handle:
        handle.write(text)
    line = public_line(private)
    from signing import release_keys as rk
    if line.split()[0] not in rk.trusted_ids():
        with open(rk.TRUST_FILE, "a", encoding="utf-8") as handle:
            handle.write(line + "\n")
    print("vault written:", out)
    print("public key:", line)


def verify(path: Path) -> None:
    private = unseal(path.read_text("utf-8"), getpass.getpass("Passphrase: "))
    selftest(private, json.loads(path.read_text("utf-8"))["public"])
    print("OK: decrypted, public key matches, test signature verified:", public_line(private))


def main(argv=None) -> int:
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument("action", choices=("create", "verify"))
    parser.add_argument("--out", type=Path)
    parser.add_argument("--vault", type=Path)
    args = parser.parse_args(argv)
    if args.action == "create":
        create(args.out or parser.error("--out required"))
    else:
        verify(args.vault or parser.error("--vault required"))
    return 0


if __name__ == "__main__":
    sys.exit(main())
