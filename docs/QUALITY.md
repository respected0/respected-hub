# Kalite kapilari (QA-001)

QA-001 surekli bir kapi: her PR ve her surum adayinda asagidaki komutlar kosar. Windows native E2E
kirmizi ise surum yayimlanmaz. Kosulmayan veya kaniti olmayan her satir `dogrulanmadi` yazar.

## Komut matrisi

| Komut | Ne dogrular | Nerede |
|---|---|---|
| `npm run check` | `tsc --noEmit` + 134 arayuz birim testi | CI: windows, macOS, Linux |
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

- `npm run check`: 134/134 gecti (2026-11-05).
- `cargo test --no-fail-fast`: 151 birim + tum entegrasyon ikilileri gecti (0 basarisiz).
- `cargo test --release --lib production_build`: gecti (ayrinti icin CI isi).
- Native E2E: 30 PASS, 0 FAIL; rapor ve ekran goruntuleri bu makinede
  `.build-cache/hub-native-qa001-e2e-fixed/` altinda. Sahte kullanici klasoru Hub'in kendi istemci kesfine de
  uygulanir; test gercek kullanici AI yapilandirmasini okumaz. Son satir: `real AI client configuration untouched by this run`.
- CI: [run 41976817349](https://github.com/respected0/respected-hub/actions/runs/41976817349) uc iste de basarili
  (commit `8276c56`): windows 8m36s, ubuntu 5m3s, macos 4m58s. Windows `native-e2e` artefakti 30 PASS, 0 FAIL.

- CI: [run 41981505058](https://github.com/respected0/respected-hub/actions/runs/41981505058) uc iste de basarili
  (commit `4d72a6c`): ubuntu 4m53s, macos 5m13s, windows 11m9s. Windows `native-e2e` artefakti 30 PASS, 0 FAIL.

- CI: [run 41985091963](https://github.com/respected0/respected-hub/actions/runs/41985091963) uc iste de basarili
  (commit `fe75a62`): ubuntu 6m22s, macos 4m45s, windows 11m25s. Windows `native-e2e` artefakti 30 PASS, 0 FAIL.

- CI: [run 41988123558](https://github.com/respected0/respected-hub/actions/runs/41988123558) uc iste de basarili
  (commit `1716894`): macos 3m45s, ubuntu 5m59s, windows 8m30s. Windows `native-e2e` artefakti 30 PASS, 0 FAIL.
- CI: [run 41992973591](https://github.com/respected0/respected-hub/actions/runs/41992973591) uc iste de basarili
  (commit `94bab61`): ubuntu 5m57s, macos 5m22s, windows 8m34s. Windows `native-e2e` artefakti 30 PASS, 0 FAIL.
- Yerel `remote_catalog_e2e`: ag yokken onbellekten onarim basarili; bos onbellekte ilk indirme `CATALOG_UNAVAILABLE` ile reddedildi (2026-11-05).

Entegrasyon kapsam denetimi (2026-11-05, yerel calisma agaci):

- `cargo test` ile 8 entegrasyon ikilisi: **35/35 gecti** (catalog 6, claude 5, codex 4, cursor/gemini/opencode 5, inventory 3, plugins 4, resolver 5, skills 3).
- `cargo test --lib integrations::`: **108 gecti**, 43 filtrelendi.
- `npm run check`: **134/134 gecti**.
- Derlenmis `src-tauri/assets/third-party-catalog.json`: **62 kayit**; `catalog_e2e` ag olmadan okur.

Yayin hijyeni denetimi (2026-11-05, yayimlanmis `catalog` ve `v0.1.0-beta.1` asset'leri indirilerek):

- `catalog.json` (2586 bayt) ayrik Ed25519 imzasi derlenmis uretim anahtari `dabcb5265227bbdf` ile
  `respected-catalog-v1\n` + dosya baytlari uzerinde dogrulandi; `catalog.json.sig` 106 bayt.
- `catalog.json` SHA-256 `f2f9261797cc5771b119461cf3f45cd70b09ae636b2479ad1879cc226d564553`; GitHub asset ozetiyle ayni.
- Hub kurucusu SHA-256 `38e8f32cd4622727ed467c4b28c43c187f7263442aa54a7b8ff7d648ceccf194` (4182482 bayt);
  katalog kaydi ve GitHub asset ozetiyle ayni.
- `Get-AuthenticodeSignature`: `NotSigned`. SmartScreen uyarisi suruyor; kod imzalama sertifikasi yok.
- `0.1.0b1` (urun) ile `0.1.0-beta.1` (etiket) farki hata degil: `lifecycle::version_key` iki bicimi esit sayar ve
  `lifecycle/mod.rs:287` bunu test eder.

## Dogrulanmadi

- `--real-semantic`: anlamsal arama modelinin gercek indirmesi bu kosuda calistirilmadi.
- Temiz VM'de ilk kurulum, Hub'in kendini guncellemesi ve Yüklü uygulamalar'dan kaldirma elle denenmedi
  (bkz. `docs/ARCHITECTURE.md` acik sinirlar).
- Gercek eski surumden guncelleme/geri alma ve kendini guncelleme simdilik denenemez: yayimlanmis tek surum ve
  gelistirme surumu de `0.1.0-beta.1`; daha yeni bir surum yok.
- macOS/Linux'ta native GUI: `docs/PLATFORMS.md` geregi desteklenmiyor/dogrulanmadi.
