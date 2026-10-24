# Hub mimarisi

Hub'ın ürünleri nasıl bulduğu, kurucuları nasıl indirip çalıştırdığı, neye güvendiği ve web görünümüne neyi yasakladığı.
Giriş: [README.md](../README.md). Yayın: [release/RELEASE.md](../release/RELEASE.md). Ortak kurallar
[PRODUCT_CONTRACT.md](PRODUCT_CONTRACT.md) dosyasındadır; burada tekrarlanmaz.

## 1. Ürünleri bulma ve durum

- Bir ürün, standart klasöründe (`%LOCALAPPDATA%\Respected\<Ürün>\app`) programı ve `distribution.json`'ı varsa kuruludur
  (`hub/installed.rs`). Sürüm `distribution.json`'dan, durum ürünün kendi `manifest` yanıtından gelir: ürün
  `repair-needed` derse kart "Onarım gerekli" gösterir; yanıt yoksa da öyle. Katalogda daha yeni sürüm varsa
  "Güncelleme var".
- Ayarlar, sağlık, hafıza ve bağlantılar her seferinde ürünün `rpp` komutuyla sorulur. Hub ürünlere klasör geçirmez;
  ürün kendi klasörlerini bilir.

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

`trust.rs`, imzalama `release/tools/signing/bundle_signing.py`.

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
| `release/tools/acceptance/accept_hub.py` | `npm run check` + `cargo test` + ortak sözleşme, gerçek AI aracı ayarları önce/sonra karşılaştırılarak |

## 9. Açık sınırlar

- Kurucular Authenticode ile imzalı değil; SmartScreen uyarır. Yalnızca Windows.
- Temiz makinede ilk kurulum, Hub'ın kendini güncellemesi ve Yüklü uygulamalar'dan
  kaldırma bir kişi tarafından denenmedi (kod ve uçtan uca testlerle kapsanıyor).
- Ürünler bir kez internetten indirilir; kurumsal vekil sunucular Windows'un `curl.exe`'sinin desteklediği kadardır.
