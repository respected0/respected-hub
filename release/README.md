# Release tooling and documents (lives in the Hub repo)

Everything that builds, signs and verifies the Windows beta. The scripts expect the three product folders side by side
(`respected-hub`, `respected-secondbrain`, `respected-orchestrator`); `release/tools/common/ecosystem_root.py` finds that parent folder
(or use `RESPECTED_ECOSYSTEM_ROOT`). Output goes to `<ecosystem>/dist`, caches to `<ecosystem>/.build-cache`.

* `tools/`, one folder per job (scripts import each other as `common.*`, `signing.*`):
  * `build/`: `build_bundle.py <product>` (freezes a product with its own build tool and wraps it as a signed catalog bundle),
    `build_hub_installer.py`, `publish_release.py build|catalog|upload`, `release_package.py build|verify` (SHA256SUMS).
  * `signing/`: `bundle_signing.py`, `release_keys.py`, `release_vault.py` (key custody and signing).
  * `acceptance/`: `accept_hub.py`, `real_env_guard.py` (local acceptance), `rpp_conformance.py` (RPP check against both products).
  * `common/`: `ecosystem_root.py`.
* `tests/test_release_tools.py`: self-check of the tooling (run in CI).
* `BETA_NOTICE.txt`: the page the installer shows before installing.
* `BUILD_FROM_SCRATCH.md`: continue from nothing but the three repositories.
* `docs/`: `MASTER_HANDOFF.md` (where things stand), `RELEASE_NOTES_BETA.md` (download text), `INSTALL_LAYOUT.md`,
  `RESPECTED_MASTER_DIRECTIVE.md` (product vision).

No key material is stored here. The release key is a passphrase-encrypted vault kept outside every repository.
