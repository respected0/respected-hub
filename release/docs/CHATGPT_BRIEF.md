# Respected: project brief for planning (state at v0.1.0-beta.1, 2026-10-18)

## What it is
Three separate GitHub repos (github.com/respected0): **respected-hub**, **respectedbrain** (local folder `respected-secondbrain`), **respected-orchestrator**. Windows beta, all at version **0.1.0**.
* **SecondBrain**: local, provider-neutral shared memory for AI tools, stored in the user's own Markdown/Obsidian vault. Frozen Windows engine (`respectedbrain.exe`; commands like `flush`, `compile`, `search`, `hook`, `mcp` are subcommands). Writes global hooks, skills and MCP entries into AI clients (Claude Code, Codex, Cursor, Gemini, Antigravity, Windsurf, Claude Desktop, VS Code agents) and can add a morning-briefing scheduled task and a desktop shortcut. The vault itself gets only notes + `.obsidian`.
* **Orchestrator**: multi-agent task engine (Python, `agent-framework-core`). No window of its own: used through the Hub workspace and an MCP server that AI clients can call.
* **Hub**: Tauri 2 + React desktop app. Installs/updates/repairs/removes the other two from a signed catalog shipped inside its installer; talks to products over their own protocol (RPP v1.0/1.1); shows activity, MCP access, product status.

## Release facts
* Installers (3 files, all = Hub + catalog; the product ones just open the Hub on that product): per-user, Inno Setup, **not Authenticode-signed** (no certificate budget) so SmartScreen warns.
* Authenticity: Ed25519. The Hub compiles in the public release key (`dabcb5265227bbdf`) and refuses any bundle not signed by it; `SHA256SUMS.txt` is signed too. The private key exists only as a passphrase-encrypted vault (scrypt + AES-256-GCM) outside the repos.
* Final folder layout: `%LOCALAPPDATA%\Respected\{RespectedHub\{app,managed}, RespectedBrain\{app,data}, RespectedOrchestrator\data}`. Vault stays in Documents.
* Uninstall rule: remove everything ours (program, data, backups, hooks, skills, MCP entries, task, shortcut), keep only the vault and notes. Updates are staged and verified before switching.
* Windows only. No auto-update (install the next beta over the old one). Catalog is bundled, not downloaded.

## Decisions already taken (do not re-argue without new evidence)
1. No Authenticode for now; Ed25519 chain + published SHA-256 instead.
2. SecondBrain's own SLSA/GitHub-Actions attestation is skipped in the beta (it cannot be produced locally); the Hub's Ed25519 check is the gate. Re-enable when releases come from CI.
3. Hooks are global only; nothing is written into the vault except notes and `.obsidian`.
4. One `Respected` folder, never move it again.
5. Three repos only; release tooling and docs live in the Hub repo under `release/`.
6. One version number for everything (0.1.0).
7. Uninstall is irreversible by design (no undo copies).

## Verified vs not verified
Verified: signing/verification chain, production build, clean-machine (Windows Sandbox) install/launch/uninstall of Hub and SecondBrain, hook/skill/MCP/task/shortcut removal with foreign entries preserved (fake AI-client folders), fresh-clone build checks.
Not verified: real sessions of Codex/Cursor/Gemini/Antigravity (Claude Code verified end to end), Orchestrator MCP end-to-end in a clean machine, repair, upgrade A→B→A, cancel/Hub-running uninstall, SmartScreen and the beta notice page seen by a human; the last three fixes (empty-folder pruning on uninstall, desktop shortcut, uninstall-log cleanup) are unit-tested only.

## Known weaknesses / candidate next work (for prioritising)
* Hub UX is functional, not good: "Uygulamalarım"/workspace flow, Orchestrator has no obvious front door (needs an "open workspace" entry + explanation), the SecondBrain "Kurulum" tab after install should show settings not the wizard (wording from the user was ambiguous), "Ürün durumu" page is diagnostics.
* After an Orchestrator/runtime update one previous version stays on disk (rollback copy); the user prefers no leftovers.
* Catalog could come from GitHub Releases (HTTPS download, signed, no git needed on the user's PC) instead of being bundled.
* Auto-update, macOS/Linux, Gemini workers on Windows, real-client test sessions, CI that builds releases (would also allow SLSA), Authenticode when affordable.
* Engine re-serialises JSON client configs (content equal, formatting changes).
* Orchestrator data (run records) is deleted on uninstall; consider an optional export.

## How to continue
Clone the three repos side by side and follow `respected-hub/release/BUILD_FROM_SCRATCH.md`. Full history and evidence: `respected-hub/release/docs/MASTER_HANDOFF.md`. Install/remove details: `INSTALL_LAYOUT.md`. Download text and limits: `RELEASE_NOTES_BETA.md`.
