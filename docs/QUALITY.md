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

Son kanit (2026-11-05, Windows dev build, `npx tauri build --debug --no-bundle`):

- `npm run check`: 134/134 gecti (2026-11-06'da `0.1.0-beta.2` uzerinde tekrar dogrulandi).
- `cargo test` (2026-11-05, `14b1609` oncesi calisma agaci): 151 birim + 35 entegrasyon kontrati (8 ikili: catalog 6,
  claude 5, codex 4, cursor/gemini/opencode 5, inventory 3, plugins 4, resolver 5, skills 3) + 1 uretim guven testi
  gecti; 1 kurucu testi bilerek ignored. beta.2 uzerinde yeniden kosulmadi.
- Derlenmis `src-tauri/assets/third-party-catalog.json`: 62 kayit; `catalog_e2e` ag olmadan okur.
- Yerel `remote_catalog_e2e`: ag yokken onbellekten onarim basarili; bos onbellekte ilk indirme `CATALOG_UNAVAILABLE`
  ile reddedildi.
- CI: [run 42026915332](https://github.com/respected0/respected-hub/actions/runs/42026915332) (commit `14b1609`,
  belge-only): Windows 8m9s, macOS 4m49s, Ubuntu 5m55s; `native-e2e` **30 PASS, 0 FAIL**, `ALL PASSED`; rapor son satiri
  `real AI client configuration untouched by this run`. Ilk iki denemede test kosucusu kaynakli flake (Windows
  `Orchestrator settings applied and read back` zaman asimi, gecici dizin temizliginde `EPERM`); ayni kod `b9f205d`
  kosusunda ilk denemede yesildi. Bu kosudan sonraki commit'ler (beta.2 surumu, `accept_update.py`) icin CI sonucu
  burada kayitli degil.

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
- Gercek eski surumden guncelleme/geri alma ve Hub'in kendini guncellemesi: `0.1.0-beta.2` yayimlandi
  (2026-11-05), CI `update` isi (`accept_update.py`) yazildi ama yayimlanmis kurucularla henuz kosulup
  dogrulanmadi.
- macOS/Linux'ta masaustu GUI ve paketleme: `docs/PLATFORMS.md` geregi dogrulanmadi; v0.1.0 resmi surum kapsamindadir, beta.1 sonrasi surumun kapisi degildir.
