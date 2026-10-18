# Respected Windows Beta 0.1.0

Respected Hub installs and manages **SecondBrain** (shared AI memory in your own note vault) and **Orchestrator** (multi-agent task engine).
Windows 10/11, per-user install, no administrator rights.

## Download
| File | What it does |
|---|---|
| `RespectedHub-Windows-Setup.exe` | Installs the Hub |
| `RespectedSecondBrain-Windows-Setup.exe` | The same Hub; opens on SecondBrain's install page |
| `RespectedOrchestrator-Windows-Setup.exe` | The same Hub; opens on Orchestrator's install page |

Running an installer installs only the Hub. Products are installed inside the Hub after you confirm.

## Windows will warn you (unsigned beta)
The installers are **not Authenticode-signed** (no paid certificate yet). Windows shows "Unknown publisher" / SmartScreen: choose
*More info > Run anyway* only if the checksum below matches.
Verify before running: `Get-FileHash <file> -Algorithm SHA256` must equal the line in `SHA256SUMS.txt`.
`SHA256SUMS.sig` is an Ed25519 signature of that list (release key id `dabcb5265227bbdf`); the products inside the Hub are signed with the same key and the Hub refuses anything else.

## Where things go
Everything lives under `%LOCALAPPDATA%\Respected\` (`RespectedHub`, `RespectedBrain`, `RespectedOrchestrator`). Your note vault stays where you choose (default `Documents\SecondBrain`).
Uninstalling a product removes its program, data, backups and the Respected entries it added to your AI tools (hooks, skills, MCP, scheduled task, shortcut); **only the vault and your notes stay**. This cannot be undone.

## Known limits of this beta
* Windows only. No automatic updates: install the next beta over the old one.
* Not code-signed; SmartScreen warning as above.
* SecondBrain's own SLSA build attestation is not used in this beta; authenticity is the Ed25519 signature checked by the Hub.
* AI clients: Claude Code is verified end to end; Codex, Cursor, Gemini and Antigravity registrations are covered by tests but not yet by real sessions. Gemini workers are unavailable on Windows.
* After an Orchestrator or Python-runtime update one previous version stays on disk until the next update.
* The Hub's interface is functional, not final.
* Report problems with the Hub's *Etkinlik* log text and your Windows version.
