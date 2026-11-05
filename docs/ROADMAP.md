# Respected Hub — Geliştirme Yol Haritası

> **Durum (2026-11-06):** `0.1.0-beta.2` yayımlı (GitHub Release `v0.1.0-beta.2`, 2026-11-05). Bu belge beta.2'den sonraki **tek kalıcı plandır**: beta.3 → beta.4 → beta.5 → v1.0 RC → v1.0 Stable → v1.1+.
> Bu belge bir **hedef plandır**; işlerin tamamlandığını iddia etmez. Bir madde yalnızca *kaynak kod + test + sürüm kanıtı* varsa `[x]` olur. Belgedeki "Mevcut durum" satırları 2026-11-06'da çalışma ağacında (`main`, `fec33c3`) kodun ve yayımlı sürümlerin gerçekten incelenmesiyle yazılmıştır.
> Bu belge tek kalıcı kaynaktır; başka bir planlama belgesine dayanmaz. Silinmiş eski kök roadmap dosyalarının (`ROADMAP.md`, `IMPLEMENTATION_ROADMAP.md`, `fec33c3` öncesi) karşılığı §12'deki eşleştirme tablosudur.

## İçindekiler

1. Ürün vizyonu ve v1.0 kapsamı
2. Depoda mevcut altyapı ve doğrulama bulguları
3. Ortak güvenlik, gizlilik ve test ilkeleri (öncelik, durum işaretleri, Definition of Done)
4. Sürüm kapıları, bağımlılık düzeni ve PR sırası
5. Beta.2 — yayımlı sürümün açık doğrulama işleri
6. Beta.3 — Bağımsız Hub (B3-01..B3-10)
7. Beta.4 — Gerçek mağaza (B4-01..B4-16)
8. Beta.5 — Salt okunur yerel AI kütüphanesi (B5-01..B5-13)
9. v1.0 Release Candidate (RC-01..RC-07)
10. v1.0 Stable (V1-01..V1-03)
11. v1.1+ — Bilinçle ertelenenler (PARK-01..PARK-10)
12. Eski roadmap görevlerinin eşleştirme tablosu
13. Kalite göstergeleri ve tek bakışta kabul tablosu
14. Araştırma, rakip kanıtı ve deneyler
15. Riskler ve risk azaltma
16. Açık kararlar ve çelişki çözümleri
17. Sürüm kapsamı özeti ve değişiklik kaydı

---

# 1. Ürün vizyonu ve v1.0 kapsamı

## 1.1. Tek cümlelik ürün tanımı

**Respected Hub, vibe coder'lar için Steam hissinde, GitHub ve açık kaynak ekosistemini keşfetmeye odaklı bir masaüstü mağazası ve kullanıcının bilgisayarındaki AI geliştirme ekosistemini salt okunur görüntüleyen bir kütüphanedir.**

Kullanıcı akışı: **Keşfet → anla → karşılaştır → kaydet/favorile → kaynak/dokümantasyona git**; isterse **Kütüphanem → hangi AI aracında, nerede, hangi kapsamda tanımlı bak**.

"Steam" benzetmesi arayüzü birebir kopyalamak değildir: vitrin, kart kalitesi, kategori, editoryal koleksiyon, ürün sayfası, keşif hissi ve favoriler içindir. "GitHub" benzetmesi kaynak şeffaflığı, lisans, geliştirici bilgisi ve açık depo bağlantıları içindir.

## 1.2. v1.0 kapsamına GİREN işler

- Gerçek bir mağaza: vitrin, editoryal seçkiler, keşif akışları, kategoriler, konu etiketleri, arama, sıralama, çok boyutlu filtre, detay sayfası, benzer ürünler, açık kaynak/doğrulama bilgisi, yerel favoriler / sonra bak.
- MCP, Skill, Plugin, uygulama (App), Rule/Instructions ve gerekiyorsa başka türleri **doğru sınıflandıran** katalog. Bir repo birden çok artifact içerebilir.
- Katalog metadata'sının **offline açılması**; kullanıcının başlattığı, doğrulanan imzalı katalog yenilemesi. Harici GitHub/dokümantasyon bağlantılarında internet gerekeceği şeffaf belirtilir.
- Yerel AI araçlarını bulma ve **yalnızca okuma**: ilk resmi matris **Claude Code, Codex, Cursor, Gemini CLI, OpenCode**. Yeteneği doğrulanmayan alan "desteklenmiyor / bilinmiyor" olur.
- Kullanıcı hesabı olmadan; kullanıcı onaylı proje kökü seçimiyle proje/global görünürlüğü. Hub yalnızca **kendi** favori ve proje kayıtlarını yazabilir.
- Katalog girdisi ile yerel envanterin **kanıta dayalı** eşleştirilmesi. "Bu araçla uyumlu" ile "bu araçta yapılandırılmış" kesinlikle ayrılır.
- Sağlam gizlilik, erişilebilirlik, Türkçe/İngilizce içerik arama, anlaşılır hata/boş durumlar, görsel kalite, Windows native testleri, kaynak/lisans dürüstlüğü.
- Brain ve Orchestrator'ın **normal katalog öğesi** gibi gösterilebilmesi; hiçbir özel yetki, sayfa veya yönetim akışı olmaması.
- Hub'ın **kendi** self-update, installer/uninstaller ve imzalı katalog güncellemesi (korunur).

## 1.3. v1.0 kapsamından ÇIKARILAN işler (açık ürün kararı)

- Herhangi bir AI aracı konfigürasyonunda otomatik **enable/disable, toggle, bind/unbind, move/copy scope, global↔proje taşıma**.
- MCP/Skill/Plugin otomatik **kur/kaldır/güncelle**, otomatik kurulum komutu, `npx`/`uvx`/shell çalıştırma, dosya üstüne yazma, symlink oluşturma/kaldırma.
- `integrations.apply`, `rollback`, `undo`, işlemsel yazıcıyı UI'dan erişilebilir yapma; "düzelt" diyerek konfigürasyona müdahale (Setup Doctor yalnızca bulgu gösterir).
- Brain/Orchestrator'ın Hub üzerinden özel yönetimi: kurulum, güncelleme, onarım, kaldırma, Brain hafızası, Orchestrator görev/kampanya, özel ayarlar, RPP bakım akışları.
- GitHub depolarına PR/fork yazma, kullanıcı yorumlarının sunucuda tutulması, profil hesabı, bulut senkronizasyon, telemetri, AI API proxy/gateway, token/kota yönetimi.
- Kanıtsız "AI şu Skill'i kullandı", "kesin güvenli", "en iyi/en popüler" sıralama. Sıralama varsa yöntemi ve kanıtı olmalı.
- Doğrulanmamış macOS/Linux desteğini yayımlanmış yetenek olarak pazarlama (v1.0 yalnızca **Windows x64**).

**Önemli ürün kararı:** v1.0'da "Kur" düğmesi yoktur; üçüncü taraf ürünler için **"GitHub'da aç", "Dokümantasyonu aç", "Favorilere ekle"** eylemleri vardır. Harici kaynak bağlantısının açılması kurulum değildir.

## 1.4. Kullanıcıya verilecek iki temel söz

1. **"Aradığın AI aracını bulmana ve anlamana yardım edeceğim."** Arama/filtre, ürün sayfası, örnek senaryolar, doğru kaynak, alternatifler, gereksinimler.
2. **"Bilgisayarındaki AI kurulumlarının nerede tanımlı olduğunu göstereceğim; onları değiştirmeyeceğim."** MCP/Skill/Plugin, istemci, global/proje, kaynak ve belirsizlik kanıtı.

## 1.5. Mimari kırmızı çizgi

```text
                 React / TypeScript (Tauri WebView)
 Store · Search · Detail · Saved · Library · Projects · Settings
                           |
                  dar/tipli Tauri IPC
                           |
             Rust (çevrimdışı ve yerel çekirdek)
    +----------------------+---------------------+
    |                      |                     |
 Store Metadata       Read-only Inventory     Hub Self-Update
 katalog/imza/cache   adapters/resolver       installer/release
    |                      |                     |
    v                      v                     v
 kaynak GitHub URL   izinli config okuma      yalnız Hub'ın kendi dosyaları

YASAK: webview'dan rastgele shell/URL/dosya yazma, AI config'e yazma,
       üçüncü taraf kaynak kodu indirme/çalıştırma/kurma.
```

Üç veri kategorisi birbirine karıştırılmaz:

1. **Katalog metadata'sı:** sürümlü, kaynak gösterilen, imzalı Hub kataloğu (derlenmiş snapshot + doğrulanmış uzak güncelleme + önbellek).
2. **Yerel envanter:** AI istemcisinin **gerçek** yapılandırması; Hub'ın önbelleği değil.
3. **Hub'ın kendi kayıtları:** favoriler, kullanıcının seçtiği proje kökleri, Hub ayarları.

---

# 2. Depoda mevcut altyapı ve doğrulama bulguları

> 2026-11-06'da `main` (`fec33c3`) üzerinde kod okunarak ve komut çalıştırılarak çıkarılmış başlangıç fotoğrafıdır. Her görev başlarken ilgili satır yeniden doğrulanır. İşaretler: **Var** = kod mevcut, **Doğrula** = kod var ama bu planın beklediği davranış kanıtlanmadı, **Eksik** = yok, **Borç** = var ama plana aykırı/hatalı.

## 2.1. Korunacak / dönüştürülecek / kaldırılacak yatırımlar

| Alan | Mevcut konum | Durum | Karar |
|---|---|---|---|
| Ürün gezinmesi | `src/shell/App.tsx` (rotalar `home`/`discover`/`apps`/`app`/`settings`), `src/shell/apps.ts` (`HUB_NAV`: Ana Sayfa, Keşfet, Kütüphane, Ayarlar; `APPS` = yalnız Brain ve Orchestrator) | Borç | Beta.3'te özel yolları kaldır; Store merkezli IA kur (B3-03, B3-07) |
| Ürün sayfaları | `src/products/{brain,orchestrator}/` (hafıza, kampanya, görev, ajan, MCP erişimi, kanıt modelleri ve görünümleri), `src/shell/views/{AppWorkspace,Products,LifecycleControls,InstallFacts,Hello}.tsx`, `Discover.tsx` (öne çıkan = Brain/Orchestrator kurulum paneli), `Library.tsx` | Borç | Kaldır (B3-03) |
| Ürün yönetim backend'i | `src-tauri/src/{hub,lifecycle,products,rpp.rs,commands/{engine,manage,memory}.rs}`, `settings.rs` (Brain/Orchestrator kök ve exe yolları), `process.rs` | Borç / karışık | Ürün yönetimini kaldır; Hub self-update'i ayır ve koru (B3-04) |
| Kayıtlı Tauri komutları | `lib.rs` `generate_handler!`: 37 komut — `engine_request`, `rpp_request`, `product_status`, `development_build`, `campaign_list`, `lifecycle_status/plan/apply`, `orchestrator_mcp_plan/apply`, 12 adet `memory_*`, `startup_intent`, `activity_list`, `catalog_refresh`, `catalog_set_channel`, `hub_update_status/install`, `detect_ai_clients`, `integrations_request`, `projects_list/add/remove`, `get_settings`, `save_settings` | Karışık | §6 B3-04 tablosundaki sınıflandırma |
| Üçüncü taraf katalog | `src-tauri/src/integrations/catalog.rs` (iç içe `include_str!`, `REMOTE_URL`, `REMOTE_DOMAIN`, 4 MiB sınır, kanallar stable/beta, `deny_unknown_fields`), `assets/third-party-catalog.json` | Var | Yeniden kullan; şema v2 (B4-02) |
| Katalog güvenlik önizlemesi | `integrations/security.rs`, `reviewed()` | Var | Yeniden kullan; "güvenli" iddiası yok |
| Yerel keşif | `integrations/adapters/{claude,codex,cursor,gemini,opencode,common}.rs`, `discovery.rs` | Var, Doğrula | Mevcut doğruluğu test et; yeniden yazma (B5-01..03) |
| Skill/Plugin | `integrations/{skills,plugins,resolver,inventory}.rs` | Var, Doğrula | Kapsam/kimlik/kanıt ayrımını iyileştir (B5-04..06) |
| Envanter UI | `views/IntegrationsLibrary.tsx`, `integrationMatrix.ts` | Var | Beta.5'te ürünleştir |
| Proje seçimi | `integrations/projects.rs` (`load/add/remove`), `views/Projects.tsx`, `projectScope.ts`, IPC `projects_list/add/remove` | Var | Koru (B5-09) |
| Yazma/planlama kodu | `integrations/planner.rs` (1307 satır), `apply.rs` (2197), `undo.rs` (424), `profiles.rs` (370) | **Borç** | Üretim yoluna kapat/çıkar (B3-05) |
| Yayın | `common/release/`, `tests/native/hub_native_e2e.mjs` (363 satır), `packaging/windows/hub_setup.iss`, `.github/workflows/{ci,audit}.yml` | Var | Yeni Hub testlerine uyarla; kendi updater/installer'ı koru |

## 2.2. Doğrulama bulguları (2026-11-06)

Plana doğrudan etki eden, kodda doğrulanmış gerçekler. Her bulgunun karşılık geldiği görev sağda.

