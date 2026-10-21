# Release tooling and documents (lives in the Hub repo)

Everything that builds, signs and verifies the Windows beta. The scripts expect the three product folders side by side
(`respected-hub`, `respected-secondbrain`, `respected-orchestrator`); `tools/ecosystem_root.py` finds that parent folder
(or use `RESPECTED_ECOSYSTEM_ROOT`). Output goes to `<ecosystem>/dist`, caches to `<ecosystem>/.build-cache`.

* `tools/` : `publish_release.py build|catalog|upload` (installers, signed product archives, the signed catalog on GitHub Releases), `release_package.py build|verify` (SHA256SUMS);
  `release_keys.py`, `release_vault.py` (key custody), `bundle_signing.py`, `build_*`, acceptance and native-test drivers.
* `BETA_NOTICE.txt` : the page the installer shows before installing.
* `docs/` : `MASTER_HANDOFF.md` (state and history), `RELEASE_NOTES_BETA.md` (download text), `INSTALL_LAYOUT.md`, plan, audit, directive.

No key material is stored here. The release key is a passphrase-encrypted vault kept outside every repository.
