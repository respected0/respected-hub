# Hub: gelecekte yapılacaklar

Ürün beklentisiyle bugünkü durum arasındaki açıklar. Yapılan madde silinir.
## Beta.1'den sonraki sürüm

Beta.1'den sonraki ilk sürüm **yeni özellik eklemez**: yalnızca doğrulama eksiklerini kapatır, hataları düzeltir ve var olan
özellikleri iyileştirir (bir özelliği başka bir özelliğe çevirmeden, mevcut olanı düzenleyerek). Üç ürün için geçerlidir.
- [ ] Doğrulama eksikleri: gerçek eski sürümden (beta.1) güncelleme CI işi, Hub'ın kendini güncellemesi, Brain'de Gemini CLI
  ve Cursor, Orchestrator'da Gemini ve Cursor token kullanımı, güncellemede sağlık dalıyla geri alma, temiz makinede ilk kurulum.
- [ ] Hata düzeltmeleri: beta.1 sonrası bulunanlar.
- [ ] Var olanı iyileştirme: örneğin Hub'sız kullanıcıya sürüm bildirimi.
## Dış eklenti ve MCP keşfi
- [x] Keşfet sayfasında GitHub repoları, eklentiler ve MCP sunucularını listelemek.
  - Kanıt: `src/shell/views/ThirdPartyCatalog.tsx`, `src/shell/thirdPartyCatalog.ts`, `integrations/catalog.rs`; 62 kayıtlık derlenmiş katalog ve `catalog_e2e`. Salt okunur, GitHub bağlantılı.
- [ ] Bunları Hub'dan kurmak (örnek: ponytail) ve kaldırmak; ürünlerimizdeki gibi yalnızca kendi girdisini yazıp silerek.
- [x] "Hangi AI aracında kurulu" görünümü: her MCP/eklentinin Claude Code, Codex, Cursor, Gemini… ayarlarında olup
  olmadığını göstermek.
  - Kanıt: istemci × kapsam matrisi (`IntegrationsLibrary.tsx`, `integrationMatrix.ts`), proje görünümü (`Projects.tsx`) ve beş istemcili `resolver_e2e`.
- [x] Dış eklentiler ve MCP sunucuları için imzalı katalog kanalı ve doğrulama modeli.
  - Kanıt: `SEC-002` alan ayrımlı Ed25519 imza/sequence doğrulaması, `CAT-003` isteğe bağlı tazeleme ve çevrimdışı son doğrulanmış katalog. Yayımlanmış bir dış katalog içeriği ve Hub'dan kurma yoktur.
## Yayın
- [ ] Temiz bir makinede ilk kurulum henüz denenmedi (`tests/native/hub_native_e2e.mjs` kurulu ürün olan makinede başlamaz).
  Hub'ın katalogdan kendini güncellemesi (Ayarlar > Güncellemeler > Hub'ı güncelle) ilk sürümde denenemez, çünkü güncellenecek daha yeni sürüm yok; bir sonraki sürüm yayınlanınca gerçek yeni sürümle denenecek.
- [ ] Kurucuyu Authenticode ile imzalamak (SmartScreen uyarısı). Kod imzalama sertifikası gerekir; yok.
  Yayımlanan `v0.1.0-beta.1` kurucusu 2026-11-05'te `NotSigned` olarak doğrulandı.