| # | Bulgu | Kanıt | İlgili görev |
|---|---|---|---|
| F-01 | `integrations_request` yalnızca `inventory`, `clients`, `catalog`, `catalog_refresh`, `plan` kabul eder; `apply`/`undo`/`shell`/yol/URL/komut alanları `deny_unknown_fields` ve `only_closed_actions_are_accepted` testiyle reddedilir. **`apply`/`undo` IPC'den erişilemez — doğrulandı.** | `integrations/api.rs` | B3-05 |
| F-02 | **`plan` eylemi canlıdır:** `remove_mcp`, `detach_mcp`, `enable_mcp`, `disable_mcp`, `move_mcp`, `copy_mcp`, `bind_mcp` önizlemesi üretir. Yazmaz (test: dosya baytı değişmez) ama v1.0'da plan önizlemesi ürün özelliği değildir. | `api.rs` `Action::Plan`, `planner.rs` | B3-05 |
| F-03 | Üretim yolunda her açılışta `integrations::undo::Journal::default().recover()` çalışır (çökmüş bir üçüncü taraf yazımının dosyalarını geri yükler). v1.0'da yazma olmadığı için bu çağrı **kaldırılmalıdır**; kalırsa "AI config'e yazma 0" iddiasıyla çelişir. | `lib.rs` `.setup` | B3-05 |
| F-04 | `planner/apply/undo/profiles` `lib.rs`/`integrations/mod.rs` içinde koşulsuz derlenir; yalnız `dev-trust` özelliği vardır, yazma için feature kapısı yoktur. Adapter'lar `plan = true`, `apply = false`, `undo = false` ilan eder. | `Cargo.toml [features]`, `ARCHITECTURE.md` | B3-05 |
| F-05 | `orchestrator_mcp_plan` / `orchestrator_mcp_apply` Hub komutlarıdır: Orchestrator'ın `integrations.apply`'ını çağırarak **AI istemci yapılandırmalarına dolaylı yazma** yapar. | `commands/manage.rs`, `ARCHITECTURE.md` "Orchestrator'ın AI araçlarına MCP kaydı" | B3-04, B3-05 |
| F-06 | Hub self-update kodu **Hub + ürün** için ortak imzalı indeksi kullanır: `lifecycle/catalog.rs` (`hub_update`, `install_hub_update`, `entry(&index,"hub",...)`), indeks şeması `{schema, sequence, hub:{stable,beta}, products:{brain,orchestrator}, blocked}`. İndiriş SHA-256+boyut doğrular, `/SILENT /NOCANCEL /RELAUNCH=1` ile başlatır. Yayımlı `catalog` Release'i bu indeksi taşır. Self-update ve ürün yaşam döngüsü aynı modülde iç içe. | `lifecycle/catalog.rs` | B3-04 |
| F-07 | **Yayımlı beta.2 Hub'ları aynı `catalog` indeksini okur.** İndeksten `products` kaldırılırsa/şema değişirse beta.2 Hub'ının self-update'i bozulabilir; bu yayın tarafı kararı kod değişikliğinden önce doğrulanmalıdır. | `gh release list` (3 release: `v0.1.0-beta.2`, `catalog`, `v0.1.0-beta.1`) | B3-04 |
| F-08 | Üçüncü taraf katalog için **uzak yenileme kaynağı yayımlı değildir:** `REMOTE_URL` `…/releases/download/third-party-catalog/catalog.json` gösterir, ancak GitHub Release listesinde `third-party-catalog` yoktur. Uzak yenileme yalnızca yerel `remote_catalog_e2e` ile kanıtlıdır. | `catalog.rs:19`, `gh release list` | B4-12 |
| F-09 | Katalog içeriği: **62 kayıt** — 33 MCP, 7 Skill, 18 Plugin, 2 Rule, 2 Other; tamamı `github.com`; `provenance` 45 `official` / 17 `community`; benzersiz id 62; **lisans alanı 0/62 dolu**; ayrı repo başına tek kayıt (çoklu artifact yok). | `third-party-catalog.json` analizi | B3-08, B4-02, B4-03 |
| F-10 | `CatalogKind` yalnız `mcp/skill/plugin/rule/other`; **`app` yok.** 9 istemci uygulaması (Claude Code, Codex CLI, Gemini CLI, OpenCode, Goose, Cline, Continue, Roo Code, Aider) ve sohbet uygulamaları (LibreChat, Open WebUI, LobeChat) `plugin` tipindedir. Brain/Orchestrator için de tür yoktur. | `catalog.rs`, katalog analizi | B3-06 |
| F-11 | Katalog kalite sorunları: kategori yazımı tutarsız (`Veritabanı`/`Veritabani`, `Geliştirici Araçları`/`Gelistirici Araclari`, `Dokümantasyon`/`Dokumantasyon`); 62 kaydın 9'unda Türkçe karakter var, çoğu ASCII'leştirilmiş; `Koleksiyon` kategorisinde "Awesome …" listeleri `skill`/`other` olarak tek artifact gibi duruyor. | katalog analizi | B3-08, B4-03 |
| F-12 | `CatalogEntry` **`deny_unknown_fields`** ile ayrıştırılır. Yeni alanlı (v2) bir uzak katalog, eski Hub'lar (beta.2/beta.3) tarafından **bütünüyle geçersiz** sayılır. Şema v2 yayını ayrı kanal/URL ile yapılmalı ya da eski istemci davranışı bilinçli kabul edilmelidir. | `catalog.rs` `CatalogEntry` | B4-02, B4-12 |
| F-13 | Harici bağlantı: `ThirdPartyCatalog.tsx` `<a href={source_url} target="_blank" rel="noreferrer">GitHub'da aç</a>` kullanır. Tauri tarafında **opener/shell eklentisi yoktur** (yalnız `dialog`, `single-instance`); capability yalnız `dialog:allow-open` + pencere izinleri; CSP `default-src 'self'; connect-src ipc: …; style-src 'self'; img-src 'self' data:`. WebView2'de `target=_blank` bağlantısının sistem tarayıcısında **gerçekten açıldığı doğrulanmadı.** URL doğrulaması (`is_verified_source_url`) yalnız `https://github.com/{owner}/{repo}` kabul eder. | `capabilities/default.json`, `tauri.conf.json`, `Cargo.toml` | B4-11 |
| F-14 | Favori / Sonra bak yok. Hub'ın kalıcı kayıtları: `connections.json` (yalnız dev), `projects.json`, `profiles.json`, `integration-baseline.json`, yaşam döngüsü journal'ları. Tarayıcı tarafı: `sessionStorage 'hub.route'` (+`hub.intent.done`), `localStorage` yalnız `WhatsNew`. Bugünkü arama: `thirdPartyCatalog.ts` `filterCatalog` (id/ad/açıklama/kategori üzerinde `toLocaleLowerCase('tr')` alt-dize); eşanlam, sıralama, vitrin, detay sayfası yok. | `settings.rs`, `App.tsx`, `thirdPartyCatalog.ts` | B4-04, B4-05, B4-10 |
| F-15 | Eski rota kurtarma zaten vardır (`App.tsx` `restore()`: "eski Hub'ın kaydettiği rota"); Brain/Orchestrator rotaları kaldırıldığında bu fonksiyon yeni rota kümesine göre güncellenmelidir. | `App.tsx:16,28` | B3-03 |
| F-16 | Test envanteri: `npm run check` **134/134 geçti (2026-11-06'da bu oturumda yeniden koşuldu)**; QUALITY.md: 151 birim + 35 entegrasyon + 1 üretim güven testi (cargo bu oturumda yeniden koşulmadı; `src-tauri/tests/integrations/*.rs` dosyalarında 36 `#[test]` sayıldı — `inventory_e2e` 3 yerine 4: QUALITY.md sayısı **doğrulanacak**). Brain/Orchestrator'a bağlı testler: `tests/brain/*`, `tests/orchestrator/*`, `tests/shared/{rpp,rppCache,productClient,productsModel}.test.ts`, `src-tauri/tests/{rpp_e2e.rs,brain/,orchestrator/,lifecycle/installer_e2e.rs}`, `tests/native/hub_native_e2e.mjs`. | komut çıktısı, dosya sayımı | B3-09 |
| F-17 | Çapraz depo sözleşmesi: `docs/PRODUCT_CONTRACT.md` Brain ve Orchestrator depolarında **aynı kopya** olarak tutulur; uyum testi `common/conformance/test_product_contract.py`. Hub'ın ürün yönetimi kalktığında bu sözleşmenin Hub'a ait bölümleri üç depoda birlikte değişmelidir. | `AGENTS.md`, `docs/PRODUCT_CONTRACT.md` | B3-02 |
| F-18 | Mevcut belgeler Hub'ın ürünleri kurduğunu/yönettiğini ve plan/apply motorunu anlatır: `AGENTS.md` ("Sınırlar"), `docs/ARCHITECTURE.md` (satır 20–29, 58, 73, 142–156), `docs/FOOTPRINT.md` (14, 17, 55, 70, 92), `PRIVACY.md`, `README.md`, `docs/PLATFORMS.md` (ürün kurucuları/motor/plan-apply satırları), `docs/QUALITY.md` (fixture matrisi: Sahiplik, Rollback/undo, Native GUI). Hepsi beta.3'te tutarlı hale getirilecek. | grep | B3-02, B3-10 |
| F-19 | Hub kurucusu **Authenticode ile imzasızdır** (`NotSigned`, 2026-11-05 denetimi); SmartScreen uyarısı sürer. | `docs/QUALITY.md` | RC-07, V1-01 |
| F-20 | beta.2 yayımlandı ancak: `ci.yml` `update` işi (`accept_update.py`, yayımlı kurucunun üzerine yeni kurucuyu kurar) yayımlı kurucularla **henüz doğrulanmadı**; temiz VM kurulumu ve Hub'ın gerçek kendini güncellemesi denenmedi; yerelde `v0.1.0-beta.2` git etiketi yok (yalnız `v0.1.0-beta.1`, `catalog`) — GitHub Release'te var. | `QUALITY.md` "Doğrulanmadı", `git tag` | §5 |

## 2.3. Katalog başlangıç durumu

62 kayıt (F-09): 33 MCP, 7 Skill, 18 Plugin, 2 Rule, 2 Other. Bu bir test edilmiş pazar kataloğu değil, **içerik tabanıdır**. Her kaydın hedef sayfası, lisansı, uyumluluk iddiası ve güncelliği ayrı doğrulanacak (B3-08, B4-03, RC-06). Bu sayı güncel varsayılmaz.

---

# 3. Ortak güvenlik, gizlilik ve test ilkeleri

## 3.1. Kırmızı çizgiler (hiçbir sürümde taviz yok)

1. Üçüncü taraf AI konfigürasyonuna **yazma 0**; yalnız Hub'ın kendi favori/proje/ayar kayıtları yazılabilir.
2. Üçüncü taraf kodu/installer/komut **çalıştırma 0**; Hub hiçbir kaynak betiği, hook'u, MCP komutunu çalıştırmaz.
3. Sır (token, parola, API anahtarı, PEM) UI'a, IPC JSON'a, loga veya hata mesajına **taşınmaz**.
4. Webview'dan keyfî shell/URL/dosya yazma yok; genel komut satırı yok; yalnız açıkça izinli, dar ve tipli IPC.
5. Telemetri yok; ağ yalnızca kullanıcı başlatan katalog yenilemesi/Hub güncellemesi ve harici bağlantı açma için.
6. "Kullanıyor / etkin / kurulu / güvenli / resmî / en popüler" iddiaları yalnızca kanıtla yapılır; kanıt yoksa `unknown`.
7. Geçiş dönemi/uyumluluk katmanı eklenmez (`AGENTS.md`); eski durum tolere edilir ve yok sayılır/yeniden üretilir (bkz. K-01).

## 3.2. Öncelik ve durum etiketleri

- **P0:** Sürüm engelleyici; başarısızsa sürüm çıkmaz. **P1:** Güçlü deneyim için gerekli; vazgeçilecekse gerekçe ve sürüm kapısı etkisi yazılır. **P2:** Kalite artıran; zaman baskısında RC/v1.1'e taşınabilir, sahip ve erteleme kararı yazılır. **BLOCKED:** Dış bağımlılık, eksik resmî spec veya güvenlik doğrulaması var.
- Durum işaretleri: `[ ]` yapılacak · `[~]` kod/parça var, doğrulanacak · `[x]` kaynak kod + test + sürüm kanıtı var · `[!]` bloke.
- **DONE:** kod mevcut + test/kanıt mevcut + belge doğru. Yalnızca dosyanın bulunması "bitti" değildir.

## 3.3. Definition of Done (her görev için)

İlgili kod/metadata/UX hazır; negatif senaryolar test edildi; güvenlik ve offline etkisi değerlendirildi; manuel/native gerekiyorsa raporlandı; belgeler aynı değişiklikte doğru; sürüm kapısına katkısı açıklandı. Tamamlandı demeden önce: `npm run check` ve `cargo test` (veya `python common/release/tools/acceptance/accept_hub.py`); arayüz/kurulum değişikliğinde `npx tauri build --debug --no-bundle` ardından `node tests/native/hub_native_e2e.mjs src-tauri/target/debug/respected-hub.exe`.

## 3.4. Test ilkeleri

- Testler geçici kökler ve sahte kullanıcı klasörü kullanır (`RESPECTED_HUB_TEST_HOME`); gerçek AI ayarları ve gerçek Brain verisi değişmez. Native sürücü makinede kurulu Respected ürünü görürse başlamaz.
- Her sürüm kapısında "AI konfigürasyonu önce/sonra **byte hash**" raporu zorunludur.
- Belge cümlelerine bağlı test yazılmaz; belge yolları çalışma zamanı bağımlılığı olmaz.
- Ölçüm yapılmadan "hızlı/güvenli/destekli" yazılmaz; hedef revize edilecekse gerekçe ve kullanıcı etkisi bu belgeye işlenir.
- Hub-only yazma istisnası (favoriler, proje defteri, Hub ayarları, katalog önbelleği, Hub kendi güncelleme indirmeleri) her sürümde ayrı bir izin modeli ve testle sınırlıdır.

## 3.5. Sürüm numaralama

Depo SemVer ön-sürüm düzenini kullanır (`0.1.0-beta.2` bugün `package.json`, `tauri.conf.json`, `Cargo.toml`). "Beta.3/4/5" **ürün kilometre taşlarıdır**; etiketler `v0.1.0-beta.N` (veya depo kararına göre), RC `v1.0.0-rc.N`, kararlı `v1.0.0`. RC numarası tarihe değil kalite kapısına göre artar. Güncelleme sözleşmesi korunur: her sürümden en yeniye doğrudan güncelleme (silinmeyen basamaklar, sürüm başına gerçek veri fixture'ı); zincir uzarsa `min_from` ile basamak sürüm istenir.

---

# 4. Sürüm kapıları, bağımlılık düzeni ve PR sırası

## 4.1. Genel sürüm kapıları

| Sürüm | Kullanıcıya sunulan somut değer | Çıkış şartı |
|---|---|---|
| Beta.2 | Yayımlı sürüm; açık doğrulama işleri §5'te | Mevcut kapsam; bu plan yeniden tanımlamaz |
| **Beta.3** | Bağımsız/ürün-tarafsız Hub, yeni mağaza temeli | Özel Brain/Orchestrator yönetim yoluna **0** erişilebilir giriş; Hub self-update çalışıyor |
| **Beta.4** | "Gerçek mağaza": keşif, detay, filtre, editoryal içerik | Kullanıcı bir ihtiyaca göre seçim yapıp kaynak sayfasına gidebiliyor; katalog offline açılıyor |
| **Beta.5** | "Bu bilgisayarda neler var?": salt okunur envanter/AI/kapsam | 5 hedef istemci için doğrulanmış read-only destek matrisi; 0 config yazımı |
| **v1.0 RC** | Dondurulmuş özellikler için test, UX, güvenlik, performans | Kritik hata yok; ölçüm ve gerçek Windows kurulum/güncelleme kanıtı |
| **v1.0 Stable** | Genel yayına hazır Store + Read-only Library | Bütün P0 sürüm kapıları sağlandı ve sürüm artefaktları doğrulandı |

## 4.2. Bağımlılık şeması (kaba akış)

```text
BETA.2 (yayımlı; §5 açık işler paralel yürür, B3'ü bloke etmez)
  |
BETA.3: B3-01 → B3-02 → B3-03 → B3-04 → B3-05
                  |             ├→ B3-06 → B3-07
                  |             └→ B3-08 → B3-09 → B3-10
                  v
BETA.4: B4-01 / B4-02 → B4-03 → B4-04 → B4-05 → B4-06
                                ├→ B4-07 → B4-08 → B4-09
                                ├→ B4-10 → B4-11 → B4-12
                                └→ B4-13 → B4-14 → B4-15
                                   └→ B4-16 (kapı)
                  v
BETA.5: B5-01 → B5-02 → B5-03 → B5-04 → B5-05
                    ├→ B5-06 → B5-07 → B5-08
                    ├→ B5-09 → B5-10 → B5-11
                    └→ B5-12 → B5-13 (kapı)
                  v
v1.0 RC: RC-01 → RC-02 / RC-03 / RC-04 / RC-05 / RC-06 → RC-07
                  v
v1.0 Stable: V1-01 → V1-02 → V1-03
```

Bu şema kaba akıştır; **her kartın kendi `Bağımlılık` alanı belirleyicidir.** Bağımlılık çözülmeden görev tamamlandı işaretlenmez. UI ve metadata hazırlığı birbirine bağlı olmayan yerlerde paralel yürüyebilir. Kart bağımlılıkları döngü içermez (aktarım sonunda betikle doğrulandı, bkz. §17).

## 4.3. PR sırası (her PR tek anlamlı değişiklik)

| Sıra | PR / iş paketi | Kapsam | Birleştirme şartı |
|---:|---|---|---|
| 01 | B3-01 + B3-02 | Mevcut durum ve ürün kararlarının dondurulması | Eski ürün/Hub self-update sınırı belirli |
| 02 | B3-03 | Özel Brain/Orch UI kaldırma | Eski rota testleri başarılı |
| 03 | B3-04 | Özel backend/RPP/lifecycle kaldırma | Self-update çalışıyor, diğer uygulamalara dokunulmuyor |
| 04 | B3-05 | Salt okunur kapı | Config yazma/exec dışa açık değil |
| 05 | B3-06 + B3-08 | Repo/artifact model ve eski katalog göçü | 62 kayıt için muhasebe var |
| 06 | B3-07 | Genel mağaza route iskeleti | Navigation + fallback testleri |
| 07 | B3-09 + B3-10 | Beta.3 test + belgeler + yayın | Beta.3 gate yeşil |
| 08 | B4-01 + B4-02 | Mağaza UX IA ve şema v2 | UI/model sözleşmeleri net |
| 09 | B4-03 + B4-11 | Editoryal pipeline ve güvenli URL | Kaynak doğrulama CI'da |
| 10 | B4-04 + B4-05 | Offline arama / filtreleme | Altın küme ve performans testi |
| 11 | B4-07 + B4-08 | Detay sayfaları | Örnek 5 sayfa QA |
| 12 | B4-06 + B4-09 | Vitrin ve koleksiyon | Editoryal içerik, sahte popülerlik yok |
| 13 | B4-10 | Favoriler | Hub-only local state persist |
| 14 | B4-12 + B4-13 | İmza/cache, medya/attribution | Offline + saldırı testleri |
| 15 | B4-14 + B4-15 + B4-16 | UX, perf ve beta.4 gate | Native smoke/pilot test |
| 16 | B5-01 + B5-02 | Adapter gerçek kapsam + güvenli keşif | 5 client matrisi doğrulanmış |
| 17 | B5-03 + B5-04 | MCP + Skill/Plugin parser | Fixture setleri yeşil |
| 18 | B5-05 + B5-06 | Envanter ve effective resolver | Scope/status yanlış pozitif 0 |
| 19 | B5-07 + B5-08 | Library ve detay | UI no-write, erişilebilirlik |
| 20 | B5-09 | Proje görünümü | Yalnız kullanıcı seçimi ve Hub defteri |
| 21 | B5-10 | Store ↔ Library | Kanıtsız installed rozeti yok |
| 22 | B5-11 + B5-13 | Privacy + beta.5 gate | Hash/no-exec testleri |
| 23 | B5-12 (opsiyonel) | Setup Doctor salt okunur | Varsa yalnız kanıtlı bulgu |
| 24 | RC-01..RC-04 | Freeze + CI + güvenlik + UX | Release engelleyici kalmamış |
| 25 | RC-05..RC-07 | Perf + içerik/visual polish + go/no-go | RC gate başarılı |
| 26 | V1-01..V1-03 | Stable release ve kamu desteği | Windows installer/release ve belgeler hazır |

**Paralel yapılabilir:** B4-01 UI araştırması ile B4-02 şema çalışması; B4-03 editoryal pipeline ile tasarım; B5 adapter fixture'ları ile Store hata düzeltmeleri. **Paralel yapılmamalı:** B3 özel ürün backend kaldırması sürerken aynı modüllere bağımsız geniş refactor; şema sabitlenmeden tüm detay sayfaları; resolver güvenilir değilken "etkin" rozeti.

---

# 5. Beta.2 — yayımlı sürümün açık doğrulama işleri

`0.1.0-beta.2` yayımlıdır (2026-11-05). Kapsamı yeniden açılmaz; aşağıdaki işler beta.2 çalışmasından **devralınır ve kaybolmaz**. Her biri bir sonraki sürüm kapısıyla ilişkilendirilmiştir. Beta.3 özel ürün yönetimini kaldırdığı için ürün-spesifik kısımlar Hub kapsamından çıkar (ayrıntı §12).

| ID | Madde | Durum | Beta.3+ karşılığı |
|---|---|---|---|
| REL-001 | Eski sürümden gerçek güncelleme ve geri alma CI matrisi. Hazır: `ci.yml` `update` işi (`accept_update.py`) yayımlı kurucunun üzerine bu derlemenin kurucusunu sessiz kurar; sürüm, exe, kullanıcı verisi ve kaldırmayı doğrular. Kalan: yayımlı beta.2 kurucusuyla gerçek koşu (beta.2 şimdi mevcut). | `[ ]` Yayımlı beta.2 kurucusuyla CI'da koşturulacak; kanıtı yok | Beta.3 sürümü için yeniden anlam kazanır: beta.2→beta.3 (B3-09, RC-02) |
| REL-002 | Temiz Windows VM kurulumu ve Hub'ın gerçek kendini güncelleme testi (Ayarlar > Güncellemeler > Hub'ı güncelle). Kendi güncelleme kodu yazılı ve birim testli; gerçek yeni sürümle uçtan uca deneme yapılmadı. | `[ ]` Temiz VM yok; kendini güncelleme için artık daha yeni sürüm (beta.3) çıkabilir | B3-04 (1 testli başarılı senaryo), B3-09, RC-02, V1-01 |
| REL-003 | Güncelleme bildirimi, çalışan görev uyarısı, aynı sürüm farklı digest ve yayın hataları. | `[x]` Çalışan Orchestrator görevi uyarısı (`8276c56`), aynı sürüm farklı digest (`4d72a6c`), çevrimdışı onarım ve bildirim testleri (`1716894`) — **ürün yönetimiyle birlikte beta.3'te kalkar**. `[ ]` Hub'sız kurulan ürünlerin kendi sürüm bildirimi: ürün depolarında | Hub'a ait olmayan; PARK-07 / kardeş depo notu |
| REL-004 (yeni kimlik) | Kurucuyu Authenticode ile imzalamak (SmartScreen). Kod imzalama sertifikası yok; `v0.1.0-beta.1` `NotSigned`. | `[!]` Sertifika gerekir | RC-07, V1-01 |
| PORT-002 (yeni kimlik) | macOS/Linux masaüstü GUI ve paketleme doğrulaması. CI'da yalnız keşif/envanter/plan-apply kontratları koşar. | `[ ]` Doğrulanmadı | PARK-06 (v1.0 yalnız Windows x64) |
| QA-002 (yeni kimlik) | `--real-semantic` anlamsal arama modelinin gerçek indirmesi koşulmadı. | `[ ]` Brain'e özgü | B3-04 ile **anlamsız hale gelir** (Hub'da Brain arama yok); kapatılır |
| REL-005 (yeni kimlik) | Güncelleme yolunda "sağlık dalıyla geri alma" (ürünün sağlık sözleşmesi + önceki kurucu). | `[ ]` | Ürün yönetimi kalktığı için **Hub'ın kendi** geri alma sorusuna indirgenir: B3-04 notu |
| DOC-001 (yeni kimlik) | `docs/QUALITY.md` "Doğrulanmadı" listesi ve `docs/PLATFORMS.md` kanıt satırları beta.2 sonrası CI koşusuyla güncellenmedi (son kanıt commit `14b1609`). | `[ ]` | B3-10 |
| HYG-001 (yeni kimlik) | Yerel git'te `v0.1.0-beta.2` etiketi yok; yayın GitHub'da var. B3-01'de tag/commit pini yazılmalı. | `[ ]` | B3-01 |

