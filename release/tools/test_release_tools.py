"""Self-check of the release tooling in a temp directory (no real key, repo files untouched): python tools/test_release_tools.py"""
import base64, shutil, sys, tempfile
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
import bundle_signing as bs, release_keys as rk, release_package as rp
from cryptography.hazmat.primitives.asymmetric.ed25519 import Ed25519PrivateKey

work = Path(tempfile.mkdtemp())
try:
    # refuses to keep a private key inside the repo, and refuses to overwrite
    for bad in (bs.ROOT / "x.key",):
        try:
            rk.generate(bad); raise SystemExit("repo path accepted")
        except SystemExit as error:
            assert "outside this repository" in str(error), error
    assert not (bs.ROOT / "x.key").exists()
    existing = work / "k.key"; existing.write_text("x")
    try:
        rk.generate(existing); raise SystemExit("overwrite accepted")
    except SystemExit as error:
        assert "overwrite" in str(error), error

    # a checksum list signed by a trusted key verifies; wrong key, tampered file and tampered list do not
    private = Ed25519PrivateKey.generate()
    key = work / "r.key"
    from cryptography.hazmat.primitives import serialization
    key.write_text(base64.b64encode(private.private_bytes(serialization.Encoding.Raw, serialization.PrivateFormat.Raw, serialization.NoEncryption())).decode())
    out = work / "dist"; out.mkdir()
    for name in rp.INSTALLERS:
        (out / f"{name}-Windows-Setup.exe").write_bytes(name.encode())
    rp.build(key, out)
    trusted = {bs.key_id(bs.public_bytes(private)): bs.public_bytes(private)}
    other = Ed25519PrivateKey.generate()
    assert rp.verify(out, trusted)
    assert not rp.verify(out, {bs.key_id(bs.public_bytes(private)): bs.public_bytes(other)}), "wrong public key accepted"
    assert not rp.verify(out, {}), "unknown key accepted"
    (out / "RespectedHub-Windows-Setup.exe").write_bytes(b"tampered")
    assert not rp.verify(out, trusted), "tampered installer accepted"
    (out / "RespectedHub-Windows-Setup.exe").write_bytes(b"RespectedHub")
    (out / "SHA256SUMS.txt").write_text("0" * 64 + " *RespectedHub-Windows-Setup.exe\n")
    assert not rp.verify(out, trusted), "tampered list accepted"
    print("release tools OK")
finally:
    shutil.rmtree(work, ignore_errors=True)
