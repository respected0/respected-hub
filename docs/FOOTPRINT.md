# Hub: ne yaparsan nerede ne oluşur

Bu belge, Hub'ı kurduğunda, ürün kurup kaldırdığında ve güncellediğinde **bilgisayarında hangi dosyanın nerede, hangi şartla ve
neden oluştuğunu** anlatır. Kod okunarak ve gerçek kurucu ile derlenmiş Hub boş bir kullanıcı klasöründe çalıştırılarak
doğrulandı (2026-10-24, `0.1.0-beta.1`). Brain ve Orchestrator'ın kendi dosyaları için onların `FOOTPRINT.md` belgelerine bak:
[Brain](https://github.com/respected0/respected-brain/blob/main/docs/FOOTPRINT.md) ·
[Orchestrator](https://github.com/respected0/respected-orchestrator/blob/main/docs/FOOTPRINT.md).

## 1. Hub ne yapar, ne yapmaz

- Hub **bir yönetici ve görüntüleyicidir.** Ürünleri kendi başına değiştirmez: bir ürünü kurmak, güncellemek, onarmak ya da
  kaldırmak için o ürünün **kendi imzalı kurucusunu / kaldırıcısını** sessiz modda çalıştırır.
- Hub **AI araçlarının hiçbir dosyasına yazmaz** (`.claude`, `.codex`, `.gemini` vb.). Bunları Brain ve Orchestrator yazar;
  Hub yalnızca "şunu uygula" der ve ürünün komutunu çağırır.
- Hub **projelerine ve not kasana yazmaz.** Brain sayfasında kasadan son kayıtları yerelde okuyup gösterir.
- Hub ürünler hakkında veri saklamaz; "kurulu mu, hangi sürüm" bilgisini ürünlerin kendisinden ve "Yüklü uygulamalar" kayıtlarından okur.

## 2. Tek bakışta

| Ne | Nerede | Kim oluşturur | Ne zaman |
|---|---|---|---|
| Program (`respected-hub.exe`, kaldırıcı) | `%LOCALAPPDATA%\Respected\RespectedHub\app` | Kurucu | Kurulumda |
| Windows "Yüklü uygulamalar" kaydı | `HKCU\Software\Microsoft\Windows\CurrentVersion\Uninstall\{5C0F6A1E-3B7D-4E2A-9D55-7A1C2F0E8B31}_is1` | Kurucu | Kurulumda |
| Başlat menüsü kısayolu | `Respected Hub` (Başlat menüsü programlar) | Kurucu | Kurulumda |
| Masaüstü kısayolu | `Masaüstü\Respected Hub` | Kurucu | Kurulumda |
| İndirme önbelleği, katalog, tercihler, etkinlik günlüğü | `%LOCALAPPDATA%\Respected\RespectedHub\managed` | Hub | İlk açılışta ve her denetim/indirmede |
| Pencere (WebView2) verisi | `%LOCALAPPDATA%\com.respected.hub` ve `%APPDATA%\com.respected.hub` | Windows WebView2 / Tauri | İlk açılışta |

Başka hiçbir yere dosya, kayıt defteri girdisi ya da zamanlanmış görev yazmaz.

## 3. Kurulum

1. Kurucu önce **beta bildirimini** gösterir (imzasız kurucu uyarısı, neyin kurulup neye dokunulmayacağı).
2. Programı `app` klasörüne açar, iki kısayolu ve "Yüklü uygulamalar" kaydını yazar. Yönetici izni istemez.
   `%TEMP%` içinde kısa süreli `is-XXXX.tmp` klasörü kullanır ve siler.
3. **Microsoft Edge WebView2** yoksa (Windows 11'de hazır gelir) kurucu seni uyarır; Hub arayüzü onsuz açılmaz. Kurucu
   WebView2'yi **kendisi indirmez**, karar sana kalır.
4. Sonunda "Respected Hub'ı başlat" seçeneği (varsayılan işaretli).

## 4. Hub'ın kendi klasörü: `managed`

| Dosya / klasör | Ne için |
|---|---|
| `catalog-index.json` ve `.sig` | İmzalı ürün kataloğunun son indirilen hâli; imza doğrulanmadan kullanılmaz |
| `hub-preferences.json` | Seçtiğin güncelleme kanalı (`beta` / `stable`) |
| `catalog\` | Katalogdan indirilen ürün kurucuları (imza ve özet doğrulanmış paketler), çevrimdışı onarım için |
| `installer-staging\<zaman>\` | Bir ürünü kurarken kurucunun **geçici** kopyası; iş bitince silinir |
| `downloads\` | Hub'ın kendi güncellemesi (`RespectedHub-Setup-<sürüm>.exe`) |
| etkinlik günlüğü | Etkinlik sayfasında gördüğün işlem kayıtları |

**Güncelleme ve ürün indirmeleri** Windows'un kendi `curl.exe` programıyla yalnızca `github.com/respected0` altındaki adreslerden
yapılır. İndirilen dosyanın imzası ve SHA-256 özeti katalogla eşleşmezse **çalıştırılmaz** (Hub güncellemesinde ayrıca silinir).

**Pencere verisi:** "Yenilikler" penceresinin hangi sürümde gösterildiği gibi küçük ayarlar WebView2'nin kendi
`localStorage` alanında durur (`com.respected.hub` klasörleri).

## 5. Ürün kurunca / kaldırınca

| Sen | Hub ne yapar | Sonuç |
|---|---|---|
| Brain ya da Orchestrator'ı kur | Kurucuyu `catalog`'dan alır, imzayı ve özeti doğrular, `installer-staging`'e kopyalar, sessiz çalıştırır (`/VERYSILENT /SUPPRESSMSGBOXES /NORESTART`; Brain için `/VAULT=` ve seçtiğin ayarlar `/SET=anahtar=değer`) | Ürünü kendi kurucusu kurar; sonuç, kurucuyu elle çalıştırmakla aynıdır. Ne oluştuğu için ürünün `FOOTPRINT.md` belgesi |
| Ürünün ayarlarını değiştir | Ürünün kendi `settings.apply` komutunu çağırır | AI araçlarındaki değişiklikleri **ürün** yapar; Hub önce "Değişecekler" önizlemesini gösterir, hiçbir şey yazmaz |
| Ürünü güncelle / onar | Yeni kurucuyu aynı biçimde çalıştırır | Ürünün güncelleme yedeği ve geri alma kuralları geçerli |
| Ürünü kaldır | Ürünün `unins000.exe` kaldırıcısını çalıştırır (isteğe bağlı `/PURGE=1`) | Ürün kendi girdilerini AI araçlarından geri alır; **kasa ve projeler silinmez** |
| Hub'ı güncelle | Yeni `RespectedHub-Setup-<sürüm>.exe` dosyasını indirir, doğrular, `/SILENT /NOCANCEL /RELAUNCH=1` ile çalıştırır | Hub kapanır, güncellenir, yeniden açılır |

Hub, kurmadan, güncellemeden ve kaldırmadan önce **sana sorar**; sessizce güncelleme yapmaz.

## 6. Hub'ı kaldırma

Windows ayarlarından ya da kurulum klasöründeki kaldırıcıyla: programı, kısayolları, "Yüklü uygulamalar" kaydını, `managed`
klasörünü (önbellek ve günlük) ve iki `com.respected.hub` klasörünü siler. `%LOCALAPPDATA%\Respected` klasörü, içinde Brain ya da
Orchestrator varsa **kalır**. **Kurduğun ürünlere hiç dokunulmaz;** Hub'ı sonradan yeniden kurarsan onları olduğu gibi bulur.

## 7. Sık sorulanlar

**Hub Claude/Codex ayarlarıma bir şey yazıyor mu?** Hayır. Yazan Brain ve Orchestrator'dır, o da yalnızca sen bağlantıyı açtıysan
ve araç bilgisayarında kuruluysa. Hub yalnızca önizleme gösterir ve "uygula" dediğinde ürünün komutunu çağırır.

**`Temp` klasöründeki dosyalar ne?** `is-XXXX.tmp` kurucunun geçici açılımıdır; bitince silinir. Hub ve ürünler
dosyaları güvenli yazmak için hedefin yanına geçici `.tmp` açıp yer değiştirir; işlem sonunda kalmaz.

**Ne gönderiliyor?** Bkz. [PRIVACY.md](../PRIVACY.md): Hub yalnızca GitHub'dan katalog ve kurucu indirir; telemetri yoktur.