- [ ] macOS / Linux: doğrulanmadı. Bu geliştirme makinesi Windows.
- [ ] Güncelleme yolu karara bağlandı: her sürümden en yeniye doğrudan (sözleşme §4: yedekli güncelleme, silinmeyen `STEPS`,
  sürüm başına gerçek veri fixture'ları). Kalan: yayınlanmış eski kurucuları gerçek Windows'ta yenisinin üzerine kuran CI işi
  (beta.2 yayınlanınca beta.1 ile başlar). Zincir gerçekten uzarsa `min_from` ile basamak sürüm istenir.
- [ ] Hub'sız kullanıcı yeni sürümden haberdar olmuyor: ürünlerde sürüm denetimi yok.
- [x] Aynı sürüm numarasıyla düzeltilmiş yayın: `catalog` aynı ürün/sürüm için farklı içeriği reddeder; düzeltme yeni numarayla çıkar.

## Genişletilmiş ürün vizyonu — entegrasyon kataloğu ve yerel yönetici

> **Durum (2026-11-05):** M1–M5 kaynak kodu ve testleri uygulandı; M0 yayın engelleri açıktır.
> Her işaretli maddede kısa kanıt verilir. Ayrıntı: [`IMPLEMENTATION_ROADMAP.md`](IMPLEMENTATION_ROADMAP.md),
> [`docs/QUALITY.md`](docs/QUALITY.md), [`docs/ADR-THIRD-PARTY-INTEGRATIONS.md`](docs/ADR-THIRD-PARTY-INTEGRATIONS.md).
> M3–M5 motoru hazırdır; üretim yazma IPC'si kullanıcı onayı ve sözleşme kapısı kapalı olduğu için açılmaz.

### M0 — Önce mevcut ürünün yayın kalitesi (yeni özellik yok)
- [ ] `REL-001` Eski sürümden gerçek güncelleme ve geri alma CI matrisi.
  - Engel: yayımlanmış tek sürüm ve geliştirme sürümü de `0.1.0-beta.1`; gerçek eski→yeni güncelleme için yeni sürüm gerekir.
- [ ] `REL-002` Temiz Windows VM kurulumu ve Hub'ın gerçek kendini güncelleme testi.
  - Engel: temiz Windows VM yok; yayımlanan kurucu Authenticode imzasız ve kendini güncelleme için daha yeni sürüm yok.
- [ ] `REL-003` Güncelleme bildirimi, çalışan görev uyarısı, aynı sürüm farklı digest ve yayın hataları.
  - [x] Çalışan Orchestrator görevi uyarısı (`8276c56`), aynı sürüm farklı digest (`4d72a6c`), çevrimdışı onarım ve bildirim testleri (`1716894`).
  - [ ] Hub'sız kurulan ürünlerin kendi sürüm bildirimi: ürün depolarında; bu depoda yapılamaz.

### M1 — Envanter motorunun temel sözleşmesi
- [x] `CORE-001` Brain/Orchestrator sahipliği ve üçüncü taraf konfigürasyon yazma sınırı için ADR; kod ve belge uyumu.
  - Kanıt: `ADR-THIRD-PARTY-INTEGRATIONS.md`; planner ve apply `managed`, `Owner::Product`, `Owner::Catalog` için `OWNER_PROTECTED`.
- [x] `CORE-002` `Asset`, `Client`, `Source`, `Binding`, `Scope`, `Capability`, `ObservedStatus` veri modeli.
  - Kanıt: `src-tauri/src/integrations/model.rs`; yol kimliği ve serde round-trip testleri.
- [x] `CORE-003` Sahte Windows kullanıcı kökü, config ve symlink fixture'ları, native E2E koruması.
  - Kanıt: `inventory_e2e` (3 test), `RESPECTED_HUB_TEST_HOME` ile native E2E; gerçek AI ayarları karşılaştırması.
- [x] `CORE-004` Rust `integrations` modülü ve tipli/allowlist'li Tauri IPC; genel shell/CLI yok.
  - Kanıt: `commands/integrations.rs`, `lib.rs`; kapalı eylem kümesi `api.rs` ve `only_closed_actions_are_accepted` testi.
- [x] `SEC-001` Kaynak sahipliği, path sınırları, sır redaksiyonu ve güvenlik tehdit modeli.
  - Kanıt: `security.rs` redaksiyon/yol/symlink/boyut testleri, adapter E2E sır sızıntısı kontrolleri, ADR tehdit modeli.

### M2 — Local Discovery MVP: offline katalog + salt okunur kurulum envanteri
- [x] `DISC-001` Var olan `detect_ai_clients` işlevini kanıta dayalı istemci keşfine genişlet.
  - Kanıt: `integrations/discovery.rs`; klasör varlığı tek başına `enabled` sayılmaz, okunamayan yol kontrollü uyarıdır.
- [x] `DISC-002` Claude Code MCP/Skill/Plugin konfigürasyonlarını salt okunur tespit et.
  - Kanıt: `adapters/claude.rs`, `claude_adapter_e2e` 5 test.
- [x] `DISC-003` Codex MCP/Skill konfigürasyonlarını salt okunur tespit et.
  - Kanıt: `adapters/codex.rs`, `codex_adapter_e2e` 4 test.
- [x] `DISC-004` Cursor, Gemini CLI ve OpenCode adapter'larını ekle.
  - Kanıt: `adapters/{cursor,gemini,opencode}.rs`, `cursor_gemini_opencode_adapter_e2e` 5 test.
- [x] `DISC-005` Skills/symlink envanteri, managed/unmanaged kaynak ayrımı.
  - Kanıt: `integrations/skills.rs`, `skills_e2e` 3 test.
- [x] `DISC-006` Plugin ve alt bileşen (MCP/Skill/Hook/Command) ilişkileri.
  - Kanıt: `integrations/plugins.rs`, `plugins_e2e` 4 test.
- [x] `DISC-007` Araç bazlı etkin kapsam ve çakışma çözümleyicisi.
  - Kanıt: `integrations/resolver.rs`, `resolver_e2e`; beş istemci × altı kapsam.
- [x] `CAT-001` Ürün kurucusu kataloğundan ayrı, uygulamayla gelen yerel üçüncü taraf katalog.
  - Kanıt: derlenmiş `src-tauri/assets/third-party-catalog.json` (62 kayıt), `integrations/catalog.rs`, `catalog_e2e` 6 test.
- [x] `CAT-002` Keşfet'te arama, kategori/istemci filtresi ve GitHub sayfası bağlantısı.
  - Kanıt: `ThirdPartyCatalog.tsx`, `thirdPartyCatalog.ts`; 4 arayüz filtre/arama testi.
- [x] `UI-001` Kütüphanem'de MCP/Skills/Plugin × AI aracı × kapsam matrisi.
  - Kanıt: `IntegrationsLibrary.tsx`, `integrationMatrix.ts`; matris/filtre testleri.
- [x] `UI-002` Kullanıcının seçtiği projeler için salt okunur global/proje görünümü.
  - Kanıt: `Projects.tsx`, `projectScope.ts`, `integrations/projects.rs`; global/proje gölgelenmesi ve eksik kök testleri.

### M3 — Safe Control Engine: aç/kapat, proje/global, güvenli geri alma
- [x] `OPS-001` Mutasyonsuz işlem planı, maskeleme ve değişiklik önizlemesi.
- [x] `OPS-002` Sahipliği/biçimi koruyan, hash kontrollü atomik yazıcı ve readback.
- [x] `OPS-003` Yalnızca desteklenen client capability'leriyle etkinleştir/devre dışı bırak.
- [x] `OPS-004` Global ↔ proje taşıma/kopyalama, diğer projelere etki analizi.
- [x] `OPS-005` Transaction journal, undo ve çökme sonrası güvenli kurtarma.

### M4 — Güvenli harici katalog ve kurulum yaşam döngüsü
- [x] `SEC-002` Üçüncü taraf katalog için ayrı alan ayrımlı imza/sequence doğrulaması.
- [x] `CAT-003` İsteğe bağlı imzalı katalog tazeleme, internet yokken son doğrulanmış katalog.
- [x] `SEC-003` Kaynak/komut/izin risk önizlemesi; güvenliği garanti eden etiket kullanma.
- [x] `OPS-006` Kullanıcı onaylı var olan entegrasyonu içe alma/bağlama.
- [x] `OPS-007` Kaynağın sahibini koruyan kaldırma/ayırma ve güncelleme.
- [x] `OPS-008` Birden çok AI aracına toplu uygulama ve kısmi hata kurtarma.

### M5 — Farklılaştırıcı özellikler ve platform desteği
- [x] `ADV-001` Proje profilleri ve idempotent toplu ayar şablonları.
- [x] `ADV-002` Drift, gölgelenme, bozuk symlink ve kopya çakışması analizi.
- [x] `PORT-001` Gerçek CI/VM üzerinde doğrulanmış macOS ve Linux yetenek matrisi.

### Bütün aşamalarda kalite kapısı
- [x] `QA-001` Her PR için `npm run check`, `cargo test`, ilgili native E2E, gerçek kullanıcı ayarlarını koruma ve belgelerin güncellenmesi.

**MVP sınırı:** M2 bitene kadar üçüncü taraf MCP/Skill/Plugin kurma veya AI ayarlarına yazma yok.
**M3 güvenlik kapısı:** `AGENTS.md`, `ARCHITECTURE.md`, `FOOTPRINT.md`, `PRIVACY.md` ve gerekiyorsa ürün sözleşmesi, açık onaylı üçüncü taraf yönetim kapsamı için güncellenmeden yazma komutları üretimde açılmaz.