**Hub dışı (kardeş depo) işler — bu depoda yapılamaz, kayıp değildir:** Brain'de Gemini CLI ve Cursor token kullanımı (`respected-brain`); Orchestrator'da Gemini ve Cursor token kullanımı (`respected-orchestrator`); Hub'sız kurulan ürünlerin sürüm bildirimi (ürün depoları). Bunlar Hub yönetimi kalktıktan sonra da ürün depolarında kalır.

**Bilinen açık hata:** Bu depoda düzeltilebilir bilinen açık hata yoktur (beta.2 sonrası bulunanlar buraya eklenecek). Başlangıçta tespit edilenler: F-03 (açılışta `recover()`), F-02 (plan IPC) → B3-05; F-12 → B4-02; F-13 → B4-11.

---

# 6. Beta.3 — Bağımsız Hub ve Store temeli

**Ürün hedefi:** Brain/Orchestrator yönetimini özel sistem olarak tamamen çıkarmak; mevcut katalog ve salt okunur keşif yatırımlarını koruyarak tarafsız "AI ekosistem mağazası" temeli kurmak. Beta.4 mağaza cilası, Beta.5 envanter ürünleşmesi için temiz zemin.

**Çıkış kapısı:** Özel Brain/Orchestrator yönetim yoluna **0** erişilebilir giriş (UI rota + backend komut taraması); Hub self-update **1 testli başarılı senaryo**; `npm run check`, `cargo test`, native Windows smoke yeşil; Brain/Orchestrator kurulumları bozulmadan ve klasörlerine **0 yazma**.

### B3-01 [P0] — Beta.2 kapsamını kilitle, repo/envanter denetimi yap `[~]`
- **Mevcut durum:** beta.2 yayımlı; yerelde etiket yok (HYG-001). Envanterin büyük kısmı §2'de çıkarıldı; kaldırma haritası B3-04 tablosunda başladı.
- **Bağımlılık:** Beta.2'nin çalışan geliştirmesini bozmamak; temel dal/tag netliği.
- **Adımlar:**
  1. Beta.2 tag/commit'ini sabitle (`git fetch --tags`; `v0.1.0-beta.2` karşılık commit'i) ve devam eden görevleri (§5) tespit et.
  2. `ROADMAP`, README, ürün-spesifik UI/komutlar, test dosyaları, updater ve integrations modüllerinin haritasını çıkar.
  3. "Korunacak / kaldırılacak / dönüştürülecek / doğrulanacak" listesi oluştur (envanter ayrı bir belge olabilir; kararlar bu belgede kalır).
- **Mimari/dosyalar:** `docs/ROADMAP.md`, `AGENTS.md`, `README.md`, `src/shell`, `src-tauri/src`, `common/release`, testler.
- **Test:** Kaldırılacak her özel UI/IPC kaydı için referans araması; self-update dosyaları listesi; eski test matrisi.
- **Kabul:** Bütün özel yaşam döngüsü giriş noktaları ve korunacak Hub işlemleri listelenmiş; değişiklik kapsamı peer review ile onaylanmış.

### B3-02 [P0] — Yeni ürün sınırları ve güncel ADR `[ ]`
- **Mevcut durum:** `AGENTS.md` "Sınırlar" ve `ARCHITECTURE.md` Hub'ı ürünlerin kurucusu/yöneticisi olarak tanımlar (F-18). `ADR-THIRD-PARTY-INTEGRATIONS.md` M1/M2'de okuma, M3'te yazma kapısı tanımlar; v1.0 kararı bunu kapatır.
- **Bağımlılık:** B3-01.
- **Adımlar:**
  1. Hub ürün tanımını "mağaza + salt okunur kütüphane" olarak yaz.
  2. Brain/Orchestrator özel sahiplik akışlarını kaldırma kararını işaretle.
  3. İstemci config okuma sınırını ve Hub'ın yalnızca kendi ayar/favori/proje defterine yazma yetkisini açıkla.
  4. v1.1+ yazma kapısını kapat (ADR revizyonu; PARK-01..04 önkoşulları).
  5. Çapraz depo sözleşmesi: `docs/PRODUCT_CONTRACT.md` ve `common/conformance/` değişikliği Brain/Orchestrator depolarındaki kopyalarla birlikte planlanır (F-17); kardeş depolar bu depoda değiştirilmez, gerekli değişiklik notu çıkarılır.
- **Mimari/dosyalar:** `README.md`, `docs/ARCHITECTURE.md`, `docs/ADR-THIRD-PARTY-INTEGRATIONS.md`, `docs/FOOTPRINT.md`, `PRIVACY.md`, gerekirse `docs/PRODUCT_CONTRACT.md`, `AGENTS.md`.
- **Test:** Belgelerde "Hub Brain'i/Orchestrator'ı kurar, yönetir" sözleri için tarama; AI config'e yazma iddialarının ayıklanması.
- **Kabul:** Ürün tanımı ve dosya sahipliği birbiriyle çelişmiyor; kullanıcıya v1.0'da olmayan aksiyon vaat edilmiyor.

### B3-03 [P0] — Brain/Orchestrator özel UI akışlarını ve sayfalarını kaldır `[ ]`
- **Mevcut durum:** `APPS` yalnızca Brain+Orchestrator; `Discover.tsx` bu iki ürünün kurulum paneli; `Library.tsx`/`AppWorkspace.tsx` ürün çalışma alanı; `HubRoute = 'home'|'discover'|'apps'|'settings'`; `App.tsx` `restore()` eski rotayı toparlar (F-15).
- **Bağımlılık:** B3-02.
- **Adımlar:**
  1. Hafıza, görevler, kampanya, kurulum sihirbazı, ürün özel ayarlar/güncelle/onarma/kaldır UI'larını çıkar.
  2. `APPS` ve ürün-spesifik rotaları genel Store/Library rotalarıyla değiştir.
  3. Eski deep link'leri (`startup-intent`, `route.kind === 'app'`, saklı `hub.route`) kontrollü Store/Library sayfasına yönlendir veya zararsız hata ver.
  4. Brain/Orchestrator normal katalog öğesi olabilir.
- **Mimari/dosyalar:** `src/shell/App.tsx`, `src/shell/apps.ts`, `src/products/{brain,orchestrator}/`, `src/shell/views/{Discover,Library,Home,AppWorkspace,Products,LifecycleControls,InstallFacts,Hello}.tsx`, `src/shell/{lifecycle,installFacts,workspaceModel,homeModel,useEcosystem}.ts`, ürün kontrol bileşenleri.
- **Test:** Eski hafıza/görev/ürün ayar sayfalarına rota yok; eski `sessionStorage` rotası uygulamayı kilitlemiyor; Store'da Brain/Orchestrator yalnız katalog kimliğiyle açılıyor.
- **Kabul:** Ürün özel React sayfaları/aksiyonları **0**; Hub self-update menüsü mevcut; bilinmeyen eski rotada crash **0**.

### B3-04 [P0] — Backend'de özel RPP / ürün yönetim yüzeyini kaldır `[ ]`
- **Mevcut durum:** 37 kayıtlı komut ve ürün modülleri (§2.1). Self-update ürün indeksiyle aynı modülde (F-06); yayımlı indeks beta.2 Hub'ları tarafından da okunur (F-07). `settings.rs` Brain/Orchestrator yollarını taşır.
- **Bağımlılık:** B3-03; Hub update sınırı B3-01.
- **Adımlar:**
  1. Brain/Orchestrator'a özel kurulum, onarım, kaldırma, RPP/iş akışı, memory/connection komutlarını ve özel kurucu niyetlerini ayır.
  2. Artık kullanılmayan API/permission/allowlist ve test fixture'larını temizle.
  3. Hub self-update kodunu ürün lifecycle modüllerinden gerekiyorsa ayrıştır (`hub_update`/`install_hub_update`/indeks doğrulaması ayrı modül).
  4. Kodda hiç çağrılmayan kurucu/yürütücü yetkisi bırakma.
  5. **Yayın tarafı:** `catalog` Release'indeki indeksin `products` alanını ve şema sürümünü beta.2 Hub'ının okuyabileceği biçimde bırakma/değiştirme kararını yazılı al (F-07); kararı verirken `AGENTS.md`'deki "uyumluluk katmanı yok" kuralını (K-01) gözet.
  6. Güncelleme geri alma (REL-005) sorusu: Hub'ın kendi self-update'i için önceki kurucuyu saklama ihtiyacı değerlendirilir; yoksa "geri alma yok, yeni sürümle ileri düzeltme" olarak belgelenir.
- **Komut sınıflandırması (başlangıç önerisi; dosyaları gözle kontrol et, lifecycle modülünü körlemesine silme):**

| Komut / modül | Karar |
|---|---|
| `engine_request`, `rpp_request`, `product_status`, `development_build`, `campaign_list`, `src-tauri/src/{rpp.rs,products/*,process.rs}` | Kaldır |
| `memory_access_*`, `memory_index_*`, `memory_semantic_*`, `memory_capture_*`, `memory_search`, `memory_note_read`, `memory_briefing_run`, `commands/memory.rs` | Kaldır |
| `lifecycle_status/plan/apply`, `hub/{installed,installer,startup,mod}.rs` ürün kurulum kısmı | Kaldır (Hub'ın kendi kurucusu `packaging/windows/hub_setup.iss`'te kalır) |
| `orchestrator_mcp_plan/apply`, `products/orchestrator/mcp.rs` | Kaldır (üçüncü taraf config'e dolaylı yazma; F-05) |
| `startup_intent`, `hub/startup.rs` | Kaldır veya Hub-only niyete indir |
| `hub_update_status`, `hub_update_install`, `lifecycle/catalog.rs` Hub kısmı | **Koru**, ayrıştır |
| `catalog_refresh`, `catalog_set_channel` (kanal stable/beta) | Koru; Hub indeksi ve üçüncü taraf katalog ayrımını netleştir |
| `activity_list` | Hub etkinliği kalıyorsa koru, ürün günlükleri kalkar |
| `detect_ai_clients`, `integrations_request`, `projects_list/add/remove` | Koru (B3-05'te sınırlandır) |
| `get_settings`, `save_settings` (`connections.json` ürün yolları) | Hub ayarlarına indir veya kaldır |

- **Mimari/dosyalar:** `src-tauri/src/hub/*`, `lifecycle/*`, `rpp.rs`, `products/*`, `commands/*`, `settings.rs`, `lib.rs`, `tauri.conf.json`, `capabilities/`, `packaging/windows/*`, ilgili release/test betikleri, `common/release/tools/build/*`.
- **Test:** Eski ürün yönetim IPC'leri liste dışı; `rpp` ürün komutu çalıştırılmıyor; Hub kendi update/signature doğrulaması çalışıyor (mevcut `lifecycle/catalog.rs` birim testleri korunur/taşınır); Brain/Orchestrator program dosyalarına dokunulmuyor.
- **Kabul:** Üretim derlemesinde Brain/Orchestrator'a özel erişilebilir komut ve installer yolu **0**; self-update **1 testli başarılı senaryo** (REL-002 ile birlikte gerçek sürüm geçişi RC-02'de).

### B3-05 [P0] — v1.0 salt okunurluk kapısını kod ve API'de kilitle `[~]`
- **Mevcut durum:** `apply`/`undo` IPC'den erişilemez (F-01, doğrulandı). Açık kalanlar: `plan` eylemi (F-02), açılışta `recover()` (F-03), koşulsuz derlenen yazma kodu (F-04), `orchestrator_mcp_apply` (F-05).
- **Bağımlılık:** B3-02, B3-04.
- **Adımlar:**
  1. Public Tauri API'lerinde üçüncü taraf config yazabilen eylemleri kapat (`orchestrator_mcp_apply` kalkar).
  2. `Action::Plan` ve `PlanRequest`'i **sil** (plan önizlemesi v1.0 ürünü değil; K-02 kararı: silinir). `planner.rs` `apply.rs`/`undo.rs`/`profiles.rs` ile iç içe olduğundan bu dosyaların akıbeti adım 3'te birlikte karara bağlanır.
  3. `planner/apply/undo/profiles` kodunu **üretim binary'sinden çıkar** (önerilen yol: `third-party-write` cargo feature'ı, varsayılan kapalı; testler feature ile korunur) ya da tümüyle sil; karar PR'da yazılır.
  4. `lib.rs` `.setup` içindeki `Journal::default().recover()` çağrısını kaldır (yazma yoksa kurtarma da yok).
  5. UI'da toggle, taşı, kur, kaldır, fix, sync butonları olmadığını doğrula/kaldır.
  6. Dar file dialog yalnız proje seçimi; Hub kayıtları yazımı ayrı izin modeli (`dialog:allow-open` dışı izin yok).
- **Mimari/dosyalar:** `commands/integrations.rs`, `integrations/api.rs`, `src/shared/api.ts`, `integrations/{planner,apply,undo,profiles}.rs`, `integrations/mod.rs`, `Cargo.toml [features]`, `src/shell/views/*`, `capabilities/default.json`.
- **Test:** Bilinmeyen action, kötü niyetli selector/raw path/command, stale plan vb. IPC çağrısı reddi (mevcut `only_closed_actions_are_accepted` genişletilir: `plan` de reddedilir); config fixture hash'leri tüm akışlarda sabit; `cargo test --release --lib production_build` üretim kümesine "yazma sembolü/komutu yok" denetimi eklenir. Favori/proje kaydı istisna.
- **Kabul:** Üretim v1 yolunda **AI config yazabilen komut: 0**; kullanıcı onayı olmadan üçüncü taraf kodu çalıştırma **0**.

### B3-06 [P0] — Katalog nesne modeli: Repo ≠ Artifact ≠ Uygulama `[ ]`
- **Mevcut durum:** `CatalogKind` `app` içermez (F-10); her kayıt tek `source_url` ve tek tür.
- **Bağımlılık:** B3-02, B3-03.
- **Adımlar:**
  1. `Repository`, `Publisher`, `Artifact`, `AssetKind` kavramlarını ayır.
  2. `app`, `mcp`, `skill`, `plugin`, `rule`, `tool`, `other` türlerini doğrula.
  3. Bir repo 5 skill barındırabiliyor; repo kartı/alt öğe kimliği/sürüm ilişkisi kaybolmuyor.
  4. Brain/Orchestrator yalnız `app` türünde genel kayıt olabilir, ayrı kurucu şartı yok.
- **Mimari/dosyalar:** `src-tauri/src/integrations/catalog.rs` / `model.rs`, `src/shared/integrationsTypes.ts`, mağaza model/selector'ları.
- **Test:** Tek repo/çoklu artifact; farklı repo aynı ad; aynı artifact için farklı tür; missing AI compatibility; `app` türünün geçerli oluşu.
- **Kabul:** Repo ile alt bileşenleri yanlışlıkla birleştiren vaka **0**; kararsız ID yeniden yayında bozulmuyor; `app` kategorisi tek başına AI client gerektirmiyor.

### B3-07 [P1] — Mağaza rota ve boş durum iskeleti `[ ]`
- **Bağımlılık:** B3-03, B3-06.
- **Adımlar:**
  1. Ana gezinmeyi "Mağaza/Keşfet, Kütüphane, Projeler, Kaydedilenler, Ayarlar" ilkesine göre kur.
  2. Store Home, Search/Explore, Category, Item Detail, Collection için route iskeletleri aç.
  3. Küçük pencere ve geçmiş rota dönüşlerini ele al.
  4. Henüz olmayan aksiyonlar arayüzde pasif vaade dönüşmesin.
- **Mimari/dosyalar:** `src/shell/App.tsx`, `src/shell/apps.ts`, yeni önerilen `src/shell/store/*`, `src/shell/views/*`.
- **Test:** Üst/geri gezinme, search→detail→geri, bilinmeyen item, boş içerik, eski rota, yeniden başlatma.
- **Kabul:** Her katalog öğesi açılabilir bir `item_id` taşır; tüm rotalar crash olmadan çalışır; özel Brain/Orch rotası yok.

### B3-08 [P1] — Önceki 62 kayıt ve mevcut entegrasyon altyapısını kayıpsız taşı `[ ]`
- **Mevcut durum:** 62 kayıt analiz edildi (F-09, F-11); lisans 0/62; 9 istemci `plugin` tipinde; kategori yazım tutarsızlığı.
- **Bağımlılık:** B3-06.
- **Adımlar:**
  1. Mevcut katalogdaki kayıtları şema geçiş haritasına al (kayıt başına: *korundu / yeniden adlandırıldı / birleştirildi / kaynak geçersiz*).
  2. Bozuk/eskimiş GitHub linklerini ayrı kalite listesine kaydet (ağ doğrulaması B4-03/B4-11 işidir; burada offline yapısal denetim).
  3. İmzalı güncelleme, statik fallback ve adapter testlerini koru.
  4. Bu aşamada katalog zenginleştirmeyi Beta.4'e bırak.
- **Mimari/dosyalar:** `src-tauri/assets/third-party-catalog.json`, `src-tauri/src/integrations/catalog.rs`, migration test fixtures.
- **Test:** Eski kayıt → yeni ID → yeni detay linki tutarlılığı; imza/önbellek uyumsuzluğu; bozuk kayıt fail-closed.
- **Kabul:** Kayda değer bir öğe sessizce yok olmuyor; eski 62 giriş için *korundu/yeniden adlandırıldı/birleştirildi/kaynak geçersiz* sonucunun tamamı raporlandı.

