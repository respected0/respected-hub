# Respected Windows beta: what each installer / action creates

Everything is per-user (no administrator). `%LOCALAPPDATA%` = `C:\Users\<you>\AppData\Local`.
Status: derived from the code and from two Windows Sandbox inventories. Items marked (unverified) have not been seen on a real machine yet.

## 0. The three downloads are the same Hub installer
`RespectedHub-Windows-Setup.exe`, `RespectedSecondBrain-Windows-Setup.exe`, `RespectedOrchestrator-Windows-Setup.exe` install exactly the same files
(Hub + the shipped catalog containing SecondBrain, Orchestrator and the Python runtime). They differ only in the last page: the product ones open the Hub
on that product's install page (`--install=<product>`). **No product is installed by running an installer**; the person confirms the install inside the Hub.

## 1. Running any installer (Hub files only)
| Where | What |
|---|---|
| `%LOCALAPPDATA%\Respected\RespectedHub\app\` | `respected-hub.exe`, `unins000.exe`, `unins000.dat`, `catalog\` (signed bundles: `orchestrator-0.1.0`, `secondbrain-dist-0.0.1`, `runtime-3.13.15`, each with `bundle.json` + payload) |
| Desktop | `Respected Hub.lnk` |
| Start Menu | `Programs\Respected Hub.lnk` |
| Registry (HKCU) | `Software\Microsoft\Windows\CurrentVersion\Uninstall\{5C0F6A1E-3B7D-4E2A-9D55-7A1C2F0E8B31}_is1` (Apps & features entry) |
| Created on first Hub start | `%LOCALAPPDATA%\com.respected.hub\` (WebView2 cache `EBWebView`, window state), `%APPDATA%\com.respected.hub\` (Hub settings/connections), `%LOCALAPPDATA%\Respected\RespectedHub\managed\` (journal; created when something is installed) |
| NOT created | no PATH change, no services, no scheduled task, no AI-client config, no vault, no Python outside the Hub |

## 2. Hub: install SecondBrain (wizard)
| Where | What |
|---|---|
| `%LOCALAPPDATA%\Respected\RespectedHub\managed\secondbrain\dist\0.0.1\` | verified staged copy of the engine bundle (used for read-only status calls and by the installer step) |
| `%LOCALAPPDATA%\Respected\RespectedHub\managed\journal\install-<id>.json` | Activity log entry |
| `%LOCALAPPDATA%\Respected\RespectedBrain\app\` | `respectedbrain.exe` (frozen; `flush`, `compile`, `search`, `hook`, `mcp`... are sub-commands, not loose scripts), `app\` runtime DLLs/tcl/tk, `respectedbrain\resources` (skills, instructions, vault template), `distribution.json` |
| `%LOCALAPPDATA%\Respected\RespectedBrain\data\` | `install-manifest.json` (ownership list), config store, `vaults\<id>\{state,cache}`, `logs\`, `backups\` (transaction/registration backups), `external-locks\`, `receipts\`, journals |
| The vault (chosen in the wizard, default `Documents\SecondBrain`) | notes folders (`000-Inbox` ... `900-Archive`, `daily`, `knowledge`), `.obsidian\`, `.gitignore`, `.respected.json` (vault identity). **No** `.agents/.claude/.cursor/.codex/.gemini` |
| Only for AI clients the person ticked (and only if the toggle "Global hooks" is on) | `~\.claude\settings.json`, `~\.claude\CLAUDE.md`, `~\.claude\skills\`; `~\.codex\hooks.json`, `config.toml`, `AGENTS.md`; `~\.cursor\hooks.json`, `rules\respected-brain.mdc`, `skills\`; `~\.gemini\GEMINI.md`, `config\hooks.json`, `config\skills\`, `settings.json`; `~\.agents\skills\` (Antigravity) |
| Only if "MCP" is on, per ticked client | an MCP entry in `~\.claude.json`, `~\.cursor\mcp.json`, `~\.codeium\windsurf\mcp_config.json`, `%APPDATA%\Claude\claude_desktop_config.json`, `~\.gemini\config\mcp_config.json`, Cline/Roo settings under `%APPDATA%\Code\User\globalStorage\...` (existing files are edited, only our entry is added) |
| Only if "Schedule" is on | a Windows scheduled task named `respected-morning-briefing-<vault id>` (unverified name on Windows) |
| Only if "Shortcut" is on | `Desktop\Respected Brain.lnk` |
| Python | none (engine is frozen) |

## 3. Hub: install Orchestrator
| Where | What |
|---|---|
| `...\RespectedHub\managed\runtime\{current.json, versions\3.13.15\}` | the Hub's own CPython + agent framework (auto-installed first, ~40 MB; needed only by Orchestrator) |
| `...\RespectedHub\managed\orchestrator\{current.json, versions\0.1.0\}` | the Orchestrator program (`src\`, `bundle.json`, `.respected\install.json`) |
| `%LOCALAPPDATA%\Respected\RespectedOrchestrator\data\` | run records, receipts, `integrations.json` (unverified exact file list; created when used) |
| Only if the person enables MCP access for clients | `...\managed\orchestrator\mcp\respected-orchestrator-mcp.cmd` + an entry in the ticked clients' MCP files (`~\.claude.json`, `~\.codex\config.toml`, `~\.cursor\mcp.json`, `~\.gemini\settings.json`, `~\.codeium\windsurf\mcp_config.json`, Claude Desktop config) |

## 4. Uninstall rules (what must be gone)
* Hub > Kaldır on a product: removes that product's program, its `managed` copy/staging, its `data` folder (incl. backups), our hook/MCP/schedule/shortcut entries (others' entries untouched), and the runtime after the last Python product. The `Respected` root is removed if empty. **Stays: the vault and its notes.**
* Uninstalling the Hub (Apps & features): asks whether to remove the products too (Yes = steps above for every product, then `managed`); then removes the Hub files, desktop/start-menu shortcuts, WebView cache, Hub settings, the uninstall log, and empty `com.respected.hub` / `Respected` folders. If "No": products and their data stay under `Respected\`.
* Never touched: the vault, notes, anything not listed in our ownership manifest.

## 5. Known gaps (will not surprise us later)
* Old test layout `Programs\RespectedHub` / `Programs\RespectedBrain` / `RespectedOrchestrator` is not migrated (clean installs only).
* Windows scheduled task naming/removal, Cline/Roo and Windsurf paths, and the exact Orchestrator data file list are unverified on a real client setup.
* The Sandbox has no AI clients, so client config writes/removals are covered by unit/e2e tests, not by a Sandbox run.
