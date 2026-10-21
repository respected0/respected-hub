# Respected Hub — Kaynak Atlası

> Bu belge `tools/repository_map.py` tarafından `docs/repository_inventory.json` içindeki gözden geçirilmiş açıklamalardan üretilir. Doğrudan bu Markdown dosyasını düzenlemeyin.

**Kapsam:** 172 proje dosyası. Git çalıştırmadan dosya sistemi ve kök ignore kurallarıyla keşfedilen Tauri ürün kaynağıdır. Bağımlılık/üretim önbellekleri ayrı kategoriler olarak açıklanır.

## Eksiksiz kaynak ağacı

```text
respected-hub/
├── .gitattributes
├── .github/
│   └── workflows/
│       └── ci.yml
├── .gitignore
├── AGENTS.md
├── README.md
├── docs/
│   ├── REPOSITORY_MAP.md
│   ├── decisions/
│   │   ├── BUNDLE_TRUST.md
│   │   ├── DEV_OVERRIDES_AND_UNINSTALL.md
│   │   ├── HUB_CAMPAIGNS_MOUNT.md
│   │   ├── HUB_CAMPAIGN_TRACKING.md
│   │   ├── HUB_NATIVE_RPP.md
│   │   ├── HUB_PRODUCT_LIFECYCLE.md
│   │   ├── HUB_V1_READINESS.md
│   │   ├── HUB_V1_RELEASE_GATES.md
│   │   ├── HUB_WORKSPACE_AGENTS.md
│   │   ├── HUB_WORKSPACE_EVIDENCE.md
│   │   ├── HUB_WORKSPACE_MOUNT.md
│   │   ├── HUB_WORKSPACE_OVERVIEW.md
│   │   └── REMOTE_CATALOG_AND_LIFECYCLE.md
│   ├── records/
│   │   └── 2026-10-ux-track-b.md
│   └── repository_inventory.json
├── index.html
├── package-lock.json
├── package.json
├── packaging/
│   └── windows/
│       └── hub_setup.iss
├── release/
│   ├── BETA_NOTICE.txt
│   ├── BUILD_FROM_SCRATCH.md
│   ├── README.md
│   ├── docs/
│   │   ├── AUDIT_REPORT_2026-10-17.md
│   │   ├── CHATGPT_BRIEF.md
│   │   ├── INSTALL_LAYOUT.md
│   │   ├── MASTER_HANDOFF.md
│   │   ├── RELEASE_NOTES_BETA.md
│   │   ├── RELEASE_PHASE_PLAN.md
│   │   └── RESPECTED_MASTER_DIRECTIVE.md
│   └── tools/
│       ├── accept_hub.py
│       ├── build_brain_dist.py
│       ├── build_bundle.py
│       ├── build_hub_installer.py
│       ├── build_runtime.py
│       ├── bundle_signing.py
│       ├── ecosystem_root.py
│       ├── publish_release.py
│       ├── real_env_guard.py
│       ├── release_keys.py
│       ├── release_package.py
│       ├── release_vault.py
│       ├── rpp_conformance.py
│       └── test_release_tools.py
├── src/
│   ├── main.tsx
│   ├── products/
│   │   ├── orchestrator/
│   │   │   ├── campaigns.ts
│   │   │   ├── mcpAccess.ts
│   │   │   ├── tasks.ts
│   │   │   └── views/
│   │   │       ├── AgentsPanel.tsx
│   │   │       ├── Campaigns.tsx
│   │   │       ├── EvidencePanel.tsx
│   │   │       ├── OrchestratorMcp.tsx
│   │   │       ├── OverviewPanel.tsx
│   │   │       ├── Tasks.tsx
│   │   │       ├── agents.css
│   │   │       ├── agentsModel.ts
│   │   │       ├── evidence.css
│   │   │       ├── evidenceModel.ts
│   │   │       ├── overview.css
│   │   │       ├── overviewModel.ts
│   │   │       ├── settings.css
│   │   │       └── workspaceModel.ts
│   │   └── secondbrain/
│   │       ├── brainClients.ts
│   │       ├── memoryAccess.ts
│   │       ├── memoryIndex.ts
│   │       ├── memorySemantic.ts
│   │       ├── memoryWorkspace.ts
│   │       └── views/
│   │           ├── BrainInstall.tsx
│   │           ├── BrainIntegrations.tsx
│   │           ├── BrainSetup.tsx
│   │           ├── ClientPicker.tsx
│   │           ├── IntegrationPreview.tsx
│   │           ├── MemoryAccess.tsx
│   │           ├── MemoryIndex.tsx
│   │           └── MemoryWorkspace.tsx
│   ├── shared/
│   │   ├── api.ts
│   │   ├── design.css
│   │   ├── productClient.ts
│   │   ├── productsModel.ts
│   │   ├── rpp.ts
│   │   └── ui.tsx
│   └── shell/
│       ├── App.tsx
│       ├── apps.ts
│       ├── homeModel.ts
│       ├── lifecycle.ts
│       ├── shellModel.ts
│       ├── useEcosystem.ts
│       └── views/
│           ├── Activity.tsx
│           ├── AppWorkspace.tsx
│           ├── Connections.tsx
│           ├── Discover.tsx
│           ├── Home.tsx
│           ├── LifecycleControls.tsx
│           ├── MyApps.tsx
│           ├── Products.tsx
│           └── Settings.tsx
├── src-tauri/
│   ├── Cargo.lock
│   ├── Cargo.toml
│   ├── build.rs
│   ├── capabilities/
│   │   └── default.json
│   ├── icons/
│   │   ├── 128x128.png
│   │   ├── 128x128@2x.png
│   │   ├── 32x32.png
│   │   ├── 64x64.png
│   │   ├── Square107x107Logo.png
│   │   ├── Square142x142Logo.png
│   │   ├── Square150x150Logo.png
│   │   ├── Square284x284Logo.png
│   │   ├── Square30x30Logo.png
│   │   ├── Square310x310Logo.png
│   │   ├── Square44x44Logo.png
│   │   ├── Square71x71Logo.png
│   │   ├── Square89x89Logo.png
│   │   ├── StoreLogo.png
│   │   ├── icon.icns
│   │   ├── icon.ico
│   │   └── icon.png
│   ├── release-keys.txt
│   ├── src/
│   │   ├── catalog.rs
│   │   ├── cleanup.rs
│   │   ├── lib.rs
│   │   ├── lifecycle.rs
│   │   ├── main.rs
│   │   ├── products/
│   │   │   ├── brain.rs
│   │   │   ├── mcp_access.rs
│   │   │   ├── memory_access.rs
│   │   │   ├── memory_index.rs
│   │   │   ├── memory_semantic.rs
│   │   │   └── mod.rs
│   │   ├── registration.rs
│   │   ├── rpp.rs
│   │   └── trust.rs
│   ├── tauri.conf.json
│   └── tests/
│       ├── brain_concurrency_e2e.rs
│       ├── brain_e2e.rs
│       ├── campaign_e2e.rs
│       ├── cleanup_e2e.rs
│       ├── lifecycle_e2e.rs
│       ├── mcp_access_e2e.rs
│       ├── memory_access_e2e.rs
│       ├── memory_index_e2e.rs
│       ├── remote_catalog_e2e.rs
│       ├── rpp_e2e.rs
│       └── trust_catalog_e2e.rs
├── tests/
│   ├── native/
│   │   └── hub_native_e2e.mjs
│   ├── orchestrator/
│   │   ├── agents.test.ts
│   │   ├── campaigns.test.ts
│   │   ├── evidence.test.ts
│   │   ├── mcpAccess.test.ts
│   │   ├── overview.test.ts
│   │   ├── tasks.test.ts
│   │   └── workspace.test.ts
│   ├── secondbrain/
│   │   ├── brainClients.test.ts
│   │   ├── memoryAccess.test.ts
│   │   ├── memoryIndex.test.ts
│   │   ├── memorySemantic.test.ts
│   │   └── memoryWorkspace.test.ts
│   ├── shared/
│   │   ├── productClient.test.ts
│   │   ├── productsModel.test.ts
│   │   └── rpp.test.ts
│   └── shell/
│       ├── appNav.test.ts
│       ├── homeModel.test.ts
│       ├── lifecycleModel.test.ts
│       └── shellModel.test.ts
├── tools/
│   └── repository_map.py
├── tsconfig.json
└── vite.config.ts
```

## Dosya dosya sorumluluk ve ilişkiler

Her kayıt bir dosyayı açıklar; boş `.gitkeep` ve paket `__init__.py` dosyaları da dahildir. Aşağıdaki yollar depo köküne göredir. Tarihsel belgeler canlı davranışın yetkili kaynağı değildir.

### Depo kökü

#### [`.gitattributes`](../.gitattributes)

**Rol:** Satır sonu kuralı.

**Amaç / sorumluluk:** Depodaki metinleri LF olarak normalleştirir; .ps1/.bat/.cmd/.iss CRLF, görseller ve exe ikili kalır.

**İlişkiler ve sınır:** Git tarafından okunur; çalışma zamanı etkisi yoktur.

### .github/workflows

#### [`.github/workflows/ci.yml`](../.github/workflows/ci.yml)

**Rol:** Sürekli entegrasyon.

**Amaç / sorumluluk:** Windows'ta Hub, SecondBrain ve Orchestrator'ı ekosistem klasörü gibi yan yana çeker; ürünleri ve sürüm araçlarını kurar, atlas kapısını, TypeScript kontrolü + birim testlerini (npm run check), sürüm aracı öz testini ve gerçek ürünleri tek kullanımlık köklerde süren Rust birim/uçtan uca testlerini (cargo test) çalıştırır.

**İlişkiler ve sınır:** Kardeş repolar public GitHub depolarından okunur; RESPECTED_ECOSYSTEM_ROOT çalışma alanını gösterir. Yerel eşdeğeri release/tools/accept_hub.py'dir.

### Depo kökü

#### [`.gitignore`](../.gitignore)

**Rol:** Kaynak kapsamı.

**Amaç / sorumluluk:** Bağımlılık, cache, build ve yerel test artifacts kaynak envanteri dışında tutar.

**İlişkiler ve sınır:** Yeni kaynak dosyaları atlas kapısından kaçırılmaz.

#### [`AGENTS.md`](../AGENTS.md)

**Rol:** Çalışma sözleşmesi.

**Amaç / sorumluluk:** Hub kaynağı için ürün sınırlarını tanımlar: GUI'den yönetilen yaşam döngüsü, Safe Integration Layer ve SecondBrain sihirbazı yalnızca disposable root'larda geliştirilir; mevcut AI config'i veya canlı Brain verisi değiştirilmez, genel shell çalıştırılmaz. Atlas/envanter bakım adımlarını ve karar/kanıt dizinlerini de belirtir.

**İlişkiler ve sınır:** Kaynakta çalışan tüm ajan ve geliştiricilere uygulanır; docs/repository_inventory.json ve tools/repository_map.py akışını zorunlu kılar. Kullanıcı istemedikçe source Git/commit veya paralel ajan çalıştırılmaz; native kabul ve release signing ayrı kapılardır.

#### [`README.md`](../README.md)

**Rol:** Ürün geliştirici rehberi.

**Amaç / sorumluluk:** Bağımsız ürün girişini, geliştirme komutlarını ve native/production kabul sınırlarını açıklar.

**İlişkiler ve sınır:** Runtime kaynaklarına ve tarihli extraction kaydına bağlanır.

### docs

#### [`docs/REPOSITORY_MAP.md`](../docs/REPOSITORY_MAP.md)

**Rol:** Üretilmiş atlas.

**Amaç / sorumluluk:** Ürünün bütün kaynak dosyalarının incelenmiş sorumluluk ve ilişkilerini gösterir.

**İlişkiler ve sınır:** tools/repository_map.py JSON kaynağından üretir; elle düzenlenmez.

### docs/decisions

#### [`docs/decisions/BUNDLE_TRUST.md`](../docs/decisions/BUNDLE_TRUST.md)

**Rol:** Paket güveni kararı.

**Amaç / sorumluluk:** İmza kapsamı, üretim ve geliştirme güveninin anahtar ve derleme ile ayrılması, kanıt ve yayın aşamasında kalan işleri (üretim anahtarı, Authenticode) kaydeder.

**İlişkiler ve sınır:** trust.rs, tools/bundle_signing.py ve tools/build_hub_installer.py ile birlikte okunur.

#### [`docs/decisions/DEV_OVERRIDES_AND_UNINSTALL.md`](../docs/decisions/DEV_OVERRIDES_AND_UNINSTALL.md)

**Rol:** Geliştirme geçersiz kılmaları ve Hub kaldırma kararı.

**Amaç / sorumluluk:** Test ortam değişkenleri, kaynak dizin varsayılanları ve güvensiz paket izninin yalnızca debug/dev-trust derlemelerinde bulunduğunu, webview'in memory_* ayarlarını ve çalıştırılabilir yolları değiştiremediğini, Hub kaldırılırken ürünlerin isteğe bağlı temizliğini (--uninstall-products), ikinci başlatmanın mevcut pencereye yönlendirilmesini ve motor kapısı zaman aşımını kaydeder.

**İlişkiler ve sınır:** trust.rs, cleanup.rs, hub_setup.iss, rpp.rs ve lib.rs ile birlikte okunur; kanıtı tests/cleanup_e2e.rs ve trust.rs birim testleridir.

#### [`docs/decisions/HUB_CAMPAIGNS_MOUNT.md`](../docs/decisions/HUB_CAMPAIGNS_MOUNT.md)

**Rol:** Mimari karar kaydı.

**Amaç / sorumluluk:** Mevcut Campaigns görünümünün Orchestrator çalışma alanına 'Kampanyalar' sayfası olarak bağlanmasını kaydeder. Yeni engine eylemi, bağımlılık veya yönlendirme katmanı eklenmez; kampanya klasörü kullanıcının Hub veri kökünden okunur, salt okunur gösterilir ve oluşturma yalnızca açık buton basışıyla başlar.

**İlişkiler ve sınır:** src/shell/apps.ts, src/shell/views/AppWorkspace.tsx ve src/products/orchestrator/views/Campaigns.tsx değişikliğini anlatır; listCampaigns/campaignStatus/startCampaign sarmalayıcılarını yeniden kullanır. HUB_CAMPAIGN_TRACKING.md ve HUB_WORKSPACE_MOUNT.md ile birlikte okunur.

#### [`docs/decisions/HUB_CAMPAIGN_TRACKING.md`](../docs/decisions/HUB_CAMPAIGN_TRACKING.md)

**Rol:** Mimari karar kaydı.

**Amaç / sorumluluk:** Orchestrator kampanyalarının Hub'dan izlenmesi ve açıkça başlatılması kararını kaydeder: engine_request izin listesine yalnızca campaign-start (64 KiB JSON gövde sınırı) ve campaign-status (camp- kimliği süreç başlamadan doğrulanır) eklenir. Salt okunur campaign_list komutu en yeni 30 kampanya adını döndürür, symlink/junction'ı yok sayar ve veri kökü dışına çıkmaz; genel shell açılmaz.

**İlişkiler ve sınır:** src-tauri/src/lib.rs (engine_request, campaign_list, valid_campaign_id) ve src/products/orchestrator/campaigns.ts sarmalayıcılarının tasarım gerekçesidir. tests/orchestrator/campaigns.test.ts ve src-tauri/tests/campaign_e2e.rs bu kararı doğrular.

#### [`docs/decisions/HUB_NATIVE_RPP.md`](../docs/decisions/HUB_NATIVE_RPP.md)

**Rol:** Mimari karar kaydı.

**Amaç / sorumluluk:** Hub'ın Rust arka ucundan Orchestrator ve SecondBrain ürünlerine RPP v1.0 ile erişmesini tanımlar: ürünler ve işlemler (manifest, health, settings.*) izin listesindedir, geçersiz olanlar süreç başlamadan reddedilir. Engine shell olmadan, açık argv/cwd/PYTHONPATH ile, 30 sn zaman aşımı ve çıktı sınırıyla çalıştırılır; rpp_call her zaman eksiksiz zarf döndürür ve panik yapmaz.

