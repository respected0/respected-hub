# Hub mimarisi

Hub'ın ürünleri nasıl bulduğu, kurucuları nasıl indirip çalıştırdığı, neye güvendiği ve web görünümüne neyi yasakladığı.
Giriş: [README.md](../README.md). Yayın: [common/release/RELEASE.md](../common/release/RELEASE.md). Ortak kurallar
[PRODUCT_CONTRACT.md](PRODUCT_CONTRACT.md) dosyasındadır; burada tekrarlanmaz.

## 1. Ürünleri bulma ve durum

- Bir ürün, standart klasöründe (`%LOCALAPPDATA%\Respected\<Ürün>\app`) programı ve `distribution.json`'ı varsa kuruludur
  (`hub/installed.rs`). Sürüm `distribution.json`'dan, durum ürünün kendi `manifest` yanıtından gelir: ürün
  `repair-needed` derse kart "Onarım gerekli" gösterir; yanıt yoksa da öyle. Katalogda daha yeni sürüm varsa
  "Güncelleme var".
- Ayarlar, sağlık, hafıza ve bağlantılar her seferinde ürünün `rpp` komutuyla sorulur. Hub ürünlere klasör geçirmez;
  ürün kendi klasörlerini bilir.
- Üçüncü taraf MCP/Skill/Plugin envanteri ayrı `integrations` motorundadır ve M1/M2'de salt okunurdur. Sahiplik,
  redaksiyon ve yazma kapısı: [ADR-THIRD-PARTY-INTEGRATIONS.md](ADR-THIRD-PARTY-INTEGRATIONS.md).