### B3-09 [P0] — Beta.3 test migrasyonu ve Windows smoke `[ ]`
- **Mevcut durum:** Native E2E ürün kurucularını, sihirbazı, MCP, hafıza, onarım, kaldırmayı sınar (30 PASS, QUALITY.md); `ci.yml` `test` işi kardeş Brain/Orchestrator depolarını checkout eder; `update` işi yayımlı kurucudan başlar.
- **Bağımlılık:** B3-03..08.
- **Adımlar:**
  1. Ürün kurulumuna dayalı eski UI/E2E'leri çıkar veya Hub-only smoke ile değiştir (F-16 listesi).
  2. Native önizleme, self-update, offline katalog, mağaza rotaları, RPP/kurucu yokluğu testlerini ekle.
  3. Beta.2 → beta.3 güncellemede kullanılmayacak eski Hub durumunu güvenle yoksay (sessionStorage rota, `connections.json`, ürün journal'ları).
  4. CI'dan kardeş depo checkout/`pip install -e` adımlarını kaldır.
  5. REL-001 (beta.2 kurucusu → beta.3) bu görevde kanıtlanır.
- **Mimari/dosyalar:** `tests/native/hub_native_e2e.mjs`, `src-tauri/tests/*`, `common/release/tools/acceptance/accept_hub.py`, `accept_update.py`, `.github/workflows/ci.yml`.
- **Test:** Temiz Windows test kullanıcısı; Hub uninstall/reinstall; Brain/Orch önceden kurulu senaryo; eski açılış niyeti; AI config önce/sonra hash.
- **Kabul:** `npm run check`, `cargo test`, native Windows smoke başarılı; Brain/Orch kurulumu/çalışması bozulmadan, onların klasörlerine **0 yazma**.

### B3-10 [P1] — Beta.3 dokümantasyon ve release gate `[ ]`
- **Bağımlılık:** B3-09.
- **Adımlar:**
  1. README, ürün tanımı, ekranlar, footer, About, release notes'i yeni işleve göre güncelle.
  2. "Store'dan kur" vaatlerini kaldır.
  3. Erişilebilir komut / telemetri / katalog güveni konusunda güncel açıklama ver.
  4. Yeni sürümün UI/kapsam ekran görüntülerini doğrula.
  5. **Tutarsız belge listesi (F-18)** tek PR'da düzeltilir: `AGENTS.md` (Sınırlar), `ARCHITECTURE.md` (ürün kurulumu/RPP/plan-apply anlatımı), `FOOTPRINT.md` (ürün kurulumu ve üçüncü taraf yazma motoru satırları), `PRIVACY.md`, `PLATFORMS.md`, `QUALITY.md` (fixture matrisi, kanıt tarihleri, DOC-001), `packaging/windows/{hub_info.txt,UPDATE_NOTES.md}`, `PRODUCT_CONTRACT.md` (çapraz depo, B3-02).
- **Mimari/dosyalar:** `README.md`, `docs/{ARCHITECTURE,ROADMAP,FOOTPRINT,PRODUCT_CONTRACT,ADR-THIRD-PARTY-INTEGRATIONS,PLATFORMS,QUALITY}.md`, `PRIVACY.md`, release workflow.
- **Test:** Belgelerde eski Brain/Orch kontrol vaadi kalmıyor; installer/updater yolları doğru; release artefakt linkleri çalışıyor.
- **Kabul:** Sürüm notu/yayın sayfasında yeni ürün sözü doğru; Beta.3 tag'i yalnızca gate geçildikten sonra önerilir.

---

# 7. Beta.4 — Gerçek mağaza deneyimi

**Ürün hedefi:** Kullanıcı Hub'ı ilk kez açtığında "GitHub bağlantılarından oluşan liste" değil, **araştırılabilir, bilgi bakımından zengin ve premium bir AI araçları mağazası** görmeli. Ürünleri kurmayacak; onların ne olduğunu, ne işe yaradığını, ne zaman gerekli olduğunu, nereden geldiğini ve alternatiflerini anlayacak.

**Sürüm kapağı:** Kullanıcı "tarayıcı otomasyonu yapacak güvenilir bir MCP", "React geliştirmeye yönelik Skill", "Claude Code ile uyumlu Plugin" gibi ihtiyaçtan başlayıp ürünü bulabilmeli, detay sayfasında bilinçli değerlendirme yapabilmeli, favorilere kaydedebilmeli ve kaynak/dokümantasyon bağlantısını açabilmeli. Varsayılan katalog **offline** çalışmalı.

### B4-01 [P0] — Mağaza tasarım ilkeleri, bilgi mimarisi ve ekran sözleşmesi `[ ]`
- **Bağımlılık:** B3-07.
- **Adımlar:**
  1. Kullanıcı profillerini yaz: yeni vibe coder, deneyimli AI coder, açık kaynak geliştirici.
  2. Mağaza ana sayfası, Keşfet/arama, kategori, koleksiyon, öğe detay, favoriler için bilgi hiyerarşisi tasarla.
  3. Steam'den yalnız işlevsel keşif desenlerini al; logo/görsel/layout birebir kopyalama.
  4. İlk 5 kullanıcı görevini akış şemasıyla doğrula.
  5. Genel navigasyonu mümkün olduğunca yalın tut.
- **Mimari/dosyalar:** `src/shell/App.tsx`, `src/shell/views/Discover.tsx` ve yeni `src/shell/store/*` bileşenleri; tasarım token'ları.
- **Test:** İlk açılışta keşif, kategori→ürün, arama→ürün, favori→ürün, detay→harici kaynak, geri/ileri/yeniden aç.
- **Kabul:** Beş yolun her biri arayüz prototipinde ulaşılabilir; kullanıcıya hiçbir gizli yazma/kurulum eylemi sunulmaz; her önemli görünümün loading/error/empty hali tanımlıdır.

### B4-02 [P0] — Mağaza katalog şeması v2 ve sabit kimlikler `[ ]`
- **Mevcut durum:** Şema v1: `id, name, type, description, category, source_url, supported_clients, license?, provenance?, publisher?, package_digest?, install_commands[], permissions[], manifest?`, `deny_unknown_fields` (F-12). Lisans 0/62.
- **Bağımlılık:** B3-06, B3-08.
- **Adımlar:**
  1. `Repository` (kaynak depo), `Artifact` (MCP/Skill/Plugin/App/Rule/Tool), `Publisher` ve `StoreItem` ayrımı kur.
  2. `id`, tür, dil bilgisi, kısa/uzun açıklama, ne işe yarar, kullanım alanları, öz/alt bileşenler, gerekli araçlar, kaynak ve dokümantasyon bağlantıları, lisans (biliniyorsa), yayıncı, son kontrol tarihi, metadata provenance, medya/atıf alanlarını ekle.
  3. AI uyumunu `supported / unknown / unverified` olarak kanıta bağla.
  4. Eski katalog şemasından sürümlü migrasyon stratejisi ekle (K-01: çift şema desteği değil, tek yönlü göç + eski önbelleği yok sayma).
  5. Eski Hub'ların v2 uzak kataloğu geçersiz sayacağı (F-12) sonucu için yayın kararı ver (ayrı kanal/URL).
- **Mimari/dosyalar:** `src-tauri/src/integrations/catalog.rs`, `src-tauri/assets/third-party-catalog.json`, `src/shared/integrationsTypes.ts`; yeni önerilen `storeSchema`/`storeModel`.
- **Test:** Aynı repodan 4 Skill, başka repoda aynı isimli Skill, farklı türlerin aynı adı, açıklama eksik, lisans bilinmiyor, AI uyumu bilinmiyor, şema sürümü yanlış, bozuk URL, çoklu dil metni.
- **Kabul:** Kaynak kimliği (`repository_id`) ile artifact kimliği (`item_id`) ayrıdır; 0 istemsiz kimlik çakışması; bilinmeyen olgulara uydurma varsayılan atanmaz; eski kayıtların göçü için kayıt bazlı rapor vardır.

### B4-03 [P0] — İçerik edinme ve editoryal kalite sistemi `[ ]`
- **Bağımlılık:** B4-02.
- **Adımlar:**
  1. Resmî MCP Registry, skills.sh, GitHub ve ilgili dokümanları **kaynak ve lisans/robots/API şartlarına uyarak** katalog adaylarına dönüştüren *geliştirici tarafı* süreç tasarla (kullanıcı istemcisine GitHub API toplu tarama yükü bindirme).
  2. Kaynakların yaşam döngüsünü `candidate → review → approved → published → stale/deprecated` durumlarıyla izle.
  3. Her kayıt için insan/editör onayı ve gerekçeli kaynak belirt.
  4. Kaynak kontrol/yenileme tarihini sakla.
  5. Yayımlanacak imzalı snapshot'ı deterministik üret.
  6. Katalog kalite iyileştirmeleri F-11'den: kategori adlarını tekilleştir, Türkçe karakterleri düzelt, "Awesome …" listelerini tek artifact yerine koleksiyon/kaynak olarak ele al.
- **Mimari/dosyalar:** Önerilen `scripts/catalog/`, `src-tauri/assets/third-party-catalog.json`, doğrulama testleri, katalog CI işi.
- **Test:** Silinmiş/taşınmış depo, lisanssız giriş, sahte "official", GitHub redirect, 404, farklı deposu olan aynı isim, kaynağı değişen yayıncı, eski metadata, rate limit.
- **Kabul:** Beta.4 için **en az 100 editoryal kontrol edilmiş giriş hedefi**; her onaylı öğede kaynak URL, tür, açıklama, provenance ve kontrol tarihi vardır; kalite kapısından geçmeyen kayıt yayınlanmaz. v1.0 için **150–250 iyi kayıt hedef**; sayı doğruluk/lisans aleyhine zorlanmaz.

### B4-04 [P0] — Arama motoru: ihtiyaç odaklı, Türkçe/İngilizce, offline `[ ]`
- **Mevcut durum:** Yalnızca alt-dize eşleşmesi (`filterCatalog`, F-14); sıralama/eşanlam/yazım hatası yok.
- **Bağımlılık:** B4-02, B4-03.
- **Adımlar:**
  1. Başlık, kısa açıklama, uzun açıklama, kullanım senaryosu, kategori, etiket, repo/publisher üzerinde yerel indeks.
  2. Türkçe/İngilizce eşanlamlar ve harf normalizasyonu (`ı/i`, aksan ve Unicode) kur.
  3. İlgililik puanı: başlık/etiket eşleşmesine öncelik; açıklamada rastgele kelime doldurma sonucu manipüle etmemeli.
  4. Yazım hatası önerileri ve boş sonuç alternatifleri.
  5. Arama tamamen yerel; canlı GitHub çağrısı gerektirmez.
- **Mimari/dosyalar:** Önerilen `src/shell/store/search.ts`, `src/shell/store/searchIndex.ts`, arama/indeks birim testleri; büyük katalogda ağır iş gerekirse Rust/worker mimarisini ölçümle değerlendir (gereksiz karmaşıklık ekleme).
- **Test:** "browser automation", "tarayıcı otomasyonu", "veritabanı", "database", "playwrite/playwright", Türkçe diakritikler, harf büyüklüğü, sıfır sonuç, uzun/zararlı metin, offline.
- **Kabul:** Önceden belirlenmiş **30 sorgunun en az 27'sinde** aranılan bilinen alakalı öğe ilk 5'te; 5.000 sentetik girişte sıcak arama p95 ≤ **200 ms hedefi** (test cihazı/ölçüm protokolü raporlanır); doğru çalışmayan durumda arama "başarılı" sayılmaz.

### B4-05 [P0] — Faceted filtreleme ve açık sıralama modeli `[ ]`
- **Mevcut durum:** Tür, istemci, kategori filtreleri `ThirdPartyCatalog.tsx`'te (4 birim test).
- **Bağımlılık:** B4-02, B4-04.
- **Adımlar:**
  1. Tür, kategori, AI aracı uyumu, lisans sınıfı, platform (biliniyorsa), kullanım amacı, yayıncı ve diğer doğrulanmış alanlarla bileşik filtre yap.
  2. Filtre chip'leri ve temizle eylemi ekle.
  3. Sıralamalar `alakalılık`, `yeni eklendi`, `editör seçimi`; popülerlik ancak kaynağı, zamanı ve metriği varsa.
  4. Sıfır sonuçta "filtreleri temizle / benzer öğeler" önerisi.
  5. Sonuç sayısı ve URL/route durumunu tutarlı sakla.
- **Mimari/dosyalar:** `src/shell/views/ThirdPartyCatalog.tsx` içindeki mevcut filtreler → birleşik `store/filter.ts` ve Store UI; TS store modelleri.
- **Test:** 3+ filtrenin kesişimi; bilinmeyen lisans/uyumun yanlış "destekli" görünmemesi; filtre geri alma; uzun liste performansı; boş sonuç.
- **Kabul:** En az 6 test edilmiş filtre boyutu; bütün kombinasyon test fixture'larında beklenen küme; sıralama sonucu deterministik; "en popüler" iddiası kanıtsız gösterilmiyor.

### B4-06 [P0] — Etkileyici Store ana sayfası ve vitrin `[ ]`
- **Bağımlılık:** B4-01, B4-02, B4-03, B4-05.
- **Adımlar:**
  1. Hero vitrini (editoryal seçilen gerçek öğe).
  2. "Editörün seçtikleri", "Yeni eklenenler", "Vibe coding için başlangıç", "Web geliştirme", "Veritabanı", "Tarayıcı otomasyonu" gibi senaryolu koleksiyonlar.
  3. MCP/Skill/Plugin/App türleri için keşif girişleri.
  4. Yeterli içerik yoksa boş sahte carousel göstermeme; sorunsuz fallback.
  5. Vitrini kategori ve detay sayfalarına bağla.
- **Mimari/dosyalar:** Yeni önerilen `StoreHome.tsx`, `StoreHero.tsx`, `StoreSection.tsx`, `StoreItemCard.tsx`, editoryal koleksiyon metadata'sı.
- **Test:** 0/1/100 ürün, offline, broken media, hero tıklama, klavye, küçük pencere, 125/150% Windows ekran ölçekleme.
- **Kabul:** Kullanıcı açılıştan **en fazla 2 etkileşimle** ilgi duyduğu bir kategoriye, **en fazla 3 etkileşimle** öğe detayına ulaşabilir; yatay taşma/ölü CTA yok; vitrin gerçek ve editoryal kaynaklı kayıtlar kullanıyor.

### B4-07 [P0] — Ürün detay sayfası: GitHub README yerine anlaşılır anlatım `[ ]`
- **Bağımlılık:** B4-02, B4-03, B3-07.
- **Adımlar:**
  1. Net kısa tanıtım: "Nedir?", "Ne işe yarar?", "Hangi problemi çözer?".
  2. 2–3 **doğrulanmış/örnek** kullanım senaryosu.
  3. Desteklenen AI araçları, gereksinimler, sınırlar, tür, lisans, geliştirici/yayıncı, kaynak/dokümantasyon, son doğrulama zamanı.
  4. Repo + alt artifact hiyerarşisini ve ilgili eklenti içeriğini göster.
  5. GitHub'da aç, doküman, favori düğmeleri.
  6. "Bu uygulama Hub tarafından kurulmaz" açık bilgisi.
- **Mimari/dosyalar:** Önerilen `StoreItemDetail.tsx`, `StoreItemMedia.tsx`, `StoreItemFacts.tsx`, `StoreItemRelated.tsx`; model ve route.
- **Test:** Tam/zayıf metadata; repo çoklu artifact; 404/güncelliği bilinmeyen repo; bilinmeyen sürüm/lisans; devasa açıklama; XSS tehlikesi yaratan kaynak metni.
- **Kabul:** Onaylı **her katalog öğesi** kendi detay ekranında açılır; kaynak ve lisans durumları dürüst; 2–3 örnek senaryosunu destekleyen içerik kontrolü yapılır (gerçek bilgi yoksa uydurma senaryo eklenmez). Beş temsilî detay sayfası içeriği elle QA'dan geçer.

### B4-08 [P1] — İçerik türüne göre özel detay blokları `[ ]`
- **Bağımlılık:** B4-07.
- **Adımlar:**
  1. MCP için `tools/resources/prompts` gibi yetenekleri yalnız kanıt varsa açıkla; çalıştırmadan spekülasyon yapma.
  2. Skill için `SKILL.md` amacı, ön koşullar ve örnek iş akışı; izinli kısa alıntı ve attribution.
  3. Plugin için içerdiği Skills/Commands/Agents/Hooks/MCP alt bileşenleri, manifest kaynaklıysa.
  4. App için OS/platform ve proje linkleri.
  5. Rule/Instructions için kapsam/format açıklaması.
- **Mimari/dosyalar:** `StoreItemDetail.tsx`, `StoreItemFacts.tsx`, şema v2, opsiyonel `artifactComponents`.
- **Test:** MCP hiç tool listesi yok, plugin 3 alt tür içeriyor, farklı repo alt skill, app'te AI client geçersiz, readme eksik.
- **Kabul:** Her desteklenen artifact türünde boş veri nedeniyle yanlış başlık veya uydurma "yetenek" gösterilmiyor; alt bileşen varsa doğru kaynağa bağlı.

### B4-09 [P1] — Benzerler, alternatifler ve editoryal koleksiyon sayfaları `[ ]`
- **Bağımlılık:** B4-02, B4-03, B4-07.
- **Adımlar:**
  1. Benzerlik için ortak kullanım amacı, kategori, tür ve kanıtlı uyumluluğu kullan.
  2. "Alternatif" etiketini aynı işi gerçekten yapan adaylarla sınırla.
  3. Koleksiyon açıklaması, içerik gerekçesi ve listeleme tarihi.
  4. Aynı öğeyi tekrar tekrar gösteren döngüleri temizle.
- **Mimari/dosyalar:** Önerilen `store/recommendations.ts`, `StoreCollection.tsx` ve metadata `collections.json` (öneri).
- **Test:** Aynı repo alt öğeleri, türü farklı ama işlevi benzer, hiç alternatif yok, 500 kayıt, koleksiyonun bir öğesi kaldırıldı.
- **Kabul:** Doğrulanan örnek öğelerde en az 3 anlamlı ilgili öneri (varsa) ve bunların açıklanabilir eşleşme nedeni; yetersiz veri için uydurma öneri yapılmaz.

### B4-10 [P1] — Yerel favoriler / Sonra bak `[ ]`
- **Mevcut durum:** Yok (F-14). Hub'ın kendi kayıt altyapısı (`projects.json` deseni, atomik yazma) örnek alınabilir.
- **Bağımlılık:** B3-05 (Hub-only write istisnası), B4-02, B4-07.
- **Adımlar:**
  1. Bir kullanıcı hesabı oluşturmadan favoriye ekle/çıkar.
  2. Sürüm güncellemesinde korunacak Hub-owned kayıt (schema version, toleranslı parse, atomik yazma).
  3. Mağaza, arama, detay, kaydedilenler ekranları aynı state'i görür.
  4. Katalogdan kaldırılan kayıt favoride kaybolmak yerine "katalogdan kaldırıldı" gibi açıklamalı durum alır.
- **Mimari/dosyalar:** Önerilen `integrations/favorites.rs` veya Hub settings storage, Tauri dar IPC, `SavedItems.tsx`, TS durum yönetimi.
- **Test:** 100 favori, çift tık/idempotence, Hub yeniden başlatma, bozuk kaydedilenler JSON, katalog kaydı silinmesi, offline.
- **Kabul:** Favori kaydı yalnız Hub alanını değiştirir; 0 AI config yazımı; 100 öğelik fixture'da state tutarlı; offline okunur.

### B4-11 [P0] — Güvenilir bağlantılar ve kaynak doğruluğu `[~]`
- **Mevcut durum:** `is_verified_source_url` yalnızca `https://github.com/{owner}/{repo}` kabul eder; ancak `target=_blank` açılışının WebView2'de gerçekten sistem tarayıcısına yönlendiği ve opener eklentisi/izin ihtiyacı doğrulanmadı (F-13).
- **Bağımlılık:** B4-02, B4-03.
- **Adımlar:**
  1. GitHub source URL doğrulamasını sürdür.
  2. Dokümantasyon URL'leri eklenirse açık host/şema/redirect politikası ve görünür hostname.
  3. Sahte resmî etiketleri engelle; "verified publisher" yalnız somut kanıta göre.
  4. Tıklanınca harici açma işlemi yetki sınırında kalır (gerekirse dar bir opener izni; keyfî URL açan genel komut yok); link açma otomatik download/kurulum tetiklemez. Önce mevcut `target=_blank` davranışını native testte doğrula, sonra gerekçeyle karar ver.
  5. Toplu link denetimini yayın pipeline'ına al.
- **Mimari/dosyalar:** `integrations/catalog.rs`, `security.rs`, frontend link bileşenleri, Tauri URL/open allowlist, `capabilities/default.json` ve CSP.
- **Test:** `javascript:`, `file:`, `http:`, sahte `github.com.evil`, kontrol karakterleri, phishing/redirect, silinen link, eksik `docs_url`.
- **Kabul:** Uygulama içinden keyfî protokol ile yürütme yok; kaynaklar harici tarayıcıda açılır ve kullanıcı gitmeden önce açıkça kaynak adı görülür; geçersiz URL'ler katalog kalite kapısında yakalanır.

### B4-12 [P0] — İmzalı katalog, offline cache ve sürüm migrasyonu `[~]`
- **Mevcut durum:** Ed25519 + alan ayrımlı `respected-thirdparty-catalog-v1\n`, 4 MiB sınır, sequence, stable/beta kanal, son doğrulanmış önbellek, `remote_catalog_e2e` var. **Yayımlı uzak kaynak yok** (F-08); v2 şemanın eski istemcilere etkisi (F-12).
- **Bağımlılık:** B4-02, B4-03.
- **Adımlar:**
  1. Mevcut Ed25519 domain-separated katalog imzasını koru.
  2. Şema v2 sürüm geçişinin eski önbellek ve yeni binary ile uyumunu açık tanımla.
  3. Güncellemeyi yalnız kullanıcı başlattığında veya açıkça tercih verdiyse yap; kapalı ağda son doğrulanmış snapshot fallback'i.
  4. Size/TTL/downgrade/sequence kontrolleri.
  5. Katalog imzasının yalnız metadata bütünlüğü olduğunu UI'da açıkla (`SIGNATURE_SCOPE_NOTE` korunur).
  6. İlk gerçek uzak katalog yayınını (`third-party-catalog` Release'i) üret, imzala ve yayın pipeline'ına bağla.
  7. Bugünkü açılışta arka planda çalışan `lifecycle::catalog::refresh` (Hub indeksi) ile bu "kullanıcı başlatır" ilkesi arasındaki farkı belgele: Hub self-update denetimi de kullanıcıya şeffaf olmalı.
- **Mimari/dosyalar:** `integrations/catalog.rs`, `trust.rs`, katalog cache alanı ve güncelleme komutları; mevcut lifecycle ürün kataloğundan ayrım.
- **Test:** İmza bozuk, digest bozuk, geri giden sequence, eski format, yarım indirme, ağ timeout, 4 MiB üstü, cache bozuk, offline start.
- **Kabul:** Başarısız güncelleme **önceki çalışan kataloğu bozmaz**; imzasız veri kullanılmaz; 0 zorunlu ağ isteğiyle mağaza açılır; şema geçişinin sonucunu test eden fixture vardır. Şema büyüklüğü 4 MiB sınırını aşarsa yükseltme kararı ve gerekçesi ayrıca belgelenir.

### B4-13 [P1] — Medya, yayıncı kimliği ve telif/attribution kuralları `[ ]`
- **Mevcut durum:** CSP `img-src 'self' data:` — uzak görsel zaten engelli; medya sistemi yok.
- **Bağımlılık:** B4-02, B4-06.
- **Adımlar:**
  1. Her medya için kullanım izni/lisans ve orijinal URL/kaynak kaydı.
  2. İndirilen/derlemeye eklenen resimler için boyut/format/metadata doğrulama.
  3. Kullanıcı cihazında izinsiz üçüncü taraf tracker veya keyfî uzak görsel fetch yok.
  4. Bozuk görselde premium ama doğru placeholder.
  5. Logo olmayan projeye uydurma resmî logo üretme; editoryal kapaksa açıkça editoryal olduğunu belirt.
- **Mimari/dosyalar:** Mağaza medya sistemi, içerik schema v2, `assets/store/` (öneri), CSP ve yayın/attribution manifest'i.
- **Test:** Bozuk jpeg/svg, dış tracker URL, 20 MB görsel, erişilemeyen imaj, light/dark okunabilirlik, attribution eksik.
- **Kabul:** Yayınlanan tüm paketli medya kullanım hakkı ve kaynak alanına sahip; lisans veya kaynak doğrulanmamış medya eklenmez; offline mağazada bozuk görsel yerine kontrollü placeholder vardır.

### B4-14 [P1] — Görsel sistem, mikro etkileşim ve erişilebilirlik `[ ]`
- **Bağımlılık:** B4-01, B4-06, B4-07.
- **Adımlar:**
  1. Tipografi, spacing, kart yoğunluğu, görsel hiyerarşi, focus/hover/active, skeleton/empty/error state token'ları.
  2. Ekran okuyucu etiketleri, semantik başlıklar, klavye ve odak sırası.
  3. Animasyon tercihleri (`prefers-reduced-motion`) ve performans bütçesi.
  4. Türkçe metinlerde kırılma, Windows text scaling.
  5. Steam hissini özgün Respected marka kimliğine uyarla.
- **Mimari/dosyalar:** `src/shared/ui.tsx`, `src/shared/design.css`, mağaza bileşenleri, erişilebilirlik testleri.
- **Test:** 1280×720, 1366×768, 1920×1080, %125/%150 ölçek, klavye-only, renk kontrastı, light/dark, reduced-motion.
- **Kabul:** Bloke eden klavye/okuma sorunu **0**; hiçbir CTA örtüşmüyor/taşmıyor; modal/harici link odak davranışı test edildi; yeni mağaza kartlarında tutarlı tasarım token kullanımı.

### B4-15 [P1] — Mağaza performansı ve içerik hacmi dayanıklılığı `[ ]`
- **Bağımlılık:** B4-04, B4-05, B4-06.
- **Adımlar:**
  1. 100, 250, 1.000, 5.000 öğelik sentetik fixture ile arama, kategori ve detay ölçümü.
  2. Kart render işlerini sınırla, gerekiyorsa liste virtualize et.
  3. Katalog parse ve indeks oluşturma maliyetini ölç.
  4. WebView memory/leak kontrolü; UI ana thread'de takılma önlemi.
- **Mimari/dosyalar:** Store arama/filtre/veri seçiciler, performans testleri, gerekirse memo/cache.
- **Test:** 5.000 öğe, hızla yazılan 100 sorgu, filtreleri art arda aç/kapat, 30 dakika gezinme, offline/online geçiş.
- **Kabul:** Ölçümlü arama p95 hedefi ≤200 ms (B4-04 ile aynı metodoloji; tek ölçüm protokolü kullanılır); ilk yükleme ve bellek bütçesi RC'de baseline+hedef olarak raporlanır; ölçüm yapmadan "çok hızlı" iddiası yazılmaz.

### B4-16 [P0] — Beta.4 mağaza kabul testi ve release gate `[ ]`
- **Bağımlılık:** B4-01..15 (P2 işleri sürüm kararıyla ertelenebilir, P0 atlanamaz).
- **Adımlar:**
  1. 8–12 hedef kullanıcıyla pilot test; yeni başlayan/tecrübeli karışık.
  2. Üç görev ver: "bir browser MCP bul", "iki aracı karşılaştır", "lisans/kaynak ve uyumluluğu doğrula".
  3. İçerik ve görsel QA.
  4. Windows native smoke, offline/imza/arama/favori testleri.
  5. Eksikleri beta.5/RC önceliklerine taşı.
- **Mimari/dosyalar:** `tests/store/*`, `tests/native/*`, araştırma özeti bu belgede (§14), release acceptance.
- **Test:** Pilot görev tamamlama, arama altın küme testleri, offline açılış, invalid katalog, Windows UI/klavye, favorites restart.
- **Kabul:** Pilot testte **≥%90 görev tamamlama öneri hedefi** (küçük örneklem; dışa dönük istatistik iddiası değil); 30 sorgu hedefi sağlandı; P0 mağaza hatası **0**; katalogda sahte verified/yanlış install vaadi **0**.

---

# 8. Beta.5 — Yerel AI ekosistemi / salt okunur Library

**Ürün hedefi:** Bilgisayarda tanımlı MCP, Skills, Plugins ve desteklenen rule/instruction kaynaklarını bulup doğru AI istemcisi ve doğru kapsamla göstermek. "Kullanıyor" yerine **kanıtlı statü**. Kullanıcı isterse seçtiği projeler için özel görünüm alabilir; **hiçbir AI yapılandırması değiştirilmez**.

**Kritik ayrım:** "Otomatik tarama" kullanıcının bütün diskini/iş projelerini sessizce tarama izni değildir. Hub, belgelenmiş **global istemci konfigürasyonlarını** otomatik okur. Proje bazlı görünürlük, **kullanıcının açıkça seçtiği proje kökleri** için sunulur. Sistem neyi taramadığını dürüstçe anlatır.

**Genel mevcut durum:** Keşif, envanter, çözümleyici, skill/plugin tarayıcıları ve 5 adapter koddadır; fixture testleri vardır (`claude_adapter_e2e` 5, `codex_adapter_e2e` 4, `cursor_gemini_opencode_adapter_e2e` 5, `inventory_e2e`, `plugins_e2e` 4, `resolver_e2e` 5, `skills_e2e` 3 — QUALITY.md). Beta.5 işlerinin çoğu *doğrulama, sıkılaştırma ve ürünleştirmedir*; sıfırdan ikinci bir sistem kurulmaz.

**Çıkış kapısı:** Resmî supported yetenek testleri %100; üretim derlemesinde AI config yazma **0**, üçüncü taraf kod çalıştırma **0**, kritik gizlilik ve yanlış etiketleme **0**; Windows native smoke başarılı.

### B5-01 [P0] — Adapter kapasite matrisi ve tarama kapsamı `[~]`
- **Mevcut durum:** 5 adapter ve `discovery.rs` mevcut; adapter'lar `read=true, plan=true, apply=false, undo=false` ilan eder (ARCHITECTURE.md). Klasör varlığı tek başına `enabled` sayılmaz (DISC-001). Matris belgesi ve sürüm/kaynak tarihleri yok.
- **Bağımlılık:** B3-05; mevcut read-only altyapının güncel testlerle doğrulanması.
- **Adımlar:**
  1. Claude Code, Codex, Cursor, Gemini CLI, OpenCode mevcut adapter'larını incele.
  2. Her birinde hangi OS/istemci sürümü/konfigürasyon katmanının okunduğunu bir tabloda topla.
  3. AI uygulamasının kurulu olması, config bulunması, ilgili özelliğin desteklenmesi ayrımını yap.
  4. "Okunur / bilinmiyor / desteklenmiyor / izin verilmedi" yeteneklerini ayrı belirt.
  5. Kaynakları ve sürüm tarihlerini dokümante et; tahminle yeni path ekleme.
  6. `plan`/`apply`/`undo` yetenek bayraklarını (B3-05 sonrası) veri modelinden ve UI matrisinden kaldır.
- **Mimari/dosyalar:** `integrations/adapters/*`, `integrations/discovery.rs`, `integrations/model.rs`, `docs/ARCHITECTURE.md`, `README.md` destek matrisi.
- **Test:** Uygulama yok, uygulama var ama ayar dosyası yok, ayar var ama app kurulu değil, uygulama çok eski/yeni, izin reddi, farklı kullanıcı kökü.
- **Kabul:** 5 istemci için belgelendirilmiş **en az global read-only keşif**; her kaynak/kapsamın supported/unsupported/unknown bildirimi var; kanıtsız "tam destek" yok.

### B5-02 [P0] — Güvenli, sınırlandırılmış istemci tespiti `[~]`
- **Bağımlılık:** B5-01.
- **Adımlar:**
  1. Yalnız bilinen global konfigürasyon yollarını otomatik tara.
  2. Uygulama çalıştırmadan ve hiçbir kaynak dosyaya yazmadan varlık kanıtı topla.
  3. Tarama derinliği, dosya boyutu, symlink sayısı, timeout ve iptal sınırı koy.
  4. Dosya bulunmadığında ve izin verilmediğinde farklı hata/boş durum üret.
  5. Proje taramasını B5-09'daki açık kullanıcı seçimine bağla.
- **Mimari/dosyalar:** `integrations/discovery.rs`, `integrations/adapters/*`, Tauri asenkron read-only IPC.
- **Test:** 5 sahte kullanıcı profili, büyük klasör, symlink loop, izin reddi, ağ sürücüsü, kilitli dosya, iptal, çoklu tarama isteği.
- **Kabul:** Uygulama kurulmadı/yoksa yanlış pozitif **0** (tanımlı fixture kümesinde); iptal/tarama kilidi UI'ı dondurmuyor; üçüncü taraf config'lerinde 0 değişiklik ve 0 program yürütme.

### B5-03 [P0] — MCP konfigürasyonlarını yorumlama ve ilişki üretme `[~]`
- **Bağımlılık:** B5-01, B5-02.
- **Adımlar:**
  1. JSON, JSONC, TOML farklarını adapter başına oku.
  2. Config içinde kayıt, explicit disabled, aktif proje katmanı, config override ayrımını koru.
  3. Sır içerebilen env/command/URL alanlarının içeriğini UI/log'a taşımadan tanımsal alanları çıkar.
  4. Her MCP Binding için kaynak dosya, client, scope, kanıt ve belirsizlik ekle.
- **Mimari/dosyalar:** `integrations/adapters/*`, `integrations/model.rs`, `integrations/resolver.rs`.
- **Test:** Geçerli/bozuk `claude.json`, `.mcp.json`, `config.toml`, Cursor MCP JSON, Gemini CLI settings, OpenCode JSONC; unknown key, yorum, trailing comma, disabled false, iki scope aynı ad.
- **Kabul:** Her hedef istemcide en az **8 MCP parser/status fixture**; testlenen bütün fixture'larda kaynak ve scope doğru, gizli veri ekrana/log'a düşmüyor; bozuk kaynak diğer istemcinin envanterini engellemiyor.

### B5-04 [P0] — Skills, Plugins ve alt bileşenleri güvenle tespit et `[~]`
- **Bağımlılık:** B5-01, B5-02.
- **Adımlar:**
  1. Geçerli `SKILL.md` kaynaklarını ve manifest/izinli klasörleri tara.
  2. Sadece isim eşleşmesine göre varlık birleştirme; symlink canonical hedef, `source_path`, hash ve `asset_key` ilişkileri ile ayır.
  3. Bir Plugin içindeki Skills, Commands, Agents, Hooks, MCP'leri **manifest/sabit kaynak kanıtı varsa** parent-child ilişkisiyle göster.
  4. Kaynağın script/hook/komutunu **asla çalıştırma**.
  5. Çeşitli AI araçlarının farklı Skills keşif kurallarını tek ortak kuralmış gibi varsayma.
- **Mimari/dosyalar:** `integrations/skills.rs`, `integrations/plugins.rs`, `integrations/adapters/*`, `integrations/model.rs`.
- **Test:** Boş `SKILL.md`, hatalı YAML/frontmatter, git submodule/symlink, aynı dosyaya 2 bağ, iki farklı kopya, plugin içinde 3 skill ve 2 hook, bozuk plugin manifest, 1000 klasör.
- **Kabul:** Fixture'larda plugin alt bileşen kaybı **0**, farklı içeriklerin yanlış tek asset'e birleşmesi **0**, kırık link açık uyarı, otomatik üçüncü taraf kod yürütme **0**.

### B5-05 [P0] — Ortak envanter modeli: Asset / Source / Binding / Client / Scope `[~]`
- **Mevcut durum:** `model.rs` `Asset, Client, Source, Binding, Scope, Capability, ObservedStatus` modelini içerir (CORE-002).
- **Bağımlılık:** B5-03, B5-04, B4-02.
- **Adımlar:**
  1. Adı aynı kaynakları değil **kimliği doğrulanmış** varlıkları birleştir.
  2. Aynı MCP farklı client'a bağlandıysa tek `Asset` altında birden çok `Binding` göster.
  3. Source hangi dosya/klasör/manifestte bulundu, parmak izi ve sahibi ne, kaydet.
  4. Katalogda olmayan yerel öğeyi de envantere dahil et.
  5. Veri snapshot'ını yeniden oluşturulabilir ve gerçek kaynağa dayalı tut.
  6. `Owner::Product`/`Owner::Catalog` sahiplik kavramı (Brain/Orchestrator girdilerini "yazılamaz" işaretlemek için eklenmişti) v1.0'da yalnız *bilgi* olarak kalır; yazma korumasına dayalı mantık sadeleştirilir.
- **Mimari/dosyalar:** `integrations/inventory.rs`, `integrations/model.rs`, `src/shared/integrationsTypes.ts`.
- **Test:** Aynı MCP 3 client; aynı adlı ama farklı repo Skill; Plugin alt Skill vs bağımsız Skill; değişen canonical path; katalogda olmayan öğe; kaybolan config.
- **Kabul:** Aynı varlığın 3 client ve 2 scope bağları ayrı kayıtlar olarak korunur; belirlenmiş fixture kümesinde yanlış birleştirme **0**; bir config'in hatası toplam taramayı çökertmez.

### B5-06 [P0] — Gerçekte etkili yapılandırmayı (effective) hesapla `[~]`
- **Mevcut durum:** `resolver.rs` beş istemci × altı kapsam fixture'ıyla testli (DISC-007, `resolver_e2e` 5 test; 10 birim test); gölgelenme/drift analizi vardır (ADV-002). "unknown/untrusted/overridden" tam kanıt modeli doğrulanacak.
- **Bağımlılık:** B5-03, B5-05.
- **Adımlar:**
  1. Client başına gerçek config öncelik kuralını belgeyle/modelle: global, kullanıcı, proje, environment, trust ve özel override katmanları.
  2. `configured`, `effective`, `disabled`, `overridden`, `untrusted`, `unknown` durumları için açık kanıt üret.
  3. Bir dosya var diye AI'nın onu kullandığı sonucuna varma.
  4. Kanıt azsa `unknown` döndür; özellikle eksik/okunamayan katmanda yanlış "etkin" gösterme.
  5. "Gerçek kullanım kanıtlandı" statüsü ayrı tutulur: bulundu / tanımlı / etkin olabilir / devre dışı / gölgelenmiş / bilinmiyor / gerçek kullanım kanıtlandı ayrımı kaybolmaz (Hub kullanım izlemez; son statü yalnızca ileride gerçek kanıt kaynağı olursa kullanılır).
- **Mimari/dosyalar:** `integrations/resolver.rs`, adapter durum metadata'sı, `src/shared/integrationsTypes.ts`.
- **Test:** Aynı MCP global ve proje config'te, iki ayrı proje, explicit disabled, trust edilmeyen Codex projesi, plugin disabled ama skill dosyası mevcut, env override bilinmiyor, client var ama kurulu değil.
- **Kabul:** İstemci başına **en az 8 scope/effective fixture** (toplam ≥40); dokümante edilmiş her durumda beklenen sonuç %100; kanıtsız `used` durumu **0**.

### B5-07 [P0] — Kütüphane ekranını ürün seviyesinde yeniden tasarla `[~]`
- **Mevcut durum:** `IntegrationsLibrary.tsx` + `integrationMatrix.ts` istemci × kapsam matrisi ve filtreler (UI-001) vardır; `Library.tsx` şu an ürün (Brain/Orchestrator) çalışma alanıdır ve B3-03'te kalkar.
- **Bağımlılık:** B5-05, B5-06, B4-14.
- **Adımlar:**
  1. Mevcut `IntegrationsLibrary.tsx` tablosunu hem kart/liste hem ayrıntılı matris olarak iyileştir.
  2. Üstte kullanıcıya anlaşılır özet: kaç AI bulundu, kaç tanımlı MCP/Skill/Plugin, kapsam dağılımı, kaç belirsiz/broken kaynak.
  3. Tür, AI client, proje/global ve statü filtreleri; arama.
  4. Kaynak ve kanıt ayrıntılarına erişim.
  5. Hiçbir toggle, taşınma, düzeltme veya "Kur" eylemi gösterme.
- **Mimari/dosyalar:** `src/shell/views/IntegrationsLibrary.tsx`, `src/shell/integrationMatrix.ts`, UI ortak bileşenler.
- **Test:** 0/1/50/200 item, uzun path, 5 client, aynı asset'in 6 binding'i, klavye, dar pencere, offline, renk/görme erişilebilirliği.
- **Kabul:** En az 3 tür + 5 client + scope + status filtresi doğru; her Binding'in gerçek kaynağı gösterilebilir; gösterilmeyen durumlar "bulunamadı/bilinmiyor" ayrımıyla açık; yazma CTA'sı **0**.

### B5-08 [P1] — Her entegrasyon için ayrıntılı "nerede ve neden" görünümü `[ ]`
- **Bağımlılık:** B5-07. Store bağlantısı B5-10 bitince aynı ekrana bağlanır.
- **Adımlar:**
  1. Kütüphane item detayında `Client × Scope × Source × Status` matrisi.
  2. "Neden etkin / neden devre dışı / neden bilinmiyor?" kısa açıklaması.
  3. Dosya yolunu kopyalama ve dosyayı **OS dosya yöneticisinde gösterme** yalnız kullanıcı eylemiyle; uygulamanın içeriğini yürütme/açma komutu yok.
  4. Sır içeren command/URL/env değerlerini gizle.
  5. Katalogda karşılığı varsa Store detayına git.
- **Mimari/dosyalar:** Önerilen `LibraryItemDetail.tsx`, inventory source metadata'sı; dar ve güvenli `reveal-in-file-manager` eylemi varsa doğrula (şu an böyle bir izin/komut yok; eklenecekse capability kapsamı B5-08'de tasarlanır).
- **Test:** Uzun UNC path, escape karakteri, missing file, secrets in source, non-ASCII path, symlink and broken link, anlık silinen kaynak.
- **Kabul:** Kullanıcı her test vakasında somut kaynak + kanıtı görebilir; redaksiyon testleri 100% başarılı; OS eylemi ile hiçbir config içeriği değiştirilmez.

### B5-09 [P0] — Projeler: yalnızca kullanıcının seçtiği kökleri göster `[~]`
- **Mevcut durum:** `projects.rs` (`load/add/remove`, 6 birim test), `Projects.tsx`, `projectScope.ts`, IPC `projects_*` var (UI-002); `inventory_*` testi 3 proje + eksik kök fixture'ı içerir. Projeler ayrı üst gezinme öğesi değil (`HUB_NAV`'da yok).
- **Bağımlılık:** B5-02, B5-06, B5-07.
- **Adımlar:**
  1. Mevcut `projects.rs`/`Projects.tsx` kayıt defterini koru.
  2. Kullanıcının seçtiği proje kökünü canonical normalize et.
  3. Proje ekle/çıkar, mevcut/missing, farklı client'larda global/proje binding'lerini göster.
  4. Kullanıcının genel diskinde otomatik proje araması yapma.
  5. Proje listesi yazımı ile AI config okumasını açıkça ayır.
