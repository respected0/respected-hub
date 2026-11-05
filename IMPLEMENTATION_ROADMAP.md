# Respected Hub — Uygulanabilir Geliştirme Yol Haritası

> **Durum (2026-11-05):** M1–M5 kaynak kodu ve testleri tamam; üretim yazma IPC'si bilinçli olarak kapalı. M0 yayın engelleri yeni sürüm artefaktı/temiz VM bekliyor. Authenticode imzası ile macOS/Linux masaüstü GUI/paketleme doğrulaması v0.1.0 resmi sürüm kapsamındadır; beta.1 sonrası sürümün kabul kapısı değildir. **Tarih:** 2026-11-03, güncelleme 2026-11-05. **Hedef:** Local-first, hesapsız, Steam-benzeri MCP/Skills/Plugin kataloğu + kurulu entegrasyon envanteri + AI aracı/proje/global kapsamında güvenli yönetim.
>
> **Kaynaklar:** `docs/ROADMAP.md`, `docs/ARCHITECTURE.md`, `docs/PRODUCT_CONTRACT.md`, `docs/FOOTPRINT.md`, `AGENTS.md`, `PRIVACY.md`, `src-tauri/src/commands/manage.rs`, `src-tauri/src/lib.rs`, `src/shell/views/{Discover,Library}.tsx`, `src/shell/apps.ts`, `tests/native/hub_native_e2e.mjs` (respected0/respected-hub, main, 2026-11-03; durum 2026-11-05).
>
> **Önemli:** Aşağıdaki kartlar hâlâ sözleşmedir. `önerilen` etiketli yollar artık büyük ölçüde mevcuttur; kabul kanıtı `ROADMAP.md` ve `docs/QUALITY.md` içinde işaretli maddelerle izlenir. Üretim yazma IPC'si sözleşme kapısı kapalı olduğu için açılmaz. Kalan doğrulama işleri `ROADMAP.md` içindeki üç sınıfa göre izlenir.

## 0. Karar kayıtları ve kırmızı çizgiler

1. **Beta.1 sonrası ilk sürümde yeni özellik yok.** Mevcut ROADMAP bu sürümü doğrulama/hata düzeltme/iyileştirmeye ayırıyor. M1 ve sonrası, o sürümün kabulünden sonra açılır. Yeni sürüm numarası önceden varsayılmaz.
2. **Ürün sahipliği korunur.** Brain/Orchestrator'ın kurulumları ve kendi MCP kayıtlarının sahibi o ürünlerdir. Hub mevcut ürün işlemlerinde RPP/ürün kurucularını çağırmaya devam eder. Üçüncü taraf entegrasyon yönetimi ayrı, açıkça izin verilmiş ve kullanıcı onaylı `integrations` motorunda tanımlanır. Brain/Orchestrator kayıtlarını bu motor doğrudan yeniden yazmaz; varsa ürünün kendi arayüzü/komutu kullanılır.
3. **Politika uyumu önkoşulu.** `AGENTS.md` ve `FOOTPRINT.md` şu an Hub'ın AI yapılandırmalarını yazmamasını şart koşuyor. M3 öncesi ürün sahibinin onayıyla üçüncü taraf ve kullanıcı onaylı işlemlere özel istisna / yeni sözleşme açık biçimde yazılır; `ARCHITECTURE.md`, `PRODUCT_CONTRACT.md` (yalnızca gerekli ve üç repo ile uyumlu değişiklik), `FOOTPRINT.md`, `PRIVACY.md` aynı PR'da güncellenir. Belgeler güncellenmeden üretimde yazma komutları açılmaz. Geliştirme/test makinesindeki gerçek AI ayarlarına hiçbir otomatik test yazmaz.
4. **Kaynak doğruluğu:** Gerçek AI yapılandırması, dosya sistemi ve native istemci kuralı esas kaynaktır. Hub cache'i yalnızca yeniden oluşturulabilir snapshot, işlem makbuzu ve geri alma yedeği saklar. Respected ürünlerinin ayarlarını Hub'da kopyalamayız.
5. **Yüklü ≠ yapılandırılmış ≠ etkin ≠ bağlantısı doğrulandı ≠ kullanıldığı kanıtlandı.** Durum modeli bu beş seviyeyi ayırır; hiçbir algılama yalnızca klasörün varlığına dayanarak MCP etkinliğini iddia etmez.
6. **Yerel ve hesapsız başlangıç:** Telemetri, login, server, daemon, gateway, genel CLI veya üçüncü taraf kodu otomatik çalıştırma yok. Katalog internetsiz açılır; isteğe bağlı bağlantı/indirme daha sonra eklenir. Üretim WebView'e keyfî komut/ağ erişimi açılmaz.
7. **Platform sırası:** Windows birincil hedef. Mac/Linux ancak gerçek platform kabul testleri bulunduğunda destekli olarak işaretlenir.

## 1. Mevcut ROADMAP eşleştirmesi

| ROADMAP başlığı / mevcut satır | Yeni ID'ler | Teslim kapsamı |
|---|---|---|
| Beta.1'den sonraki sürüm: doğrulama açıkları | REL-001, REL-002, REL-003 | Önce stabilizasyon; yeni özellik yok |
| Dış eklenti ve MCP keşfi: Keşfet'te listeleme | CAT-001, CAT-002, CAT-003, UI-001 | Yerel katalog, sonra doğrulanmış güncelleme |
| Dış eklenti ve MCP keşfi: kurma/kaldırma | OPS-006, OPS-007, OPS-008, SEC-003 | İsteğe bağlı güvenli kurulum ve kaldırma |
| Dış eklenti ve MCP keşfi: hangi AI'da kurulu | CORE-002, DISC-001..007, UI-001..002 | Gerçek kurulum/kapsam matrisi |
| Dış eklenti ve MCP keşfi: imzalı katalog | SEC-001..003, CAT-003 | Ayrı güven modeli; imza güvenlik denetimi değildir |
| Yayın: temiz makine/kurucu/geri alma/sürüm bildirimi | REL-001..003, QA-001 | Testli sürüm kapıları |
| Yayın: macOS/Linux doğrulanmadı | PORT-001 | Windows sonrası platform genişlemesi |
| ROADMAP'te henüz ayrıntısı olmayan ihtiyaçlar | CORE-001..004, OPS-001..005, ADV-001..002 | Envanter çekirdeği, güvenli uygulama, profil/çakışma |

