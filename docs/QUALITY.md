# Kalite kapilari (QA-001)

QA-001 surekli bir kapi: her PR ve her surum adayinda asagidaki komutlar kosar. Windows native E2E
kirmizi ise surum yayimlanmaz. Kosulmayan veya kaniti olmayan her satir `dogrulanmadi` yazar.

## Komut matrisi

| Komut | Ne dogrular | Nerede |
|---|---|---|
| `npm run check` | `tsc --noEmit` + 133 arayuz birim testi | CI: windows, macOS, Linux |
| `cargo test --manifest-path src-tauri/Cargo.toml --no-fail-fast` | 151 birim + tum fixture/entegrasyon ikilileri | CI: windows |
| `cargo test --release --manifest-path src-tauri/Cargo.toml --lib production_build` | Uretim guven kumesi; gelistirme anahtari kabul edilmez | CI: windows, macOS, Linux |
| `python common/release/tools/acceptance/accept_hub.py` | Yukaridaki Rust + TS adimlari, ortak urun sozlesmesi ve gercek AI ayarlarinin once/sonra karsilastirmasi | elle |
| `npx tauri build --debug --no-bundle` ardindan `node tests/native/hub_native_e2e.mjs src-tauri/target/debug/respected-hub.exe --report <dir>` | Derlenmis exe, WebView2 arayuzu, urunlerin kendi imzali kuruculari | CI: windows |

Native surucu on kosullari: kardes `respected-brain` ve `respected-orchestrator` depolarina `pip install -e` yapilmis
olmalidir; Inno Setup 6 (`ISCC.exe`) PATH'te veya `INNO_COMPILER` ile gosterilmis olmalidir; gelistirme imza anahtari
`python common/release/tools/signing/bundle_signing.py init` ile uretilir. Surucu makinede kurulu bir Respected urunu
gorurse baslamaz, sahte bir kullanici klasoru kullanir ve gercek AI istemci ayarlarini once/sonra karsilastirir.

## Zorunlu fixture matrisi

| Durum | Kapsayan testler |
|---|---|
| Normal | `inventory_e2e`, `claude_adapter_e2e`, `codex_adapter_e2e`, `cursor_gemini_opencode_adapter_e2e`, `catalog_e2e` |
| Bozuk/bos config | Adapter e2e bozuk JSON/TOML vakalari; `inventory_e2e` `Case::Broken` fixture'lari |
| Ag yok | `catalog_e2e` derlenmis katalog; `remote_catalog_e2e` onarim/onbellek yolu; native E2E yerel katalog |
| Non-ASCII yol | `inventory_e2e` unicode ve bosluklu yol fixture'i; `model` yol kimligi testi |
| Crash/fault injection | `undo` bozuk/gelecek kayit ve islem kimligi testleri; `installer` basarisiz dogrulama testi |
| Secret redaction | `codex_adapter_e2e` ve `cursor_gemini_opencode_adapter_e2e` token/`sk-live-*` vakalari |
| Sahiplik | `planner` detach/Respected sahiplik testleri; `apply` yalniz secilen istemciyi ayirma testi |
| Rollback/undo | `apply` undo sonrasi hash geri donusu, coklu istemci tek islem, dis degisiklikte yazmama testleri |
| Native GUI | `tests/native/hub_native_e2e.mjs` (kurulum, sihirbaz, ayar, MCP, hafiza, onarim, kaldirma) |

## Kanit

2026-11-05, Windows dev build (`npx tauri build --debug --no-bundle`):

- `npm run check`: 133/133 gecti.
- `cargo test --no-fail-fast`: 151 birim + tum entegrasyon ikilileri gecti (0 basarisiz).
- `cargo test --release --lib production_build`: gecti (ayrinti icin CI isi).
- Native E2E: 30 PASS, 0 FAIL; rapor ve ekran goruntuleri bu makinede
  `.build-cache/hub-native-qa001-e2e-fixed/` altinda. Sahte kullanici klasoru Hub'in kendi istemci kesfine de
  uygulanir; test gercek kullanici AI yapilandirmasini okumaz. Son satir: `real AI client configuration untouched by this run`.
- CI: [run 41975371304](https://github.com/respected0/respected-hub/actions/runs/41975371304) uc iste de basarili
  (commit `3f90dbd`): windows 10m56s, ubuntu 5m3s, macos 3m16s. Windows `native-e2e` artefakti 30 PASS, 0 FAIL.

## Dogrulanmadi

- `--real-semantic`: anlamsal arama modelinin gercek indirmesi bu kosuda calistirilmadi.
- Temiz VM'de ilk kurulum, Hub'in kendini guncellemesi ve Yüklü uygulamalar'dan kaldirma elle denenmedi
  (bkz. `docs/ARCHITECTURE.md` acik sinirlar).
- macOS/Linux'ta native GUI: `docs/PLATFORMS.md` geregi desteklenmiyor/dogrulanmadi.
