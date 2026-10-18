# Release phase plan – Windows beta without a paid certificate

Updated 2026-10-18. Goal: a safe, working, honestly described **Windows beta** of Hub, Orchestrator and SecondBrain.
Authenticode is deferred (no budget). Bundle authenticity stays on the free Ed25519 chain.

## What "beta without a certificate" means
| Layer | State |
|---|---|
| Product bundles (SecondBrain, Orchestrator, runtime) | Ed25519-signed; the production Hub installs nothing else (fail closed) |
| Release checksums | `SHA256SUMS.txt` signed with the same key (`SHA256SUMS.sig`), verifiable with `tools/release_package.py verify` |
| Installer / exe | **Not Authenticode-signed.** Windows shows SmartScreen / "Unknown publisher". Stated in the wizard (`release/BETA_NOTICE.txt`) and in the download folder |
| Dev trust / overrides | Compiled out of production (proven by `cargo test --release --lib trust` and an executable string audit in the build) |

## Done in this phase (no approval needed)
* `respected-hub/src-tauri/release-keys.txt` – public release keys compiled into production builds (currently none, so production installs nothing).
* `tools/release_keys.py` – key ceremony: generate (refuses repo paths / overwrite), print public, check trust.
* `tools/build_hub_installer.py --release-key FILE` – now refuses a key the Hub would not trust or one kept inside the repo, and fails if the production exe still contains a dev override.
* `tools/release_package.py` – signed checksum list for the three installers + notice; `tools/test_release_tools.py` self-check.
* Installer: pre-install beta/SmartScreen notice (TR+EN), WebView2 presence check (no download; warns and lets the person decide).
* VC runtime: not needed (the executable imports no `vcruntime`/`msvcp` DLL).

## Needs your decision / action (the only blockers)
1. **Generate the real release key** – `python tools/release_keys.py generate --out <path outside the repo>`. I can run it for you only if you approve a location; you must back it up in two places.
2. **Download restic** (third-party binary, ~10 MB) – approve the pinned version and I verify its published SHA-256, or install it yourself.
3. **A clean Windows environment** for the production install, clean-machine and interactive-uninstall checks. A production exe ignores every test override, so on this PC it would install into your real profile. Options: Windows Sandbox (needs an admin to enable the feature), a VM, or a throwaway local Windows user.
4. **A person to click** the uninstall prompt (Yes / No / cancel) and the SmartScreen dialog once.

## Sequence once the inputs exist
1. Real key → `release-keys.txt` → `build_hub_installer.py --release-key KEY` → `release_package.py build --key KEY`.
2. Verify the artefacts: `release_package.py verify`; exe has no dev strings; installer has no `dev-trust.pub`.
3. Real restic run in a disposable vault: first backup (init), password source, verification, `--restore-to`, refusal of non-empty/in-vault targets, corrupted snapshot detected. Fix the restored-path assumption if the real layout differs.
4. Clean environment: install via the wizard, confirm the notice page, walk the Hub wizard, install SecondBrain + Orchestrator, one MCP call, repair, uninstall (Yes and No), cancel mid-way, Hub running during uninstall, upgrade A→B→A.
5. Release gate: all suites on the final state, native E2E (dev build) once more, then publish the folder.

## Honest beta limits (to state on the download page)
* No verified publisher: SmartScreen warning; compare SHA-256.
* Windows only. No macOS/Linux, no remote catalog/auto-update (update by installing the next beta).
* Gemini workers unavailable on Windows; Codex/Cursor/Antigravity hooks not driven by a real session.
* Open audit items are listed in `AUDIT_REPORT_2026-10-17.md` (resolution status).


## Status at pause (2026-10-18) – resume here
Verified this session:
* Key plumbing, `release_keys.py`, `build_hub_installer.py --release-key` guards, `release_package.py`, installer beta notice + WebView2 check; production rehearsal with a throwaway key passed (exe string audit, bundles signed, checksums verified, tamper detected).
* **Real restic 0.19.1** (downloaded with approval; SHA-256 matched the release SHA256SUMS, GPG signature not checked): all scenarios pass after fixing a real Windows restore bug (restore now uses `snapshot:/path`). The binary sits in `%TEMP%\restic-dl` (not on PATH); delete it when no longer needed.
* Residue check: `release-keys.txt` has **no key lines**; the throwaway key and its folder are deleted; no production/real release key exists anywhere; `dist/` holds the earlier DEVELOPMENT installers (not for distribution); only the pre-existing git-ignored dev key remains in `.build-cache/signing`.

Not done (in this order when resuming):
1. **Real release key** – D: does not exist and no USB is available. Decide a custody location (an offline/removable drive, or a password-manager vault) and run `python tools/release_keys.py generate --out <path outside the repo>`; back it up in two places. Nothing else in the production chain can proceed without it.
2. Production build: `python tools/build_hub_installer.py --release-key <key>` then `python tools/release_package.py build --key <key>`; `release_package.py verify`.
3. Clean Windows environment (VM / Windows Sandbox / throwaway user) for the production install, wizard, MCP call, repair, upgrade A→B→A, uninstall Yes/No/cancel/Hub-running. A production exe ignores test overrides, so never run it on this PC's real profile.
4. A person to see the notice page, the WebView2-missing dialog, SmartScreen and the interactive uninstall prompt (compiled, not yet seen on screen).
5. Real sessions for Codex, Gemini (direct node launch still missing), Cursor, Antigravity.
6. Release gate: full suites on the final state (SecondBrain full regression was not re-run after the restore change; only the 14 backup/maintenance tests were), then publish `dist/` + `SHA256SUMS.*` + `BETA_NOTICE.txt`.

Decisions needed from you next session: key custody location; clean-machine option (VM / Sandbox needs admin to enable / throwaway user); whether to also delete `%TEMP%\restic-dl` and the older test residue listed in the handoff (§16).
