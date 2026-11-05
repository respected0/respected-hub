# Hub: gelecekte yapılacaklar

Ürün beklentisiyle bugünkü durum arasındaki açıklar. Yapılan madde silinir.
## Beta.1'den sonraki sürüm

Beta.1'den sonraki ilk sürüm **yeni özellik eklemez**: yalnızca doğrulama eksiklerini kapatır, hataları düzeltir ve var olan
özellikleri iyileştirir (bir özelliği başka bir özelliğe çevirmeden, mevcut olanı düzenleyerek). Üç ürün için geçerlidir.
- [ ] Doğrulama eksikleri: gerçek eski sürümden (beta.1) güncelleme CI işi, Hub'ın kendini güncellemesi, Brain'de Gemini CLI
  ve Cursor, Orchestrator'da Gemini ve Cursor token kullanımı, güncellemede sağlık dalıyla geri alma, temiz makinede ilk kurulum.
- [ ] Hata düzeltmeleri: beta.1 sonrası bulunanlar.
- [ ] Var olanı iyileştirme: örneğin çalışan Orchestrator görevi varken güncelleme uyarısı, Hub'sız kullanıcıya sürüm
  bildirimi, aynı sürüm numarasıyla düzeltilmiş yayın.
## Dış eklenti ve MCP keşfi
- [ ] Keşfet sayfasında GitHub repoları, eklentiler ve MCP sunucularını listelemek (bugün yalnızca Brain ve
  Orchestrator; `src/shell/views/Discover.tsx`).
- [ ] Bunları Hub'dan kurmak (örnek: ponytail) ve kaldırmak; ürünlerimizdeki gibi yalnızca kendi girdisini yazıp silerek.
- [ ] "Hangi AI aracında kurulu" görünümü: her MCP/eklentinin Claude Code, Codex, Cursor, Gemini… ayarlarında olup
  olmadığını göstermek.
- [ ] Dış eklentiler ve MCP sunucuları için de imzalı katalog: ürünlerimizin kataloğu GitHub'dan imzalı gelir (Hub imzayı
  ve dosya özetlerini doğrular); dış eklentiler için böyle bir katalog ve güven modeli henüz yok.
## Yayın
- [ ] Temiz bir makinede ilk kurulum henüz denenmedi (`tests/native/hub_native_e2e.mjs` kurulu ürün olan makinede başlamaz).
  Hub'ın katalogdan kendini güncellemesi (Ayarlar > Güncellemeler > Hub'ı güncelle) ilk sürümde denenemez, çünkü güncellenecek daha yeni sürüm yok; bir sonraki sürüm yayınlanınca gerçek yeni sürümle denenecek.
