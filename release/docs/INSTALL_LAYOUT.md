# Respected Windows beta: what each step creates, updates and removes

Everything is per-user (no administrator). `%LOCALAPPDATA%` = `C:\Users\<you>\AppData\Local`.
Design and frozen contracts: `respected-hub/docs/decisions/REMOTE_CATALOG_AND_LIFECYCLE.md`.

## 0. Downloads
`RespectedHub-Windows-Setup.exe`, `RespectedSecondBrain-Windows-Setup.exe` and `RespectedOrchestrator-Windows-Setup.exe` are the
same small Hub installer. The product ones only open the Hub on that product (`--install=<product>`). **No product is inside an
installer**: the Hub downloads it from the signed catalog on GitHub Releases when the person confirms the install.

## 1. Running an installer (Hub only)
| Where | What |
|---|---|
| `%LOCALAPPDATA%\Respected\RespectedHub\app\` | `respected-hub.exe`, `unins000.exe`, `unins000.dat` |
| Desktop, Start Menu | `Respected Hub.lnk` |
| Installed apps (HKCU) | `Respected Hub` (Inno AppId `{5C0F6A1E-3B7D-4E2A-9D55-7A1C2F0E8B31}_is1`) |
| First Hub start | `%LOCALAPPDATA%\com.respected.hub\` (WebView2 cache, window state), `%APPDATA%\com.respected.hub\` (Hub settings), `...\RespectedHub\managed\catalog-index.json` (+ `.sig`, the verified catalog) |
| NOT created | no PATH change, no service, no scheduled task, no AI-client config, no vault, no Python outside the Hub |

## 2. Installing SecondBrain (Hub wizard)
| Where | What |
|---|---|
| `...\RespectedHub\managed\catalog\secondbrain-<version>\` | the downloaded, verified package (kept for offline repair; the engine installs from it and the Hub runs read-only status calls from it) |
| `...\RespectedHub\managed\journal\` | Activity log entries |
| `%LOCALAPPDATA%\Respected\RespectedBrain\app\` | `respectedbrain.exe` (frozen; `flush`, `compile`, `search`, `hook`, `mcp`... are sub-commands), `app\` runtime, resources, `distribution.json` |
| `%LOCALAPPDATA%\Respected\RespectedBrain\data\` | ownership manifest, config, `vaults\<id>\{state,cache}`, logs, transaction backups, receipts |
| The vault (chosen in the wizard, default `Documents\SecondBrain`) | notes folders, `.obsidian\`, `.gitignore`, `.respected.json`. **Nothing else** (no `.claude/.cursor/...`) |
| Only for AI clients the person ticked | global hooks/skills/instructions in `~\.claude`, `~\.codex`, `~\.cursor`, `~\.gemini`, `~\.agents`; MCP entries in their config files (only our entry is added) |
| Optional | scheduled task `respected-morning-briefing-<vault id>`, `Desktop\Respected Brain.lnk` |
| Installed apps | `Respected SecondBrain` (Uninstall = Hub confirmation, Modify = Hub product page) |

## 3. Installing Orchestrator
| Where | What |
|---|---|
| `...\RespectedHub\managed\catalog\{orchestrator,runtime}-<version>\` | downloaded, verified packages (Python runtime ~44 MB, only the first time) |
| `...\RespectedHub\managed\runtime\{current.json, versions\<python>\}` | the Hub's own CPython + agent framework |
| `...\RespectedHub\managed\orchestrator\{current.json, versions\<version>\}` | the Orchestrator program |
| `%LOCALAPPDATA%\Respected\RespectedOrchestrator\data\` | run records, receipts, settings (created when used) |
| Only if MCP access is enabled | `...\managed\orchestrator\mcp\respected-orchestrator-mcp.cmd` + an entry in the ticked clients' MCP files; the server hands each client the kod-orkestrasyon instructions |
| Installed apps | `Respected Orchestrator` |

## 4. Updates
* The Hub checks the catalog at start (and on *Ayarlar › Güncellemeleri denetle*). A newer release is **offered**: "Güncelle" / "Sonra".
* **Hub update**: the verified installer runs with `/SILENT /RELAUNCH=1` over the old Hub (same AppId); `managed`, products,
  data and vault are untouched; the Hub reopens.
* **Product update**: the new archive is downloaded and verified, installed as a new version, the product's own check runs, then
  the switch happens in one step. The older version and its archive are deleted; settings, data and vault stay. An update cannot be
  undone.
* Skipping releases is fine: the newest Hub and the newest product package do the work. Downgrades are not supported.
* `min_hub` in the catalog makes an old Hub ask for its own update first.

## 5. Uninstall
* **A product** (Hub › Kaldır, or Windows Installed apps › Uninstall): its program, its `managed` folder and cached archive, its
  data folder (with backups), our hook/MCP/schedule/shortcut entries (others' entries untouched) and its Installed-apps entry; the
  runtime goes with the last Python product. **Stays: the vault and its notes.** Not undoable.
* **The Hub** (Installed apps): asks "products too?" (default *No*).
  * *Yes*: everything above for every product, then `managed`, the Hub files, shortcuts, WebView cache and Hub settings.
  * *No*: products keep working (they do not need the Hub to run) but cannot be updated or repaired. The Hub executable is copied to
    `RespectedHub\maintenance\` and their Installed-apps entries point there; uninstalling the last product from Installed apps removes
    that copy too. Installing the Hub again finds the products as they are.
* Never touched: the vault, notes, anything not in our ownership records.

## 6. Known gaps
* Not seen on a clean machine yet with the remote catalog: first install, Hub self-update, Installed-apps uninstall/modify and the
  kept-products path (all covered by code and the end-to-end download test, not by a person clicking).
* Windows scheduled task naming/removal, Cline/Roo and Windsurf paths have unit/e2e coverage only.
* Products need the internet once to download; corporate proxies are whatever Windows' curl.exe supports.