**İlişkiler ve sınır:** src-tauri/src/rpp.rs içindeki rpp_call/rpp_call_version tasarımını anlatır. src-tauri/tests/rpp_e2e.rs ve rpp.rs birim testleri ile doğrulanır.

#### [`docs/decisions/HUB_PRODUCT_LIFECYCLE.md`](../docs/decisions/HUB_PRODUCT_LIFECYCLE.md)

**Rol:** Mimari karar kaydı.

**Amaç / sorumluluk:** GUI ile ürün yaşam döngüsünü (kur/güncelle/onar/kaldır), Safe Integration Layer'ı ve Windows dağıtımını tanımlar: katalog paketleri hash'li ve imzasızdır, Hub-yönetimli kopyalar doğrulanmış staging, journal, atomik current.json ve rollback ile çalışır, SecondBrain kendi setup/update/repair/uninstall komutlarıyla kurulur. Yalnızca tek kullanımlık köklerde test edildi (2026-10-14); imzalı sürüm veya native kabul iddia edilmez.

**İlişkiler ve sınır:** tools/build_bundle.py, tools/build_runtime.py, packaging/windows/hub_setup.iss ve src/shell/views/LifecycleControls.tsx ile ilişkili uygulamanın karar gerekçesidir. HUB_V1_RELEASE_GATES.md kapı durumlarını bu karara göre değerlendirir.

#### [`docs/decisions/HUB_V1_READINESS.md`](../docs/decisions/HUB_V1_READINESS.md)

**Rol:** Hazırlık denetimi.

**Amaç / sorumluluk:** Hub'ın V1 hazırlığını alan alan (kurulum, ilk kurulum, UX, ürün entegrasyonu, test, sürüm hazırlığı) REAL/PARTIAL/ABSENT olarak değerlendirir; her iddia bu çalışma ağacındaki koda dayanır ve README'de 'sonraki iş' denen alanlar yükseltilmez. Yalnızca geliştirme kanıtı kaydeder, OS düzeyi kabul iddia etmez.

**İlişkiler ve sınır:** src-tauri/src/rpp.rs, lib.rs, src/shell/apps.ts, src/products/secondbrain/views/BrainSetup.tsx ve tests/ dosyalarına atıf yapar. HUB_V1_RELEASE_GATES.md ile birlikte okunur; doğrulama kapıları ve özellik hazırlığı olarak ayrılırlar.

#### [`docs/decisions/HUB_V1_RELEASE_GATES.md`](../docs/decisions/HUB_V1_RELEASE_GATES.md)

**Rol:** Sürüm kapıları matrisi.

**Amaç / sorumluluk:** Hub V1 sürümünün geçmesi gereken kapıları OPEN, DEV-VERIFIED ve NOT APPLICABLE durumlarıyla listeler; her durum kayıtlı komuta veya çalıştırmaya dayanır, kanıt gösterilemiyorsa OPEN kalır. Hiçbir kapı kapalı sayılmaz; DEV-VERIFIED tek başına sürüm kabulü değildir.

**İlişkiler ve sınır:** package.json komutlarına ve docs/records altındaki tarihli kayıtlara atıf yapar. HUB_V1_READINESS.md özellik hazırlığı denetimini tamamlar.

#### [`docs/decisions/HUB_WORKSPACE_AGENTS.md`](../docs/decisions/HUB_WORKSPACE_AGENTS.md)

**Rol:** Mimari karar kaydı.

**Amaç / sorumluluk:** Orchestrator çalışma alanında ajan rolleri, worker CLI'lar ve model/provider profilleri kavramlarını ayıran salt okunur 'Ajanlar ve modeller' görünümünü tanımlar. Saf agentsModel.ts, engine status.workers ve RPP settings.readback verisini eşler; veri yoksa uydurma yerine dürüst boş durum gösterir.

**İlişkiler ve sınır:** src/products/orchestrator/views/agentsModel.ts ve AgentsPanel.tsx tasarım gerekçesidir. HUB_WORKSPACE_MOUNT.md sayfanın bağlanmasını, tests/ altındaki agents testi davranışı doğrular.

#### [`docs/decisions/HUB_WORKSPACE_EVIDENCE.md`](../docs/decisions/HUB_WORKSPACE_EVIDENCE.md)

**Rol:** Mimari karar kaydı.

**Amaç / sorumluluk:** Orchestrator için salt okunur inceleme, kanıt ve onay görünümünü tanımlar: inspect ve campaign-status verisi gösterilir, ancak apply ve maf-approve izin listesi dışında kalır; Hub hiçbir şeyi onaylıyor veya uyguluyor gibi davranmaz.

**İlişkiler ve sınır:** src/products/orchestrator/views/EvidencePanel.tsx ve ilgili saf model modülünün tasarım gerekçesidir. HUB_CAMPAIGN_TRACKING.md engine_request izin listesine dayanır; HUB_WORKSPACE_MOUNT.md sayfayı bağlar.

#### [`docs/decisions/HUB_WORKSPACE_MOUNT.md`](../docs/decisions/HUB_WORKSPACE_MOUNT.md)

**Rol:** Mimari karar kaydı.

**Amaç / sorumluluk:** Orchestrator uygulama sekmelerinin sırasını (genel bakış, ajanlar, kanıtlar, kampanyalar, görevler, ayarlar) ve her sekmenin gerçek engine verisiyle nasıl beslendiğini kaydeder. SecondBrain sekmeleri değişmez; sayfa/bağlama mantığı saf bir modülde tutulur.

**İlişkiler ve sınır:** src/shell/apps.ts, src/shell/views/AppWorkspace.tsx ve src/views/orchestrator/ panellerini bağlayan kararı anlatır. HUB_WORKSPACE_OVERVIEW.md, HUB_WORKSPACE_AGENTS.md ve HUB_WORKSPACE_EVIDENCE.md ile birlikte okunur.

#### [`docs/decisions/HUB_WORKSPACE_OVERVIEW.md`](../docs/decisions/HUB_WORKSPACE_OVERVIEW.md)

**Rol:** Mimari karar kaydı.

**Amaç / sorumluluk:** Orchestrator genel bakış sayfasını tanımlar: saf overviewModel.ts engine status ve list yanıtını çalışma sayıları, son beş çalıştırma ve worker durumuna çevirir; bozuk kayıtları eler, eksik veriyi demo veri yerine uyarıya dönüştürür. Zaman damgası olmadığından 'en yeni' run_id önekiyle yaklaşıklanır.

**İlişkiler ve sınır:** src/products/orchestrator/views/overviewModel.ts ve OverviewPanel.tsx tasarım gerekçesidir; src/products/orchestrator/tasks.ts terminal() yardımcısını kullanır. tests/ altındaki overview testi ve HUB_WORKSPACE_MOUNT.md ile ilişkilidir.

#### [`docs/decisions/REMOTE_CATALOG_AND_LIFECYCLE.md`](../docs/decisions/REMOTE_CATALOG_AND_LIFECYCLE.md)

**Rol:** Karar kaydı.

**Amaç / sorumluluk:** Başlatıcı modeli (2026-10-21): ürünler GitHub Releases'teki imzalı katalogdan indirilir, güncellemeler sorularak önerilir, beta/kararlı kanallar, Windows Yüklü uygulamalar kayıtları (Kaldır/Değiştir), Hub kaldırılınca ürünlerin bakım kopyasıyla çalışmaya devam etmesi; ilk sürümle donan sözleşmeler (katalog URL'si, imza alanı, şema, sürüm sırası, kayıt defteri anahtarları, Hub argümanları, klasörler).

**İlişkiler ve sınır:** src-tauri/src/catalog.rs, registration.rs, lifecycle.rs ve packaging/windows/hub_setup.iss bu kararı uygular; release/tools/publish_release.py yayın tarafıdır.

### docs/records

#### [`docs/records/2026-10-ux-track-b.md`](../docs/records/2026-10-ux-track-b.md)

**Rol:** Çalışma kaydı.

