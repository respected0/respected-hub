# Bundle trust: Ed25519 signatures, fail closed

Status: implemented 2026-10-17. Audit finding P0 #8. Production key provisioning and Authenticode are the release phase.

## Problem
`signed` meant only "the `signature` field is not null"; `bundle_digest` lives in the same writable `bundle.json`, so
anyone who could write the catalog folder could install code as "signed". SecondBrain installs always passed
`--allow-unattested`, switching off the product's own release attestation.

## Decision
* Every bundle carries an Ed25519 signature over a canonical, domain-separated statement
  (`respected-bundle-v1\n` + sorted compact JSON of schema, product, kind, version, bundle_digest); the digest already
  binds every file path, size and hash, so a signature cannot be moved to another product, kind, version or payload.
  Signing: `release/tools/signing/bundle_signing.py`; `build_bundle.py` signs every bundle.
* `src/trust.rs` verifies at catalog read time (`read_bundle`), which gates both "available" and install. Anything that
  does not verify is skipped with a reason; nothing is accepted silently.
* **Trust is separated by key and by build, not by a flag in the data:**
  * PRODUCTION keys are compiled in (`PRODUCTION_KEYS`, empty until the release owner provisions one). A production
    build therefore trusts no bundle until then.
  * DEVELOPMENT trust (the local dev public key, file `dev-trust.pub` beside the exe or `RESPECTED_HUB_DEV_TRUST`) and the
    explicit unsigned opt-in `RESPECTED_HUB_ALLOW_UNSIGNED=1` exist only in debug builds or builds with
    `--features dev-trust`. In a production build that code is compiled out; `cargo test --release --lib trust` proves an
    environment variable cannot re-enable it.
  * The dev PRIVATE key lives in `.build-cache/signing` (ignored by git, created on first use) and is never copied into an
    installer or bundle. Installers ship only its public half, and only in development installers.
* SecondBrain gets `--allow-unattested` only when the staged package is not production-signed.
* `release/tools/build/build_hub_installer.py` builds a DEVELOPMENT installer by default (banner says so); `--release-key FILE` builds a
  PRODUCTION one (no dev-trust feature, no dev public key, bundles signed with that key).

## Proven
`trust.rs` unit tests (including a Python-made signature verified by the Rust code) and `src-tauri/tests/lifecycle/trust_catalog_e2e.rs`
with real bundles: version/kind swap, re-digested payload, flipped signature byte, unknown algorithm, stripped signature,
attacker-signed bundle (unknown key) are all rejected; the unsigned bundle is accepted only behind the development opt-in.

## Release phase (not done here)
Generate the production key pair offline, put the public key in `PRODUCTION_KEYS`, sign release bundles with
`--release-key`, add Authenticode for the exe and installers, then ship production installers without `dev-trust`.