- [ ] Kurucuyu Authenticode ile imzalamak (SmartScreen uyarısı). Kod imzalama sertifikası gerekir; yok.
- [ ] macOS / Linux: doğrulanmadı. Bu geliştirme makinesi Windows.
- [ ] Güncelleme yolu karara bağlandı: her sürümden en yeniye doğrudan (sözleşme §4: yedekli güncelleme, silinmeyen `STEPS`,
  sürüm başına gerçek veri fixture'ları). Kalan: yayınlanmış eski kurucuları gerçek Windows'ta yenisinin üzerine kuran CI işi
  (beta.2 yayınlanınca beta.1 ile başlar). Zincir gerçekten uzarsa `min_from` ile basamak sürüm istenir.
- [ ] Hub'sız kullanıcı yeni sürümden haberdar olmuyor: ürünlerde sürüm denetimi yok.
- [ ] Aynı sürüm numarasıyla düzeltilmiş yayın: Hub önbellekteki eski kurucuyla onarabilir ve "Güncelleme var" göstermez;
  kurmuş olanlar düzeltmeyi kurucuyu elle çalıştırarak alır.

## Genişletilmiş ürün vizyonu — entegrasyon kataloğu ve yerel yönetici

> **Bu bölüm beta.1 sonrası ilk stabilizasyon sürümünde uygulanmaz.** Detaylar, mimari değişiklik,
> bağımlılık, test ve ölçülebilir kabul kriterleri: [`IMPLEMENTATION_ROADMAP.md`](IMPLEMENTATION_ROADMAP.md).
> Buradaki görevler öneridir ve henüz gerçekleştirilmemiştir.

### M0 — Önce mevcut ürünün yayın kalitesi (yeni özellik yok)
- [ ] `REL-001` Eski sürümden gerçek güncelleme ve geri alma CI matrisi.
- [ ] `REL-002` Temiz Windows VM kurulumu ve Hub'ın gerçek kendini güncelleme testi.
- [ ] `REL-003` Güncelleme bildirimi, çalışan görev uyarısı, aynı sürüm farklı digest ve yayın hataları.

### M1 — Envanter motorunun temel sözleşmesi
- [ ] `CORE-001` Brain/Orchestrator sahipliği ve üçüncü taraf konfigürasyon yazma sınırı için ADR; kod ve belge uyumu.
- [ ] `CORE-002` `Asset`, `Client`, `Source`, `Binding`, `Scope`, `Capability`, `ObservedStatus` veri modeli.
- [ ] `CORE-003` Sahte Windows kullanıcı kökü, config ve symlink fixture'ları, native E2E koruması.
- [ ] `CORE-004` Rust `integrations` modülü ve tipli/allowlist'li Tauri IPC; genel shell/CLI yok.
- [ ] `SEC-001` Kaynak sahipliği, path sınırları, sır redaksiyonu ve güvenlik tehdit modeli.

### M2 — Local Discovery MVP: offline katalog + salt okunur kurulum envanteri
- [ ] `DISC-001` Var olan `detect_ai_clients` işlevini kanıta dayalı istemci keşfine genişlet.
- [ ] `DISC-002` Claude Code MCP/Skill/Plugin konfigürasyonlarını salt okunur tespit et.
- [ ] `DISC-003` Codex MCP/Skill konfigürasyonlarını salt okunur tespit et.
- [ ] `DISC-004` Cursor, Gemini CLI ve OpenCode adapter'larını ekle.
- [ ] `DISC-005` Skills/symlink envanteri, managed/unmanaged kaynak ayrımı.
- [ ] `DISC-006` Plugin ve alt bileşen (MCP/Skill/Hook/Command) ilişkileri.
- [ ] `DISC-007` Araç bazlı etkin kapsam ve çakışma çözümleyicisi.
- [ ] `CAT-001` Ürün kurucusu kataloğundan ayrı, uygulamayla gelen yerel üçüncü taraf katalog.
- [ ] `CAT-002` Keşfet'te arama, kategori/istemci filtresi ve GitHub sayfası bağlantısı.
- [ ] `UI-001` Kütüphanem'de MCP/Skills/Plugin × AI aracı × kapsam matrisi.
- [ ] `UI-002` Kullanıcının seçtiği projeler için salt okunur global/proje görünümü.

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
- [ ] `OPS-008` Birden çok AI aracına toplu uygulama ve kısmi hata kurtarma.

### M5 — Farklılaştırıcı özellikler ve platform desteği
- [ ] `ADV-001` Proje profilleri ve idempotent toplu ayar şablonları.
- [ ] `ADV-002` Drift, gölgelenme, bozuk symlink ve kopya çakışması analizi.
- [ ] `PORT-001` Gerçek CI/VM üzerinde doğrulanmış macOS ve Linux yetenek matrisi.

### Bütün aşamalarda kalite kapısı
- [ ] `QA-001` Her PR için `npm run check`, `cargo test`, ilgili native E2E, gerçek kullanıcı ayarlarını koruma ve belgelerin güncellenmesi.

**MVP sınırı:** M2 bitene kadar üçüncü taraf MCP/Skill/Plugin kurma veya AI ayarlarına yazma yok.
**M3 güvenlik kapısı:** `AGENTS.md`, `ARCHITECTURE.md`, `FOOTPRINT.md`, `PRIVACY.md` ve gerekiyorsa ürün sözleşmesi, açık onaylı üçüncü taraf yönetim kapsamı için güncellenmeden yazma komutları üretimde açılmaz.