- **Mimari/dosyalar:** `integrations/projects.rs`, `src/shell/views/Projects.tsx`, `src/shell/projectScope.ts`.
- **Test:** Proje ekle/sil/yeniden başlat, kök symlink, taşınmış proje, nested projeler, izin reddi, Codex untrusted proje, 3 proje farklı scope.
- **Kabul:** En az 3 proje fixture'ında doğru bağ/kapsam; Hub yalnız **kendi proje defterine** yazıyor; bütün AI config hash'leri işlem önce/sonra birebir eşit.

### B5-10 [P0] — Store ↔ Library bağını yanlış pozitif olmadan kur `[ ]`
- **Bağımlılık:** B4-02, B4-07, B5-05.
- **Adımlar:**
  1. Katalog metadata `item_id/source` ile yerel `asset_key/source` eşleştirme stratejisi.
  2. Repo adı/başlık benzerliği tek başına eşleşme kanıtı sayılmaz; doğrulanabilir kimlik/provenance veya güvenli fingerprint.
  3. "Uyumlu", "yerelde tanımlı", "başka projede tanımlı", "katalogda yok" ayrımı.
  4. Store öğesinde **kanıtlı** yerel varlık varsa Library'ye bağlantı.
  5. Yerel ama katalogda bulunmayan öğeler Library'de görünmeye devam eder.
