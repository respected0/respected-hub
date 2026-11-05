# Gizlilik: Respected Hub

Bu metin Hub `0.1.0-beta.2` için geçerlidir (2026-11-06). Kod okunarak yazıldı; kod değişirse metin de değişir.

## Kısaca

- Hub **hesap istemez, telemetri ya da kullanım istatistiği göndermez**, Respected'e hiçbir veri iletmez.
- Hub'ın arayüzü, içerik güvenlik kuralı gereği ağa doğrudan bağlanamaz; ağ işlerini yalnızca Hub'ın kendi imzalı yolu yapar.
- Hub ürünler hakkında veri saklamaz: kurulu olanı ürünlerin kendisinden okur.

## Bilgisayarınızda ne saklanır

| Ne | Nerede |
|---|---|
| Hub programı | `%LOCALAPPDATA%\Respected\RespectedHub\app` |
| Güncelleme kanalı tercihi, indirilen katalog ve imzası | `%LOCALAPPDATA%\Respected\RespectedHub\managed` |
| Üçüncü taraf katalog önbelleği (imzalı `catalog.json` ve imzası) | `%LOCALAPPDATA%\Respected\RespectedHub\managed\integrations\third-party-catalog` |
| Envanter karşılaştırma kaydı (`integration-baseline.json`), sizin eklediğiniz proje klasörlerinin yolları (`projects.json`), profiller (`profiles.json`); yapılandırma içeriği ya da sır saklanmaz | `%APPDATA%\com.respected.hub` |
| Pencere ve WebView2 verisi | `%LOCALAPPDATA%\com.respected.hub`, `%APPDATA%\com.respected.hub` |

Hub, Brain ve Orchestrator'ın durumunu ve (Brain sayfasında) notlarınızdan son kayıtları **yerelde okuyup gösterir**; bunları
bir yere göndermez. Ürünlerin ne yaptığı için kendi gizlilik metinlerine bakın:
[Brain](https://github.com/respected0/respected-brain/blob/main/PRIVACY.md) ·
[Orchestrator](https://github.com/respected0/respected-orchestrator/blob/main/PRIVACY.md).

Hub, üçüncü taraf AI istemcilerinin MCP/Skill/Plugin yapılandırmalarını yalnızca **yerelde ve salt okunur** olarak
tarar; dosyaların kopyasını saklamaz, dışarı göndermez. Tespit edilen token, parola ve benzeri değerler arayüzde
redakte edilir; ${DATABASE_URL} gibi ortam değişkeni referansları gösterilir, değerleri gösterilmez. Tarama yalnızca sınırlı derinlikte ve boyutta okur; symlink hedefi kökün dışına çıkıyorsa ya da döngü oluşturuyorsa izlenmez ve kontrollü bir uyarı olarak raporlanır. Bozuk yapılandırma dosyasının içeriği gösterilmez. Ayrıntı: [ADR](docs/ADR-THIRD-PARTY-INTEGRATIONS.md).

## Bilgisayardan ne çıkar, kime gider

| Ne zaman | Ne gider | Nereye |
|---|---|---|
| Güncelleme denetiminde | Sıradan bir indirme isteği (hiçbir kişisel veri eklenmez) | `github.com/respected0/respected-hub` sürümleri: imzalı ürün kataloğu |
| Siz Keşfet'te katalog tazeleme düğmesine bastığınızda (kendiliğinden hiç) | Sıradan bir indirme isteği | `github.com/respected0/respected-hub` sürümleri: imzalı üçüncü taraf katalog; imza doğrulanmazsa kullanılmaz |
| Siz bir ürünü kurmayı, güncellemeyi ya da onarmayı onayladığınızda | Sıradan bir indirme isteği | Katalogda adı geçen GitHub sürüm dosyası; indirilen dosya imza ve özetle doğrulanmadan çalıştırılmaz |

GitHub, her web isteğinde olduğu gibi IP adresinizi görür ve isteği kendi gizlilik politikasına göre kaydedebilir. Hub bunun
dışında bir şey göndermez.

## Dürüst sınırlar

- Kurucular Authenticode ile imzalı değildir; Windows uyarı gösterebilir. İndirdiğiniz dosyanın özetini sürüm sayfasında dosyanın yanında GitHub'ın gösterdiği SHA-256 değeriyle karşılaştırın.
- Bu metin hukuki danışmanlık ya da sözleşme metni değildir; neyin nereye gittiğini anlatır.

## Silme

Hub'ı kaldırmak yalnızca Hub'ın programını ve kendi klasörlerini siler; **kurduğu ürünlere dokunmaz**.

## Soru ve bildirim

[GitHub Issues](https://github.com/respected0/respected-hub/issues)
