# Gizlilik: Respected Hub

Bu metin Hub `0.1.0-beta.1` için geçerlidir (2026-10-24). Kod okunarak yazıldı; kod değişirse metin de değişir.

## Kısaca

- Hub **hesap istemez, telemetri ya da kullanım istatistiği göndermez**, Respected'e hiçbir veri iletmez.
- Hub'ın arayüzü, içerik güvenlik kuralı gereği ağa doğrudan bağlanamaz; ağ işlerini yalnızca Hub'ın kendi imzalı yolu yapar.
- Hub ürünler hakkında veri saklamaz: kurulu olanı ürünlerin kendisinden okur.

## Bilgisayarınızda ne saklanır

| Ne | Nerede |
|---|---|
| Hub programı | `%LOCALAPPDATA%\Respected\RespectedHub\app` |
| Güncelleme kanalı tercihi, indirilen katalog ve imzası | `%LOCALAPPDATA%\Respected\RespectedHub\managed` |
| Pencere ve WebView2 verisi | `%LOCALAPPDATA%\com.respected.hub`, `%APPDATA%\com.respected.hub` |

Hub, Brain ve Orchestrator'ın durumunu ve (Brain sayfasında) notlarınızdan son kayıtları **yerelde okuyup gösterir**; bunları
bir yere göndermez. Ürünlerin ne yaptığı için kendi gizlilik metinlerine bakın:
[Brain](https://github.com/respected0/respected-brain/blob/main/PRIVACY.md) ·
[Orchestrator](https://github.com/respected0/respected-orchestrator/blob/main/PRIVACY.md).

## Bilgisayardan ne çıkar, kime gider

| Ne zaman | Ne gider | Nereye |
|---|---|---|
| Güncelleme denetiminde | Sıradan bir indirme isteği (hiçbir kişisel veri eklenmez) | `github.com/respected0/respected-hub` sürümleri: imzalı ürün kataloğu |
| Siz bir ürünü kurmayı, güncellemeyi ya da onarmayı onayladığınızda | Sıradan bir indirme isteği | Katalogda adı geçen GitHub sürüm dosyası; indirilen dosya imza ve özetle doğrulanmadan çalıştırılmaz |

GitHub, her web isteğinde olduğu gibi IP adresinizi görür ve isteği kendi gizlilik politikasına göre kaydedebilir. Hub bunun
dışında bir şey göndermez.

## Dürüst sınırlar

- Kurucular Authenticode ile imzalı değildir; Windows uyarı gösterebilir. İndirdiğiniz dosyanın özetini `SHA256SUMS.txt` ile karşılaştırın.
- Bu metin hukuki danışmanlık ya da sözleşme metni değildir; neyin nereye gittiğini anlatır.

## Silme

Hub'ı kaldırmak yalnızca Hub'ın programını ve kendi klasörlerini siler; **kurduğu ürünlere dokunmaz**.

## Soru ve bildirim

[GitHub Issues](https://github.com/respected0/respected-hub/issues)