- Keşfet kataloğu (`integrations/catalog.rs`) `lifecycle::catalog`tan ayrıdır: `assets/third-party-catalog.json` derleme anında ikiliye gömülür, ağ veya kurulum yapmaz. Yalnızca `https://github.com/<sahip>/<repo>` bağlantıları, benzersiz kimlikler ve geçerli istemciler kabul edilir; tek geçersiz kayıt belgenin tamamını reddeder. Keşfet bu kayıtları `integrations_request`/`catalog` ile salt okunur listeler; kartlarda yalnızca GitHub bağlantısı vardır, kurma eylemi yoktur.
- Keşif tek motordur (`integrations/discovery.rs`): klasör/dosya varlığı yalnızca kaynak kanıtıdır; programın yolu `installed`, ikisi birlikte `configured` demektir. Hiçbir durum tek başına `enabled` veya kullanımda sayılmaz; okunamayan yol kontrollü bir uyarıdır.
- Claude Code adapter'ı (`integrations/adapters/claude.rs`) yalnızca doğrulanmış kaynakları okur: kullanıcı `~/.claude.json`, proje `<kök>/.mcp.json`, kullanıcı/proje `skills/<ad>/SKILL.md` klasörleri ve `.claude-plugin/plugin.json` manifestli plugin'ler. Proje kökü kullanıcı seçimidir; adapter diskte proje taraması yapmaz. Brain/Orchestrator girdileri `Owner::Product` olarak işaretlenir ve yazılmaz.
- Codex adapter'ı (`integrations/adapters/codex.rs`) `$CODEX_HOME/config.toml` (yoksa `~/.codex/config.toml`), `skills/` ve yalnızca `[projects."<yol>"] trust_level = "trusted"` olan projelerin `.codex` kaynaklarını okur. Güvenilmeyen proje dosyası okunmaz; `project_untrusted` uyarısı dosya yolunu gösterir. Aynı MCP kimliği Claude ve Codex'te tek `asset_key` taşır.
- Cursor ve Gemini CLI adapter'ları (`integrations/adapters/{cursor,gemini}.rs`) salt okunurdur: yalnızca kullanıcı ve seçili proje ayarlarını okur, iki katmanı ayrı binding olarak bildirir; etkin değeri `DISC-007` resolver'ı hesaplar. OpenCode adapter'ı (`integrations/adapters/opencode.rs`) aynı kaynakları okur ve yalnızca `mcp.<ad>.enabled` alanını toggle edebilir. Kaynaklar: Cursor `~/.cursor/mcp.json` + `.cursor/mcp.json` (+ `.cursor`, `.agents`, `.claude`, `.codex` skill kökleri), Gemini `~/.gemini/settings.json` + `.gemini/settings.json`, OpenCode `$XDG_CONFIG_HOME/opencode` altındaki global katmanlar, `$OPENCODE_CONFIG` ve proje `opencode.json(c)`.
- JSONC okuyan adapter'lar yorumları ve sondaki virgülleri ayıklar; hatalı dosya yalnızca yol ve sabit neden bildirir, içerik yankılanmaz. Tüm adaptörler `read = true`, `plan = true`, `apply = false`, `undo = false` ilan eder. MCP aç/kapa yeteneği istemci bazlıdır: Codex (`mcp_servers.<ad>.enabled`) ve OpenCode (`mcp.<ad>.enabled`) `enable/disable = true`; Claude, Cursor ve Gemini `false` ve gerekçeli `toggle_reason` taşır. Toggle planı yalnızca hedef dosyadaki `enabled` üyesini değiştirir; `apply/undo` M3 kapısı açılana kadar kapalıdır.
- Skill envanteri (`integrations/skills.rs`) kimliği `SKILL.md`'nin kanonik hedef yolundan üretir: aynı klasöre işaret eden symlink'ler tek asset, farklı kopyalar ayrı asset olur. `source.path` dağıtım konumunu, `source.symlink_target` orijinal hedefi, `fingerprint` içerik hash'ini gösterir. Boş manifest `skill_empty`, kapanmayan frontmatter `skill_invalid_manifest`, çözülemeyen hedef `orphan_skill`/`broken_link` uyarısıdır; hiçbir Skill kodu çalıştırılmaz.
- Plugin envanteri (`integrations/plugins.rs`) `.claude-plugin/plugin.json` manifestini okur ve plugin'i kendi `asset_key`'iyle ekler. Alt öğeler (plugin içi `skills/`, `commands/*.md`, `agents/*.md`, `hooks/hooks.json`, Claude MCP'leri) ayrı kimlik ve `parent_asset_key` taşır; iki farklı plugin'deki aynı adlı alt öğe birleşmez. Manifestteki `enabled` yalnızca ipucudur: devre dışı plugin'in dosyaları görünür kalır ama `enabled` durumuna yükselmez; istemci seviyesindeki etkin plugin çözümü `DISC-007` resolver işidir. Bozuk manifest `invalid_json` uyarısı üretir ve hayali alt öğe oluşturmaz. Komut/agent kaynakları `Markdown`, hook kaynağı `Json` biçimindedir.
- Etkin kapsam çözümleyicisi (`integrations/resolver.rs`) adapter'ların düşük→yüksek öncelik sırasıyla eklediği binding'leri `configured`/`effective`/`disabled`/`unknown`/`overridden` durumlarına indirger. Global kayıt tüm projelerde geçerlidir; aynı MCP/plugin adı proje dosyasında varsa yalnız o projede global kayıt gölgelenir. Açık `enabled=false`, bozuk/kırık kaynak ve devre dışı plugin'in alt öğeleri etkin sayılmaz. Codex `project_untrusted` kayıtları `untrusted` listesinde döner ve proje dosyasından binding üretilmez. Her çözümlenmiş giriş kaynak sahibini (`owner`) ve Hub yazımı olup olmadığını (`managed`) taşır; dosya varlığı tek başına 'kullanılıyor' kanıtı değildir.
- Değişiklik planlayıcısı (`integrations/planner.rs`) salt okunur önizleme üretir: kapalı seçici (`operation=remove_mcp|enable_mcp|disable_mcp|move_mcp|copy_mcp`, `asset_key`, istemci, kaynak kapsam/proje, hedef `target_scope`/`target_project`), hedef dosya, önce/sonra SHA-256, maskelenmiş girdi diff'i, sahiplik ve etkilenebilecek projeler. Toggle planı hedef `<container>.<ad>.enabled` değerini `true/false` yapan tek üyelik diff, istemci yeniden başlatma notu ve global kapsam etkisi taşır; desteklenmeyen istemcide adapter gerekçesiyle `UNSUPPORTED_CAPABILITY` döner. `move_mcp`/`copy_mcp` yalnızca aynı istemcide kapsam değiştirir: hedef dosya mevcut olmalı (`TARGET_NOT_FOUND`), aynı ad hedefte varsa `TARGET_EXISTS`, kaynak ve hedef aynıysa `BAD_REQUEST` döner; Codex'te güvenilmeyen proje hedef olamaz. Plan, hedef dosyanın önce/sonra hash'ini, taşınan girdiyi, tüm etkilenen projeleri ve global move/gölgeleme/yeniden başlatma risklerini gösterir. Plan üretimi hiçbir dosyaya yazmaz; kaynak `managed`, `Owner::Product` veya `Owner::Catalog` ise `OWNER_PROTECTED`, yetenek kapalıysa `UNSUPPORTED_CAPABILITY` döner. `plan_digest` deterministiktir ve 5 dakikalık `expires_at` taşır; `verify` sahte/süresi dolmuş planı `INVALID_PLAN`/`PLAN_EXPIRED`, taramadan sonra değişen dosyayı `SOURCE_CHANGED` ile reddeder. `apply/undo` hâlâ kapalıdır.
- Yama motoru (`integrations/apply.rs`) yalnızca sunucuda üretilmiş `OperationPlan`ı kabul eder; web görünümünden yol, komut veya ham bayt almaz. Uygulamadan önce plan digest/TTL/hash'ini yeniden doğrular, taze envanterden binding'i yeniden çözer ve hedefin planla aynı kanonik yol olduğunu (Windows'ta büyük/küçük harf duyarsız) kanıtlar. `managed`, `Owner::Product` ve `Owner::Catalog` sahipliğini, symlink'i ve beklenmeyen dosya türünü reddeder. JSON/JSONC/TOML'de hedefli üye siler veya mevcut `enabled` boolean'ını çevirir/eksikse ekler; boolean olmayan değeri reddeder; bilinmeyen anahtar, sıra, boşluk ve yorumlar korunur. Aynı dizinde atomik replace, tek kullanımlık yedek, fsync ve readback hash doğrulaması yapar; readback başarısızsa önceki baytları geri yükler, bu da başarısızsa `RECOVERY_REQUIRED` ve yedek yolunu döndürür. Çok dosyalı `move_mcp`/`copy_mcp` tek transaction'da yazılır: her dosya yedeklenir, readback sonrası journal `committed` olur; süreç yazma ortasında ölürse sonraki açılış yalnızca `prepared` kayıtları yedek hash'iyle geri yükler, dışarıdan değişen dosyayı `CONFLICT` olarak bırakır. Tamamlanmış işlem `integrations/undo.rs` üzerinden `undo` ile önceki baytlara döner; journal ve undo motoru iç testlerle doğrulanır. `integrations.apply` IPC eylemi ve `Capability.apply` M3 yazma kapısı ürün sahibi/ADR onayıyla açılana kadar kapalıdır.
- Envanter birleştirici (`integrations/inventory.rs`) Claude, Codex, Cursor, Gemini CLI ve OpenCode taramalarını tek snapshot'ta toplar; aynı `asset_key` tek varlık, her istemci/kapsam ayrı binding olur. Aynı hata kodu ve kaynak yolu tek kez bildirilir; `clients` beş istemcinin gerçek keşif durumundan gelir.
- Kütüphane'nin entegrasyon görünümü (`src/shell/views/IntegrationsLibrary.tsx`, `src/shell/integrationMatrix.ts`) bu snapshot'ı asset × istemci × kapsam matrisi olarak gösterir: tür/istemci/kapsam filtreleri, kaynak yolu, sahiplik ve `managed` etiketi vardır. Her satır istemcinin toggle yeteneğini ve gerekçesini rozet olarak gösterir; toggle eylemi henüz UI'dan çağrılmaz. Görünüm salt okunurdur; "kullanılıyor" yargısını resolver üretir, UI kendi başına etkinlik uydurmaz.
- Proje kayıt defteri (`integrations/projects.rs`) yalnızca kullanıcının dosya seçiciden açıkça seçtiği kökleri `%APPDATA%\com.respected.hub\projects.json` altında tutar. Yol mutlak, var olan ve klasör olmalıdır; symlink kanonik hedefe çözülür ve aynı kök iki kez eklenmez. AI istemci ayarları yazılmaz. Kayıt sonradan taşınır/silinirse envanter `project_missing` uyarısı üretir ve o kök taranmaz; bozuk kayıt defteri `projects_invalid` ile kapalı biçimde reddedilir.
- Kütüphane'nin Projeler görünümü (`src/shell/views/Projects.tsx`, `src/shell/projectScope.ts`) kayıtlı kökleri ve her istemci için etkin binding'leri gösterir. Proje MCP/plugin kaydı aynı adlı global kaydı yalnız o projede gölgeler; görünüm bu hesabı resolver snapshot'ından yapar ve diskte tarama başlatmaz. Proje ekleme/kaldırma yalnızca Hub'ın kendi proje listesini değiştirir; salt okunur aşamada istemci yapılandırmasına 0 yazma yapılır.

## 2. Kurucuyu çalıştırma

`hub/installer.rs`. Her eylem ürünün kendi kurucusu ya da kaldırıcısıdır:

| Eylem | Ne çalışır |
|---|---|
| Kur, Güncelle, Onar | Katalogdaki kurucu: `/VERYSILENT /SUPPRESSMSGBOXES /NORESTART` + seçilenler için `/VAULT=` ve `/SET=anahtar=değer` |
| Kaldır | Ürünün kaldırıcısı (`app\unins000.exe`; yarım kalmış bir kurulum numaralı bir çift bırakmışsa sağlam olan en yenisi) aynı sessiz anahtarlarla, istenirse `/PURGE=1` |

1. **Plan:** kurucu paketinin imzası ve kurucunun SHA-256'sı denetlenir, kişiye ne çalışacağı gösterilir ("…imzalı
   kurucusunu çalıştırır"). Planda hiçbir şey yazılmaz.
2. **Onay** sonrası kurucu (en fazla 15 dakika) çalışır. Bilinmeyen bir anahtar ya da parametre reddedilir, kurucuya
   geçirilmez. Kurucu sıfırdan farklı bir kodla biterse işlem `LIFECYCLE_PARTIAL` olarak kapanır.
3. **Doğrulama:** kurulumdan sonra ürünün `manifest`'i `installed: true` demelidir; kaldırmadan sonra program klasörü
   gitmiş olmalıdır.
4. **Önbellek:** yalnızca şu an kurulu sürümün kurucusu kalır (çevrimdışı onarım için); kaldırınca hiçbiri kalmaz.

Hub geri alma tutmaz: kurucu programı yerinde değiştirir, kaldırıcı siler; tutarlılık ürünün işidir. Her eylem
`managed\journal\` altına bir kayıt yazar (Ayarlar › Etkinlik). Hub bir işlemin ortasında kapanırsa o kayıt bir sonraki
açılışta "başarısız" olarak kapatılır; ürün meşgul görünüp kalmaz. Aynı ürün için ikinci bir işlem, birincisi bitmeden
başlamaz.

**Açılış niyetleri:** ürün kurucusunun "Hub ile devam et" yolu Hub'ı `--install=<ürün>` ile açar; `--manage=<ürün>` ürün
sayfasını açar. Hub zaten açıksa ikinci pencere açılmaz; çalışan pencere öne gelir ve niyeti alır.

## 3. Katalog ve indirme

`lifecycle/catalog.rs`. İnsanların indirdiği tek dosya küçük Hub kurucusudur; ürün kurucuları yalnızca kurulurken
indirilir.

| Sabit sözleşme (yayınlanmış her Hub buna bağlıdır) | Değer |
|---|---|
| Katalog adresi | `https://github.com/respected0/respected-hub/releases/download/catalog/catalog.json` (+ `.sig`) |
| Katalog imzası | Ed25519 yayın anahtarı, `b"respected-catalog-v1\n"` + dosyanın tam baytları üzerinde; `.sig` = `<anahtar_kimliği> <base64>` |
| Katalog şeması 1 | `{schema, sequence, published, hub:{stable,beta}, products:{brain,orchestrator:{stable,beta}}}`; bir sürüm `{version, url, sha256, size, min_hub?, min_from?, notes?}`; kök `blocked?: {ürün: [sürüm]}` geri çekilen sürümleri listeler; `notes` en çok 600 karakterlik düz metindir, güncelleme önerilirken gösterilir |
| Geri gitme | Önbellektekinden küçük `sequence` reddedilir |
| İndirme sınırı | `curl --max-filesize`: katalog 4 MiB, imza 4 KiB, arşiv ve Hub kurucusu katalogda yazan `size`; fazlası indirilirken kesilir |
| Arşiv | Kökünde tek imzalı paket klasörü (`bundle.json` + `payload/<Ürün>-Windows-Setup.exe`) olan zip |
| Sürüm sırası | SemVer (`0.1.0-beta.1 < 0.1.0-rc.1 < 0.1.0 < 0.1.1-beta.1`); PEP 440 yazımları eşittir (`0.1.0b1`) |
| Kanallar | `beta` (varsayılan; beta ve kararlıyı görür) ve `stable`; `managed\hub-preferences.json` |

- İndirme Windows'un kendi `curl.exe` ve `tar.exe`'siyle (System32'den, PATH'ten değil) yapılır; üretimde yalnızca HTTPS.
  Her bayt iki kez denetlenir: katalogdaki SHA-256, sonra paketin kendi imzası ve dosya hash'leri.
- Katalog Hub açılışında ve "Güncellemeleri denetle" ile tazelenir; plan ağa çıkmaz.
- `min_hub` eski bir Hub'ın önce kendini güncellemesini ister (`HUB_TOO_OLD`). Güncellemeleri yeni Hub yapar; eski bir
  Hub'daki hata bir sonraki sürümle düzelir.
- Hub'ın kendi güncellemesi: doğrulanmış Hub kurucusu `/SILENT /RELAUNCH=1` ile eski Hub'ın üzerine çalışır (aynı AppId
  `{5C0F6A1E-3B7D-4E2A-9D55-7A1C2F0E8B31}`); Hub yeniden açılır, ürünlere dokunulmaz.

## 4. Paket güveni

`trust.rs`, imzalama `common/release/tools/signing/bundle_signing.py`.

- Her paket, kanonik ve alan ayrımlı bir ifade üzerinde Ed25519 imzası taşır: `respected-bundle-v1\n` + şema, ürün, tür,
  sürüm ve `bundle_digest` alanlarının sıralı sıkı JSON'u. Özet her dosyanın yolunu, boyutunu ve hash'ini bağlar; bir imza
  başka bir ürüne, türe, sürüme ya da içeriğe taşınamaz.
- Paketler katalog okunurken doğrulanır; doğrulanmayan paket gerekçesiyle atlanır, hiçbir şey sessizce kabul edilmez.
- **Güven derleme ve anahtarla ayrılır, veride bir bayrakla değil:**
  - Üretim anahtarları derlemeye gömülüdür (`PRODUCTION_KEYS`; açık satır `src-tauri/release-keys.txt`, anahtar kimliği
    `dabcb5265227bbdf`).
  - Geliştirme güveni (yerel geliştirme anahtarının açık yarısı: exe'nin yanındaki `dev-trust.pub` ya da
    `RESPECTED_HUB_DEV_TRUST`) ve imzasız paket izni (`RESPECTED_HUB_ALLOW_UNSIGNED=1`) yalnızca hata ayıklama derlemesinde
    ya da `--features dev-trust` ile vardır; üretim derlemesinde kod hiç yoktur. `cargo test --release --lib trust`, hiçbir
    ortam değişkeninin bunu geri açamadığını kanıtlar.
  - Geliştirme özel anahtarı `.build-cache/signing` altındadır, Git'e girmez, hiçbir kurucuya ya da pakete kopyalanmaz.
- Kanıt: `trust.rs` birim testleri (Python'la üretilmiş bir imzanın Rust'ta doğrulanması dahil) ve
  `src-tauri/tests/lifecycle/trust_catalog_e2e.rs`: sürüm/tür değiştirme, yeniden özetlenmiş içerik, imzada bir bayt,
  bilinmeyen algoritma, silinmiş imza ve yabancı anahtarla imzalanmış paket reddedilir.

## 5. Geliştirme değişkenleri

Yalnızca hata ayıklama derlemesinde ya da `--features dev-trust` ile okunur (`trust::dev_enabled()`); üretim derlemesi
bunları hiç okumaz:

| Değişken | Ne için |
|---|---|
| `RESPECTED_HUB_TEST_HOME` | Ürün süreçlerine ve kuruculara sahte kullanıcı klasörü (USERPROFILE, HOME, LOCALAPPDATA, APPDATA birlikte; Windows kullanıcı klasörlerini USERPROFILE'dan türettiği için kurucu ürünü de oraya kurar) |
| `RESPECTED_HUB_PRODUCTS_ROOT` | Kurulu ürünlerin aranacağı `Respected` klasörü |
| `RESPECTED_HUB_MANAGED_ROOT`, `RESPECTED_HUB_CATALOG_ROOT`, `RESPECTED_HUB_CATALOG_URL` | Hub'ın kendi klasörü, yerel katalog klasörü, katalog adresi |
| `RESPECTED_HUB_DEV_TRUST`, `RESPECTED_HUB_ALLOW_UNSIGNED` | Geliştirme anahtarı, imzasız paket |
| `RESPECTED_APP_DIR`, `RESPECTED_DATA_DIR` | Kaynaktan çalışan ürünlere geçirilen kökler |

Geliştirme sürümü katalog yoksa `<ekosistem>/catalog` klasörünü kullanır ve kardeş ürünlerin kaynağını Python ile
çalıştırabilir (Ayarlar › kaynak arama yolları). Üretimde kaynak yolları kaydedilemez ve o panel gizlidir: ele geçirilmiş
bir web görünümü Hub'ın hangi programı başlatacağını seçemez.

## 6. RPP köprüsü

### Entegrasyon IPC'si

`commands/integrations.rs` yalnızca kapalı biçimli `integrations_request` isteğini sunar. İstek gövdesi
`{ "action": "inventory" | "clients" | "catalog" | "plan" }` ile sınırlıdır; `plan` yalnızca iç içe kapalı `operation/asset_key/client/scope/project/target_scope/target_project` seçicisini taşır. Bilinmeyen eylem veya alan reddedilir. Web görünümü bu
yüzeyden dosya yolu, kabuk komutu, URL, plan digest'i ya da rastgele işlem adı geçiremez. M1/M2'de yanıt salt okunurdur; `plan` yalnızca önizleme üretir.
`inventory` beş istemcinin okuma adapter'larını birleştirir, `resolution` her binding'in kaynak yolunu, sahipliğini,
`managed` bilgisini ve `effective|overridden|disabled|unknown|untrusted` durumunu döndürür. "Etkin" yalnızca istemcinin
kendi bağlamında kullanacağı kaydı anlatır; gerçek çağrı kanıtı değildir. Yazma komutu M3 kapısı açılana kadar yoktur.

`rpp.rs`, `commands/`. Ürünler bir alt süreç olarak çağrılır: `<ürün>.exe rpp`, stdin'de bir JSON istek, stdout'ta bir
yanıt. Ağ dinleyicisi, servis ya da kabuk yoktur.

- Süreç doğrudan (`Command`, kabuksuz), açık argümanlarla, zaman aşımı ve öldürme ile, çıktı sınırıyla başlatılır. Adı sır
  gibi görünen ortam değişkenleri (key, token, secret, passw, credential, auth, cookie, session) alt sürece geçmez.
- Yerel hatalar da yanıt zarfı olur: `ENGINE_NOT_FOUND`, `ENGINE_UNAVAILABLE`, `ENGINE_TIMEOUT`, `BAD_RESPONSE`.
- Ürünler (`orchestrator`, `brain`) ve işlemler izin listesindedir; liste dışı istek süreç başlatılmadan reddedilir.
  `settings.apply`, `integrations.apply` ve `integrations.rollback` işlem kimliği ister (yoksa üretilir).
- **Web görünümü** yalnızca `rpp_request` ile okuma ve ayar işlemlerini yapabilir. Köprüde yasak olanlar: bütün `memory.*`
  işlemleri, `integrations.apply|rollback` ve `memory_` ile başlayan ayar anahtarları. Bunları yalnızca onay ekranı olan
  Hub komutları çalıştırır (`memory_access_*`, `orchestrator_mcp_*`, hafıza sayfası komutları). Kurma ve kaldırma
  `lifecycle_*` komutlarıyla yapılır.

## 7. Ürünleri birbirine bağlama

- **Brain bağlantıları** (hangi AI aracı yakalanır, MCP, zamanlayıcı, kısayol): önce Brain'in salt okunur
  `integrations.plan` önizlemesi gösterilir, onay sonrası `settings.apply`. Ürün kendi kayıtlarını aynı işlemde günceller.
- **Orchestrator'ın Brain hafızası:** Orchestrator'ın `memory.connect` / `memory.disconnect` işlemi (komut satırındaki
  `memory connect` ile aynı kod). İzni Brain verir, Orchestrator saklar; Hub anahtarı hiç görmez.
- **Orchestrator'ın AI araçlarına MCP kaydı:** Orchestrator'ın `integrations.plan` / `apply`; girdi kurulu programı gösterir
  (`...\RespectedOrchestrator\app\respected-orchestrator.exe mcp`), Hub'ın hiçbir dosyasını değil. Başka girdiler korunur.

## 8. Testler

| Test | Kapsam |
|---|---|
| `npm run check` | TypeScript tür denetimi ve arayüz birim testleri |
| `cargo test` | Rust birim testleri; `rpp_e2e` (gerçek ürün kaynakları, geçici kökler), `remote_catalog_e2e` (indirme yolu, file:// katalog), `trust_catalog_e2e`, `campaign_e2e`, `memory_index_e2e` |
| `installer_e2e` (elle, `--ignored`) | İki ürünü gerçek kurucularıyla gerçek klasörlere kurup kaldırır |
| `tests/native/hub_native_e2e.mjs` (elle) | Derlenmiş Hub'ı arayüzden sürer; kurulum, ayarlar, bağlantılar, hafıza, onarım, kaldırma |
| `common/release/tools/acceptance/accept_hub.py` | `npm run check` + `cargo test` + ortak sözleşme, gerçek AI aracı ayarları önce/sonra karşılaştırılarak |

## 9. Açık sınırlar

- Kurucular Authenticode ile imzalı değil; SmartScreen uyarır. Yalnızca Windows.
- Temiz makinede ilk kurulum, Hub'ın kendini güncellemesi ve Yüklü uygulamalar'dan
  kaldırma bir kişi tarafından denenmedi (kod ve uçtan uca testlerle kapsanıyor).
- Ürünler bir kez internetten indirilir; kurumsal vekil sunucular Windows'un `curl.exe`'sinin desteklediği kadardır.
