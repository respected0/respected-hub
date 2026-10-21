# Respected Windows Beta 0.1.0-beta.1

Respected Hub installs and manages **SecondBrain** (shared AI memory in your own note vault) and **Orchestrator** (multi-agent task engine).
Windows 10/11, per-user install, no administrator rights.

## Download
| File | What it does |
|---|---|
| `RespectedHub-Windows-Setup.exe` | Installs the Hub |
| `RespectedSecondBrain-Windows-Setup.exe` | The same Hub; opens on SecondBrain's install page |
| `RespectedOrchestrator-Windows-Setup.exe` | The same Hub; opens on Orchestrator's install page |

The installer installs only the Hub. Products are downloaded and installed inside the Hub after you confirm (internet needed once).

## Windows will warn you (unsigned beta)
The installers are **not Authenticode-signed** (no paid certificate yet). Windows shows "Unknown publisher" / SmartScreen: choose
*More info > Run anyway* only if the checksum below matches.
Verify before running: `Get-FileHash <file> -Algorithm SHA256` must equal the line in `SHA256SUMS.txt`.
`SHA256SUMS.sig` is an Ed25519 signature of that list (release key id `dabcb5265227bbdf`); the product catalog and every product
package are signed with the same key and the Hub refuses anything else.

## Updates
The Hub tells you when a new Hub or product version exists and asks before installing it. Nothing updates silently.
Updates keep your settings, data and vault; the previous version is removed. Going back to an older version is not supported.

## Where things go
Everything lives under `%LOCALAPPDATA%\Respected\` (`RespectedHub`, `RespectedBrain`, `RespectedOrchestrator`). Your note vault stays
where you choose (default `Documents\SecondBrain`; **do not put it in a OneDrive-synced folder** for this beta).
SecondBrain and Orchestrator also appear in *Windows Settings › Apps*, where you can uninstall or modify them.
Uninstalling a product removes its program, data, backups and the Respected entries it added to your AI tools (hooks, skills, MCP,
scheduled task, shortcut); **only the vault and your notes stay**. This cannot be undone.

## Known limits of this beta
* Windows only.
* Not code-signed; SmartScreen warning as above.
* AI clients: Claude Code is verified end to end; Codex, Cursor, Gemini and Antigravity registrations are covered by tests but not yet by
  real sessions (treat them as experimental). Gemini workers are unavailable on Windows.
* The Hub's interface is functional, not final.
* Report problems with the Hub's *Etkinlik* log text and your Windows version at
  https://github.com/respected0/respected-hub/issues.