- **Mimari/dosyalar:** `integrations/inventory.rs`, Store↔Library eşleştirme katmanı (önerilen), `StoreItemDetail.tsx`, `IntegrationsLibrary.tsx`.
- **Test:** Başlık aynı fakat farklı repo, katalogda var yerelde yok, yerelde var katalogda yok, aynı repo 4 Skill, 3 farklı client, eksik/yanlış URL.
- **Kabul:** Tanımlı test setinde yanlış "bilgisayarda kurulu" rozeti **0**; kaynak/istemci/kapsam bağlantısı takip edilebiliyor; belirsiz eşleşme `unknown`.

### B5-11 [P0] — Gizlilik: sır redaksiyonu, izin sınırı, otomatik kod çalıştırmama `[~]`
- **Mevcut durum:** `security.rs` redaksiyon/yol/symlink/boyut testleri (9 birim), adapter e2e'lerde `sk-live-*`/token sızıntı kontrolleri (SEC-001); inventory/IPC DTO düzeyinde toplu denetim ve byte-eşitliği entegrasyon testi doğrulanacak.
- **Bağımlılık:** B5-02..10.
- **Adımlar:**
  1. Tarayıcıya/React state'ine asla ham token/password/API key taşıma.
  2. Log, hata, diff/önizleme artığı, telemetry yokluğu kontrolü.
  3. Symlink traversal/loop, path canonicalization, file limit, gereksiz disk tarama önleme.
  4. Üçüncü taraf kaynağı import ederek komut çalıştırma yok.
  5. AI config dosyalarında tarama sonrası **byte eşitliği** kanıtlayan entegrasyon testleri.
- **Mimari/dosyalar:** `integrations/security.rs`, adapters, inventory API DTO, logging, `PRIVACY.md`, `docs/FOOTPRINT.md`.
- **Test:** URL/env içine gömülmüş token, PEM/private key, büyük Markdown, erişilemez dosya, symlink loop, çeşitli Unicode path, çoklu paralel tarama.
- **Kabul:** Test setinde **0 sır sızıntısı**, **0 AI config yazımı**, **0 üçüncü taraf komut yürütme**; hatalar gizli dosya içeriğini yankılamıyor; sınırı aşan tarama kontrollü hata üretir.

### B5-12 [P2] — Salt okunur Setup Doctor: tanılama, otomatik düzeltme yok `[ ]`
- **Bağımlılık:** B5-06, B5-07, B5-11.
- **Adımlar:**
  1. Mevcut resolver/adapter issue'larını kullanıcı diline çevir.
  2. Kırık symlink, eksik dosya, bozuk manifest, gölgelenmiş yapılandırma, güvenilmeyen proje gibi **kanıtlı** durumları sınıflandır.
  3. "Nasıl değerlendirilir?" açıklaması koy.
  4. Düzelt butonu veya config yazması ekleme.
  5. Yanlış pozitifleri test edip yüksek belirsizlikte yalnız uyarı ver.
- **Mimari/dosyalar:** `integrations/resolver.rs` issue DTO'su, önerilen `LibraryIssues.tsx`.
- **Test:** Her bulgu tipi için pozitif/negatif fixture, sahte conflict, projeden çıkmış dosya, farklı AI client kuralı.
- **Kabul:** Her gösterilen bulguda kaynağı/kanıtı var; otomatik repair **0**; zaman baskısı olursa **v1.1'e ertelenebilir** ve yayın engeli değil.

### B5-13 [P0] — Beta.5 bütünleşik kabul / gerçek cihaz doğrulaması `[ ]`
- **Bağımlılık:** B5-01..11; B5-12 opsiyonel.
- **Adımlar:**
  1. Katalog + envanter + proje birlikteliğini gerçek Windows uygulamasında test et.
  2. Beş AI adapter'ın parse/scope/status testlerini koştur.
  3. Eski kurulu olmayan client, bozuk config ve kapalı ağ durumlarını simüle et.
  4. AI konfigürasyon dosyalarının önce/sonra byte hash raporunu çıkar.
  5. P0 açık hata listesini sıfırla.
- **Mimari/dosyalar:** `src-tauri/tests/*`, `tests/native/*`, acceptance script'leri (`real_env_guard.py` gerçek ayar karşılaştırması), fixture kapsam tablosu.
- **Test:** En az 5 client × 8 fixture, 3 kullanıcı projesi, plugin içeriği, symlink, secrets, offline, Store↔Library eşleme, restart.
- **Kabul:** Resmî supported yetenek testleri %100; üretim derlemesinde AI config yazma **0**, üçüncü taraf kod çalıştırma **0**, kritik gizlilik ve yanlış etiketleme **0**; Windows native smoke başarılı.

---

# 9. v1.0 Release Candidate — özellik dondurma, kanıt, görsel düzeltmeler

**Ürün hedefi:** Beta.3–5'te yapılanları artık yeni özelliklerle dağıtmadan, **ilk gerçek kullanıcıya dağıtılabilecek ürün** seviyesine çıkar. `v1.0.0-rc.1`, gerekirse `rc.2`, `rc.3` olabilir. RC numarası tarih değil kalite kapısına göre artar.

### RC-01 [P0] — Feature freeze ve v1.0 destek tablosunu sabitle `[ ]`
- **Bağımlılık:** B4-16, B5-13.
- **Adımlar:**
  1. Beta.4 ve beta.5 görev kanıtlarını topla.
  2. P0/P1 eksikleri ve ertelenenleri kararlaştır.
  3. Her AI için gerçekten desteklenen global/proje kaynaklarını yayımla.
  4. Yazma/kurulum özelliklerinin yasak kaldığını kontrol et.
  5. Geç kalan yeni fikirleri v1.1+ listesine taşı.
- **Mimari/dosyalar:** `docs/ROADMAP.md`, `README.md`, release checklist, destek matrisi.
- **Test:** UI buton taraması ve IPC allowlist; 5 client capability matrisi; bütün P0 bağımlılıkları.
- **Kabul:** v1.0 özellik listesi çelişkisiz ve sabit; eksik P0 **0**, v1.1+ ayrımı açık; AI config'e yazma yolu **0**.

### RC-02 [P0] — Birim/entegrasyon/native testleri ile sürüm yükseltme kanıtı `[ ]`
- **Bağımlılık:** RC-01.
- **Adımlar:**
  1. `npm run check`, Rust unit/integration ve tüm katalog/adapters fixture setlerini CI zorunlusu yap.
  2. Gerçek derlenmiş Windows WebView2 arayüz testleri: Store→Search→Detail→External link, Favorites, Library, Projects.
  3. Temiz Windows kullanıcısında ilk kurulum (REL-002).
  4. Önceki beta sürümlerinden yükseltme: beta.2→beta.3→beta.4→beta.5→RC ve desteklenen direkt yükseltmeler (REL-001).
  5. Hub self-update; kaldırıcı sonrası Brain/Orchestrator dosyaları ve AI config hash'leri korunuyor mu.
- **Mimari/dosyalar:** `tests/native/hub_native_e2e.mjs`, `src-tauri/tests/*`, `common/release/tools/acceptance/accept_hub.py`, `accept_update.py`, `.github/workflows/*`, `packaging/windows/*`.
- **Test:** Çevrimdışı, yüklü AI yok, kısmi kurulum, eski Hub route state, sürüm atlama, bozuk önbellek, iki kere güncelleme.
- **Kabul:** Resmî release CI ve Windows native smoke **yeşil**; temiz kurulum ve en az bir gerçek beta→RC yükseltme zinciri kanıtlı; önce/sonra hash farkı AI config'lerde **0**.

### RC-03 [P0] — Katalog, güven ve gizlilik için adversarial test seti `[ ]`
- **Bağımlılık:** RC-01, B4-11/12/13, B5-11.
- **Adımlar:**
  1. İmza/digest/sequence/schema, dış bağlantı, HTML/Markdown/render içeriği ve görsel kaynakları için kötü niyetli fixture.
  2. İstemci config parser'ına bozuk dosya/çok büyük dosya/symlink/izin reddi enjekte et.
  3. Sırların IPC JSON ve UI/log'a girmediğini kontrol et.
  4. Üçüncü taraf executable komutu çalıştırabilen IPC kalmadığını bağımsız denetle.
  5. Yalnız read-only tarama sırasında bile fazla izin istememeye dikkat et.
- **Mimari/dosyalar:** `integrations/security.rs`, `catalog.rs`, `api.rs`, `commands/*`, Store render/URL sistemi, Tauri permissions/CSP.
- **Test:** `javascript:` URL, `github.com.evil`, XSS/prompt enjeksiyonu içeren açıklama, token/env leakage, 4 MiB+ catalog, stale cache, symlink traversal, yarım JSONC/TOML, renderer crash/reload.
- **Kabul:** Bilinen kritik/yüksek güvenlik açığı **0**, sır sızıntısı **0**, AI config yazma **0**, üçüncü taraf komut çalıştırma **0**; tüm tehlikeli fixture'lar reddediliyor/güvenli gösteriliyor.

### RC-04 [P0] — Gerçek kullanıcı deneyimi, araştırma ve erişilebilirlik doğrulaması `[ ]`
- **Bağımlılık:** RC-01.
- **Adımlar:**
  1. 8–12 pilot kullanıcıyla Beta.4 mağaza görevlerini, Beta.5 envanter görevlerini ölç.
  2. Her görevde başarı, süre, hatalı tıklama, kullanıcı açıklaması kaydet; kişisel repo yollarını rapora ham kopyalama.
  3. En çok başarısız olunan 5 UX noktasına düzeltme issue'su aç.
  4. WCAG 2.2 AA prensiplerini masaüstü arayüzüne uygulayarak klavye/focus/kontrast kontrol et.
  5. "Bu AI kullanıyor" yanlış anlama ölçümlerine göre metinleri düzelt.
- **Mimari/dosyalar:** UX metinleri, Store/Library sayfaları, tasarım sistemi, erişilebilirlik testleri, araştırma notları (özet bu belgede, §14).
- **Test:** Mağaza ürün bulma, karşılaştırma, kaynak görme; Kütüphane'de global/proje farkını anlama; yüklü olmayan istemciyi tanıma; keyboard-only.
- **Kabul:** Pilot çalışma için **≥%90 görev tamamlama hedefi**; küçük örneklem sonuçları "pazarın %90'ı" gibi sunulmaz; kritik a11y engeli **0**, UX başarısızlıklarının tamamına durum/karar verilir.

### RC-05 [P1] — Performans, bellek ve soğuk açılış bütçesi `[ ]`
- **Bağımlılık:** RC-01.
- **Adımlar:**
  1. Gerçek Windows test makinesinin CPU/RAM/OS sürümünü kaydet.
  2. Offline soğuk açılış, 250/5.000 öğe katalog, arama p95, 200 yerel binding, tarama zamanı, listede scroll ve memory tüketimini ölç.
  3. Temel ölçüm ve hedef arası farkı raporla.
  4. UI bloklama ve parsing/yüklenme darboğazlarını düzelt.
  5. 30 dakikalık gezinme/yinelemeli tarama stress testi.
- **Mimari/dosyalar:** Store search/selectors, inventory async, native perf tests, release kanıt dokümanı.
- **Test:** Offline ilk açılış, yeniden başlatma, arka arkaya 20 arama, 5 tarama, 200 satır render, düşük RAM senaryosu.
- **Kabul:** Arama p95 ≤200ms hedefi ölçümle teyit veya ölçüme dayanarak revize edilip gerekçelenir; uzun işlemler UI'ı kilitlemiyor; bellek/başlangıç benchmark raporu bulunuyor.

### RC-06 [P1] — Katalog editoryal denetimi, marka ve görsel polish `[ ]`
- **Bağımlılık:** RC-01, B4-03, B4-14.
- **Adımlar:**
  1. Katalogdaki 150–250 hedef kayıt için kaynak, ad, tür, açıklama, uyumluluk, lisans, varsa medya haklarını kontrol et.
  2. Yazım/dil/sıralama düzenlemesi; gereksiz tekrar, AI tarafından uydurulmuş içerik ve resmî olmayan "official" etiketlerini çıkar.
  3. Light/dark, boş/hata durumları, görsel tutarlılık, kart oranları, tipografi.
  4. Dış sitelere açılan bağlantılar/güncel adresler denetlenir.
- **Mimari/dosyalar:** Katalog snapshot, editoryal pipeline, UI, marka/attribution manifest, release notları.
- **Test:** Bütün ürün türlerinden örneklem ve yüksek görünürlüklü 20 vitrin öğesi %100 manuel içerik QA; bozuk kaynak/yanlış lisans taraması.
- **Kabul:** Hero ve editör seçimi alanındaki **tüm** girdiler manuel doğrulanmış; kritik yanlış kaynak/uyumluluk **0**; katalog hedef adedi içerik kalitesinden üstün tutulmaz.

### RC-07 [P0] — Release Candidate çıkış onayı / no-go kararı `[ ]`
- **Bağımlılık:** RC-02, RC-03, RC-04, RC-05, RC-06.
- **Adımlar:**
  1. RC-02..06 kanıt paketini incele.
  2. P0/P1 bug listesini kapat veya P1 ertelemesine açık gerekçe ekle.
  3. Güncelleyici, installer, metadata imzaları, offline çalışabilme ve gizlilik sözleşmesini son kez denetle.
  4. Yayın notunda v1 sınırlarını net yaz.
  5. İmzalama/SmartScreen durumu çözülemiyorsa (REL-004) kullanıcıyı yanıltmayan açık not ile ayrıca release kararı ver; güvenlik doğrulamasını asla atlama.