## 2. Mimari hedef

```text
React UI (src/shell/views/*)
  ├─ Keşfet: katalog ve GitHub bağlantıları
  ├─ Kütüphane: asset × client × scope matrisi
  └─ Projeler: kullanıcı seçimi, etkin kapsam
      │ dar, tipli Tauri IPC
      ▼
Rust src-tauri/src/integrations/                  [YENİ]
  ├─ model.rs          Asset, Client, Source, Binding, Scope, Capability, Status
  ├─ discovery.rs      tarama orkestrasyonu, bounded traversal
  ├─ adapters/         claude.rs / codex.rs / cursor.rs / gemini.rs / opencode.rs
  ├─ resolver.rs       etkin kayıt, öncelik, gölgelenme, çakışma
  ├─ inventory.rs      yeniden oluşturulabilir snapshot + provenance
  ├─ catalog.rs        uygulama içi statik üçüncü taraf kayıtları
  ├─ planner.rs        mutasyonsuz değişiklik planı, hash/ön koşullar
  ├─ apply.rs          izinli ve atomik yazıcı + doğrulama
  ├─ undo.rs           geri alma kayıtları, kurtarma
  └─ security.rs       path policy, secret redaction, source risk

Mevcut src-tauri/src/lifecycle/* + hub/* + rpp.rs  [DEĞİŞMEDEN KORU]
  └─ Yalnızca Brain ve Orchestrator ürün yaşam döngüsü
```

**Yeni API komutları (öneri):** `integrations_detect_clients`, `integrations_inventory`, `integrations_catalog_list`, `integrations_plan`, `integrations_apply`, `integrations_undo`, `integrations_operation_status`. `integrations_apply` sadece onaylanan plan kimliği/digest ve tek kullanımlık transaction ID ile çalışır; tarayıcıdan rastgele dosya yolu, kabuk komutu veya executable parametreleri kabul edilmez. Plan her zaman taze dosya hash'leriyle yeniden doğrulanır. Asıl dosyalara uygulanan yazma politikası varsayılan olarak kapalıdır ve yalnızca ilgili capability açık olduğunda devreye girer.

**Temel ilişkiler:**
- `Asset`: tür (`mcp|skill|plugin|rule|other`), kaynak kimliği, sürüm, parent/child bileşenler.
- `Client`: kurulu/varlık kanıtı, sürüm (varsa), platform, adapter yetenekleri.
- `Source`: canonical path ya da yapılandırma kaydı; biçim, hash, sahibi (`respected-product|hub-managed|third-party|unknown`), readonly sebebi.
- `Binding`: `asset_id + client_id + scope(global|project) + project_root + source_id + target_ref`; durum ve kanıt.
- `ObservedStatus`: `found|configured|enabled|connection-validated|usage-observed|unknown|conflict|error` ayrı bayrak/kanıtlarla ifade edilir; tek sıralı enum yanlış çıkarımlara yol açabilir.
- `OperationPlan`: `plan_id`, kaynak hash'leri, hedef diff, beklenen sonuç, riskler, geri alma metadata'sı, son kullanma süresi.

**Veri konumu (öneri):** `%LOCALAPPDATA%\Respected\RespectedHub\managed\integrations\` içinde yalnızca cache, kullanıcı projesi seçimleri, plan geçmişi ve transaction yedekleri. `managed\catalog\` altındaki mevcut imzalı ürün kurucu kataloğuna karıştırılmaz; ayrı `third-party-catalog` alanı açılır. Kalıcı snapshot `schema_version` içerir, bozuksa güvenle yeniden oluşturulur.

## 3. Bağımlılık grafiği ve sürüm kapıları

```text
REL-001/002/003 ──(stabil sürüm kabulü)──> CORE-001
CORE-001 ──> CORE-002 ──> DISC-001 ──> DISC-002/003/004 ──> DISC-007
           └─> CORE-003 ────────────────────────┘
           └─> CORE-004 ──> UI-001/002
