"""Self-check of the release tooling in a temp directory (no real key, repo files untouched): python release/tests/test_release_tools.py"""
import base64, shutil, sys, tempfile
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent.parent / "tools"))
from signing import bundle_signing as bs, release_keys as rk
from build import release_package as rp
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
    for product in rp.PRODUCTS:
        (out / f"Respected-{product}-0.1.0-beta.1-Windows-x64-Setup.exe").write_bytes(product.encode())
    hub = out / "Respected-Hub-0.1.0-beta.1-Windows-x64-Setup.exe"
    rp.build(key, out)
    trusted = {bs.key_id(bs.public_bytes(private)): bs.public_bytes(private)}
    other = Ed25519PrivateKey.generate()
    assert rp.verify(out, trusted)
    assert not rp.verify(out, {bs.key_id(bs.public_bytes(private)): bs.public_bytes(other)}), "wrong public key accepted"
    assert not rp.verify(out, {}), "unknown key accepted"
    hub.write_bytes(b"tampered")
    assert not rp.verify(out, trusted), "tampered installer accepted"
    hub.write_bytes(b"Hub")
    (out / "SHA256SUMS.txt").write_text("0" * 64 + " *" + hub.name + "\n")
    assert not rp.verify(out, trusted), "tampered list accepted"

    # the catalog: merged over the published one, signed over the exact bytes, archives are bundle folders zipped at the root
    import json, zipfile
    from build import publish_release as pr
    from cryptography.hazmat.primitives.asymmetric.ed25519 import Ed25519PublicKey
    bundle = work / "bundle"; (bundle / "payload" / "src").mkdir(parents=True)
    (bundle / "bundle.json").write_text('{"version": "0.1.0-beta.1"}'); (bundle / "payload" / "src" / "a.py").write_text("x")
    archive = work / pr.package_name("orchestrator", "0.1.0-beta.1")
    pr.zip_bundle(bundle, archive)
    assert sorted(zipfile.ZipFile(archive).namelist()) == ["bundle.json", "payload/src/a.py"]
    base = {"schema": 1, "sequence": 99999999999999, "hub": {"stable": {"version": "0.1.0"}}, "products": {"brain": {"beta": {"version": "0.1.0-beta.1"}}}}
    entries = {"orchestrator": pr.entry(archive, "0.1.0-beta.1", "respected-orchestrator", "v0.1.0-beta.1")}
    index = pr.merged_catalog(base, entries, "beta", "0.1.0-beta.1")
    assert index["sequence"] == base["sequence"] + 1, "sequence always grows"
    assert index["products"]["brain"] == base["products"]["brain"] and index["hub"] == base["hub"], "unreleased parts are kept"
    release = index["products"]["orchestrator"]["beta"]
    assert release["url"] == "https://github.com/respected0/respected-orchestrator/releases/download/v0.1.0-beta.1/Respected-Orchestrator-0.1.0-beta.1-Windows-x64-Package.zip"
    assert release["min_hub"] == "0.1.0-beta.1" and release["size"] == archive.stat().st_size
    assert "notes" not in release and pr.merged_catalog(base, entries, "beta", None, "Düzeltmeler")["products"]["orchestrator"]["beta"]["notes"] == "Düzeltmeler"
    pr.sign_index(index, key, work)
    body = (work / "catalog.json").read_bytes()
    key_id, value = (work / "catalog.json.sig").read_text().split()
    assert key_id == bs.key_id(bs.public_bytes(private))
    Ed25519PublicKey.from_public_bytes(bs.public_bytes(private)).verify(base64.b64decode(value), pr.DOMAIN + body)
    assert json.loads(body) == index
    print("release tools OK")
finally:
    shutil.rmtree(work, ignore_errors=True)