- **Mimari/dosyalar:** Release checklist, changelog, sürüm etiketleri, dağıtım artefaktları.
- **Test:** RC sürüm paketi temiz Windows VM'de ve mevcut betanın üzerine; bütün release çıktısında checksum ve versiyon tutarlılığı.
- **Kabul:** Açık P0 **0**; P1 yayın engelleyici **0**; kanıtı eksik test "geçti" sayılmaz; release onayı yazılı.

---

# 10. v1.0.0 Stable — ilk ciddi yayın

### V1-01 [P0] — Tekrarlanabilir release üretimi ve sürüm bütünlüğü `[ ]`
- **Bağımlılık:** RC-07.
- **Adımlar:**
  1. RC ile aynı kaynak revizyonundan production build al.
  2. Git tag, Windows x64 installer, sürüm açıklaması, SHA-256 checksum ve mümkünse Authenticode imzasını yayınla (REL-004).
  3. Hub self-update kanalına doğru artefakt/sequence/hash bilgilerini koy (`catalog` Release indeksi; aynı sürüm numarasıyla farklı içerik reddi korunur).
  4. Kritik bağımlılık ve lisans manifestini dağıtımda tut.
- **Mimari/dosyalar:** `common/release/*`, `.github/workflows/*`, `packaging/windows/*`, changelog/release notes.
- **Test:** Paket checksum doğrulaması, eski stable/beta üzerinden desteklenen güncelleme, manuel installer, clean Windows.
- **Kabul:** Tekrarlanabilir paketleme kanıtı, doğrulanmış checksum ve kaynak tag'i; bozuk veya yanlış imzalı paket kabul edilmiyor; imza yoksa bu durum net açıklanıyor ve risk kabulü kaydediliyor.

### V1-02 [P0] — Kamuya dönük ürün anlatımı ve destek dokümanları `[ ]`
- **Bağımlılık:** V1-01.
- **Adımlar:**
  1. `README` ve release sayfası: Store + Read-only Library sözünü doğru anlat.
  2. 5 istemci için gerçek capability desteği ve sınırlı proje taramasını açık yaz.
  3. "Mağazadan nasıl keşfederim?", "Kütüphanede nasıl görürüm?", "Neden bilinmiyor?", "Neden Kur düğmesi yok?" sorularını yanıtla.
  4. Privacy, cache, uninstall/leftover, güvenlik, lisans, issue template, bilinen sınırlamalar.
- **Mimari/dosyalar:** `README.md`, `PRIVACY.md`, `docs/FOOTPRINT.md`, `docs/ARCHITECTURE.md`, docs yardım sayfaları, GitHub Releases.
- **Test:** Belge ekran görselleri ve metin ile gerçek binary karşılaştırması; bütün linkler ve indirme notları.
- **Kabul:** Dokümanlarda **hiçbir üçüncü taraf kurulum/taşıma vaadi yok**; 5 istemci destek sınırı gerçek; kullanıcıların temel 4 sorusunun cevabı mevcut.

### V1-03 [P0] — Stable sonrası ilk ölçüm ve hata triage protokolü `[ ]`
- **Bağımlılık:** V1-01, V1-02.
- **Adımlar:**
  1. GitHub Issues hata/özellik şablonları; kullanıcıdan sır veya bütün config içeriği istemeyen güvenli hata bildirme talimatı.
  2. İlk kritik hatalarda hotfix/rollback release politikası.
  3. v1.1+ fikir havuzunu v1 hedefleriyle karıştırmadan sürdür.
  4. Telemetri olmadan, kullanıcıların **isteğe bağlı bildirdiği** geri bildirim ve support issue'ları üzerinden iyileştirme.
- **Mimari/dosyalar:** GitHub issue templates, release runbook, dokümanlar.
- **Test:** Bozuk config bildirimi için sırları gizleme şablonu, "kaynak linki 404" issue'su, kritik hotfix karar senaryosu.
- **Kabul:** Açık hata sınıflandırma/triage politikası, güvenli geri bildirim kanalı ve sonraki sürüme taşınan görevlerin eksiksiz listesi.

---

# 11. v1.1+ — Bilinçle ertelenenler (PARK)

Bu liste **silinmiş fikirler değildir**; v1.0 kapsamı ve güvenlik sınırını korumak için ertelenmiştir. Açılmaları **yeni ürün kararı, ADR revizyonu, tehdit modeli ve bağımsız release planı** gerektirir. "Planlandı" demek "v1'de yapılacak" demek değildir.

| ID | Ertelenen konu | Neden şimdi yok? | Gelecekte açma önkoşulu |
|---|---|---|---|
| PARK-01 | Üçüncü taraf MCP/Skill/Plugin kurma, kaldırma, güncelleme | Keyfî kod/komut çalıştırma, kaynak sahipliği ve platform riskleri | Tehdit modeli, kullanıcı onayı, kaynak sahipliği, installer testleri |
| PARK-02 | Config dosyası yazma, diff/apply, transaction, undo, rollback | Kullanıcı ayarı kaybı riski; ilk sürüm salt okunur | ADR, gerçek parser round-trip, hash/atomik yazma/crash recovery testleri |
| PARK-03 | Agent bazlı enable/disable ve global↔proje taşıma | İstemciler arası farklı scope/precedence; yanlış değişiklik başka projeleri bozar | Client capability sözleşmeleri, etkiler önizlemesi, bağımsız rollback kapısı |
| PARK-04 | Profile/preset uygulama, toplu sync, dışarıdan import | Çoklu dosya/binding sahiplik karmaşıklığı | PARK-02/03 geçmeden açılamaz |
| PARK-05 | Canlı MCP tool çağrısı, token/kota analizi, API proxy/gateway | Ürünü kontrol merkezi yerine altyapı motoruna çevirir; kapsam büyür | Bağımsız ürün doğrulama ve kullanıcı değeri kanıtı |
| PARK-06 | macOS/Linux resmî destek (ve PORT-002 GUI/paketleme doğrulaması) | Platform adapter yolları ve gerçek native test yok | Platform CI, installer ve gerçek test cihazı |
| PARK-07 | Brain/Orchestrator özel Hub yönetimi | **Ürün kararıyla kalıcı kaldırıldı**, normal katalog öğesi olabilir | Varsayılan olarak **geri açılmayacak** |
| PARK-08 | Bulut hesapları, online yorumlar, community ratings, sync | Hesapsız/offline deneyime ters; backend/moderasyon maliyeti | Ayrı MVP, gizlilik ve maliyet kararı |
| PARK-09 | Otomatik AI önerileri ve kişiselleştirme | Veri toplama/önyargı/ölçüm maliyeti | Yerel-öncelikli deney ve fayda ölçümü |
| PARK-10 | Gelişmiş Git diff/push, CLI otomasyonu ve eklenti editörü | v1'in keşif + gözlem misyonu dışında | Kullanıcı araştırması ve yeni release |

**Kritik not:** Setup Doctor v1'de yalnızca **bulgu görüntüleme** düzeyindedir (B5-12 opsiyonel). "Düzelt/Onar" tıklaması PARK-01/02/03 güvenlik alanına girer.

**Kodda zaten var olup ertelenen yetenekler (kanıtlı, silinmedi):** Mutasyonsuz plan, hash kontrollü atomik yazıcı, journal/undo, toggle, taşıma, bağlama, ayırma, toplu uygulama, profiller, drift (eski `OPS-001..008`, `ADV-001..002` — kod ve birim testleri tamam, §12). Bunlar PARK-02/03/04'ün başlangıç birikimidir; B3-05 kararı (feature arkasında saklama veya silme) PARK'ın yeniden açılma maliyetini belirler.

---

# 12. Eski roadmap görevlerinin eşleştirme tablosu

> Eski `ROADMAP.md`, `docs/ROADMAP.md` ve `IMPLEMENTATION_ROADMAP.md` dosyaları `fec33c3` öncesinde silinmiştir; içerikleri git geçmişinde `fec33c3^` altındadır. Aşağıdaki durumlar o dosyalardaki işaretlerin kayıpsız kopyasıdır; doğrulanmış durum 2026-11-06 koduyla yeniden değerlendirilmiştir.

## 12.1. Eski görev kimlikleri → yeni karşılık