**Amaç / sorumluluk:** 2026-10-12 tarihli Track B premium arayüz entegrasyonunun durumunu kaydeder: tasarım sistemi, kabuk, Home/My Apps/Discover/Activity yeniden yapımı ve yapılmayan işler (yönetimli kurulum UI'ı, imzalı katalog, rol-model düzenleme). Bilinen Rust test hatasını (RPP 1.0/1.1 uyuşmazlığı) ve envanterdeki eski kayıtları not eder.

**İlişkiler ve sınır:** src/shared/design.css, src/shared/ui.tsx, src/shell/shellModel.ts ve src/shell/views/AppWorkspace.tsx değişikliklerini özetler. docs/repository_inventory.json güncellemesi için bakım notu olarak kullanılır.

### docs

#### [`docs/repository_inventory.json`](../docs/repository_inventory.json)

**Rol:** Anlamsal atlas kaynağı.

**Amaç / sorumluluk:** Dosya rol, amaç, ilişki ve içerik inceleme hash kayıtlarını tutar.

**İlişkiler ve sınır:** repository_map update/accept-reviewed/write/check kapılarının tek anlamsal kaynağıdır.

### Depo kökü

#### [`index.html`](../index.html)

**Rol:** HTML giriş.

**Amaç / sorumluluk:** Türkçe frontend root ve React TypeScript module girişini tanımlar.

**İlişkiler ve sınır:** Vite src/main.tsx yükler; Tauri frontendDist buradan üretilir.

#### [`package-lock.json`](../package-lock.json)

**Rol:** Dependency kilidi.

**Amaç / sorumluluk:** Local Tauri/React/Vite/TypeScript sürüm ve integrity çözümünü korur.

**İlişkiler ve sınır:** npm ci bu kilidi kullanır; global araç kurulumu değildir.

#### [`package.json`](../package.json)

**Rol:** Frontend paket tarifi.

**Amaç / sorumluluk:** Respected Hub kimliği ve local dev/build/test/Tauri bağımlılıklarını tanımlar. Sürüm 0.1.0-beta.1. test betiği tests/**/*.test.ts düzenini, check betiği tsc --noEmit + testleri çalıştırır.

**İlişkiler ve sınır:** npm local node_modules ile çalışır; ürün public CLI eklemez.

### packaging/windows

#### [`packaging/windows/hub_setup.iss`](../packaging/windows/hub_setup.iss)

**Rol:** Windows paket tarifi.

**Amaç / sorumluluk:** Inno Setup betiği: yönetici hakkı gerektirmeyen, kullanıcı başına Windows kurulum paketi (RespectedHub-Windows-Setup) üretir; respected-hub.exe'yi kurar, Başlat menüsü kısayolu ekler. Kaldırma yalnızca kendi koyduğu dosyaları siler; Hub ile kurulan ürünlere, verilerine ve notlara dokunmaz. varsa dev-trust.pub (yalnızca geliştirme kurulumları) exe yanına kopyalanır. Kaldırma sırasında 'ürünler de kaldırılsın mı' sorulur (varsayılan hayır; sessiz kaldırmada /REMOVEPRODUCTS=1); evet ise CurUninstallStepChanged ile respected-hub.exe --uninstall-products çalıştırır. Dosya UTF-8 BOM ile kaydedilir. Kurulum öncesi beta/SmartScreen bildirimi gösterir ve WebView2 varlığını denetler (indirme yapmaz, uyarır). 2026-10-21: kurulumda ürün kataloğu yoktur (Hub indirir); eski {app}\catalog silinir; ürün sorusu yalnızca Yüklü uygulamalar kaydı varken sorulur, Hayır seçilirse exe maintenance klasörüne kopyalanıp --sync-registrations çalıştırılır; /RELAUNCH=1 ile sessiz güncellemeden sonra Hub yeniden açılır.

**İlişkiler ve sınır:** release/tools/build_hub_installer.py staging ve çıktı dizinlerini vererek bu betiği derler. docs/decisions/REMOTE_CATALOG_AND_LIFECYCLE.md kaldırma ve güncelleme davranışını açıklar.

### release

#### [`release/BETA_NOTICE.txt`](../release/BETA_NOTICE.txt)

**Rol:** Yayın belgesi.

**Amaç / sorumluluk:** Kurulumdan önce gösterilen TR/EN beta ve SmartScreen bildirimi: imzasız kurulum, Ed25519 ile imzalı ürünler, SHA-256 karşılaştırması, kullanıcı başına kurulum ve kaldırmada kasanın korunması.

**İlişkiler ve sınır:** hub_setup.iss InfoBeforeFile olarak gösterir; build_hub_installer.py ve release_package.py kopyalar.

#### [`release/BUILD_FROM_SCRATCH.md`](../release/BUILD_FROM_SCRATCH.md)

**Rol:** Yayın belgesi.

**Amaç / sorumluluk:** Yalnızca üç repodan geliştirmeye devam etme rehberi: klasör düzeni ve klonlama, kurulacak araçlar, günlük test komutları (doğru unittest deseni dahil), sürüm yükseltme ve publish_release.py ile yayın, git dışında yedeklenmesi gerekenler (imza kasası).

**İlişkiler ve sınır:** release/tools araçlarını ve release/docs/MASTER_HANDOFF.md'yi gösterir.

#### [`release/README.md`](../release/README.md)

**Rol:** Yayın belgesi.

**Amaç / sorumluluk:** release/ klasörünün dizini: araçlar (publish_release, release_package, imza, derleme), BETA_NOTICE ve belgeler; anahtar malzemesinin repoda tutulmadığını belirtir.

**İlişkiler ve sınır:** release/tools ve release/docs içeriğini tanıtır; ecosystem_root.py klasör bulma kuralını sağlar.

### release/docs

#### [`release/docs/AUDIT_REPORT_2026-10-17.md`](../release/docs/AUDIT_REPORT_2026-10-17.md)

**Rol:** Yayın belgesi.

**Amaç / sorumluluk:** 2026-10-17 bağımsız denetim raporu (P0/P1/P2 bulgular) ve 2026-10-18 çözüm durumu, yayın engelleri ve kalan riskler. Tarihsel kayıttır.

**İlişkiler ve sınır:** MASTER_HANDOFF.md §18 ve karar kayıtları bu rapora atıf yapar.

#### [`release/docs/CHATGPT_BRIEF.md`](../release/docs/CHATGPT_BRIEF.md)

**Rol:** Yayın belgesi.

**Amaç / sorumluluk:** Planlama için kısa proje özeti (2026-10-18 durumu) ve 2026-10-21 başlatıcı modeline geçiş notu; alınmış kararlar, doğrulanmış/doğrulanmamış noktalar, aday işler.

**İlişkiler ve sınır:** Ayrıntı için MASTER_HANDOFF.md, INSTALL_LAYOUT.md ve REMOTE_CATALOG_AND_LIFECYCLE.md'ye yönlendirir.

#### [`release/docs/INSTALL_LAYOUT.md`](../release/docs/INSTALL_LAYOUT.md)

**Rol:** Yayın belgesi.

**Amaç / sorumluluk:** Windows betasında her adımın (Hub kurulumu, SecondBrain ve Orchestrator kurulumu, güncellemeler, kaldırma, Hub'ı ürünleri bırakarak kaldırma) neyi nerede oluşturduğunu, güncellediğini ve sildiğini listeler; bilinen boşluklar.

**İlişkiler ve sınır:** Kod: catalog.rs, registration.rs, lifecycle.rs, cleanup.rs, hub_setup.iss; karar: docs/decisions/REMOTE_CATALOG_AND_LIFECYCLE.md.

#### [`release/docs/MASTER_HANDOFF.md`](../release/docs/MASTER_HANDOFF.md)

**Rol:** Yayın belgesi.

**Amaç / sorumluluk:** Ekosistemin tarihçe ve durum devri: ürün durumları, doğrulama kanıtları, kararlar, yayın aşaması ve §20'de başlatıcı modeli (uzak katalog, Yüklü uygulamalar, sorulan güncellemeler) ile sıradaki işler.

**İlişkiler ve sınır:** Diğer release/docs belgelerine ve karar kayıtlarına yönlendirir.

#### [`release/docs/RELEASE_NOTES_BETA.md`](../release/docs/RELEASE_NOTES_BETA.md)

**Rol:** Yayın belgesi.

**Amaç / sorumluluk:** 0.1.0-beta.1 indirme sayfası metni: dosyalar, SmartScreen uyarısı ve SHA-256 doğrulaması, sorulan güncellemeler, kurulum yerleri (OneDrive uyarısı dahil), Windows Uygulamalar listesi, kaldırma kapsamı, bilinen sınırlar ve hata bildirim yeri.

**İlişkiler ve sınır:** publish_release.py upload sürüm notu olarak kullanır.

#### [`release/docs/RELEASE_PHASE_PLAN.md`](../release/docs/RELEASE_PHASE_PLAN.md)

**Rol:** Yayın belgesi.

**Amaç / sorumluluk:** Sertifikasız Windows beta yayın planı ve durum notları (2026-10-18 duraklama, 2026-10-21 başlatıcı modeline geçiş ve yeni yayın sırası).

**İlişkiler ve sınır:** MASTER_HANDOFF.md §19-20 ile birlikte okunur.

#### [`release/docs/RESPECTED_MASTER_DIRECTIVE.md`](../release/docs/RESPECTED_MASTER_DIRECTIVE.md)

**Rol:** Yayın belgesi.

**Amaç / sorumluluk:** Ekosistemin ürün vizyonu ve yönerge metni (SecondBrain, Orchestrator, Hub hedefleri ve çalışma kuralları). Tarihsel referans.

**İlişkiler ve sınır:** MASTER_HANDOFF.md §0 vizyon kaynağı olarak gösterir.

### release/tools

#### [`release/tools/accept_hub.py`](../release/tools/accept_hub.py)

**Rol:** Yerel kabul kapısı.

**Amaç / sorumluluk:** Hub deposunda yerinde cargo test (tüm birim ve uçtan uca testler) ve npm run check çalıştırır; öncesi/sonrası real_env_guard anlık görüntüsüyle gerçek AI istemci ayarının değişmediğini kanıtlar, değişiklik varsa başarısız olur. Kopya ağaç veya TEMP artığı bırakmaz.

**İlişkiler ve sınır:** real_env_guard.py ve ecosystem_root.py kullanır; CI aynı adımları korumasız çalıştırır.

#### [`release/tools/build_brain_dist.py`](../release/tools/build_brain_dist.py)

**Rol:** Yayın aracı.

**Amaç / sorumluluk:** Hub testleri ve yayın için donmuş SecondBrain dağıtımını üretir veya yeniden kullanır (<ecosystem>/.build-cache/brain-dist/RespectedBrain); SecondBrain'in kendi build_installer.py aracını çağırır, kaynak daha yeniyse yeniden derler.

**İlişkiler ve sınır:** publish_release.py, Rust e2e testleri ve hub_native_e2e.mjs kullanır.

#### [`release/tools/build_bundle.py`](../release/tools/build_bundle.py)

**Rol:** Yayın aracı.

**Amaç / sorumluluk:** Hub kataloğu için doğrulanabilir ürün paketi üretir (orchestrator/secondbrain kaynak paketi veya secondbrain-dist dağıtım paketi): bundle.json dosya listesini boyut ve SHA-256 ile, liste özetini ve Ed25519 imzasını (geliştirme anahtarı veya --key ile yayın anahtarı) taşır.

**İlişkiler ve sınır:** bundle_signing.py ile imzalar; publish_release.py ve Rust e2e testleri çağırır.

#### [`release/tools/build_hub_installer.py`](../release/tools/build_hub_installer.py)

**Rol:** Yayın aracı.

**Amaç / sorumluluk:** Hub'ın üç Windows kurulumunu (Hub, SecondBrain, Orchestrator adlı; aynı Hub) derler: release exe'sini sahneler, --release-key ile üretim (geliştirme güveni yok, exe dize denetimi) veya geliştirme derlemesi yapar ve hub_setup.iss'i Inno ile derler. Kurulumlarda ürün paketi yoktur.

**İlişkiler ve sınır:** packaging/windows/hub_setup.iss; publish_release.py build çağırır.

#### [`release/tools/build_runtime.py`](../release/tools/build_runtime.py)

**Rol:** Yayın aracı.

**Amaç / sorumluluk:** Hub'ın yönettiği Python çalışma ortamı paketini (runtime-<python sürümü>) üretir: çalışan python.org yorumlayıcısını kopyalar, gereksizleri ayıklar, agent-framework-core bağımlılık kapanışını ekler; RESPECTED_SIGNING_KEY verilirse yayın anahtarıyla imzalar.

**İlişkiler ve sınır:** publish_release.py ve Rust e2e testleri çağırır.

#### [`release/tools/bundle_signing.py`](../release/tools/bundle_signing.py)

**Rol:** Yayın aracı.

**Amaç / sorumluluk:** Katalog paketlerinin Ed25519 imzası: geliştirme anahtar çifti oluşturma, paket imzalama/doğrulama ve yayın anahtarı kasasını açma; Hub trust.rs ile aynı ifadeyi imzalar.

**İlişkiler ve sınır:** build_bundle.py, build_runtime.py, publish_release.py ve release_package.py kullanır.

#### [`release/tools/ecosystem_root.py`](../release/tools/ecosystem_root.py)

**Rol:** Yayın aracı.

**Amaç / sorumluluk:** Üç ürün klasörünü yan yana tutan ekosistem klasörünü bulur; RESPECTED_ECOSYSTEM_ROOT ile geçersiz kılınabilir.

**İlişkiler ve sınır:** release/tools altındaki tüm araçlar içe aktarır.

#### [`release/tools/publish_release.py`](../release/tools/publish_release.py)

**Rol:** Yayın aracı.

**Amaç / sorumluluk:** Yayını üretir, imzalar ve GitHub Releases'e yükler: build (kurulumlar + imzalı ürün arşivleri + entries.json), catalog (yayındaki kataloğa birleştirir, sequence büyütür, kanal ve min_hub uygular, respected-catalog-v1 alanıyla imzalar), upload (önce ürünler, en son katalog; --yes olmadan yalnızca gh komutlarını yazar), local-catalog (testler için file:// adresli geliştirme kataloğu).

**İlişkiler ve sınır:** bundle_signing, build_bundle, build_brain_dist, build_runtime ve build_hub_installer'ı kullanır; Hub tarafı src-tauri/src/catalog.rs'dir; test_release_tools.py ve tests/remote_catalog_e2e.rs sınar.

#### [`release/tools/real_env_guard.py`](../release/tools/real_env_guard.py)

**Rol:** Yayın aracı.

**Amaç / sorumluluk:** Test çalıştırmalarının kullanıcının GERÇEK AI istemci yapılandırmasını değiştirmediğini veya test oturumu bırakmadığını snapshot/verify ile kanıtlar; çalıştırma için geçici bir ev klasörü üretir.

**İlişkiler ve sınır:** accept_hub.py ve tests/native/hub_native_e2e.mjs kullanır.

#### [`release/tools/release_keys.py`](../release/tools/release_keys.py)

**Rol:** Yayın aracı.

**Amaç / sorumluluk:** Üretim imza anahtarı töreni: anahtar üretme (repo yolunu ve üzerine yazmayı reddeder), genel anahtarı yazdırma ve bir anahtarın Hub tarafından güvenilir olup olmadığını denetleme.

**İlişkiler ve sınır:** src-tauri/release-keys.txt; build_hub_installer.py --release-key denetimi kullanır.

#### [`release/tools/release_package.py`](../release/tools/release_package.py)

**Rol:** Yayın aracı.

**Amaç / sorumluluk:** İndirme klasörü için kurulumların SHA256SUMS.txt listesini ve Ed25519 imzasını (SHA256SUMS.sig) üretir ve doğrular; BETA_NOTICE.txt'i kopyalar.

**İlişkiler ve sınır:** publish_release sonrası --dir dist/release ile çalıştırılır; test_release_tools.py sınar.

#### [`release/tools/release_vault.py`](../release/tools/release_vault.py)

**Rol:** Yayın aracı.

**Amaç / sorumluluk:** Üretim Ed25519 anahtarı için parola korumalı kasa (scrypt -> AES-256-GCM): oluşturma ve doğrulama; düz anahtar diske yazılmaz.

**İlişkiler ve sınır:** bundle_signing.load_private kasayı açar.

#### [`release/tools/rpp_conformance.py`](../release/tools/rpp_conformance.py)

**Rol:** Yayın aracı.

**Amaç / sorumluluk:** Ürünler arası RPP uyum denetimi: iki gerçek sunucuyu geçici veri köklerinde çalıştırır ve her yanıtı Hub'ın ayrıştırıcısına verir; v1.1 müzakere, manifest, lifecycle ve hata türlerini ayrıca sınar.

**İlişkiler ve sınır:** src-tauri/src/rpp.rs sözleşmesine dayanır.

#### [`release/tools/test_release_tools.py`](../release/tools/test_release_tools.py)

**Rol:** Davranış/regresyon testi.

**Amaç / sorumluluk:** Yayın araçlarının geçici klasörde öz-denetimi: anahtar üretme korumaları, imzalı SHA256SUMS doğrulama ve kurcalama reddi, katalog birleştirme (sequence, korunan girdiler, URL, min_hub), katalog imzası ve arşiv düzeni.

**İlişkiler ve sınır:** release_keys, release_package, bundle_signing ve publish_release'i sınar.

### src-tauri

#### [`src-tauri/Cargo.lock`](../src-tauri/Cargo.lock)

**Rol:** Dependency kilidi.

**Amaç / sorumluluk:** Tauri uygulamasının Rust bağımlılık sürümlerini ve özetlerini sabitler; derlemeyi tekrarlanabilir kılar. Elle düzenlenmez, cargo üretir. Yeni bağımlılıkların (single-instance, ed25519-dalek, base64) kilitli sürümleri.

**İlişkiler ve sınır:** src-tauri/Cargo.toml bağımlılıklarından cargo tarafından üretilir. cargo build/test ve Tauri paketlemesi bu kilidi okur.

#### [`src-tauri/Cargo.toml`](../src-tauri/Cargo.toml)

**Rol:** Native paket tarifi.

**Amaç / sorumluluk:** respected-hub crate'ini ve respected_hub_lib kütüphanesini (staticlib/cdylib/rlib) tanımlar; Tauri 2, dialog eklentisi, serde/serde_json ve sha2 bağımlılıklarını bildirir. tauri-plugin-single-instance, ed25519-dalek ve base64 bağımlılıklarını, yalnızca geliştirme derlemelerinde güveni açan dev-trust özelliğini bildirir. Sürüm 0.1.0-beta.1 (SemVer; beta sürümün parçasıdır).

**İlişkiler ve sınır:** build.rs, src/main.rs ve src/lib.rs ile birlikte derlenir; Python ürün kodu crate'e import edilmez. sha2 hem runtime hem dev-dependency olarak lifecycle/doğrulama ve Rust testleri tarafından kullanılır.

#### [`src-tauri/build.rs`](../src-tauri/build.rs)

**Rol:** Native build girişi.

**Amaç / sorumluluk:** Tauri config/capability code generation build adımını başlatır.

**İlişkiler ve sınır:** Cargo tauri-build kullanır; platform acceptance ayrı kapıdır.

### src-tauri/capabilities

#### [`src-tauri/capabilities/default.json`](../src-tauri/capabilities/default.json)

**Rol:** Native yetki sınırı.

**Amaç / sorumluluk:** Yerel main penceresinde core ve explicit directory dialog iznini tanımlar.

**İlişkiler ve sınır:** Remote origin/generic shell yetkisi açmaz; App proje seçimini kullanır.

### src-tauri/icons

#### [`src-tauri/icons/128x128.png`](../src-tauri/icons/128x128.png)

**Rol:** Uygulama simgesi.

**Amaç / sorumluluk:** Tauri paketlemesinde kullanılan 128x128 piksel uygulama simgesi.

**İlişkiler ve sınır:** tauri.conf.json bundle.icon listesinden okunur.

#### [`src-tauri/icons/128x128@2x.png`](../src-tauri/icons/128x128@2x.png)

**Rol:** Uygulama simgesi.

**Amaç / sorumluluk:** 128x128 simgenin yüksek çözünürlüklü (256x256) varyantı.

**İlişkiler ve sınır:** tauri.conf.json bundle.icon listesinden okunur.

#### [`src-tauri/icons/32x32.png`](../src-tauri/icons/32x32.png)

**Rol:** Uygulama simgesi.

**Amaç / sorumluluk:** Küçük boyutlu (32x32) uygulama simgesi; pencere ve görev çubuğu için.

**İlişkiler ve sınır:** tauri.conf.json bundle.icon listesinden okunur.

#### [`src-tauri/icons/64x64.png`](../src-tauri/icons/64x64.png)

**Rol:** Uygulama simgesi.

**Amaç / sorumluluk:** 64x64 piksel uygulama simgesi varyantı.

**İlişkiler ve sınır:** Tauri simge üretiminin çıktısı; paketleme sırasında kullanılabilir.

#### [`src-tauri/icons/Square107x107Logo.png`](../src-tauri/icons/Square107x107Logo.png)

**Rol:** Uygulama simgesi.

**Amaç / sorumluluk:** Windows mağaza/Appx paketi için 107x107 kare logo.

**İlişkiler ve sınır:** Tauri Windows paketlemesi tarafından kullanılır.

#### [`src-tauri/icons/Square142x142Logo.png`](../src-tauri/icons/Square142x142Logo.png)

**Rol:** Uygulama simgesi.

**Amaç / sorumluluk:** Windows mağaza/Appx paketi için 142x142 kare logo.

**İlişkiler ve sınır:** Tauri Windows paketlemesi tarafından kullanılır.

#### [`src-tauri/icons/Square150x150Logo.png`](../src-tauri/icons/Square150x150Logo.png)

**Rol:** Uygulama simgesi.

**Amaç / sorumluluk:** Windows mağaza/Appx paketi için 150x150 kare logo.

**İlişkiler ve sınır:** Tauri Windows paketlemesi tarafından kullanılır.

#### [`src-tauri/icons/Square284x284Logo.png`](../src-tauri/icons/Square284x284Logo.png)

**Rol:** Uygulama simgesi.

**Amaç / sorumluluk:** Windows mağaza/Appx paketi için 284x284 kare logo.

**İlişkiler ve sınır:** Tauri Windows paketlemesi tarafından kullanılır.

#### [`src-tauri/icons/Square30x30Logo.png`](../src-tauri/icons/Square30x30Logo.png)

**Rol:** Uygulama simgesi.

**Amaç / sorumluluk:** Windows mağaza/Appx paketi için 30x30 kare logo.

**İlişkiler ve sınır:** Tauri Windows paketlemesi tarafından kullanılır.

#### [`src-tauri/icons/Square310x310Logo.png`](../src-tauri/icons/Square310x310Logo.png)

**Rol:** Uygulama simgesi.

**Amaç / sorumluluk:** Windows mağaza/Appx paketi için 310x310 kare logo.

**İlişkiler ve sınır:** Tauri Windows paketlemesi tarafından kullanılır.

#### [`src-tauri/icons/Square44x44Logo.png`](../src-tauri/icons/Square44x44Logo.png)

**Rol:** Uygulama simgesi.

**Amaç / sorumluluk:** Windows mağaza/Appx paketi için 44x44 kare logo.

**İlişkiler ve sınır:** Tauri Windows paketlemesi tarafından kullanılır.

#### [`src-tauri/icons/Square71x71Logo.png`](../src-tauri/icons/Square71x71Logo.png)

**Rol:** Uygulama simgesi.

**Amaç / sorumluluk:** Windows mağaza/Appx paketi için 71x71 kare logo.

**İlişkiler ve sınır:** Tauri Windows paketlemesi tarafından kullanılır.

#### [`src-tauri/icons/Square89x89Logo.png`](../src-tauri/icons/Square89x89Logo.png)

**Rol:** Uygulama simgesi.

**Amaç / sorumluluk:** Windows mağaza/Appx paketi için 89x89 kare logo.

**İlişkiler ve sınır:** Tauri Windows paketlemesi tarafından kullanılır.

#### [`src-tauri/icons/StoreLogo.png`](../src-tauri/icons/StoreLogo.png)

**Rol:** Uygulama simgesi.

**Amaç / sorumluluk:** Windows mağaza girişi için StoreLogo görseli.

**İlişkiler ve sınır:** Tauri Windows paketlemesi tarafından kullanılır.

#### [`src-tauri/icons/icon.icns`](../src-tauri/icons/icon.icns)

**Rol:** Uygulama simgesi.

**Amaç / sorumluluk:** macOS paketi için çok boyutlu simge dosyası.

**İlişkiler ve sınır:** tauri.conf.json bundle.icon listesinden okunur; macOS yerel doğrulaması ayrı kapıdır.

#### [`src-tauri/icons/icon.ico`](../src-tauri/icons/icon.ico)

**Rol:** Uygulama simgesi.

**Amaç / sorumluluk:** Windows çalıştırılabilir dosyası ve yükleyici için çok boyutlu simge.

**İlişkiler ve sınır:** tauri.conf.json bundle.icon listesinden okunur; build betiği exe simgesine gömer.

#### [`src-tauri/icons/icon.png`](../src-tauri/icons/icon.png)

**Rol:** Uygulama simgesi.

**Amaç / sorumluluk:** Diğer simge boyutlarının türetildiği ana kaynak PNG ve Linux simgesi.

**İlişkiler ve sınır:** Tauri simge üretimi (tauri icon) bu dosyadan diğer varyantları çıkarır.

### src-tauri

#### [`src-tauri/release-keys.txt`](../src-tauri/release-keys.txt)

**Rol:** Üretim genel imza anahtarları.

**Amaç / sorumluluk:** Üretim Hub derlemesine gömülen Ed25519 genel anahtarlar (satır başına 'anahtar_kimliği base64'). Özel anahtar asla buraya girmez; tools/release_keys.py yazar. Anahtar satırı yokken üretim derlemesi hiçbir paketi kurmaz. Şu an tek anahtar: dabcb5265227bbdf; katalog imzası da bu anahtarla doğrulanır.

**İlişkiler ve sınır:** src-tauri/src/trust.rs include_str! ile derler; tools/release_keys.py ve build_hub_installer.py --release-key kontrol eder.

### src-tauri/src

#### [`src-tauri/src/catalog.rs`](../src-tauri/src/catalog.rs)

**Rol:** Uzak katalog.

**Amaç / sorumluluk:** GitHub Releases'teki imzalı katalog: indeksi curl.exe ile indirir, respected-catalog-v1 imzasını ve sequence'i doğrular, önbelleğe alır; beta/kararlı kanal seçimi; ürün arşivini yalnızca kur/güncelle/onar sırasında indirir, boyut ve SHA-256 ile denetler, tar.exe ile açar, paket imzası ve dosya özetleriyle yeniden doğrular ve yerel katalog önbelleğine koyar; bozuk önbelleği yeniler, eski sürümleri budar, durum çıktısına güncellemeyi ekler; Hub'ın kendi güncellemesini indirip /SILENT /RELAUNCH=1 ile başlatır.

**İlişkiler ve sınır:** lib.rs (ensure_bundles, hub_status, komutlar), trust.rs (verify_detached), lifecycle.rs (version_key, read_bundle) ile çalışır; tests/remote_catalog_e2e.rs ve birim testleri sınar.

#### [`src-tauri/src/cleanup.rs`](../src-tauri/src/cleanup.rs)

**Rol:** Ürün temizliği (Hub kaldırılırken).

**Amaç / sorumluluk:** Hub kaldırılırken kullanıcı isterse Hub'ın kurduğu ürünleri geri alır: Orchestrator MCP kayıtlarını kapatır, SecondBrain'i kendi kaldırıcısıyla (kasa ve veri kalır) kaldırır, Orchestrator ve çalışma zamanını kaldırır ve yönetilen klasörü siler. Not kasaları ve notlar asla silinmez. Kaldırılan ürünün katalog önbelleğindeki paketleri de silinir; Yüklü uygulamalar kayıtları headless kaldırmadan sonra senkronlanır.

**İlişkiler ve sınır:** lib.rs headless_uninstall_products (respected-hub.exe --uninstall-products) tarafından çağrılır; Inno kaldırıcısı hub_setup.iss (CurUninstallStepChanged) ile çalıştırır. hub_apply, brain::status ve mcp_access::disable_all ile aynı günlüklü yaşam döngüsü yollarını kullanır; tests/cleanup_e2e.rs ile doğrulanır.

#### [`src-tauri/src/lib.rs`](../src-tauri/src/lib.rs)

**Rol:** Native engine köprüsü.

**Amaç / sorumluluk:** Settings kalıcılığını ve doğrulamasını, motor/RPP isteklerini sınırlı JSON işlemleriyle yürütmeyi, ürün durumu, yaşam döngüsü (plan/apply/rollback/purge), orchestrator MCP, bellek erişimi/indeks/anlamsal arama ve etkinlik Tauri komutlarını sunar. Yalnızca açıkça izin verilen komutlar kayıtlıdır; kampanya id'si ve spec boyutu doğrulanır, açılışta yarım kalmış işlemler recover edilir. memory_search, memory_note_read, memory_capture_note ve memory_briefing_run komutları (motorun kendi yanıtını geçirir) eklendi. Tek örnek eklentisi ikinci pencereyi açmaz, mevcut pencereyi öne alır; açılış kurtarması arka plan iş parçacığında çalışır; motor süreç ağacı zaman aşımında sonlandırılır, okuyucu iş parçacıkları süre sınırlıdır, stderr 1 MB ile sınırlıdır; runtime işlemleri sonrası MCP launcher yenilenir; SecondBrain kurulum/güncellemesi sonrası Orchestrator bellek komutu yenilenir. Geliştirme ortam değişkenleri, kaynak dizin ve katalog varsayılanları yalnızca dev_enabled iken geçerlidir; save_settings üretimde reddedilir; product_status development alanı dev_enabled'dır; headless_uninstall_products ürün temizliğini başlatır; tek örnek eklentisi --install niyetini açık pencereye startup-intent olayıyla iletir. 2026-10-21: katalog kökü <managed>/catalog (standard_roots); plan/apply öncesi ensure_bundles gereken paketi indirir (yayınlanmış katalog varsa çalışma ortamı zorunlu); hub_status uzak katalog güncellemesini ekler; başarılı işlemden sonra sweep (yalnız kurulu sürüm kalır) ve Yüklü uygulamalar senkronu; catalog_refresh, catalog_set_channel, hub_update_status, hub_update_install komutları; --install/--uninstall/--manage niyetleri; açılışta kurtarma, budama, kayıt senkronu, bakım kopyası temizliği ve katalog yenileme; headless --sync-registrations.

**İlişkiler ve sınır:** React api.ts ve lifecycle.ts invoke ile çağırır; brain, catalog, registration, lifecycle, rpp, mcp_access, memory_* modüllerini kullanır. Rust birim/e2e testleri ve src-tauri/src/main.rs (run çağrısı) kapsar; native IPC kabulü ayrı kapıdır.

#### [`src-tauri/src/lifecycle.rs`](../src-tauri/src/lifecycle.rs)

**Rol:** Yaşam döngüsü çekirdeği.

**Amaç / sorumluluk:** Hub yönetimli ürünlerin (Orchestrator, SecondBrain, Python çalışma zamanı) katalog doğrulaması, sürümlü kurulum/güncelleme/onarım/kaldırma, journal'lı işlemler, geri alma, yedek temizleme ve kesintiden sonra kurtarmasını yapar. Yol geçişi, bozuk/imzasız paket ve hash uyuşmazlığını reddeder; doğrulama başarısızsa hiçbir şeyi değiştirmez. Her katalog paketi trust::verify ile doğrulanır (güven düzeyi Bundle.trust ve durum çıktısında); journal kimlikleri okuma/yazmada doğrulanır; Orchestrator kuruluyken runtime kaldırılamaz. version_key SemVer sırası ve PEP 440 yazımlarıyla karşılaştırır (her Hub sürümünde sabit kalmalı); güncellemeden sonra yalnızca etkin sürüm diskte kalır (prune_old_versions), bu yüzden güncelleme geri alınamaz.

**İlişkiler ve sınır:** brain.rs, mcp_access.rs ve main.rs tarafından çağrılır; Bundle/Roots/LifecycleError tiplerini sağlar. Birim testleri kendi içinde; src-tauri/tests/lifecycle_e2e.rs gerçek motorlarla uçtan uca test eder.

#### [`src-tauri/src/main.rs`](../src-tauri/src/main.rs)

**Rol:** GUI executable girişi.

**Amaç / sorumluluk:** Release Windows console görünürlüğünü kapatıp Hub runtime'ını başlatır. --uninstall-products bağımsız değişkeni pencere açmadan ürün temizliğini çalıştırır. --sync-registrations pencere açmadan Yüklü uygulamalar kayıtlarını bu exe'ye göre yeniler.

**İlişkiler ve sınır:** lib.rs run çağrılır; ayrı kullanıcı CLI/setup wizard değildir.

### src-tauri/src/products

#### [`src-tauri/src/products/brain.rs`](../src-tauri/src/products/brain.rs)

**Rol:** SecondBrain yaşam döngüsü.

**Amaç / sorumluluk:** SecondBrain ürününün motor-yönetimli kurulum, güncelleme, onarım, kaldırma ve geri almasını yürütür: katalog önbelleğindeki doğrulanmış dağıtımı yerinde kullanır (stage, seri; ikinci kopya tutulmaz), ürünün kendi folderlarını (BrainDirs) kullanır, güvenli entegrasyon planını ürünün kendisinden salt-okunur alır ve istemci id'lerini doğrular. Çökme sonrası ürünün yarım işlemini tamamlatır/geri aldırır; ortam sırlardan arındırılmış çalışır, kullanıcı vault'una dokunmaz. Hub Ed25519 imzasını doğruladığı için SecondBrain çağrılarına --allow-unattested eklenir (SLSA CI gelene kadar). Kurulum/güncelleme motor kapısını en çok 120 sn bekler; uzun bir işlem sürüyorsa açık bir 'meşgul' hatası verir; dizin geçersiz kılmaları yalnızca geliştirme derlemelerinde okunur. (2026-10-21: `src-tauri/src/products/brain.rs` yolundan taşındı.)

**İlişkiler ve sınır:** lifecycle.rs journal/Roots/Bundle yapılarını kullanır; rpp.rs üzerinden ürün motorunu çağırır; main.rs komutları bunu çağırır. src-tauri/tests/brain_e2e.rs ve brain_concurrency_e2e.rs ile test edilir.

#### [`src-tauri/src/products/mcp_access.rs`](../src-tauri/src/products/mcp_access.rs)

**Rol:** Orchestrator MCP erişimi.

**Amaç / sorumluluk:** Orchestrator MCP sunucusunun yapay zekâ istemcilerine kaydını planlar, onay özetine bağlı uygular ve geri alır; Orchestrator güncellense de değişmeyen kararlı bir launcher yazar ve Hub kapalıyken çalıştığını sınar (probe). Yalnızca Respected'e ait girdileri yönetir, yabancı girdilere dokunmaz. Launcher PYTHONSAFEPATH=1 ile ve kendi klasöründe çalışır (çalışma dizininden modül yükleme engellenir). (2026-10-21: `src-tauri/src/products/mcp_access.rs` yolundan taşındı.)

**İlişkiler ve sınır:** lifecycle.rs journal'ını ve rpp.rs motor çağrılarını kullanır; main.rs komutları ve Orchestrator yaşam döngüsü kancaları çağırır. src-tauri/tests/mcp_access_e2e.rs ile test edilir.

#### [`src-tauri/src/products/memory_access.rs`](../src-tauri/src/products/memory_access.rs)

**Rol:** Bellek erişimi yönetimi.

**Amaç / sorumluluk:** Orchestrator'ın SecondBrain belleğine okuma (isteğe bağlı yazma) erişimini motorlar üzerinden açar/kapatır: grant oluşturur, belirteci yalnızca Orchestrator apply'ına verir, hata durumunda yeni grant'ı geri çeker. Yanıtlardaki sır ve belirteç metinden temizlenir; yazma izni ayrı onay gerektirir. refresh_command: SecondBrain yeni sürüm klasörüne taşınınca etkin Orchestrator bellek sağlayıcısının komutunu günceller. (2026-10-21: `src-tauri/src/products/memory_access.rs` yolundan taşındı.)

**İlişkiler ve sınır:** rpp.rs trusted çağrılarıyla motorlara konuşur (CallFn enjekte edilir); main.rs komutları çağırır. Birim testleri sahte motorla; src-tauri/tests/memory_access_e2e.rs gerçek motorlarla test eder.

#### [`src-tauri/src/products/memory_index.rs`](../src-tauri/src/products/memory_index.rs)

**Rol:** Bellek arama indeksi.

**Amaç / sorumluluk:** SecondBrain bellek arama indeksinin durumunu okur ve yeniler; desteklemeyen eski ürünü 'desteklenmiyor' olarak bildirir, kasa yoksa hata değil durum döndürür. Yenileme yalnızca confirm ve force gönderir, hatalar koda göre Türkçe açıklanır. (2026-10-21: `src-tauri/src/products/memory_index.rs` yolundan taşındı.)

**İlişkiler ve sınır:** rpp.rs trusted (uzun zaman aşımlı) çağrıları kullanır; main.rs komutları çağırır. Birim testleri sahte motorla; src-tauri/tests/memory_index_e2e.rs gerçek motorla ve vault'un değişmediğini doğrular.

#### [`src-tauri/src/products/memory_semantic.rs`](../src-tauri/src/products/memory_semantic.rs)

**Rol:** Anlamsal arama yönetimi.

**Amaç / sorumluluk:** SecondBrain anlamsal aramasını açar/kapatır ve durumunu verir: yapılandır, modeli indir, vektör indeksini yenile adımlarını sırayla çalıştırır. İndirme başarısızsa vektör kurulumundan önce durur ve nedenini açıklar; eski SecondBrain'i desteklenmiyor diye bildirir. (2026-10-21: `src-tauri/src/products/memory_semantic.rs` yolundan taşındı.)

**İlişkiler ve sınır:** rpp.rs trusted çağrılarıyla SecondBrain motoruna konuşur; main.rs komutları çağırır. Yalnızca dosya içi birim testleriyle (sahte motor) kapsanır.

#### [`src-tauri/src/products/mod.rs`](../src-tauri/src/products/mod.rs)

**Rol:** Ürün modülleri.

**Amaç / sorumluluk:** Hub'ın ürüne özgü Rust parçalarını toplar: SecondBrain (kendi motoruyla yaşam döngüsü brain, hafıza erişimi/indeks/anlamsal arama) ve Orchestrator (AI istemcileri için MCP erişimi). Hub çekirdeği (lifecycle, catalog, registration, trust, rpp) crate kökünde kalır; lib.rs bu modülleri kısa adlarıyla yeniden dışa verir.

**İlişkiler ve sınır:** lib.rs `pub use products::{brain, mcp_access, memory_access, memory_index, memory_semantic}`; e2e testleri respected_hub_lib::brain gibi kısa yolları kullanır.

### src-tauri/src

#### [`src-tauri/src/registration.rs`](../src-tauri/src/registration.rs)

**Rol:** Windows kayıtları.

**Amaç / sorumluluk:** SecondBrain ve Orchestrator için Windows Yüklü uygulamalar kayıtlarını (HKCU Uninstall) kurulu duruma göre yazar/siler: Kaldır --uninstall=<ürün>, Değiştir --manage=<ürün> ile Hub'ı açar; Hub ürünler bırakılarak kaldırıldığında maintenance kopyasını tanır, tam Hub dönünce onu siler, son ürün gidince kendini ve klasörlerini temizler. Geliştirme derlemesi yalnızca açıkça verilen test anahtarına yazar.

**İlişkiler ve sınır:** lib.rs başlangıçta ve her yaşam döngüsü işleminden sonra sync çağırır; hub_setup.iss --sync-registrations ve kayıt anahtarlarını kullanır.

#### [`src-tauri/src/rpp.rs`](../src-tauri/src/rpp.rs)

**Rol:** RPP motor köprüsü.

**Amaç / sorumluluk:** Hub'ın Orchestrator ve SecondBrain motorlarıyla RPP üzerinden konuşan tek yerel köprüsü: istek/yanıt zarfları, operasyon id doğrulaması, bilinmeyen ürün/op'un süreç başlatmadan reddi, sır görünümlü ortam değişkenlerinin süzülmesi ve nötr çalışma dizini. Webview köprüsü yalnızca salt-okunur yaşam döngüsü ve ilk vault oluşturmaya izin verir; Hub-içi trusted çağrılar ayrıdır. Okuma/yazma geçidi (Gate) yazıcıya öncelik verir. İzin listesine memory.search/note.read/capture.note/briefing.run eklendi; memory.* köprüden yine engelli. isolate_python: yönetilen Python kullanıcının PYTHONHOME/PYTHONSTARTUP/kullanıcı site-packages ortamını almaz. Test HOME'u yalnızca geliştirme derlemelerinde uygulanır; köprü orchestrator için memory_* ayarlarını reddeder; Gate.write_within bekleyen yazıcıyı zaman aşımıyla bırakır.

**İlişkiler ve sınır:** main.rs, brain.rs, mcp_access.rs, memory_*.rs tarafından çağrılır; motor süreçlerini başlatır. Birim testleri kendi içinde; src-tauri/tests/rpp_e2e.rs ve campaign_e2e.rs gerçek motorlarla test eder.

#### [`src-tauri/src/trust.rs`](../src-tauri/src/trust.rs)

**Rol:** Paket özgünlüğü doğrulaması.

**Amaç / sorumluluk:** Katalog paketlerinin Ed25519 imzasını doğrular (fail-closed): imzalanan metin product, kind, version, schema ve bundle_digest'in alan ayrımlı kanonik JSON'udur; PRODUCTION_KEYS derlemeye gömülür (yayın aşamasına kadar boş), geliştirme anahtarı ve RESPECTED_HUB_ALLOW_UNSIGNED yalnızca debug veya dev-trust derlemelerinde vardır ve üretim derlemesinde koddan çıkarılır. Python aracıyla (tools/bundle_signing.py) üretilmiş bir imza birim testiyle doğrulanır. dev_env/dev_abs: geliştirme ortam değişkenleri üretim derlemesinde hiç okunmaz; üretim testi tüm geçersiz kılmaların yok sayıldığını kanıtlar. Üretim anahtarları release-keys.txt dosyasından derlenir. verify_detached: katalog indeksinin '<key_id> <base64>' biçimli ayrık imzasını aynı anahtarlarla doğrular (imzasız seçeneği yoktur).

**İlişkiler ve sınır:** lifecycle::read_bundle her katalog paketi için verify çağırır; brain.rs güven düzeyine göre --allow-unattested kararını verir. tools/bundle_signing.py aynı ifadeyi imzalar; docs/decisions/BUNDLE_TRUST.md karar kaydıdır.

### src-tauri

#### [`src-tauri/tauri.conf.json`](../src-tauri/tauri.conf.json)

**Rol:** Tauri development config.

**Amaç / sorumluluk:** Respected Hub ürün kimliğini, Vite dev URL'sini (127.0.0.1:1420), ../dist frontendDist'ini, ana pencere boyutlarını ve yalnızca self/ipc'ye izin veren CSP'yi tanımlar; bundle kapalıdır. Sürüm 0.1.0-beta.1.

**İlişkiler ve sınır:** Tauri CLI/derleyici ve native pencere tarafından okunur; npm run dev/build komutlarını tetikler. Production release veya signing iddiası taşımaz.

### src-tauri/tests

#### [`src-tauri/tests/brain_concurrency_e2e.rs`](../src-tauri/tests/brain_concurrency_e2e.rs)

**Rol:** Rust e2e testi.

**Amaç / sorumluluk:** SecondBrain kurulumu sürerken durum yoklamaları ve sihirbaz planları çalışsa da kurulumun kilitlenmeden bittiğini ve yoklamaların program klasöründen çalışmadığını kanıtlar. Geçici köklerde yürür. Yayın araçlarını respected-hub/release/tools yolundan çağırır.

**İlişkiler ve sınır:** brain.rs, rpp.rs Gate ve lifecycle.rs'yi gerçek ürün kaynaklarıyla sınar. cargo test ile çalışır; ../orchestrator ve ../secondbrain kardeş kaynaklarına bağlıdır.

#### [`src-tauri/tests/brain_e2e.rs`](../src-tauri/tests/brain_e2e.rs)

**Rol:** Rust e2e testi.

**Amaç / sorumluluk:** SecondBrain'in motor-yönetimli yaşam döngüsünü uçtan uca kanıtlar: doğrulanmış paket, salt-okunur SIL planı, AI istemci entegrasyonlu setup, RPP kontrolleri, onarım, geri alma ve kaldırma. Sahte HOME ve geçici kökler; kullanıcının gerçek yapılandırmaları okunmaz/yazılmaz. İstemci yapılandırmasını yeniden yazdıktan sonra kaldırmada yalnızca bizim girdilerimizin gittiği ve manifest/günlüklerde gizli bilginin tutulmadığı gerçek akışta doğrulanır. Yayın araçlarını respected-hub/release/tools yolundan çağırır.

**İlişkiler ve sınır:** brain.rs ve lifecycle.rs'yi gerçek donmuş launcher ile sınar. cargo test ile çalışır; kardeş secondbrain dağıtımına bağlıdır.

#### [`src-tauri/tests/campaign_e2e.rs`](../src-tauri/tests/campaign_e2e.rs)

**Rol:** Rust e2e testi.

**Amaç / sorumluluk:** Kampanya takibini gerçek motorla sınar: kimlik kuralları motorla uyumlu, liste yalnızca geçerli klasörleri yeniden eskiye döndürür, geçersiz kimlik süreç başlamadan reddedilir, geçersiz spec motor hatası verir. Geçici veri kökü; kampanya işçisi hiç başlatılmaz.

**İlişkiler ve sınır:** Hub'ın kampanya çağrılarını ve valid_campaign_id kuralını gerçek motorla sınar. cargo test ile çalışır; kardeş orchestrator motoruna bağlıdır.

#### [`src-tauri/tests/cleanup_e2e.rs`](../src-tauri/tests/cleanup_e2e.rs)

**Rol:** Hub kaldırma temizliği uçtan uca testi.

**Amaç / sorumluluk:** Gerçek respected-hub.exe --uninstall-products komutunu sahte HOME ve geçici köklerle çalıştırır: SecondBrain ve Orchestrator kurulu, Claude ve Codex yapılandırmalarına kayıtlı ve istemciler dosyalarını yeniden yazmışken kayıtlarımız kalkar, istemci ve kullanıcı değişiklikleri, kasa notu korunur, yönetilen klasör silinir; boş makinede komut hiçbir şey yaratmaz. Yayın araçlarını respected-hub/release/tools yolundan çağırır.

**İlişkiler ve sınır:** cleanup.rs, lib.rs, mcp_access ve brain yaşam döngüsünü gerçek ürünlerle çalıştırır.

#### [`src-tauri/tests/lifecycle_e2e.rs`](../src-tauri/tests/lifecycle_e2e.rs)

**Rol:** Rust e2e testi.

**Amaç / sorumluluk:** Hub yönetimli yaşam döngüsünü gerçek ürün kaynaklarından üretilen paketlerle kanıtlar: iki ürün kur/doğrula/onar/güncelle/kaldır, motorun çalıştıramadığı kopya asla etkinleştirilmez, ürünler sistem Python'u olmadan Hub'ın Python'unda çalışır. Yalnızca geçici kök. Yerel paketler geliştirme imzasıyla doğrulanır (trust: development); bozuk ama geçerli imzalı paket motor doğrulamasıyla durdurulur. Yayın araçlarını respected-hub/release/tools yolundan çağırır.

**İlişkiler ve sınır:** lifecycle.rs ve rpp.rs'yi gerçek motorlarla sınar. cargo test ile çalışır; kardeş orchestrator/secondbrain kaynaklarına bağlıdır.

#### [`src-tauri/tests/mcp_access_e2e.rs`](../src-tauri/tests/mcp_access_e2e.rs)

**Rol:** Rust e2e testi.

**Amaç / sorumluluk:** Orchestrator MCP kaydını sahte HOME'da uçtan uca sınar: plan, onaya bağlı uygulama, kayıtlı komutun Hub olmadan istemci gibi çalışması, kapatma/geri alma ve yabancı girdilerin korunması. Ayrıca env ile açılan, çalışma setine dahil olmayan bir doğrulama yardımcısı içerir.

**İlişkiler ve sınır:** mcp_access.rs ve lifecycle.rs'yi sınar. cargo test ile çalışır; RESPECTED_MCP_KEEP_DIR yalnızca yardımcı testi tetikler.

#### [`src-tauri/tests/memory_access_e2e.rs`](../src-tauri/tests/memory_access_e2e.rs)

**Rol:** Rust e2e testi.

**Amaç / sorumluluk:** SecondBrain bellek erişiminin Orchestrator için gerçek motorlarla açılıp kapatılmasını sınar; grant belirteci test API'sine hiç sızmaz. Geçici veri kökleri ve sahte home.

**İlişkiler ve sınır:** memory_access.rs ve rpp.rs'yi gerçek motorlarla sınar. cargo test ile çalışır; kardeş ürün kaynaklarına bağlıdır.

#### [`src-tauri/tests/memory_index_e2e.rs`](../src-tauri/tests/memory_index_e2e.rs)

**Rol:** Rust e2e testi.

**Amaç / sorumluluk:** SecondBrain bellek arama indeksinin gerçek motorla okunup yenilenmesini sınar ve vault notlarının bayt düzeyinde değişmediğini doğrular. Geçici veri kökleri ve sahte home.

**İlişkiler ve sınır:** memory_index.rs ve rpp.rs'yi gerçek motorla sınar. cargo test ile çalışır; kardeş secondbrain kaynağına bağlıdır.

#### [`src-tauri/tests/remote_catalog_e2e.rs`](../src-tauri/tests/remote_catalog_e2e.rs)

**Rol:** Rust e2e testi.

**Amaç / sorumluluk:** İndirme yolunu uçtan uca kanıtlar: gerçek publish_release.py ile üretilmiş, geliştirme anahtarıyla imzalı file:// katalog curl ile alınır, arşiv tar ile açılıp önbelleğe girer, onarım ağ gerektirmez, bozuk önbellek yenilenir; kurcalanmış arşiv, kurcalanmış indeks ve daha eski sequence reddedilir.

**İlişkiler ve sınır:** catalog.rs ve lifecycle.rs'yi sınar; release/tools/build_bundle.py ve publish_release.py'yi çağırır.

#### [`src-tauri/tests/rpp_e2e.rs`](../src-tauri/tests/rpp_e2e.rs)

**Rol:** Rust e2e testi.

**Amaç / sorumluluk:** Hub'ın gerçek Orchestrator ve SecondBrain motorlarını RPP 1.0/1.1 ile sürdüğünü kanıtlar: manifest/sağlık/şema, bilinmeyen anahtar reddi, uygula-geri oku ve idempotent tekrar, bilinmeyen op'un yerel reddi, motor bulunamadı, protokol uyuşmazlığı ve yaşam döngüsü planı. Geçici veri kökleri.

**İlişkiler ve sınır:** rpp.rs'yi gerçek motorlarla sınar. cargo test ile çalışır; kardeş orchestrator/secondbrain kaynaklarına bağlıdır.

#### [`src-tauri/tests/trust_catalog_e2e.rs`](../src-tauri/tests/trust_catalog_e2e.rs)

**Rol:** Katalog güveni saldırı testi.

**Amaç / sorumluluk:** Gerçek build_bundle çıktısı üzerinde katalogun yalnızca güvenilir anahtarla imzalı paketi sunduğunu kanıtlar: sürüm/tür değişimi, yeniden özetlenmiş içerik, bozulmuş imza, bilinmeyen algoritma, imza silme ve saldırgan anahtarıyla yeniden imzalama reddedilir; imzasız paket yalnızca geliştirme isteğiyle kabul edilir. Yayın araçlarını respected-hub/release/tools yolundan çağırır.

**İlişkiler ve sınır:** lifecycle::catalog ve status, trust.rs ve tools/bundle_signing.py ile birlikte çalışır.

### src

#### [`src/main.tsx`](../src/main.tsx)

**Rol:** React başlangıcı.

**Amaç / sorumluluk:** #root üzerinde React root oluşturur, App'i StrictMode içinde render eder ve design.css stilini yükler.

**İlişkiler ve sınır:** index.html tarafından yüklenir; App.tsx'i çağırır. StrictMode effect cleanup davranışı korunmalıdır.

### src/products/orchestrator

#### [`src/products/orchestrator/campaigns.ts`](../src/products/orchestrator/campaigns.ts)

**Rol:** Kampanya istemcisi.

**Amaç / sorumluluk:** Hub kampanya takibi için tipli, salt-okunur/başlatma native komut istemcisi ve kampanya durumunu sade özete (faz, şeritler, zaman çizelgesi) çeviren saf mantık. Tauri dışında çağrılar reddedilir, sahte başarı üretilmez. (2026-10-21: `src/products/orchestrator/campaigns.ts` yolundan taşındı.)

**İlişkiler ve sınır:** Campaigns.tsx, AppWorkspace.tsx ve EvidencePanel tarafından kullanılır; api.ts üzerinden native'e gider. tests/orchestrator/campaigns.test.ts ve tests/shared/productClient.test.ts kapsar.

#### [`src/products/orchestrator/mcpAccess.ts`](../src/products/orchestrator/mcpAccess.ts)

**Rol:** Orchestrator MCP erişim istemcisi.

**Amaç / sorumluluk:** Orchestrator'ın AI istemcilerine MCP sunucusu olarak bağlanması için plan/uygula native çağrıları ve anahtar durumu yardımcıları. Yapılandırmayı Orchestrator yazar; Hub planlar, onay ister ve günlüğe işler. Yalnızca farklı olan istemciler gönderilir. (2026-10-21: `src/products/orchestrator/mcpAccess.ts` yolundan taşındı.)

**İlişkiler ve sınır:** views/OrchestratorMcp.tsx kullanır; lifecycle.ts türlerine dayanır. tests/orchestrator/mcpAccess.test.ts kapsar.

#### [`src/products/orchestrator/tasks.ts`](../src/products/orchestrator/tasks.ts)

**Rol:** Form doğrulaması.

**Amaç / sorumluluk:** Project/task/scope, JSON argv acceptance/custom komut, deadline ve pending/terminal sınırlarını doğrular. (2026-10-21: `src/products/orchestrator/tasks.ts` yolundan taşındı.)

**İlişkiler ve sınır:** App submit ve 4 Node testi gerçek üretim fonksiyonlarını kullanır.

### src/products/orchestrator/views

#### [`src/products/orchestrator/views/AgentsPanel.tsx`](../src/products/orchestrator/views/AgentsPanel.tsx)

**Rol:** Ajanlar ve modeller paneli.

**Amaç / sorumluluk:** Engine'den gelen gerçek worker CLI'larını ve RPP ayarlarındaki model/sağlayıcı profillerini gösterir. Uydurma sağlayıcı veya canlı metrik içermez. (2026-10-21: `src/products/orchestrator/views/AgentsPanel.tsx` yolundan taşındı.)

**İlişkiler ve sınır:** AppWorkspace tarafından açılır; agentsModel.ts ve agents.css kullanır. Kapsam: tests/orchestrator/agents.test.ts.

#### [`src/products/orchestrator/views/Campaigns.tsx`](../src/products/orchestrator/views/Campaigns.tsx)

**Rol:** Kampanyalar görünümü.

**Amaç / sorumluluk:** Hub kampanya takibi: kampanya listesi, başlatma ve durum özeti. Gerçek veri gösterir; Tauri dışında dürüst bir uyarı verir. Zamanı tr-TR biçimler. (2026-10-21: `src/products/orchestrator/views/Campaigns.tsx` yolundan taşındı.)

**İlişkiler ve sınır:** AppWorkspace Kampanyalar sayfasında açar; campaigns.ts ve api.ts kullanır. Kapsam: tests/orchestrator/campaigns.test.ts.

#### [`src/products/orchestrator/views/EvidencePanel.tsx`](../src/products/orchestrator/views/EvidencePanel.tsx)

**Rol:** Kanıt ve inceleme paneli.

**Amaç / sorumluluk:** Seçilen koşu veya kampanya için engine 'inspect' ve kampanya durumu yanıtlarını sade Türkçe kanıt, karar ve yama özeti olarak gösterir. (2026-10-21: `src/products/orchestrator/views/EvidencePanel.tsx` yolundan taşındı.)

**İlişkiler ve sınır:** AppWorkspace tarafından açılır; evidenceModel.ts, campaigns.ts ve api.ts kullanır. Kapsam: tests/orchestrator/evidence.test.ts.

#### [`src/products/orchestrator/views/OrchestratorMcp.tsx`](../src/products/orchestrator/views/OrchestratorMcp.tsx)

**Rol:** Orchestrator MCP görünümü.

**Amaç / sorumluluk:** Orchestrator'ı AI istemcilerine MCP sunucusu olarak bağlar. Kullanıcı tam planı onaylamadan hiçbir şey yazılmaz. (2026-10-21: `src/products/orchestrator/views/OrchestratorMcp.tsx` yolundan taşındı.)

**İlişkiler ve sınır:** AppWorkspace tarafından açılır; mcpAccess.ts kullanır. Kapsam: tests/orchestrator/mcpAccess.test.ts.

#### [`src/products/orchestrator/views/OverviewPanel.tsx`](../src/products/orchestrator/views/OverviewPanel.tsx)

**Rol:** Orchestrator genel bakış paneli.

**Amaç / sorumluluk:** Engine koşu sayılarını, son koşuları ve worker CLI durumunu gösterir; eksik engine verisi dürüst uyarıya dönüşür. Görevler ve koşu ayrıntısına geçiş sunar. (2026-10-21: `src/products/orchestrator/views/OverviewPanel.tsx` yolundan taşındı.)

**İlişkiler ve sınır:** AppWorkspace tarafından açılır; overviewModel.ts ve overview.css kullanır. Kapsam: tests/orchestrator/overview.test.ts.

#### [`src/products/orchestrator/views/Tasks.tsx`](../src/products/orchestrator/views/Tasks.tsx)

**Rol:** Görevler görünümü.

**Amaç / sorumluluk:** Orchestrator görevi oluşturma ve koşu inceleme. Görev durumunu engine tutar; görünüm yalnızca izinli engine işlemlerini çağırır. (2026-10-21: `src/products/orchestrator/views/Tasks.tsx` yolundan taşındı.)

**İlişkiler ve sınır:** AppWorkspace Görevler sayfasında açar; api.ts ve tasks.ts kullanır. Kapsam: tests/shared/productClient.test.ts (native reddi).

#### [`src/products/orchestrator/views/agents.css`](../src/products/orchestrator/views/agents.css)

**Rol:** Panel stili.

**Amaç / sorumluluk:** Ajanlar ve modeller panelinin yerleşimi; genel .panel-glass, .muted, .capabilities ve .notice sınıflarını yeniden kullanır. (2026-10-21: `src/products/orchestrator/views/agents.css` yolundan taşındı.)

**İlişkiler ve sınır:** AgentsPanel.tsx içe aktarır. design.css sınıflarına dayanır.

#### [`src/products/orchestrator/views/agentsModel.ts`](../src/products/orchestrator/views/agentsModel.ts)

**Rol:** Ajanlar paneli modeli.

**Amaç / sorumluluk:** Ajanlar/modeller paneli için saf model: engine status.workers'tan worker CLI'ları ve RPP settings.readback'ten gerçekte var olan model/sağlayıcı profillerini türetir. React/Tauri içermez, veri uydurmaz. (2026-10-21: `src/products/orchestrator/views/agentsModel.ts` yolundan taşındı.)

**İlişkiler ve sınır:** AgentsPanel.tsx çağırır. tests/orchestrator/agents.test.ts kapsar.

#### [`src/products/orchestrator/views/evidence.css`](../src/products/orchestrator/views/evidence.css)

**Rol:** Panel stili.

**Amaç / sorumluluk:** Kanıt/inceleme panelinin yerleşimi ve seçim alanları; Hub tasarım token'larını güvenli yedek değerlerle kullanır. (2026-10-21: `src/products/orchestrator/views/evidence.css` yolundan taşındı.)

**İlişkiler ve sınır:** EvidencePanel.tsx içe aktarır. design.css token'larına dayanır.

#### [`src/products/orchestrator/views/evidenceModel.ts`](../src/products/orchestrator/views/evidenceModel.ts)

**Rol:** Kanıt paneli modeli.

**Amaç / sorumluluk:** Ham engine 'inspect' ve campaign-status yanıtlarını sade Türkçe kanıt, karar (verdict), inceleme ve yama bilgisine çeviren saf model. Durum ifadeleri campaigns.ts ile uyumludur ama Tauri'ye bağlı istemci içe aktarılmaz. (2026-10-21: `src/products/orchestrator/views/evidenceModel.ts` yolundan taşındı.)

**İlişkiler ve sınır:** EvidencePanel.tsx çağırır. tests/orchestrator/evidence.test.ts kapsar.

#### [`src/products/orchestrator/views/overview.css`](../src/products/orchestrator/views/overview.css)

**Rol:** Panel stili.

**Amaç / sorumluluk:** Orchestrator genel bakış panelinin yerleşimi; genel .panel-glass, .status ve .notice sınıflarını yeniden kullanır. (2026-10-21: `src/products/orchestrator/views/overview.css` yolundan taşındı.)

**İlişkiler ve sınır:** OverviewPanel.tsx içe aktarır. design.css sınıflarına dayanır.

#### [`src/products/orchestrator/views/overviewModel.ts`](../src/products/orchestrator/views/overviewModel.ts)

**Rol:** Genel bakış modeli.

**Amaç / sorumluluk:** Koşuları etkin/başarılı/başarısız/diğer olarak sayan, son koşuları (LATEST_LIMIT) seçen ve eksik engine verisini dürüst uyarıya çeviren saf model. (2026-10-21: `src/products/orchestrator/views/overviewModel.ts` yolundan taşındı.)

**İlişkiler ve sınır:** OverviewPanel.tsx çağırır; tasks.ts terminal durumunu kullanır. tests/orchestrator/overview.test.ts kapsar.

#### [`src/products/orchestrator/views/settings.css`](../src/products/orchestrator/views/settings.css)

**Rol:** Panel stili.

**Amaç / sorumluluk:** Orchestrator çalışma alanı ayarlarındaki veri kökü özet bloğunun küçük boşluk stili. (2026-10-21: `src/products/orchestrator/views/settings.css` yolundan taşındı.)

**İlişkiler ve sınır:** Orchestrator ayar/çalışma alanı görünümleri tarafından kullanılır. design.css ile birlikte yüklenir.

#### [`src/products/orchestrator/views/workspaceModel.ts`](../src/products/orchestrator/views/workspaceModel.ts)

**Rol:** Çalışma alanı modeli.

**Amaç / sorumluluk:** Orchestrator çalışma alanı sekmelerini (ORCHESTRATOR_PAGES) ve bağlama kararlarını tutan saf mantık; kayıt defteri ile bağlama mantığının ayrışmaması için tek kaynaktır. Tarayıcı önizlemesi dürüstçe etiketlenir. SecondBrain için 'memory' bağlama noktası eklendi. (2026-10-21: `src/products/orchestrator/views/workspaceModel.ts` yolundan taşındı.)

**İlişkiler ve sınır:** apps.ts ve AppWorkspace.tsx kullanır. tests/orchestrator/workspace.test.ts ve tests/shell/appNav.test.ts kapsar.

### src/products/secondbrain

#### [`src/products/secondbrain/brainClients.ts`](../src/products/secondbrain/brainClients.ts)

**Rol:** AI istemci seçimi modeli.

**Amaç / sorumluluk:** SecondBrain'in bağlanabildiği AI istemcilerini (hooks/mcp yetenekleri) ve kullanıcı seçimini ürünün beklediği `desired` nesnesine çeviren saf mantığı içerir. Kayıtlı entegrasyonlardan seçimi geri üretir; ürün her id'yi yine doğrular. (2026-10-21: `src/products/secondbrain/brainClients.ts` yolundan taşındı.)

**İlişkiler ve sınır:** ClientPicker, BrainSetup/BrainInstall ve BrainIntegrations görünümleri kullanır. tests/secondbrain/brainClients.test.ts kapsar.

#### [`src/products/secondbrain/memoryAccess.ts`](../src/products/secondbrain/memoryAccess.ts)

**Rol:** Hafıza erişim istemcisi.

**Amaç / sorumluluk:** Orchestrator'ın SecondBrain hafızasına erişimini açma/kapama/durum native çağrıları ve durumu Türkçe açıklamaya çeviren mantık. Erişim token'ı yalnızca Rust tarafında kalır; burada alınmaz, saklanmaz, gösterilmez. (2026-10-21: `src/products/secondbrain/memoryAccess.ts` yolundan taşındı.)

**İlişkiler ve sınır:** views/MemoryAccess.tsx kullanır; lifecycle.ts hata türüne dayanır. tests/secondbrain/memoryAccess.test.ts kapsar.

#### [`src/products/secondbrain/memoryIndex.ts`](../src/products/secondbrain/memoryIndex.ts)

**Rol:** Hafıza dizini istemcisi.

**Amaç / sorumluluk:** SecondBrain arama dizininin durumu, oturum yakalama ve kullanıcı onaylı yenileme için güvenilir Rust komutlarını saran tipli istemci ve durum/sonuç açıklayıcıları. (2026-10-21: `src/products/secondbrain/memoryIndex.ts` yolundan taşındı.)

**İlişkiler ve sınır:** views/MemoryIndex.tsx kullanır. tests/secondbrain/memoryIndex.test.ts kapsar.

#### [`src/products/secondbrain/memorySemantic.ts`](../src/products/secondbrain/memorySemantic.ts)

**Rol:** Anlamsal arama istemcisi.

**Amaç / sorumluluk:** SecondBrain'in isteğe bağlı yerel anlamsal aramasının durum/aç/kapat native çağrıları ve durumu başlık, ayrıntı ve ton olarak açıklayan mantık. Varsayılan kapalıdır; tam metin arama her durumda çalışır. (2026-10-21: `src/products/secondbrain/memorySemantic.ts` yolundan taşındı.)

**İlişkiler ve sınır:** views/MemoryIndex.tsx kullanır. tests/secondbrain/memorySemantic.test.ts kapsar.

#### [`src/products/secondbrain/memoryWorkspace.ts`](../src/products/secondbrain/memoryWorkspace.ts)

**Rol:** Tauri IPC sarmalayıcı.

**Amaç / sorumluluk:** SecondBrain hafıza çalışma alanı için güvenilir Rust komutlarının (memory_search, memory_note_read, memory_capture_note, memory_briefing_run) tür tanımlı sarmalayıcıları, motorun bilinen notları (Son oturum, Açık konular, Kurallar, günlük, brifing) ve dürüst Türkçe hata/arama modu metinleri. (2026-10-21: `src/products/secondbrain/memoryWorkspace.ts` yolundan taşındı.)

**İlişkiler ve sınır:** views/MemoryWorkspace.tsx kullanır; masaüstü dışında UNAVAILABLE döner. Testi tests/secondbrain/memoryWorkspace.test.ts.

### src/products/secondbrain/views

#### [`src/products/secondbrain/views/BrainInstall.tsx`](../src/products/secondbrain/views/BrainInstall.tsx)

**Rol:** SecondBrain kurulum sihirbazı.

**Amaç / sorumluluk:** SecondBrain'in kendi kurucusu üzerinden ilk kurulum: Kimlik → Hafıza → Kaynaklar → İzinler → Onay adımları. Önce salt-okunur plan gösterir, onayla uygular. Kısayol açıklaması kasa klasörünü açan kısayol olarak düzeltildi. (2026-10-21: `src/products/secondbrain/views/BrainInstall.tsx` yolundan taşındı.)

**İlişkiler ve sınır:** AppWorkspace kurulum sayfasında açar; lifecycle.ts, productClient.ts, brainClients.ts, ClientPicker ve IntegrationPreview kullanır. Kapsam: tests/secondbrain/brainClients.test.ts, tests/shell/lifecycleModel.test.ts.

#### [`src/products/secondbrain/views/BrainIntegrations.tsx`](../src/products/secondbrain/views/BrainIntegrations.tsx)

**Rol:** SecondBrain entegrasyon seçimi.

**Amaç / sorumluluk:** Kurulu SecondBrain için hangi AI istemcisinde hafıza yakalanacağını (Kaynaklar) ve kimin MCP ile kullanacağını (İzinler) seçtirir; önce açık, salt-okunur önizleme verir ve onayla uygular. Kısayol açıklaması kasa klasörünü açan kısayol olarak düzeltildi. (2026-10-21: `src/products/secondbrain/views/BrainIntegrations.tsx` yolundan taşındı.)

**İlişkiler ve sınır:** BrainSetup içinden kullanılır; ClientPicker, IntegrationPreview, brainClients.ts ve lifecycle.ts'e dayanır. Kapsam: tests/secondbrain/brainClients.test.ts.

#### [`src/products/secondbrain/views/BrainSetup.tsx`](../src/products/secondbrain/views/BrainSetup.tsx)

**Rol:** SecondBrain ayar görünümü.

**Amaç / sorumluluk:** SecondBrain'in gerçek ayarlarını ürün şemasından forma çevirir, değişenleri uygular ve geri okuma ile doğrular; asistan adı kişiselleştirmesini localStorage'da saklar. Entegrasyon seçimini içerir. (2026-10-21: `src/products/secondbrain/views/BrainSetup.tsx` yolundan taşındı.)

**İlişkiler ve sınır:** AppWorkspace ve Home tarafından kullanılır; productClient.ts, rpp.ts, productsModel.ts ve BrainIntegrations'a dayanır. Kapsam: tests/shared/productsModel.test.ts.

#### [`src/products/secondbrain/views/ClientPicker.tsx`](../src/products/secondbrain/views/ClientPicker.tsx)

**Rol:** AI istemci seçici.

**Amaç / sorumluluk:** Desteklenen her AI istemcisi için bir anahtar sunar. Bu bilgisayarda bulunmayan istemciler yeni seçilemez (klasörleri oluşturulmasın diye). (2026-10-21: `src/products/secondbrain/views/ClientPicker.tsx` yolundan taşındı.)

**İlişkiler ve sınır:** BrainInstall ve BrainIntegrations tarafından kullanılır; brainClients.ts'ten CLIENTS ve toggle alır. Kapsam: tests/secondbrain/brainClients.test.ts.

#### [`src/products/secondbrain/views/IntegrationPreview.tsx`](../src/products/secondbrain/views/IntegrationPreview.tsx)

**Rol:** Entegrasyon önizlemesi.

**Amaç / sorumluluk:** Ürünün salt-okunur planını gösterir: nelerin bulunduğu, tam olarak neyin değişeceği ve dokunulmayanlar. Değişiklik yoksa bunu açıkça söyler. (2026-10-21: `src/products/secondbrain/views/IntegrationPreview.tsx` yolundan taşındı.)

**İlişkiler ve sınır:** BrainInstall ve BrainIntegrations kullanır; lifecycle.ts IntegrationPlan türüne dayanır. Doğrudan test yok.

#### [`src/products/secondbrain/views/MemoryAccess.tsx`](../src/products/secondbrain/views/MemoryAccess.tsx)

**Rol:** Hafıza erişimi görünümü.

**Amaç / sorumluluk:** SecondBrain hafıza erişimini (salt okunur veya yazma dahil) onayla açar/kapatır ve durumu gösterir. Erişim token'ı bileşene hiç ulaşmaz. (2026-10-21: `src/products/secondbrain/views/MemoryAccess.tsx` yolundan taşındı.)

**İlişkiler ve sınır:** AppWorkspace tarafından açılır; memoryAccess.ts kullanır. Kapsam: tests/secondbrain/memoryAccess.test.ts.

#### [`src/products/secondbrain/views/MemoryIndex.tsx`](../src/products/secondbrain/views/MemoryIndex.tsx)

**Rol:** Hafıza dizini görünümü.

**Amaç / sorumluluk:** SecondBrain arama dizininin durumunu, oturum yakalamayı ve anlamsal aramayı gösterir; dizin yenilemeyi açık onaydan sonra çalıştırır. Bekleyen oturumları yakalama iki adımlı onay ister (oturumlar yapay zeka sağlayıcısına gönderilir). (2026-10-21: `src/products/secondbrain/views/MemoryIndex.tsx` yolundan taşındı.)

**İlişkiler ve sınır:** AppWorkspace tarafından açılır; memoryIndex.ts ve memorySemantic.ts kullanır. Kapsam: tests/secondbrain/memoryIndex.test.ts, tests/secondbrain/memorySemantic.test.ts.

#### [`src/products/secondbrain/views/MemoryWorkspace.tsx`](../src/products/secondbrain/views/MemoryWorkspace.tsx)

**Rol:** Kullanıcı arayüzü.

**Amaç / sorumluluk:** SecondBrain 'Hafıza' sayfası: not arama (tam metin, hazırsa anlamsal; vurgu işaretleri temizlenir), not okuyucu ve bilinen not düğmeleri, hızlı not yakalama ve 'brifingi şimdi çalıştır'. Eski yerel panelin işlevlerini Hub'a taşır; hiçbir HTML enjekte etmez. (2026-10-21: `src/products/secondbrain/views/MemoryWorkspace.tsx` yolundan taşındı.)

**İlişkiler ve sınır:** AppWorkspace.tsx 'memory' sayfasında bağlar; yalnız memoryWorkspace.ts ve react kullanır. Bu görünüm Orchestrator şeridinde yazıldı, native E2E ile doğrulandı.

### src/shared

#### [`src/shared/api.ts`](../src/shared/api.ts)

**Rol:** Tipli native API.

**Amaç / sorumluluk:** Tauri invoke ile explicit engine/settings/product işlemleri ve runtime detection/types sağlar. startup_intent komutunu startupIntent olarak sunar. StartupIntent türü: { action: install|uninstall|manage, product }. (2026-10-21: `src/shared/api.ts` yolundan taşındı.)

**İlişkiler ve sınır:** Rust command handler tüketilir; generic shell plugin veya sahte sonuç yoktur.

#### [`src/shared/design.css`](../src/shared/design.css)

**Rol:** Görsel yönetim.

**Amaç / sorumluluk:** Hub tasarım sistemi v2: renk/boşluk token'ları, kabuk, başlatıcı sayfalar, ortak bileşenler ve çalışma alanı panellerinin kullandığı eski sınıf adları. Tek stil kaynağıdır; dar ekranda ikon rayına düşer. (2026-10-21: `src/shared/design.css` yolundan taşındı.)

**İlişkiler ve sınır:** Uygulama girişi ve tüm görünümler tarafından yüklenir; orchestrator/*.css bu sınıflara dayanır. Eski hub.css ve style.css'in yerine geçer.

#### [`src/shared/productClient.ts`](../src/shared/productClient.ts)

**Rol:** RPP ürün istemcisi.

**Amaç / sorumluluk:** Özel 'rpp_request' komutu üzerine ince tipli sarmalayıcı. Tauri dışında net bir hatayla reddeder; taşıma katmanı birim testler için enjekte edilebilir. (2026-10-21: `src/shared/productClient.ts` yolundan taşındı.)

**İlişkiler ve sınır:** useEcosystem, Products, BrainSetup/BrainInstall ve AppWorkspace kullanır; rpp.ts ile istek/yanıt kurar. tests/shared/productClient.test.ts kapsar.

#### [`src/shared/productsModel.ts`](../src/shared/productsModel.ts)

**Rol:** Ürün ayarları modeli.

**Amaç / sorumluluk:** Ürünler görünümü için saf model: manifest/sağlıktan ürün özeti ve durum türü, ayar şemasından form alanları, değişen ayarlar, değer ayrıştırma ve geri okuma doğrulaması. Sır değerleri maskeli kalır. (2026-10-21: `src/shared/productsModel.ts` yolundan taşındı.)

**İlişkiler ve sınır:** Products, BrainSetup, homeModel, shellModel ve useEcosystem kullanır; rpp.ts'e dayanır. tests/shared/productsModel.test.ts kapsar.

#### [`src/shared/rpp.ts`](../src/shared/rpp.ts)

**Rol:** RPP sözleşmesi.

**Amaç / sorumluluk:** Respected Product Protocol v1.0/v1.1 için paylaşılan TypeScript sözleşmesi: istek kurma, yanıt ayrıştırma, sürüm pazarlığı, ayar şeması türleri ve sır maskeleme. Bir JSON istek, bir JSON yanıt satırı; MCP veya daemon değildir. (2026-10-21: `src/shared/rpp.ts` yolundan taşındı.)

**İlişkiler ve sınır:** productClient.ts, productsModel.ts ve Products görünümü kullanır. tests/shared/rpp.test.ts kapsar.

#### [`src/shared/ui.tsx`](../src/shared/ui.tsx)

**Rol:** Ortak sunum bileşenleri.

**Amaç / sorumluluk:** Paylaşılan sunum parçaları: ikon seti, uygulama amblemleri, dekoratif Weave ve sade dilde durum etiketi/tonu yardımcıları. İş mantığı içermez. (2026-10-21: `src/shared/ui.tsx` yolundan taşındı.)

**İlişkiler ve sınır:** Home, MyApps, Discover, Activity, LifecycleControls ve kabuk kullanır. testler doğrudan kapsamaz; görünümler üzerinden kullanılır.

### src/shell

#### [`src/shell/App.tsx`](../src/shell/App.tsx)

**Rol:** Görsel yönetim.

**Amaç / sorumluluk:** Hub kabuğunu çizer: global rail (Hub sayfaları ve uygulama çalışma alanları), üst çubuk, sağlık özeti ve yenileme; route'a göre Home, MyApps, Discover, Connections, Activity, Settings ve AppWorkspace görünümlerini gösterir. Tarayıcı önizlemesinde gerçek veri uydurmaz, uyarı gösterir. Seçili sayfa oturum boyunca sessionStorage'da tutulur; WebView yeniden yüklenirse (ör. kurulum sırasında) kullanıcı Ana Sayfa'ya atılmaz. Product kurulumcularından gelen --install niyeti oturum başına bir kez ilgili sayfayı açar (startupIntent). startup-intent olayı dinlenir: Hub açıkken başlatılan ürün kurucusunun seçimi ilgili sayfayı açar. Windows Yüklü uygulamalar ve ürün kurulumlarından gelen niyetleri (install/uninstall/manage) işler; Hub ve ürün güncellemelerini Güncelle/Sonra bildirimiyle önerir, Hub güncellemesini başlatır; Hub sürümünü gösterir. (2026-10-21: `src/shell/App.tsx` yolundan taşındı.)

**İlişkiler ve sınır:** main.tsx çağırır; apps.ts, api.ts (native), useEcosystem.ts, shellModel.ts (globalHealth), ui.tsx ve views/* kullanır. Kabuk modeli ve gezinme testleri sağlık/rota mantığını kapsar.

#### [`src/shell/apps.ts`](../src/shell/apps.ts)

**Rol:** Uygulama kayıt defteri.

**Amaç / sorumluluk:** Hub'ın tanıdığı ekosistem uygulamalarını (SecondBrain, Orchestrator), üst gezinme öğelerini ve sayfa listelerini tanımlar. Yeni uygulama eklemek bu listeye kayıt eklemektir; `rpp` yalnızca RPP konuşan ürünlerde dolar. SecondBrain sayfalarına 'Hafıza' eklendi. (2026-10-21: `src/shell/apps.ts` yolundan taşındı.)

**İlişkiler ve sınır:** useEcosystem.ts, Home/MyApps/Discover görünümleri ve shell tarafından okunur; orchestrator/workspaceModel.ts'ten ORCHESTRATOR_PAGES alır. tests/shell/appNav.test.ts kapsar.

#### [`src/shell/homeModel.ts`](../src/shell/homeModel.ts)

**Rol:** Ana sayfa karar modeli.

**Amaç / sorumluluk:** Gerçek ekosistem durumuna göre Ana Sayfa modunu (önizleme, yükleniyor, ilk açılış, dikkat, kurulum eksik, aktif) seçen saf karar fonksiyonu. React ve Tauri içermez. (2026-10-21: `src/shell/homeModel.ts` yolundan taşındı.)

**İlişkiler ve sınır:** views/Home.tsx tarafından çağrılır; productsModel.ts'ten ProductStatusKind türünü kullanır. tests/shell/homeModel.test.ts kapsar.

#### [`src/shell/lifecycle.ts`](../src/shell/lifecycle.ts)

**Rol:** Ürün yaşam döngüsü istemcisi.

**Amaç / sorumluluk:** Hub yönetimli ürünlerin kurulum/güncelleme/onarım/kaldırma/geri alma işlemleri için tipli native istemci, plan ve günlük türleri ve etiket/hata/geri alma yardımcıları. Native { ok, result, error } döner; hiçbir şey simüle edilmez. Güven düzeyi (trust) türü; SecondBrain kaldırması geri alınamaz olarak işaretlenir (canRollback). Uzak katalog ve Hub güncellemesi istemcileri (hubUpdateStatus, catalogRefresh, catalogSetChannel, hubUpdateInstall), kanal türü, yeni hata kodları; güncelleme geri alınamaz (eski sürüm silinir). (2026-10-21: `src/shell/lifecycle.ts` yolundan taşındı.)

**İlişkiler ve sınır:** LifecycleControls, BrainSetup/BrainInstall, Activity, Discover, useEcosystem ve mcpAccess kullanır. tests/shell/lifecycleModel.test.ts kapsar.

#### [`src/shell/shellModel.ts`](../src/shell/shellModel.ts)

**Rol:** Kabuk modeli.

**Amaç / sorumluluk:** Global kabuk için üst çubuk sağlık hapı ve ekran metnini üreten saf model. Native yoksa 'Tarayıcı önizlemesi', yoksa ürün durumlarından sağlık tonu ve etiket verir. (2026-10-21: `src/shell/shellModel.ts` yolundan taşındı.)

**İlişkiler ve sınır:** Uygulama kabuğu (App) tarafından çağrılır; productsModel.ts'ten durum türünü alır. tests/shell/shellModel.test.ts kapsar.

#### [`src/shell/useEcosystem.ts`](../src/shell/useEcosystem.ts)

**Rol:** Ekosistem durum kancası.

**Amaç / sorumluluk:** Kabuk için canlı ekosistem durumunu toplayan React kancası: her uygulamanın RPP manifest/sağlık/ayar özeti, yapılandırma kaynağı, engine koşuları, yaşam döngüsü genel bakışı ve günlük. Native yoksa dürüstçe boş durum döner. Yenileme tek seferde bir tane çalışır; arada gelen istek bir kez daha çalıştırılır. (2026-10-21: `src/shell/useEcosystem.ts` yolundan taşındı.)

**İlişkiler ve sınır:** Kabuk ve tüm üst düzey görünümler tarafından kullanılır; apps.ts, productClient.ts, lifecycle.ts ve productsModel.ts'e dayanır. Doğrudan test yok; modelleri homeModel/shellModel/productsModel testleriyle kapsanır.

### src/shell/views

#### [`src/shell/views/Activity.tsx`](../src/shell/views/Activity.tsx)

**Rol:** Etkinlik görünümü.

**Amaç / sorumluluk:** Engine koşularını ve ürün yaşam döngüsü günlüğünü listeler; uygun girdiler için geri alma sunar. Hata mesajlarını kullanıcıya gösterir. (2026-10-21: `src/shell/views/Activity.tsx` yolundan taşındı.)

**İlişkiler ve sınır:** Kabuk tarafından Etkinlik rotasında açılır; useEcosystem, lifecycle.ts ve tasks.ts kullanır. Kapsam: tests/shell/lifecycleModel.test.ts (günlük cümleleri).

#### [`src/shell/views/AppWorkspace.tsx`](../src/shell/views/AppWorkspace.tsx)

**Rol:** Uygulama çalışma alanı.

**Amaç / sorumluluk:** Seçilen uygulamanın sayfalarını (SecondBrain kurulum/ayarlar, Orchestrator genel bakış, ajanlar, kanıtlar, kampanyalar, görevler, MCP, hafıza) uygun panele bağlayan bağlayıcı görünüm. 'memory' sayfası MemoryWorkspace'i bağlar. (2026-10-21: `src/shell/views/AppWorkspace.tsx` yolundan taşındı.)

**İlişkiler ve sınır:** Kabuk tarafından açılır; BrainSetup, BrainInstall, Tasks, Products, Campaigns, MemoryAccess, MemoryIndex, OrchestratorMcp ve orchestrator panellerini bağlar. workspaceModel.ts karar mantığı tests/orchestrator/workspace.test.ts ile kapsanır.

#### [`src/shell/views/Connections.tsx`](../src/shell/views/Connections.tsx)

**Rol:** Bağlantılar görünümü.

**Amaç / sorumluluk:** Geliştirme kaynak bağlantılarını (python/ürün kökleri) ve her ürünün RPP ayarlarını gösterir. Ayar kaydı sonucu doğrular ve hataları kullanıcıya iletir. Form yalnızca dört bilinen kaynak alanını gösterir. Kaynak bağlantıları paneli yalnızca geliştirme derlemelerinde gösterilir. (2026-10-21: `src/shell/views/Connections.tsx` yolundan taşındı.)

**İlişkiler ve sınır:** Kabuk Bağlantılar rotasında açılır; api.ts ve Products.tsx kullanır. Doğrudan test yok; ayar modeli tests/shared/productsModel.test.ts ile kapsanır.

#### [`src/shell/views/Discover.tsx`](../src/shell/views/Discover.tsx)

**Rol:** Keşfet görünümü.

**Amaç / sorumluluk:** Yerel katalogdaki doğrulanmış paketleri ve her ürünün kurulum durumunu listeler. Kurulum/güncelleme/onarım Uygulamalarım sayfasına yönlendirilir. Paket güveni etiketi: üretim imzalı, geliştirme imzası (yerel) veya imzasız yerel. (2026-10-21: `src/shell/views/Discover.tsx` yolundan taşındı.)

**İlişkiler ve sınır:** Kabuk Keşfet rotasında açılır; APPS, useEcosystem ve lifecycle.ts kullanır. Doğrudan test yok.

#### [`src/shell/views/Home.tsx`](../src/shell/views/Home.tsx)

**Rol:** Ana sayfa görünümü.

**Amaç / sorumluluk:** Ekosistemin gerçek durumuna göre (homeMode) karşılama, dikkat gerektirenler, kurulum eksikleri ve çalışan görevleri gösteren ana sayfa. Go yönlendirme türünü tanımlar. (2026-10-21: `src/shell/views/Home.tsx` yolundan taşındı.)

**İlişkiler ve sınır:** Kabuk tarafından açılır; homeModel.ts, useEcosystem ve ui.tsx kullanır. Kapsam: tests/shell/homeModel.test.ts.

#### [`src/shell/views/LifecycleControls.tsx`](../src/shell/views/LifecycleControls.tsx)

**Rol:** Yaşam döngüsü kontrolleri.

**Amaç / sorumluluk:** Tek bir Hub yönetimli ürün için kur/güncelle/onar/kaldır: önce plan, onay, uygulama, sonra geri alma ve yedek temizleme sunar. Geri döndürülemez adımlar açıkça belirtilir. Yedekleri temizleme ve SecondBrain kaldırma metni iki adımlı onay ve doğru kapsam açıklamasıyla gösterilir. autoAction ile başka yerde seçilmiş işlemin (Yüklü uygulamalar, güncelleme bildirimi) planını doğrudan açar; paket indirilirken durum metni gösterir; katalog alınamadıysa Ayarlar'a yönlendirir. (2026-10-21: `src/shell/views/LifecycleControls.tsx` yolundan taşındı.)

**İlişkiler ve sınır:** MyApps tarafından kullanılır; lifecycle.ts işlemlerini çağırır. Kapsam: tests/shell/lifecycleModel.test.ts.

#### [`src/shell/views/MyApps.tsx`](../src/shell/views/MyApps.tsx)

**Rol:** Uygulamalarım görünümü.

**Amaç / sorumluluk:** Hub'a bağlı ürünleri durum, sürüm ve yetenekleriyle listeler (ürünün kendi sağlık yanıtından); kurulum eksikse sihirbaza yönlendirir ve yaşam döngüsü kontrollerini barındırır. App'ten gelen istenen işlemi (ask) ilgili ürünün kontrollerine iletir; ürünlerin imzalı katalogdan indirildiğini ve Windows Uygulamalar listesinde göründüğünü açıklar. (2026-10-21: `src/shell/views/MyApps.tsx` yolundan taşındı.)

**İlişkiler ve sınır:** Kabuk tarafından açılır; useEcosystem, LifecycleControls ve ui.tsx kullanır. Doğrudan test yok.

#### [`src/shell/views/Products.tsx`](../src/shell/views/Products.tsx)

**Rol:** Ürün ayarları görünümü.

**Amaç / sorumluluk:** Orchestrator ve SecondBrain için RPP ayar şemasından form üretir, doğrular, uygular ve geri okumayla doğrular. Sır alanları maskeli kalır. Mevcut ayarlar okunamadıysa Uygula kapalıdır (varsayılanlar gerçek ayarların üzerine yazılmaz). (2026-10-21: `src/shell/views/Products.tsx` yolundan taşındı.)

**İlişkiler ve sınır:** Connections ve AppWorkspace (ProductPanel) kullanır; productClient.ts, rpp.ts, productsModel.ts'e dayanır. Kapsam: tests/shared/productsModel.test.ts.

#### [`src/shell/views/Settings.tsx`](../src/shell/views/Settings.tsx)

**Rol:** Hub ayarları görünümü.

**Amaç / sorumluluk:** Python yolu, ürün kaynak klasörleri ve görev verisi klasörü gibi geliştirme ayarlarını düzenletir; kayıt sonrası durumu (kaydediliyor/uygulandı/doğrulandı/başarısız) bildirir. Güncellemeler bölümü: Hub sürümü, beta/kararlı kanal seçimi ve Güncellemeleri denetle. (2026-10-21: `src/shell/views/Settings.tsx` yolundan taşındı.)

**İlişkiler ve sınır:** Kabuk Ayarlar rotasında açılır; api.ts ve lifecycle.ts kullanır. Doğrudan test yok.

### tests/native

#### [`tests/native/hub_native_e2e.mjs`](../tests/native/hub_native_e2e.mjs)

**Rol:** Native E2E aracı.

**Amaç / sorumluluk:** Derlenmiş Hub'ı (geliştirme veya kurulu) WebView2 DevTools portu üzerinden gerçek masaüstünde uçtan uca sürer: atılabilir kökler, sahte kullanıcı evi, PATH'ten Python çıkarılmış; ürün paketlerini release/tools ile üretir, --remote-catalog ile indirme yolunu (publish_release.py local-catalog) kullanır; gerçek ortamın değişmediğini real_env_guard ile kanıtlar ve rapor yazar. Varsayılan rapor klasörü ekosistemdeki .build-cache/hub-native'dir (repo içine yazmaz).

**İlişkiler ve sınır:** release/tools/build_bundle.py, build_brain_dist.py, build_runtime.py, publish_release.py ve real_env_guard.py'yi çağırır.

### tests/orchestrator

#### [`tests/orchestrator/agents.test.ts`](../tests/orchestrator/agents.test.ts)

**Rol:** Ajanlar paneli testleri.

**Amaç / sorumluluk:** Ajanlar/modeller modelinin yalnızca gerçek worker ve ayar verisinden türetildiğini, eksik veriyle dürüstçe bozulduğunu doğrular.

**İlişkiler ve sınır:** src/products/orchestrator/views/agentsModel.ts'i sınar. node --test ile çalışır.

#### [`tests/orchestrator/campaigns.test.ts`](../tests/orchestrator/campaigns.test.ts)

**Rol:** Kampanya testleri.

**Amaç / sorumluluk:** Kampanya durumu özetinin (faz, şeritler, zaman çizelgesi, zincir) gerçek yanıtlardan doğru üretildiğini doğrular.

**İlişkiler ve sınır:** src/products/orchestrator/campaigns.ts'i sınar. node --test ile çalışır.

#### [`tests/orchestrator/evidence.test.ts`](../tests/orchestrator/evidence.test.ts)

**Rol:** Kanıt paneli testleri.

**Amaç / sorumluluk:** Engine inspect ve kampanya yanıtlarının sade kanıt, karar ve yama bilgisine doğru çevrildiğini doğrular.

**İlişkiler ve sınır:** src/products/orchestrator/views/evidenceModel.ts'i sınar. node --test ile çalışır.

#### [`tests/orchestrator/mcpAccess.test.ts`](../tests/orchestrator/mcpAccess.test.ts)

**Rol:** MCP erişim testleri.

**Amaç / sorumluluk:** MCP planından başlangıç anahtar durumu, yalnızca değişenlerin gönderilmesi, istemci etiketleri ve hata açıklamasını doğrular.

**İlişkiler ve sınır:** src/products/orchestrator/mcpAccess.ts'i sınar. node --test ile çalışır.

#### [`tests/orchestrator/overview.test.ts`](../tests/orchestrator/overview.test.ts)

**Rol:** Genel bakış testleri.

**Amaç / sorumluluk:** Koşu gruplama, sayımlar, son koşu seçimi ve eksik veri uyarılarını doğrular.

**İlişkiler ve sınır:** src/products/orchestrator/views/overviewModel.ts'i sınar. node --test ile çalışır.

#### [`tests/orchestrator/tasks.test.ts`](../tests/orchestrator/tasks.test.ts)

**Rol:** Form contract testleri.

**Amaç / sorumluluk:** Structured submission, required scope/write verification, read/custom argv ve pending/terminal ayrımını dört testle sınar.

**İlişkiler ve sınır:** Node test runner tasks.ts kullanır; native IPC kabulü yerine geçmez.

#### [`tests/orchestrator/workspace.test.ts`](../tests/orchestrator/workspace.test.ts)

**Rol:** Çalışma alanı testleri.

**Amaç / sorumluluk:** Orchestrator çalışma alanı sekme sırası ve bağlama kararlarının gerçek veriye dayandığını doğrular. secondbrain/memory bağlama beklentisi eklendi.

**İlişkiler ve sınır:** src/products/orchestrator/views/workspaceModel.ts'i sınar. node --test ile çalışır.

### tests/secondbrain

#### [`tests/secondbrain/brainClients.test.ts`](../tests/secondbrain/brainClients.test.ts)

**Rol:** İstemci seçimi testleri.

**Amaç / sorumluluk:** AI istemci listesi, seçimden desired nesnesine dönüşüm ve kayıtlı entegrasyonlardan seçim geri üretimini doğrular.

**İlişkiler ve sınır:** src/products/secondbrain/brainClients.ts'i sınar. node --test ile çalışır.

#### [`tests/secondbrain/memoryAccess.test.ts`](../tests/secondbrain/memoryAccess.test.ts)

**Rol:** Hafıza erişim testleri.

**Amaç / sorumluluk:** Hafıza erişim durumunun (desteklenmiyor, kapalı, açık, iptal bekliyor) açıklamalarını ve hata eşlemesini doğrular.

**İlişkiler ve sınır:** src/products/secondbrain/memoryAccess.ts'i sınar. node --test ile çalışır.

#### [`tests/secondbrain/memoryIndex.test.ts`](../tests/secondbrain/memoryIndex.test.ts)

**Rol:** Hafıza dizini testleri.

**Amaç / sorumluluk:** Dizin durumu ve yenileme sonucu açıklayıcılarını ve hata eşlemesini doğrular.

**İlişkiler ve sınır:** src/products/secondbrain/memoryIndex.ts'i sınar. node --test ile çalışır.

#### [`tests/secondbrain/memorySemantic.test.ts`](../tests/secondbrain/memorySemantic.test.ts)

**Rol:** Anlamsal arama testleri.

**Amaç / sorumluluk:** Anlamsal arama durumlarının başlık, ayrıntı ve ton açıklamalarını doğrular.

**İlişkiler ve sınır:** src/products/secondbrain/memorySemantic.ts'i sınar. node --test ile çalışır.

#### [`tests/secondbrain/memoryWorkspace.test.ts`](../tests/secondbrain/memoryWorkspace.test.ts)

**Rol:** Davranış/regresyon testi.

**Amaç / sorumluluk:** Bilinen not yolları, arama modu metninin anlamsal aramanın kullanılamadığını dürüstçe söylemesi ve hata metni biçimi.

**İlişkiler ve sınır:** src/products/secondbrain/memoryWorkspace.ts sınanır.

### tests/shared

#### [`tests/shared/productClient.test.ts`](../tests/shared/productClient.test.ts)

**Rol:** RPP istemci testleri.

**Amaç / sorumluluk:** Enjekte edilen taşıma ile rppCallVia'nın istek kurma ve yanıt ayrıştırmasını doğrular. Tauri dışında (düz Node) yerel IPC giriş noktalarının işi reddettiğini ve rppTry'ın erişilemeyen ürünü hata yerine null döndürdüğünü de sınar.

**İlişkiler ve sınır:** src/shared/productClient.ts ve rpp.ts'i sınar. node --test ile çalışır.

#### [`tests/shared/productsModel.test.ts`](../tests/shared/productsModel.test.ts)

**Rol:** Ürün modeli testleri.

**Amaç / sorumluluk:** Ürün özeti, ayar formu, değişen ayarlar, değer ayrıştırma ve geri okuma doğrulamasını (sır maskeleme dahil) sınar.

**İlişkiler ve sınır:** src/shared/productsModel.ts'i sınar. node --test ile çalışır.

#### [`tests/shared/rpp.test.ts`](../tests/shared/rpp.test.ts)

**Rol:** RPP sözleşme testleri.

**Amaç / sorumluluk:** RPP istek/yanıt kurma ve ayrıştırma, sürüm pazarlığı, operasyon kimliği zorunluluğu ve hata açıklamasını doğrular.

**İlişkiler ve sınır:** src/shared/rpp.ts'i sınar. node --test ile çalışır.

### tests/shell

#### [`tests/shell/appNav.test.ts`](../tests/shell/appNav.test.ts)

**Rol:** Gezinme testleri.

**Amaç / sorumluluk:** Hub gezinmesinin altı sayfayı doğru sırada ve etiketle sunduğunu ve Orchestrator çalışma alanı sayfalarının sırasını doğrular. SecondBrain sayfa listesine memory eklendi.

**İlişkiler ve sınır:** src/shell/apps.ts ve workspaceModel.ts'i sınar. node --test ile çalışır.

#### [`tests/shell/homeModel.test.ts`](../tests/shell/homeModel.test.ts)

**Rol:** Ana sayfa modeli testleri.

**Amaç / sorumluluk:** homeMode karar sırasını (önizleme, yükleniyor, ilk açılış, dikkat, kurulum eksik, aktif) doğrular.

**İlişkiler ve sınır:** src/shell/homeModel.ts'i sınar. node --test ile çalışır.

#### [`tests/shell/lifecycleModel.test.ts`](../tests/shell/lifecycleModel.test.ts)

**Rol:** Yaşam döngüsü testleri.

**Amaç / sorumluluk:** Yaşam döngüsü etiketleri, kullanılabilir eylemler, geri alma uygunluğu, hata ve günlük cümleleri ile operasyon kimliği üretimini doğrular. Güncellemenin geri alınamadığını sınar.

**İlişkiler ve sınır:** src/shell/lifecycle.ts'i sınar. node --test ile çalışır.

#### [`tests/shell/shellModel.test.ts`](../tests/shell/shellModel.test.ts)

**Rol:** Kabuk modeli testleri.

**Amaç / sorumluluk:** Üst çubuk sağlık hapının native yok, yükleniyor, hepsi hazır, bağlı değil ve karışık durumlardaki ton ve etiketini doğrular.

**İlişkiler ve sınır:** src/shell/shellModel.ts'i sınar. node --test ile çalışır.

### tools

#### [`tools/repository_map.py`](../tools/repository_map.py)

**Rol:** Atlas geliştirici aracı.

**Amaç / sorumluluk:** Bağımsız ürün adıyla metadata/hash denetimi ve no-Git filesystem keşfi sağlar.

**İlişkiler ve sınır:** Brain aracının bağımsız kopyasıdır; dış repo importu yoktur, normal Git CI modu korunur.

### Depo kökü

#### [`tsconfig.json`](../tsconfig.json)

**Rol:** TypeScript build sınırı.

**Amaç / sorumluluk:** Strict React JSX, DOM/ES modülleri ve noEmit typecheck ayarlarını sağlar.

**İlişkiler ve sınır:** npm build tsc ardından Vite çalıştırır.

#### [`vite.config.ts`](../vite.config.ts)

**Rol:** Frontend paket tarifi.

**Amaç / sorumluluk:** Vite yapılandırması: geliştirme sunucusunun dosya izlemesinde src-tauri/target ve src-tauri/gen altındaki Rust derleme çıktılarını yok sayar, böylece native derleme gereksiz yeniden yüklemeyi tetiklemez. Başka eklenti veya ayar içermez.

**İlişkiler ve sınır:** package.json build (tsc --noEmit && vite build) ve dev komutları tarafından kullanılır. Tauri geliştirme akışında frontend'i src-tauri ile birlikte sunar.

## Haritayı güncel tutma sözleşmesi

Yeni dosya keşfi otomatiktir; yeni kodun doğru anlamsal açıklaması otomatik olarak bilinemeyeceği için insan/ajan incelemesi gerekir. `--update` yeni kayıtları `NEEDS_REVIEW` olarak ekler ve indeksten silinenleri çıkarır. Dosya değişikliği açıklamanın artık doğru olduğuna dair yeniden inceleme gerektirir; LF/CRLF dönüşümü değişiklik sayılmaz.

```sh
python tools/repository_map.py --update
# JSON içindeki NEEDS_REVIEW alanlarını gerçek rol/amaç/ilişkilerle doldur.
python tools/repository_map.py --accept-reviewed path/to/changed.py
python tools/repository_map.py --write
python tools/repository_map.py --check
python -m unittest tests.repository_map_test -v
```

`--accept-reviewed` yalnız açıklaması incelenmiş açık bir yolun içerik hash'ini günceller; içerik açıklaması üretmez. Aynı komutta birden fazla seçenek verilebilir. `--write` keşfedilmiş eksikleri görünür biçimde üretebilir ama `--check` her `NEEDS_REVIEW` kaydını reddeder. CI kapısı eksik/fazla/çift kayıtları, kayıp dosyaları, eski açıklama hash'lerini ve Markdown üretim farkını hata kodu 1 ile durdurur. Staged deletion Git indeksinden kalktığı için envanterden çıkar; yalnız diskten silinmiş izlenen dosya hata olarak kalır.

JSON envanteri ve üretilmiş Markdown kendi kendini hash'leme döngüsünü önlemek için `generated` damgası taşır. JSON açıklamalarındaki değişiklik yine Markdown üretim karşılaştırmasıyla yakalanır. Diğer dosyalar SHA256 inceleme damgasıyla bağlıdır. Hash, açıklamanın doğru olduğunu matematiksel olarak kanıtlamaz; açıklamayı yazan kişi kodu ve ilişkileri okumakla sorumludur.

Her dosya ekleme/silme/yeniden adlandırma ve davranış değişikliğinde bu güncelleme aynı iş içinde yapılır. CI çalışması dosyaya yazmaz. `--root` yalnız farklı bir kaynak checkout'u denetlemek içindir; kurulu AppRoot, DataRoot veya kişisel vault üzerinde bu aracı çalıştırmayın.


`--filesystem-only` Git çalıştırmaz; kök ignore kurallarını ve gözden geçirilmiş kaynak yollarını kullanır. Eksik bilinen dosya check hatası olarak korunur. Staged durum ve iç içe ignore sözleşmelerinin Git ile eşdeğerliği bu modun kapsamı değildir; CI normal Git modunu kullanır.