SEC-001 ────────────────> DISC-005/006, OPS-001
CAT-001 ──> CAT-002 ──> UI-001
DISC-002/003 + DISC-007 ──> UI-001; DISC-004/005/006 tamamlandıkça UI genişler
UI-001/002 + CORE-004 + SEC-001 ──> OPS-001 ──> OPS-002 ──> OPS-003/004 ──> OPS-005
OPS-005 + SEC-002 + SEC-003 + CAT-003 ──> OPS-006/007/008
OPS-003/004/005 ──> ADV-001/002
QA-001 her aşamada; PORT-001 Windows kapsamının kararlılığından sonra.
```

## 4. Görev kartları

### M0 — Mevcut beta stabilizasyonu (yeni özellik YOK)

#### REL-001 — Gerçek sürüm yükseltme ve geri dönüş matrisi [P0]
- **Mimari / dosyalar:** Mevcut `common/release/`, `tests/fixtures/releases/`, `.github/workflows/` ve kurucu testlerini güçlendir; ürün yaşam döngüsünün sözleşmesini değiştirme.
- **Bağımlılıklar:** Beta.1 kurucu artefaktları ve sonraki doğrulanabilir test sürümü.
- **Test:** Beta.1 → yeni sürüm gerçek Windows CI; sağlıklı→sağlıklı, sağlıklı→bozuk, başarısız `repair` sonrası rollback, iki kez onarım; gerçek eski veri fixture'ları.
- **Kabul:** Her eski sürüm fixture'ı başarıyla taşınır; başarısız güncellemede önceki uygulama/veri hash'leri korunur; CI'da gerçek yükseltme kanıtı kayıtlıdır; yanlış sürüm/paket reddedilir.

#### REL-002 — Temiz Windows ilk kurulum ve Hub güncellemesi [P0]
- **Mimari / dosyalar:** `tests/native/hub_native_e2e.mjs` ve Windows release CI kapsamını genişlet; mevcut fake-home/real-env guard'ını koru.
- **Bağımlılıklar:** REL-001, beta.2+ doğrulanabilir yayın artefaktı.
- **Test:** Temiz VM'de ilk kurulum, Brain ve Orchestrator ilk yükleme, Hub kendi güncellemesi, kullanım sonrası kaldırma; SmartScreen/kurucu imzası durumu ayrı raporlanır.
- **Kabul:** 0 gerçek kişisel AI ayarı değişikliği; temiz VM'de tüm kritik kurulum senaryoları geçer; eksik artefakt nedeniyle test atlanırsa yayın kapısı geçilmiş sayılmaz.

#### REL-003 — Mevcut sürüm hataları ve yayın hijyeni [P0]
- **Mimari / dosyalar:** Mevcut güncelleme bildirimi, açık görev sırasında güncelleme uyarısı, aynı sürüm numarasıyla değişmiş paket senaryosu; imzalı artefakt politikası.
- **Bağımlılıklar:** REL-001.
- **Test:** Çalışan Orchestrator görevi, Hub'sız ürün kullanıcısı, aynı SemVer farklı digest, offline onarım.
- **Kabul:** Her senaryoda deterministik kullanıcı bildirimi; bozuk/güvensiz paket sessizce kabul edilmez; mevcut `npm run check`, `cargo test`, native E2E geçer.

### M1 — Envanter çekirdeği ve güvenli okuma altyapısı

#### CORE-001 — Ürün sınırları ve karar kaydı (ADR) [P0]
- **Mimari / dosyalar:** Yeni `docs/ADR-THIRD-PARTY-INTEGRATIONS.md`; `AGENTS.md`, `ARCHITECTURE.md`, `PRODUCT_CONTRACT.md`, `FOOTPRINT.md`, `PRIVACY.md` için sürüm bazlı etki planı. Brain/Orchestrator ile üçüncü taraf konfigürasyon sahipliği ayrılır.
- **Bağımlılıklar:** REL-001..003; üçüncü taraf yazmayı açmadan önce açık kullanıcı/maintainer kararı.
- **Test:** Kural kontrolü ve test fixture'larında Respected ürününe ait MCP girdisinin üçüncü taraf motorla değiştirilememesi; dokümantasyon gözden geçirme.
- **Kabul:** Ownership matrisi onaylı; 'okunabilir/yazılabilir/ürüne devredilir/yasak' eylemleri tanımlı; gelecekteki tüm yazma senaryoları bu karara dayanır.

#### CORE-002 — Ortak envanter veri modeli [P0]
- **Mimari / dosyalar:** `src-tauri/src/integrations/model.rs` [yeni], TypeScript karşılıkları `src/shared/integrationsTypes.ts` [yeni]. `Asset/Client/Source/Binding/Scope/Capability/ObservedStatus` tipleri; stable asset key + source fingerprint; plugin child graph.
- **Bağımlılıklar:** CORE-001.
- **Test:** Tek asset/çok client, aynı isim-farklı kaynak, plugin içinde iki MCP, global + proje çakışması, bozuk symlink, non-UTF-8 dosya yolu, kaybolan kaynak.
- **Kabul:** Fixture'lardaki tüm bağımsız binding'ler korunur; kimlik çarpışması 0; JSON serileştirme/yeniden okuma round-trip testleri geçer; belirsiz durumda `unknown` kullanılır.

#### CORE-003 — İzole çok-istemcili fixture/test altyapısı [P0]
- **Mimari / dosyalar:** Önerilen `src-tauri/tests/integrations/fixtures/` ve `tests/native/integrations_native_e2e.mjs`. Mevcut `RESPECTED_HUB_TEST_HOME` yalnızca dev/testte; üretimde keyfî kök değişkeni yok.
- **Bağımlılıklar:** CORE-002; `tests/native/hub_native_e2e.mjs` fake home ve real-env guard.
- **Test:** Global/proje dizinleri, geçerli/bozuk JSON, JSONC/TOML, env veya secret referansı, klasör izin reddi, symlink loop, iki kayıt aynı isim, Windows Unicode/boşluk yolları.
- **Kabul:** Her adapter en az 1 normal + 1 bozuk + 1 çakışma fixture'ı ile test edilir; testte gerçek `%USERPROFILE%` konfigürasyonuna 0 yazma; CI'da deterministik sonuç.

#### CORE-004 — Sınırlandırılmış Tauri API ve capability sözleşmesi [P0]
- **Mimari / dosyalar:** `src-tauri/src/lib.rs`, `commands/integrations.rs` [yeni], `integrations/mod.rs` [yeni]; UI yalnızca tipli IPC çağırır. `hub/*`, `lifecycle/*`, `rpp.rs` üçüncü taraf işlem API'si hâline getirilmez.
- **Bağımlılıklar:** CORE-001..003.
- **Test:** Rastgele `path`, `command`, URL, araç/işlem adı veya plan digest'i ile yetkisiz invoke denemeleri; fail-closed; eski lifecycle komutları için regresyon.
- **Kabul:** Production IPC üzerinde genel shell/exec veya arbitrary write çağrısı 0; izin listesindeki olmayan eylemler reddedilir; eski Brain/Orchestrator RPP testleri geçer.

#### SEC-001 — Kaynak/sahiplik, gizlilik ve güvenlik tehdit modeli [P0]
- **Mimari / dosyalar:** `integrations/security.rs` [yeni], redaction ve provenance alanları; `PRIVACY.md`. Read-only tarama sırasında bile credential değerleri UI/telemetri/kayıtlarda açığa çıkmaz.
- **Bağımlılıklar:** CORE-001/002.
- **Test:** API key, token, parola, `.env` env var referansı, symlink target dışarı kaçışı, symlink loop, çok büyük dosya/derin klasör, bozuk manifest, tanınmayan klasör sahipliği.
- **Kabul:** Test fixture'larındaki 100% bilinen sır örnekleri redakte; herhangi bir tarama yürütülebilir üçüncü taraf kod çalıştırmaz; tarama sınır aşımı kontrollü hata üretir; kaynak kimliği ve sahiplik UI'ya aktarılır.

### M2 — Local Discovery MVP (yalnızca okuma)

#### DISC-001 — AI client varlığı ve kaynak keşfi [P0]
- **Mimari / dosyalar:** Var olan `commands/manage.rs::detect_ai_clients` genişletilir veya keşif motoruna tek giriş olarak devredilir. `integrations/discovery.rs` [yeni]; installed/configured/uncertain durumları; sürüm/konum kanıtı.
- **Bağımlılıklar:** CORE-002..004; CORE-003.
- **Test:** Klasörü var ama program yok; program var ama konfigürasyon yok; farklı Windows profili, `APPDATA`/`LOCALAPPDATA`, özel config path, bozuk dizin erişimi.
- **Kabul:** 5 hedef client için farklı durumları ayrı raporlar; sırf `.claude` var diye MCP etkin sayılmaz; mevcut Brain kurulum sihirbazı regresyonu 0.

#### DISC-002 — Claude Code salt okunur adapter [P0]
- **Mimari / dosyalar:** `integrations/adapters/claude.rs` [yeni]. Kullanıcı/proje MCP ve Skill kaynakları; desteklenen plugin manifest ve dosya konumları; kaynak/owner ayrımı. Sadece resmi olarak doğrulanmış konfigürasyon biçimlerini parse et.
- **Bağımlılıklar:** DISC-001, SEC-001, CORE-003.
- **Test:** User/project MCP; aynı isim iki scope; plugin Skill/MCP alt bileşenleri; yorumlu veya bozuk girdi; Brain/Orchestrator'a ait kayıt.
- **Kabul:** En az 12 kurgulanmış fixture olgusunda beklenen binding ve kaynaklar 100% eşleşir; 0 yazma; hata hangi dosyaya aitse açıkça raporlanır.

#### DISC-003 — Codex salt okunur adapter [P0]
- **Mimari / dosyalar:** `integrations/adapters/codex.rs` [yeni]. Desteklenen TOML ve Skills kaynakları, proje güven bağlamı, kapsam öncelikleri; shell'e bağlanmaz.
- **Bağımlılıklar:** DISC-001, CORE-003, SEC-001.
- **Test:** Global/proje config; project trust kısıtı; geçersiz TOML; yanlışlıkla açıkta kalan sır alanı; aynı MCP'nin iki client kaydı.
- **Kabul:** En az 10 fixture, 100% beklenen durum; etkinlik `unknown` ise uydurulmaz; TOML yorum/formatı bu aşamada değiştirilmez.

#### DISC-004 — Cursor + Gemini CLI + OpenCode salt okunur adapter'ları [P1]
- **Mimari / dosyalar:** `integrations/adapters/{cursor,gemini,opencode}.rs` [yeni]; kaynak keşfi ve her istemcinin kendi config merge/override kuralı; capability matrix. Her istemci tek PR veya ayrı issue olabilir.
- **Bağımlılıklar:** DISC-001..003, CORE-003, SEC-001.
- **Test:** Her istemcide user/project; geçerli/geçersiz config; override ve çakışma; OpenCode JSONC/konum katmanları; istemci kurulu ama config yok.
- **Kabul:** Her istemcide ≥8 fixture, 100% beklenen parse; tarama sırasında 0 komut çalıştırma/0 yazma; platform desteklemiyorsa açık `unsupported`.

#### DISC-005 — Skill klasörü ve symlink envanteri [P1]
- **Mimari / dosyalar:** `integrations/skills.rs` [yeni]; adaptörden dönen Skill roots; manifest/`SKILL.md` gibi dosyalar, managed/unmanaged, content hash, symlink target, orphan.
- **Bağımlılıklar:** DISC-002..004, CORE-002, SEC-001.
- **Test:** Aynı Skill iki agent'ta symlink; kopya dosyalar farklı; bozuk symlink; boş/bozuk Skill; dış repo kaynağı.
- **Kabul:** 0 yanlış birleştirme; orijinal ve dağıtım konumları görünür; symlink loop sonsuz döngüye yol açmaz; hiçbir Skill kodu yürütülmez.

#### DISC-006 — Plugin parent/child envanteri [P1]
- **Mimari / dosyalar:** `integrations/plugins.rs` [yeni]; bir Plugin'e ait Skills/MCP/Hooks/Commands/Agents'i parent-child ilişkilerinde korur. Destek sınırları istemci bazlı.
- **Bağımlılıklar:** DISC-002..005, CORE-002, SEC-001.
- **Test:** Plugin içinde iki Skill, bir MCP, bir hook; aynı alt bileşen başka plugin'de; plugin disable ama fiziksel dosyalar var; bozuk manifest.
- **Kabul:** Örnek parent-child ilişkilerinde 100% doğru sahiplik; plugin ve alt bileşen statüleri karıştırılmaz; bilinmeyen format açıkça gösterilir.

#### DISC-007 — Etkin kapsam ve çakışma çözümleyicisi [P0]
- **Mimari / dosyalar:** `integrations/resolver.rs` [yeni]; `configured` vs `effective` vs `enabled`; global/project ve client'a özgü precedence; `unknown/untrusted` dönüşü.
- **Bağımlılıklar:** DISC-002 ve DISC-003, CORE-002; her yeni adapter/Skill/Plugin türü eklendiğinde yeni resolver fixture'ları.
- **Test:** Global+project aynı MCP; project trust kapalı; iki dosyadan override; plugin disable; gözükür ama etkin olmayan kayıt.
- **Kabul:** Her hedef adapter için ≥6 kapsam fixture'ı; toplam ≥30 senaryoda 100% beklenen etkin kayıt; asla yalnızca `is_file()` ile “kullanılıyor” iddiası kurulmaz.

#### CAT-001 — Statik ve tamamen yerel üçüncü taraf katalog [P1]
- **Mimari / dosyalar:** `integrations/catalog.rs` [yeni], `assets/third-party-catalog.json` [önerilen]. Mevcut imzalı `lifecycle/catalog.rs` ürün kurucuları kataloğundan **ayrı**. Alanlar `id,name,type,description,category,source_url,supported_clients,license?,provenance?`.
- **Bağımlılıklar:** CORE-002, SEC-001.
- **Test:** Boş katalog, 100 geçerli kayıt, yinelenen ID, yanlış URL şeması, bozuk JSON, ağ kapalı.
- **Kabul:** Uygulama ağı kullanmadan ≥50 küratörlü kayıt gösterebilir; ID çakışmaları reddedilir; yalnızca doğrulanmış `https://github.com/...` kaynak linki açılır (MVP); kurulum yapmaz.

#### CAT-002 — Keşfet sayfasında filtreli katalog [P1]
- **Mimari / dosyalar:** `src/shell/views/Discover.tsx`, yeni `ThirdPartyCatalog.tsx` [öneri]; mevcut `APPS` ve Brain/Orchestrator installer kartları korunur, ayrı kayıt türü yapılır.
- **Bağımlılıklar:** CAT-001, CORE-004.
- **Test:** Tür/kategori/istemci filtresi, arama Türkçe/Unicode, boş sonuç, internet kapalı, GitHub butonu.
- **Kabul:** ≥50 kayıt; filtre sonucu fixture ile birebir; mevcut Brain/Orchestrator yükleme akışı değişmez; 'Kur' butonu MVP'de yanlış vaat olarak görünmez.

#### UI-001 — Kurulu entegrasyon matrisi / Kütüphanem [P0]
- **Mimari / dosyalar:** `src/shell/views/Library.tsx` yeni alt görünüm veya `IntegrationsLibrary.tsx` [öneri]; mevcut ürün kütüphanesi ayrı kalır. Asset × client × scope, kaynak path, managed/unmanaged, etkinlik ve hata etiketleri.
- **Bağımlılıklar:** DISC-002/003, DISC-007, CAT-001, CORE-004; DISC-004..006 geldiğinde matris genişletilir.
- **Test:** Aynı MCP Claude ve Codex; birinde global diğerinde project; duplicate; büyük listede arama; read-only vs unsupported açıklaması.
- **Kabul:** 3 tür (MCP/Skill/Plugin) filtresi; 5 istemci kapsamı hedefi; kullanıcı hangi kaynaktan okunduğunu açabilir; yanlış 'kullanılıyor' etiketi 0.

#### UI-002 — Projeler ve kapsam görünümü (salt okunur) [P1]
- **Mimari / dosyalar:** `src/shell/views/Projects.tsx` [yeni öneri], proje kökünü seçen güvenli file dialog; kullanıcı açıkça seçtiği proje dışında izinsiz genel disk taraması yapılmaz.
- **Bağımlılıklar:** DISC-007, UI-001, CORE-004.
- **Test:** Proje seçildi, kaldırıldı, taşındı, silindi, izin reddi, global override, symlink root.
- **Kabul:** En az 3 kayıtlı proje arasında etkin binding farkı doğru gösterilir; var olmayan proje güvenli hata verir; salt okunur aşamada 0 config değişikliği.

### M3 — Safe Control Engine (ilk gerçek yönetim)

#### OPS-001 — Salt-okunur değişiklik planı ve preview [P0]
- **Mimari / dosyalar:** `integrations/planner.rs` [yeni]; sadece desteklenen capability'ler için operation plan. Diff, etkilenebilecek projeler, sahiplik, masked secret, hash ve `plan_digest`.
- **Bağımlılıklar:** CORE-004, DISC-007, UI-001/002, SEC-001, CORE-001 onayı.
- **Test:** Plan çağrısı 0 yazma; sahte veya süresi dolmuş plan; target file eş zamanlı değişiyor; Respected-owned entry; unsupported capability.
- **Kabul:** Plan üretmek dosya hash'lerini değiştirmez; 100% hedef dosyalar belirtilir; stale plan `SOURCE_CHANGED` ve owned kaynak `OWNER_PROTECTED` ile reddedilir.

#### OPS-002 — Transactional writer + dosya biçimi koruması [P0]
- **Mimari / dosyalar:** `integrations/apply.rs` [yeni]; allowlisted atomic replace/backup/readback, dosya izinlerini ve mümkünse biçimi koruma; gerekirse desteklenmeyen biçime salt okunur kilit. UI raw path taşımaz.
- **Bağımlılıklar:** OPS-001, CORE-003/004, SEC-001; sözleşme/doküman güncellemesi.
- **Test:** JSON/JSONC/TOML round-trip, yorum/unknown key koruma, izin reddi, disk dolu, yazma sırasında kapanma, stale digest, filename case ve symlink replacement.
- **Kabul:** Desteklenen fixture'larda bilinmeyen alan ve yorum kaybı 0; kaynağın hash'i işlem öncesi tekrar doğrulanır; başarısız yazma sonrası doğrulanmış önceki durum geri gelir veya kontrollü kurtarma durumu görünür.

#### OPS-003 — İstemci bazlı etkinleştir/devre dışı bırak [P0]
- **Mimari / dosyalar:** `adapters/*` için ayrı `plan_enable/plan_disable` capability; UI toggle sadece adapter garantisi varsa; unsupported durumda buton pasif ve gerekçeli.
- **Bağımlılıklar:** OPS-002, DISC-007, UI-001.
- **Test:** Aynı MCP yalnızca Claude'da disable, Codex etkilenmiyor; native `enabled` alanı; alanı olmayan config için izinli alternatif; yeniden başlatma gerektiren client.
- **Kabul:** Doğrulanan bütün destekli toggle işlemleri yeniden okumadan sonra hedef statüye ulaşır; diğer client kaynaklarında byte değişikliği 0; unsupported işlemler uygulanmaz.

#### OPS-004 — Global ↔ proje kapsam taşıma/kopyalama [P0]
- **Mimari / dosyalar:** `integrations/planner.rs`, adapter scope writer'ları ve `resolver.rs`; `move` ile `copy` farklı eylemler; global kaydı kaldırmanın diğer projelere etkisi diff'te.
- **Bağımlılıklar:** OPS-002/003, UI-002.
- **Test:** Global→proje, proje→global, çakışma, farklı projelerde ortak global kayıt, owned source, project trust kapalı, aynı target tekrar var.
- **Kabul:** Her destekli client için en az 6 kapsam senaryosu; hiçbir işlem sessizce başka projeleri bozmaz; etkiler onay öncesi görünür; yeniden okuma hedef bağlamı doğrular.

#### OPS-005 — Undo, crash recovery, operasyon günlüğü [P0]
- **Mimari / dosyalar:** `integrations/undo.rs` [yeni], mevcut `lifecycle` işlem günlüğünden ayrı third-party transaction journal; yalnızca uygulanan kayıtların yedeklerini yönetir.
- **Bağımlılıklar:** OPS-002/003/004.
- **Test:** Uygulama kapanması (yedekten önce / yazı ortası / doğrulamadan önce), çoklu dosya kısmi başarı, başka uygulama dosyayı sonradan değiştirdi, iki eşzamanlı plan.
- **Kabul:** Denenen 3 crash noktasında veriler kaybolmaz; undo başarılıysa önceki hash birebir geri gelir; dosya dışarıdan değiştiyse üzerine zorla yazmak yerine çatışma gösterilir; eşzamanlı işlem kilidi çalışır.

### M4 — Güvenli mağaza / harici içerik yaşam döngüsü

#### SEC-002 — Ayrı, imzalı üçüncü taraf katalog güncelleme kanalı [P1]
- **Mimari / dosyalar:** `integrations/catalog.rs`, `trust.rs` içindeki doğrulama yapıtaşlarını **domain separation** ile yeniden kullan; ürün kurucusu katalog şemasını sessizce genişletme. Örn. `respected-thirdparty-catalog-v1\n` alan ayrımı.
- **Bağımlılıklar:** CAT-001, SEC-001, OPS-005.
- **Test:** Signature mismatch, replay/downgrade sequence, bozuk digest, büyük dosya, offline fallback, karışık anahtar/kanal.
- **Kabul:** İmzasız/bozuk/geriye giden uzak katalog 100% reddedilir; son doğrulanmış katalog offline açılır; bu imzanın yalnızca metadata bütünlüğünü doğruladığı UI'da belirtilir.

#### SEC-003 — Kaynak güvenlik incelemesi ve izin açıklaması [P1]
- **Mimari / dosyalar:** `integrations/security.rs` ve UI kaynak kartı: repo, publisher bilgisi (kanıt varsa), paket digest, kurulum komutları, izin/risk sınıfları. Statik analiz yalnızca uyarıdır; 'tam güvenli' etiketi yok.
- **Bağımlılıklar:** SEC-001/002, CAT-003.
- **Test:** `curl|sh`, `npx`, `uvx`, script hook, env secret isteği, kaynak URL yönlendirmesi; manifest içinde gömülü komut; hash drift.
- **Kabul:** Bütün çalıştırılabilir kurulum yolları kullanıcı onayı ve önizleme ister; bilinmeyen izin 'unknown' gösterilir; gözden geçirme olmadan hiçbir komut otomatik çalıştırılmaz.

#### CAT-003 — Katalog önbelleği ve isteğe bağlı tazeleme [P1]
- **Mimari / dosyalar:** Ayrı `third-party-catalog` alanı, kullanıcı başlattığında HTTPS metadata alımı; mevcut `lifecycle/catalog.rs` imzalı Respected ürün kataloğu çalışmasını etkilemez.
- **Bağımlılıklar:** CAT-001/002, SEC-002.
- **Test:** Offline, timeout, kısmi indirme, imza hatası, geri gitme, 50–100 kayıt filtre kararlılığı.
- **Kabul:** Ağa çıkmadan statik katalog açılır; yeni imzalı katalog başarıyla yüklenir; başarısız tazeleme doğrulanmış son kaydı bozmaz.

#### OPS-006 — Var olan üçüncü taraf MCP/Skill'i kontrollü içe alma [P1]
- **Mimari / dosyalar:** Adapter `plan_bind` ile mevcut yerel kaynak kaydını ekleme; kullanıcı dosya/URL veya source seçer. Varsayılan kopyala değil, uygun olduğunda referans bağla. Respected kayıtları hariç.
- **Bağımlılıklar:** OPS-005, SEC-003, CAT-003.
- **Test:** Mevcut MCP, duplicate, geçersiz path, symlink, env secret, iki client, rollback.
- **Kabul:** Sadece izin verilen hedefe yazılır; aynı isteğin tekrar uygulanması mükerrer binding üretmez; undo çalışır.

#### OPS-007 — Güncelleme/kaldırma ve sahiplik koruması [P1]
- **Mimari / dosyalar:** Source provenance + managed-by-Hub makbuzları; user-installed ve external-owned kayıtları ‘kaldır’ yerine ‘ayır’ semantiğiyle ele al.
- **Bağımlılıklar:** OPS-006, SEC-003.
- **Test:** Sadece Claude bağını kaldır, Codex kalsın; ortak symlink; manuel değişmiş config; plugin içinde paylaşılan Skill; orijinal dosyayı silme sınırı.
- **Kabul:** Hub'ın oluşturmadığı kaynak dosya otomatik silinmez; ilişki kaldırma diğer istemcileri etkilemez; her silme etkisi önizlenir; geri alma kontrolü geçer.

#### OPS-008 — Çoklu istemciye güvenli toplu uygulama [P2]
- **Mimari / dosyalar:** Tek plan içinde birden fazla Binding, operasyon journaling ve partial-failure UX; bağımsız adımları takip edilebilir transaction'a bağla.
- **Bağımlılıklar:** OPS-005..007.
- **Test:** 3 istemcide ekle, ikinci adım permission denied; üçüncü uygulanmamalı veya tüm değişiklikler rollback ile tutarlı kalmalı (sözleşmedeki politika açık seçilmeli).
- **Kabul:** Tüm uygulamalar doğrulandığında başarı; kısmi başarı yanlışlıkla tam başarı gösterilmez; her başarısız senaryoda önceki duruma güvenli dönüş veya operatörce çözülebilir net hata.

### M5 — Farklılaşma ve platform genişlemesi

#### ADV-001 — Profiles / paketler [P2]
- **Mimari / dosyalar:** `integrations/profiles.rs` [yeni], yalnızca `Binding` hedeflerini ve istekleri saklayan yeniden uygulanabilir profil; secret içermez.
- **Bağımlılıklar:** OPS-003..005, UI-002.
- **Test:** Bir profili iki projeye uygula, aynı profile ikinci kez uygula, source bulunamadı, iki client yetenek farkı.
- **Kabul:** İdempotent plan; eksik kaynak açıkça listelenir; profil değişimi önceki AI konfigürasyonlarını izinsiz silmez; hiçbir credential profil export'una girmez.

#### ADV-002 — Drift / gölgelenme / bozuk bağlantı analizi [P2]
- **Mimari / dosyalar:** `integrations/resolver.rs` ve `inventory.rs`: kaynağın hash/mtime farkı, iki farklı kopya, shadowed profile ve orphan symlink göstergeleri.
- **Bağımlılıklar:** DISC-007, OPS-005.
- **Test:** Hub kapalıyken config haricen değişir; aynı Skill farklı içerik; silinen target; üzerine yazılmaması gereken owned entry.
- **Kabul:** Fixture'lardaki bütün drift durumları doğru kategoriye düşer; varsayılan olarak otomatik düzeltme yapılmaz; kullanıcı source ve etkilerini görür.

#### PORT-001 — macOS/Linux kontrat yetenek matrisi [P3]
- **Mimari / dosyalar:** Platform path discovery, permission/symlink değerlendirmesi ve gerçek OS test harness'i; platform başına capability matrisi.
- **Bağımlılıklar:** M2+M3 Windows kalite kapıları, CORE-003.
- **Test:** macOS ve Linux temiz kullanıcı profilinde client discovery, read-only scan, desteklenen kapsam işlemleri, crash/rollback.
- **Kabul:** Hedef OS'te aynı kontrat testleri gerçek CI üzerinde geçmeden 'destekleniyor' etiketi yok; Windows testleri gerilemez. Masaüstü GUI/paketleme doğrulaması bu görevin dışındadır ve v0.1.0 resmi sürümünde kapatılır.

#### QA-001 — Her aşamada kalite ve yayın kapıları [P0, sürekli]
- **Mimari / dosyalar:** `npm run check`, `cargo test`, ilgili `cargo test --release --lib trust`, derlenmiş Windows native driver, fixture matrisi; gerektiğinde ayrı izole `integrations_native_e2e.mjs`.
- **Bağımlılıklar:** Her iş paketi.
- **Test:** Normal, bozuk/boş config, no-network, non-ASCII path, crash/fault injection, secret redaction, ownership, rollback, native GUI.
- **Kabul:** Yeni görevler için 100% zorunlu fixture testleri ve sıfır regresyon; ilgili test raporu PR'a eklenir; Windows native E2E başarısızsa sürüm yayımlanmaz; test yapılmadıysa `doğrulanmadı` yazılır.

## 5. Uygulama sırasına göre küçük PR dilimleri

1. `PR-01` (M0) REL-001, `PR-02` REL-002, `PR-03` REL-003 — özellik eklemeden mevcut sürüm kapıları.
2. `PR-04` CORE-001 + SEC-001 — sahiplik/tehdit modeli ADR.
3. `PR-05` CORE-002 + CORE-003 — veri modeli + fixture; UI yok.
4. `PR-06` CORE-004 + DISC-001 — tipli read-only IPC, mevcut `detect_ai_clients` genişlemesi.
5. `PR-07` DISC-002 Claude; `PR-08` DISC-003 Codex; `PR-09` DISC-007 ilk 2 client resolver.
6. `PR-10` CAT-001 + CAT-002 — çevrimdışı kataloğu ve GitHub bağlantılarını çıkar (M2 kapsamına sığabilir).
7. `PR-11` UI-001 — Claude/Codex envanterini göster; **ilk gerçek kullanıcı değerinin çıktığı an**.
8. `PR-12..14` DISC-004/005/006 + UI-002 — diğer üç agent, Skills, Plugins, projeler; M2 tamamlanır.
9. `PR-15` OPS-001 — sadece plan/diff; M3 başlamadan sözleşme ve güvenlik gözden geçirmesi.
10. `PR-16` OPS-002 + OPS-005'in transaction altyapısı; ardından `PR-17` OPS-003, `PR-18` OPS-004, `PR-19` OPS-005'in tam undo/crash recovery kapsamı.
11. `PR-20..` SEC-002, CAT-003, SEC-003, OPS-006..008; M4 özelliği opsiyonel ve kademeli.
12. `PR-25..` ADV-001/002, PORT-001; ölçülen kullanıcı ihtiyacına göre.

> **Not:** PR dilimleri planlama önerisidir; eşit süre veya takım kapasitesi varsayımı değildir. Özellikle transaction/rollback PR'ları test kanıtı olmadan birleştirilmez.

## 6. Milestone bazlı 'Done' tanımı

| Milestone | Kullanıcının göreceği çıktı | Yayın kapısı |
|---|---|---|
| M0 | Mevcut Respected ürün yönetimi daha güvenilir | Gerçek eski sürüm güncelleme + temiz VM + native E2E geçer |
| M1 | Henüz görünür yeni özellik gerekmez | Sahiplik ADR onaylı, şema/fixture/IPC güvenlik testleri geçer |
| M2 | Yerel katalog + otomatik kurulum envanteri + projeye göre read-only görünüm | 5 istemcinin testli envanteri, 0 gerçek yazma, ≥50 kayıt çevrimdışı |
| M3 | Bir araca aç/kapat, proje/global taşı, preview + undo | Tüm support edilen işlemler plan→apply→readback→undo ile doğrulanır; yanlış yazma 0 |
| M4 | İsteğe bağlı güvenli katalog güncelleme, import, yükleme/kaldırma | İmza/replay testleri, komut onayı, sahiplik/rollback testleri geçer |
| M5 | Profiller, drift ve platform genişlemesi | Platform/özellik bazlı gerçek test kanıtı olmadan destek iddiası yok |

## 7. İş sıralamasındaki kritik notlar

- **İlk ürün artışı M2'de Claude + Codex ve çevrimdışı katalogla yapılabilir.** 5 istemci tam olana kadar diğer adapter'lar `read-only/unsupported` olarak dürüstçe gösterilebilir. Bu, kaliteli bir dikey dilimle başlamayı sağlar.
- **M2'de kurma/kaldırma yok.** GitHub bağlantısı açılabilir; otomatik komut çalışmaz. Rakip mağazayla görünürlük rekabeti başlarken güvenli yazma sonraki aşamaya kalır.
- **M3 olmadan 'toggle'ı üretime çıkarma.** UI düğmesinin görüntülenmesi kullanılabilir eylem olduğu anlamına gelmez; önce gerçek adapter desteği + plan + geri alma.
- **Hedefler tasarım/test kriterleridir, ölçülmüş mevcut performans değil.** 50 katalog kaydı veya 30 fixture vb. sayılar önerilen kabul eşikleridir.
- **Kabul ölçümü:** Her görevde fixture beklentisi, komut çıktısı, değiştirilmiş dosya hash'i, native E2E raporu ve ekran çıktısı saklanır; sadece “çalışıyor” yorumu yeterli değildir.

## 8. Her PR için standart kontrol listesi

- [ ] Kaynak/ownership sınırları korunuyor (Brain/Orchestrator dahil)
- [ ] Üretim koda dev-only trust bypass girmedi
- [ ] Testler disposable home kullanıyor, gerçek AI yapılandırmaları korunuyor
- [ ] İşleme özgü normal/hata/çakışma fixture'ları eklendi
- [ ] Varsa diff, preview, redaction ve undo testleri var
- [ ] `npm run check` yeşil
- [ ] `cargo test` yeşil
- [ ] UI/native değiştiyse Windows binary native E2E yeşil
- [ ] `docs/ARCHITECTURE.md`, `FOOTPRINT.md`, `PRIVACY.md`, `AGENTS.md` etkileniyorsa aynı PR'da düzeltildi
- [ ] Desteklenmeyen özellik/istemci UI'da açıkça etiketli
- [ ] Doğrulanmayan iddia release notes'a yazılmadı

## 9. Bağlantılı özgün dosyalar

- [`docs/ROADMAP.md`](https://github.com/respected0/respected-hub/blob/main/docs/ROADMAP.md)
- [`docs/ARCHITECTURE.md`](https://github.com/respected0/respected-hub/blob/main/docs/ARCHITECTURE.md)
- [`docs/PRODUCT_CONTRACT.md`](https://github.com/respected0/respected-hub/blob/main/docs/PRODUCT_CONTRACT.md)
- [`docs/FOOTPRINT.md`](https://github.com/respected0/respected-hub/blob/main/docs/FOOTPRINT.md)
- [`AGENTS.md`](https://github.com/respected0/respected-hub/blob/main/AGENTS.md)
- [`PRIVACY.md`](https://github.com/respected0/respected-hub/blob/main/PRIVACY.md)
- [`src-tauri/src/commands/manage.rs`](https://github.com/respected0/respected-hub/blob/main/src-tauri/src/commands/manage.rs)
- [`src/shell/views/Discover.tsx`](https://github.com/respected0/respected-hub/blob/main/src/shell/views/Discover.tsx)
- [`src/shell/views/Library.tsx`](https://github.com/respected0/respected-hub/blob/main/src/shell/views/Library.tsx)
- [`tests/native/hub_native_e2e.mjs`](https://github.com/respected0/respected-hub/blob/main/tests/native/hub_native_e2e.mjs)