| Eski ID / madde | Eski durum | Yeni karşılık | v1.0 kapsamı |
|---|---|---|---|
| `REL-001` Eski sürümden gerçek güncelleme ve geri alma CI matrisi | `[ ]` | §5 REL-001 → B3-09, RC-02 | **Evet** (yalnız Hub'ın kendi güncellemesi) |
| `REL-002` Temiz VM kurulumu + Hub kendini güncelleme | `[ ]` | §5 REL-002 → B3-04, B3-09, RC-02, V1-01 | **Evet** |
| `REL-003` Güncelleme bildirimi, çalışan görev uyarısı, aynı sürüm farklı digest | `[x]` Hub tarafı / `[ ]` ürünlerin kendi bildirimi | §5 REL-003: tamamlanan ürün-yönetimi kodu B3-04'te kalkar; "farklı digest reddi" Hub indeksi için korunur (V1-01); ürün bildirimi kardeş depo | Hub parçası **evet**, ürün parçası **hayır** |
| `CORE-001` Sahiplik ve yazma sınırı ADR | `[x]` | B3-02, B3-05 (ADR revize: yazma kapısı kapalı) | **Evet** (değişmiş hâli) |
| `CORE-002` Asset/Client/Source/Binding/Scope/Capability/ObservedStatus modeli | `[x]` | B3-06, B5-05 | **Evet** |
| `CORE-003` Sahte kullanıcı kökü, config/symlink fixture, native koruma | `[x]` | B5-01, B5-13, RC-02 | **Evet** |
| `CORE-004` `integrations` modülü, tipli/allowlist'li IPC | `[x]` | B3-05 (IPC daraltma: `plan` çıkar), B5-01 | **Evet** (daraltılmış) |
| `SEC-001` Sahiplik, path sınırları, sır redaksiyonu, tehdit modeli | `[x]` | B5-11, RC-03 | **Evet** |
| `DISC-001` Kanıta dayalı istemci keşfi | `[x]` | B5-01, B5-02 | **Evet** |
| `DISC-002` Claude Code MCP/Skill/Plugin okuma | `[x]` | B5-01, B5-03, B5-04 | **Evet** |
| `DISC-003` Codex MCP/Skill okuma | `[x]` | B5-01, B5-03, B5-04 | **Evet** |
| `DISC-004` Cursor, Gemini CLI, OpenCode adapter'ları | `[x]` | B5-01, B5-03 | **Evet** |
| `DISC-005` Skills/symlink envanteri, managed/unmanaged | `[x]` | B5-04 | **Evet** |
| `DISC-006` Plugin ve alt bileşen ilişkileri | `[x]` | B5-04 | **Evet** |
| `DISC-007` Araç bazlı etkin kapsam ve çakışma çözümleyicisi | `[x]` | B5-06 | **Evet** |
| `CAT-001` Uygulamayla gelen yerel üçüncü taraf katalog (62 kayıt) | `[x]` | B3-08, B4-02, B4-03 | **Evet** |
| `CAT-002` Keşfet'te arama, kategori/istemci filtresi, GitHub bağlantısı | `[x]` | B4-04, B4-05, B4-11 | **Evet** |
| `CAT-003` İsteğe bağlı imzalı katalog tazeleme + offline | `[x]` kod / uzak kaynak yayımlı değil (F-08) | B4-12 | **Evet** |
| `SEC-002` Alan ayrımlı Ed25519 imza/sequence | `[x]` | B4-12, RC-03 | **Evet** |
| `SEC-003` Kaynak/komut/izin risk önizlemesi | `[x]` | B4-11 (yalnız kaynak/lisans/metadata; kurulum komutu uyarıları PARK-01) | **Kısmen** (yalnız metadata/link) |
| `UI-001` Kütüphanem'de MCP/Skills/Plugin × AI × kapsam matrisi | `[x]` | B5-07, B5-08 | **Evet** |
| `UI-002` Seçilen projeler için salt okunur global/proje görünümü | `[x]` | B5-09 | **Evet** |
| `OPS-001` Mutasyonsuz plan, maskeleme, önizleme | `[x]` kod | PARK-02 · B3-05 (üretim IPC'den çıkar) | **Hayır** |
| `OPS-002` Hash kontrollü atomik yazıcı + readback | `[x]` kod | PARK-02 | **Hayır** |
| `OPS-003` Etkinleştir/devre dışı bırak | `[x]` kod | PARK-03 | **Hayır** |
| `OPS-004` Global↔proje taşıma/kopyalama | `[x]` kod | PARK-03 | **Hayır** |
| `OPS-005` Transaction journal, undo, çökme kurtarma | `[x]` kod | PARK-02 · B3-05 (açılış `recover()` kaldırılır) | **Hayır** |
| `OPS-006` Kullanıcı onaylı mevcut entegrasyonu içe alma/bağlama | `[x]` kod | PARK-01, PARK-04 | **Hayır** |
| `OPS-007` Sahibi koruyan kaldırma/ayırma ve güncelleme | `[x]` kod | PARK-01, PARK-02 | **Hayır** |
| `OPS-008` Çoklu araca toplu uygulama, kısmi hata kurtarma | `[x]` kod | PARK-04 | **Hayır** |
| `ADV-001` Proje profilleri, idempotent şablonlar | `[x]` kod | PARK-04 | **Hayır** |
| `ADV-002` Drift, gölgelenme, kırık symlink, kopya çakışması analizi | `[x]` | B5-06, B5-12 (salt okunur kısım) | **Evet** (yalnız okuma/bulgu) |
| `PORT-001` macOS/Linux kontrat yetenek matrisi (CI) | `[x]` | PARK-06; CI kontrat koşuları B3-04'te yeniden değerlendirilir (yalnız kalan yeteneklere indirgenir); `docs/PLATFORMS.md` yeniden yazılır (B3-10) | **Hayır** (Windows x64) |
| `QA-001` Her PR'da `check`/`cargo test`/native E2E/gerçek ayar koruma/belge | `[x]` sürekli kapı | B3-09, B4-16, B5-13, RC-02..07 | **Her sürümde zorunlu** |
| "Beta.1'den sonraki ilk sürüm yalnız doğrulama/hata düzeltme" | beta.2 yayımlandı | §5 (kapsam kapalı; açık işler devralındı) | Beta.2'de kaldı |
| Hub'ın kendini güncelleme **kodu** (imzalı kurucu, boyut/SHA-256, `/SILENT /NOCANCEL /RELAUNCH=1`) | `[x]` | B3-04 (koru/ayrıştır) | **Evet** |
| Gerçek beta.1→yeni sürüm güncelleme CI işi | `[ ]` | REL-001 | **Evet** |
| Güncellemede sağlık dalıyla geri alma | `[ ]` | REL-005 → B3-04 notu | Hub'ın kendisi için değerlendirilecek |
| Temiz makinede ilk kurulum | `[ ]` | REL-002 | **Evet** |
| Güncelleme yolu: her sürümden en yeniye doğrudan (sözleşme §4) | `[ ]` kısmen karar, CI işi yok | RC-02 (§3.5 kural korunur) | **Evet** |
| Aynı sürüm numarasıyla düzeltilmiş yayın reddi | `[x]` | V1-01 (indeks için korunur) | **Evet** |
| Brain'de Gemini CLI/Cursor token kullanımı | `[ ]` | Kardeş depo `respected-brain` | Hub dışı |
| Orchestrator'da Gemini/Cursor token kullanımı | `[ ]` | Kardeş depo `respected-orchestrator` | Hub dışı |
| Hub'sız kullanıcıya ürün sürüm bildirimi | `[ ]` | Ürün depoları (PARK-07) | Hub dışı |
| Keşfet: GitHub repoları/eklentiler/MCP listeleme | `[x]` | B3-06..08, B4-01..16 | **Evet** (gerçek mağazaya dönüşür) |
| Bunları Hub'dan kurmak (örn. ponytail) ve kaldırmak | `[ ]` | **PARK-01** | **Hayır** (aktif checkbox olarak kalmaz) |
| Kaldırma/ayırma motoru (`DetachMcp`) | `[x]` kod | PARK-01/02 | **Hayır** |
| "Hangi AI aracında kurulu" görünümü | `[x]` | B5-01..11 | **Evet** (kanıtlı statülerle) |
| Dış eklentiler için imzalı katalog kanalı ve doğrulama modeli | `[x]` | B4-11..13 | **Evet** |
| Authenticode imzası (SmartScreen) | `[ ]` | REL-004 → RC-07, V1-01 | **Evet** (imza yoksa şeffaf risk notu) |
| macOS/Linux masaüstü GUI ve paketleme doğrulaması | `[ ]` | PORT-002 → PARK-06 | **Hayır** |

## 12.2. Eski ana akış kuralları → yeni durum

- **MVP sınırı** ("M2 bitene kadar üçüncü taraf kurma/yazma yok"): v1.0'da **kalıcı** hâle gelir; M3/M4/M5 yazma motoru ürün özelliği değildir (PARK).
- **M3 güvenlik kapısı** ("`AGENTS.md`, `ARCHITECTURE.md`, `FOOTPRINT.md`, `PRIVACY.md` ve ürün sözleşmesi güncellenmeden yazma komutları açılmaz"): v1.0'da yazma açılmayacağı için kapı **kapalı kalır**; belgeler B3-02/B3-10'da bu kararı yansıtır.
- **"Yapılan madde silinmez; kanıtıyla işaretlenir"** kuralı korunur: tamamlanan maddeler `[x]` ve kanıtla bu belgede kalır.

---

# 13. Kalite göstergeleri ve tek bakışta kabul tablosu

| Ölçüt | Hedef | Kanıt/ölçüm yöntemi | Engelleyici mi? |
|---|---|---|---|
| Üçüncü taraf AI konfigürasyonuna yazma | **0** | Önce/sonra byte hash + IPC deny test | Evet |
| Üçüncü taraf kod/installer/komut çalıştırma | **0** | IPC allowlist + native smoke + kötü niyetli fixture | Evet |
| Kritik veri kaybı / sır sızıntısı | **0** | Security test matrisi ve redaksiyon kontrolü | Evet |
| Özel Brain/Orch Hub yönetim giriş noktası | **0** | UI route + backend command araması | Evet |
| Hub'ın kendi güncellemesinin çalışması | **Doğrulanmış** | Beta→RC gerçek Windows update testi | Evet |
| Offline mağaza | **Çalışır** | Network disabled Store/Search/Detail/Saved | Evet |
| Metadata bütünlüğü | **İmza + digest + downgrade kontrolü** | Kötü katalog testleri | Evet |
| Resmî AI istemcisi read-only desteği | **5 istemci** | Her biri için en az 8 scope/config fixture | Evet; desteklenmeyen alt özellik açıkça işaretlenir |
| Katalog içeriği | **Beta.4 ≥100; v1.0 hedef 150–250** | Editoryal kayıt ve kaynak QA | Hedef; kalite uğruna hacim azaltılabilir |
| Arama ilgililiği | **30 sorgunun ≥27'sinde ilk 5** | Sabit arama altın kümesi | Beta.4 P0 hedefi |
| Arama gecikmesi | **p95 ≤200 ms hedef** | Cihaz/5k fixture ve ölçüm raporu | RC'de performans kararı |
| Pilot kullanıcı görev tamamlama | **≥%90 öneri hedefi** | 8–12 katılımcı; görev ve yöntem raporu | RC'de UX değerlendirme kapısı; küçük örneklem genellenmez |
| Kritik erişilebilirlik engeli | **0** | Klavye, focus, kontrast ve AA-prensip kontrolü | Evet |
| Native Windows temiz kurulum/yükseltme | **Başarılı** | VM ve gerçek kurucu E2E | Evet |
| Popülerlik/güvenlik/uyumluluk kanıtsız iddiası | **0** | İçerik doğrulaması | Evet |

**Kabul tablosu "şimdiki sonuçlar" değildir.** Ölçüm başarısızsa görev tamamlandı işaretlenmez; hedef revize edilecekse gerekçe ve kullanıcı etkisi bu belgeye kaydedilir.

**Bugünkü kalite tabanı (2026-11-06):** `npm run check` 134/134 (bu oturumda koşuldu). `cargo test` 151 birim + 35 entegrasyon + 1 üretim güven testi (QUALITY.md; yeniden koşulmadı). CI son bilinen yeşil: run `42026915332` (commit `14b1609`), native E2E 30 PASS. Bu sayılar B3-04/B3-09'da ürün testleri çıkarıldığında düşecektir; düşüş bilinçli ve belgelenmiş olmalıdır.

---

# 14. Araştırma, rakip kanıtı ve deneyler

## 14.1. Rakip/kaynak matrisi: "özelliği al" değil "kanıtlı prensibi uyarla"

| Ürün/standart | İncelenecek yetenek veya güçlü desen | Hub'a aktarım | v1'de KOPYALANMAYACAK bölüm |
|---|---|---|---|
| **Steam** | Vitrin, etiketler, koleksiyonlar, görsel ürün sayfaları | B4-01/05/06/07/09/14 | Steam markası/görselinin kopyası, oyun satın alma/topluluk altyapısı |
| **GitHub** | Repository/yayıncı/README/lisans/kaynak şeffaflığı | B4-02/03/07/11 | Repo host etme, Git operations, OAuth ile her kullanıcı verisini okuma |
| **skills.sh** | Skill keşfi ve etiketli/dizinli bulunabilirlik | B4-03/04/05 | Otomatik skill kuran CLI davranışı |
| **Glama** | MCP kategori/filtre ve metadata derinliği | B4-03/05/08 | Yaşayan tüm kataloğu anlık scrape etme veya popülerliği uydurma |
| **Official MCP Registry** | Standart kaynak/metadata alanları | B4-02/03/11 | Registry'de olmak = güvenli kod varsayımı |
| **SkillDock** | Çoklu AI envanter ve Skill/MCP/Plugin ilişkileri | B5-01/04/05/07 | Kurulum, güncelleme, symlink senkronizasyon |
| **SkillBuddy** | Project/global effective chain, shadowing ve drift | B5-06, B5-12 (read-only) | Config değiştirme, drift "fix" |
| **CC Switch** | Çoklu AI client görünürlüğü | B5-01/07 | Model switching, provider/proxy, token/kota |
| **xjeway MCP Manager** | Scope ve güvenli değişiklik planlama fikirleri | v1'de **yalnız nasıl yanlış iddia yapmayacağımız** | `apply/undo` ve MCP kur/kaldır |
| **OWASP MCP güvenlik rehberi** | Secret exposure, supply chain, command injection, shadow MCP | B4-11/12, B5-11, RC-03 | "OWASP onaylı" gibi yanlış iddialar |
| **WCAG 2.2** | Erişilebilir gezinme, focus, kontrast ve test edilebilir kriterler | B4-14, RC-04 | Sadece renk değiştirip "erişilebilir" deme |

## 14.2. Doğrudan incelenebilecek kaynaklar

**Resmî ürün/standart kaynakları:**
- Steam etiketleri ve keşif: https://partner.steamgames.com/doc/store/tags
- GitHub REST API rate limits: https://docs.github.com/en/rest/using-the-rest-api/rate-limits-for-the-rest-api
- MCP Registry: https://registry.modelcontextprotocol.io/
- MCP standardı: https://modelcontextprotocol.io/
- Agent Skills formatı: https://agentskills.io/specification
- WCAG 2.2: https://www.w3.org/TR/WCAG22/
- OWASP MCP Top 10 (yaşayan beta rehberi): https://owasp.org/projects/mcp-top-10

**Rakipler / kataloglar:**
- SkillDock: https://github.com/wanghuan9/skilldock
- SkillBuddy: https://github.com/konnga/skill-buddy
- CC Switch: https://github.com/farion1231/cc-switch
- xjeway MCP Manager: https://github.com/xjeway/mcp-manager
- Glama MCP dizini: https://glama.ai/mcp/servers
- skills.sh: https://skills.sh
- Steam Store: https://store.steampowered.com/

**UX araştırma rehberleri:**
- Nielsen Norman Group (ecommerce search / faceted search): https://www.nngroup.com/reports/ecommerce-ux-search-including-faceted-search/
- W3C erişilebilirlik temelleri: https://www.w3.org/WAI/standards-guidelines/wcag/

> Bu bağlantılar planlama anında doğrulanmadan aktarılmıştır (ağ erişimi/erişilebilirlik ve içerikleri ilgili görevde incelendiğinde teyit edilir; rakip özellikleri tahminle yazılmaz).

## 14.3. Bilimsel araştırma yükümlülüğü

Beta.4 ve Beta.5'te her büyük teknik karar için uygun *hakemli makale/teknik rapor*, standart ve rakip implementasyonundan **en az bir** dayanak aranır. Kaynak bulunmadan "makaleye göre daha iyi" yazılmaz. Her kaynak için şu kayıt tutulur: `yazar/kurum, başlık, tarih, bağlantı, çalışma yöntemi, sınırlamalar, Hub'daki karar, testle nasıl doğrulanacağı`. Araştırma derlemesi kullanıcıya gereksiz uzun rapor olarak dayatılmaz; sonuçlar doğrudan ilgili görevin kabul testine dönüştürülür. Kalıcı özet bu belgede (bu bölümde) tutulur.

## 14.4. Önerilen araştırma deneyleri

| Deney ID | Hipotez / soru | Yöntem | Başarı kanıtı | Bağlı görev |
|---|---|---|---|---|
| RES-01 | Kullanıcı doğal dille arayınca ürünü daha hızlı buluyor mu? | 30 altın sorgu + 8–12 pilot katılımcı | İlk 5 sonuç ilgililik, görev başarısı/süre | B4-04, B4-16 |
| RES-02 | Vitrin/kategori yapısı ilk kez gelen kullanıcıya yol gösteriyor mu? | Görev bazlı navigasyon, tıklama sayısı | 2–3 etkileşimle doğru sayfaya ulaşma | B4-06, B4-16 |
| RES-03 | Detay sayfası kullanıcının "neye yarar" sorusunu karşılıyor mu? | Örnek kullanım soruları, içerik anlaşılırlığı testi | Katılımcı kendi sözleriyle 2 kullanım durumunu açıklayabiliyor | B4-07, B4-16 |
| RES-04 | Global/proje ve configured/used farkı anlaşılır mı? | 5 karmaşık scope senaryosu ile kullanıcı testi | Kritik yanlış yorum oranı raporlanır; metin düzeltmeleri yapılır | B5-06, B5-07, RC-04 |
| RES-05 | Offline katalog kullanıcı beklentisini karşılıyor mu? | Ağ tamamen kapalı Windows cihazında gezinme | Store/Search/Detail/Favorites/Library çalışıyor | B4-12, RC-02 |
| RES-06 | Kütüphane dosyaları değiştirmiyor mu? | Fixture ve gerçek Windows profilinde hash/snapshot | Okunan tüm AI config'lerinde 0 byte farkı | B5-11, B5-13 |
| RES-07 | Metadata kaynağı ve uyumluluk iddiaları doğru mu? | Katalog örneklemi ve resmî doküman kıyaslaması | Yanlış "official/verified/compatible" etiketinin azaltılması; P0 için 0 kritik hata | B4-03, RC-06 |

**Araştırmadan sonuç çıkarma kuralı:** Rakipler veya makaleler **başlangıç hipotezini** destekler; Respected Hub'ın daha iyi olduğunu ancak kendi ölçümleri gösterebilir. "Kanıt" diye uydurma kullanıcı yorumu, yıldız sayısı veya sonuç oranı eklenmez.

---

# 15. Riskler ve risk azaltma

| Risk | Kullanıcıya etkisi | Önlem | İlgili görev |
|---|---|---|---|
| Brain/Orch yönetimini kaldırırken Hub self-update bozulur | Yayın/kurulum zinciri kırılır | Self-update'i bağımsızla, gerçek update E2E | B3-04, B3-09, RC-02 |
| Yayımlı `catalog` indeksi değişince beta.2 Hub'ları güncelleme bulamaz | Mevcut kullanıcılar beta.3'e geçemez | İndeks şeması/`products` alanı kararını yayından önce doğrula (F-07) | B3-04, REL-001 |
| Eski route veya session state çakışır | Açılışta boş/crash | Güvenli route fallback ve migration | B3-03/07/09 |
| Eski metadata bir anda geçersiz olur | Mağaza boş veya yanlış | Schema migration, son iyi cache, imza/doğrulama | B3-08, B4-02/12 |
| v2 uzak katalog eski Hub'larca bütünüyle reddedilir (`deny_unknown_fields`) | Eski istemcilerde katalog yenilemesi çöker, son iyi önbellek kalır | Ayrı kanal/URL veya bilinçli kabul (F-12) | B4-02, B4-12 |
| Çok sayıda uydurma/eskimiş katalog bilgisi | Güven kaybı | Editoryal inceleme, tarih, provenance | B4-03/11, RC-06 |
| Yayıncı resmiymiş gibi gösterilir | Zararlı araca güvenilmesi | Doğrulama kanıtı, unknown state | B4-11, RC-03 |
| Yanlış "kurulu/etkin/kullanılıyor" iddiası | Kullanıcı ortamını yanlış anlar | Resolver fixture, kanıta dayalı etiketler | B5-05/06/10 |
| Config veya logdan API anahtarı ifşa olur | Gizlilik/güvenlik ihlali | Redaksiyon, ham veri dönmeme, adversarial tests | B5-11, RC-03 |
| Symlink/yol taraması UI'ı asar | Donma, uygunsuz erişim | Bounded scan, explicit project selection, iptal | B5-02/11 |
| Tauri WebView keyfî URL/komut açar | Yerel yürütme/güvenlik riski | Dar IPC, allowlist, CSP | B3-05, B4-11, RC-03 |
| Harici bağlantı WebView2'de açılmaz ya da uygulama içinde gezinir | "GitHub'da aç" çalışmaz veya CSP/izin ihlali | Native testle doğrula, dar opener izni (F-13) | B4-11 |
| Popülerlik sıralaması sahte/oynanabilir olur | Yanlış keşif | Kanıtlı ve tarihli kaynak, editoryal ayrım | B4-05, B4-09 |
| Çok kapsamlı v1'e yetişme baskısı | Kırılgan ürün | P0 kapıları ve kapsam dışı park listesi | RC-01 |
| Offline medya dış ağ çağrısı yapar | Gizlilik/offline vaadi bozulur | Paketli izinli medya veya placeholder, ağ testi | B4-13 |
| Çapraz depo sözleşmesi (`PRODUCT_CONTRACT.md`) üç depoda ayrışır | Uyum testi kırılır | Kardeş depo değişiklik notu, birlikte planlı güncelleme (F-17) | B3-02, B3-10 |
| İmzasız kurucu SmartScreen uyarısı | Güven ve kurulum terki | Şeffaf not; sertifika edinimi | REL-004, RC-07, V1-01 |
| Yazma kodunu silmek PARK'ı pahalılaştırır; bırakmak üretim yüzeyini genişletir | Güvenlik / gelecek maliyet | Feature arkasında saklama + üretimde sembol yokluğu testi | B3-05 |

---

# 16. Açık kararlar ve çelişki çözümleri

**Durum (2026-11-06):** K-01, K-03, K-04 ürün sahibince kararlaştırıldı (aşağıda "KARAR"). K-02 ve K-05..K-09 varsayılan çözümle onaylandı; sorun çıkarsa düzeltilir.

Aktarım sırasında bulunan çelişkiler/belirsizlikler. Varsayılan çözüm yazılıdır; ürün sahibi aksini söylemedikçe geçerlidir.

| ID | Konu | Çelişki / belirsizlik | Varsayılan çözüm | İlgili görev |
|---|---|---|---|---|
| K-01 | "Uyumluluk katmanı yok" kuralı ↔ göç ihtiyacı | `AGENTS.md`: "Geçiş dönemi, eski sürüm desteği ya da uyumluluk katmanı ekleme." Plan ise eski rota, eski durum, eski önbellek, katalog şeması göçü ister. | **KARAR (ürün sahibi, 2026-11-06): `AGENTS.md`'deki bu kural dikkate alınmaz.** Gerektiğinde eski durum için geçiş/uyumluluk kodu yazılabilir (eski rota, `connections.json`, eski katalog önbelleği, katalog v1→v2 göçü). Yine de en küçük çözüm tercih edilir; çift sürüm desteği ancak gerçek bir ihtiyaç varsa eklenir. `AGENTS.md` bu karara göre B3-10'da güncellenir. | B3-03, B3-09, B4-02, B4-12 |
| K-02 | `plan` IPC eylemi | Kaynak: yazma eylemleri kapatılacak, planlama UI'ı yok. Bugün `plan` salt okunur ama canlı (F-02). | **KARAR (ürün sahibi): silinir.** `plan` eylemi ve `PlanRequest` kodu üretim IPC'sinden çıkarılır, feature arkasında saklanmaz. | B3-05 |
| K-03 | Açılışta otomatik ağ isteği | `lib.rs` her açılışta arka planda Hub indeksini yeniler (`lifecycle::catalog::refresh`); plan "ağ yalnız kullanıcı başlatınca" der (B4-12). | **KARAR (ürün sahibi, 2026-11-06): Hub self-update kalır** (açılıştaki denetim dahil). Üçüncü taraf katalog yenilemesi yalnız kullanıcı başlatımıyla olur. Bu fark `PRIVACY.md`'ye yazılır. | B3-04, B4-12 |
| K-04 | Brain/Orchestrator'ın katalogdaki yeri | `app` türü yok (F-10); ayrıca "normal katalog öğesi" mi, hiç mi? | **KARAR (ürün sahibi): onaylandı.** Katalogda `app` türünde, diğer öğelerle aynı kurallarla; özel rozet/eylem yok; "GitHub'da aç" ve belgeler. | B3-06 |
| K-05 | Üçüncü taraf uzak katalog kaynağı | `REMOTE_URL` yayımlı değil (F-08). | B4-12'de ilk imzalı yayın üretilir; o zamana kadar katalog yalnız derlenmiş snapshot'tır. Bu açıkça belgelenir. | B4-12 |
| K-06 | Sürüm numaraları | Plan "beta.3/4/5" ürün kilometre taşı der; depo `0.1.0-beta.N`. | Etiketler `v0.1.0-beta.3/4/5`, `v1.0.0-rc.N`, `v1.0.0`; kilometre taşı numarası ile etiket uyuşmazsa bu belge güncellenir. | B3-10 |
| K-07 | macOS/Linux CI koşuları | Bugün CI keşif/envanter/plan-apply kontratlarını macOS/Linux'ta koşturur; v1.0 yalnız Windows x64 diyor. | CI koşuları kalabilir (yalnız kalan yetenekler: keşif/envanter/çözümleme); belgelerde "destekleniyor" etiketi verilmez (PARK-06). plan/apply testleri feature ile birlikte taşınır. | B3-04, B3-10 |
| K-08 | Katalog başlığı Türkçe karakter kullanımı | Mevcut JSON'da ASCII'leştirilmiş metinler (F-11). | Düzeltme editoryal sürecin (B4-03) parçasıdır; B3-08'de yalnız eşleme raporu çıkar. | B3-08, B4-03 |
| K-09 | `plugin` olarak etiketli istemci uygulamaları | 9 istemci `plugin` tipinde (F-10). | B3-08 göç haritasında `app`'e yeniden sınıflanır ("yeniden adlandırıldı/sınıflandı" raporu). | B3-06, B3-08 |

---

# 17. Sürüm kapsamı özeti ve değişiklik kaydı

## 17.1. Tek cümlelik sürüm kapsamı

- **Beta.3:** Brain ve Orchestrator'ın özel yönetimi **tamamen kaldırılmış**, tarafsız Hub altyapısı.
- **Beta.4:** Bir vibe coder'ın keşfetmek isteyeceği **güçlü, premium, araştırılabilir bir Store**.
- **Beta.5:** MCP, Skills, Plugins ve desteklenen kuralların **hangi AI'da ve hangi kapsamda tanımlandığını gösteren salt okunur Library**.
- **v1.0 RC:** Hatalar, içerik doğruluğu, UX, güvenlik, offline, performans ve Windows release testleri.
- **v1.0 Stable:** İlk genel yayın — **Steam hissinde keşif + yerel ekosistem görünürlüğü; config yönetimi/otomatik kurulum değil**.

Bu karardan farklı bir kapsam ancak ürün sahibinin yeni ve açık kararıyla değişir.

## 17.2. Görev sayıları

| Sürüm | Görevler | Adet | P0 | P1 | P2 |
|---|---|---:|---:|---:|---:|
| Beta.2 devralınan | REL-001..005, PORT-002, QA-002, DOC-001, HYG-001 | 9 | — | — | — |
| Beta.3 | B3-01..B3-10 | 10 | 7 | 3 | 0 |
| Beta.4 | B4-01..B4-16 | 16 | 10 | 6 | 0 |
| Beta.5 | B5-01..B5-13 | 13 | 11 | 1 | 1 |
| v1.0 RC | RC-01..RC-07 | 7 | 5 | 2 | 0 |
| v1.0 Stable | V1-01..V1-03 | 3 | 3 | 0 | 0 |
| **Toplam planlı** | | **49** | 36 | 12 | 1 |
| v1.1+ | PARK-01..PARK-10 | 10 | | | |

## 17.3. Değişiklik kaydı

| Tarih | Değişiklik |
|---|---|
| 2026-11-06 | Belge sıfırlandı hâlinden (`Şu an açık madde yok.`) tam sürüm planına genişletildi: Beta.3 → v1.0 Stable görevleri (49), beta.2'den devralınan açık işler (9), PARK listesi (10), eski görev eşleştirmesi, kabul tablosu, araştırma/rakip kaynakları, riskler. Depo doğrulama bulguları F-01..F-20 ve açık kararlar K-01..K-09 eklendi. Hiçbir uygulama kodu değiştirilmedi, sürüm yayımlanmadı. Betikle doğrulandı: 49 kartın tamamı mevcut, her kartta bağımlılık/adım/mimari/test/kabul alanı var, bağımlılıklar var olan kartlara işaret ediyor ve döngü yok. |
