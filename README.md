<div align="center">

<img src="assets/brand/hub.svg" width="128" alt="Respected Hub logosu">

# Respected Hub

### Respected ekosisteminin masaüstü uygulaması.

Brain'i ve Orchestrator'ı tek yerden kur, güncelle, onar, ayarla.

<br>

[![Sürüm](https://img.shields.io/github/v/release/respected0/respected-hub?include_prereleases&sort=semver&style=flat-square&color=4f9cff)](https://github.com/respected0/respected-hub/releases)
[![Lisans: MIT](https://img.shields.io/badge/lisans-MIT-a3e635?style=flat-square)](LICENSE)
![Platform](https://img.shields.io/badge/platform-Windows-60a5fa?style=flat-square)
![Tauri 2](https://img.shields.io/badge/Tauri-2-FFC131?style=flat-square&logo=tauri&logoColor=white)
![Rust](https://img.shields.io/badge/Rust-000000?style=flat-square&logo=rust&logoColor=white)
![React](https://img.shields.io/badge/React-TypeScript-38bdf8?style=flat-square&logo=react&logoColor=white)

**Yalnızca kolaylık** · **İmzalı kurucular** · **Hub'sız da çalışır**

[Kurulum](#kurulum) · [Ne yapar](#ne-yapar) · [Ekranlar](#ekranlar) · [Güven](#güven-ve-sınırlar) · [Geliştirme](#geliştirme) · [Belgeler](#belgeler)

</div>

---

## Neden Hub?

Respected ekosistemi üç ayrı ürün: **Hub**, **[Brain](https://github.com/respected0/respected-brain)** (ortak hafıza) ve
**[Orchestrator](https://github.com/respected0/respected-orchestrator)** (yerel AI görev motoru). Her biri kendi başına kurulur ve çalışır.
Hub, bunları komut satırı bilmeden yönetmeniz için var:

- **Tek yerden yönetim.** Kur, güncelle, onar, kaldır; ürünlerin ayarlarını ve AI araçlarıyla bağlantılarını görsel olarak değiştir.
- **Her ürüne bir çalışma alanı.** Orchestrator görevleri ve kanıtları, Brain'in hafıza sayfası.
- **Öneri var, dayatma yok.** Yeni sürümler önerilir, sessizce kurulmaz.

> **Hub yalnızca kolaylıktır.** Yaptığı her işin Hub'sız bir karşılığı vardır (ürünün kurucusunu ya da komutunu çalıştırmak) ve Hub
> o karşılığı çağırır. Hub kaldırılınca ürünler olduğu gibi çalışmaya devam eder.

## Ne yapar

| | |
|---|---|
| **Kurulum yaşam döngüsü** | İmzalı katalogdan ürünün kendi kurucusunu indirir, doğrular ve çalıştırır: kur, güncelle, onar, kaldır |
| **Güncelleme kanalları** | `beta` (varsayılan) ve `stable`; Ayarlar'dan seçilir. Güncelleme önerilir, siz onaylamadan uygulanmaz |
| **Ürün çalışma alanları** | Orchestrator için görevler, kampanyalar, kanıt, ajanlar; Brain için hafıza (arama, okuma, hızlı not, sabah özeti) |
| **Ayarlar ve bağlantılar** | Ürünün kendi şemasından form; AI araçları bağlantıları; Orchestrator'ı Brain hafızasına bağlama |
| **Etkinlik kaydı** | Hub'dan yapılan kurulum işlemleri, ürünlerin son kurulum kaydı, görev geçmişi |

Ürün hakkında hiçbir şey saklanmaz: kurulu olan, sürüm, ayarlar ve sağlık her seferinde ürünün kendisinden okunur.

## Kurulum

`Respected-Hub-<sürüm>-Windows-x64-Setup.exe` dosyasını [Releases](https://github.com/respected0/respected-hub/releases) sayfasından indirip
çalıştırın. Kurulum kullanıcı başınadır (`%LOCALAPPDATA%\Respected\RespectedHub\app`), **yönetici izni istemez**, WebView2 gerektirir
(Windows 11'de hazırdır).

Başka bir yol: Brain ya da Orchestrator'ın kurucusunda **Respected Hub ile devam et**'i seçin. Kurucu Hub'ı indirip kurar ve ürünü
Hub'da açar.

<details>
<summary><b>Hub neyi nerede saklar, kaldırınca ne olur?</b></summary>

<br>

Hub'ın sakladığı her şey kendisinindir ve kaldırınca silinir:

| Ne | Nerede |
|---|---|
| Pencere verisi | `%LOCALAPPDATA%\com.respected.hub` |
| Ayarlar | `%APPDATA%\com.respected.hub` |
| İndirilen kurucular ve işlem günlüğü | `RespectedHub\managed` |

Ürünlere dokunulmaz.

</details>

**Güncelleme.** Hub açılışta kataloğu denetler. Ürün güncellemesi, yeni sürümün kurucusunu çalıştırmaktır; Hub güncellemesi doğrulanmış
Hub kurucusunu `/SILENT /RELAUNCH=1` ile çalıştırır.

## Ekranlar

| Ekran | Ne var |
|---|---|
| **Ana Sayfa** | İlk açılışta karşılama; sonra ürünlerin gerçek durumu |
| **Keşfet** | Katalogdaki uygulamalar: ad, kısa tanıtım, sürüm; Kur ya da kuruluysa Kütüphane'de |
| **Kütüphane** | Solda kurulu uygulamalar; sağda seçilenin kurulum durumu ve Güncelle / Onar / Kaldır düğmeleri, altında kendi sayfaları. Ürün güncellemesi olduğunda rozet burada çıkar |
| **Ayarlar** | **Genel** (kanal, güncellemeleri denetle, Hub'ın neler kurduğu) ve **Etkinlik**. Hub güncellemesi olduğunda rozet burada çıkar |
| **Brain** | Genel bakış (ürün sağlığı burada), Hafıza, Kurulum sihirbazı, Ayarlar (AI aracı bağlantıları, anlamsal arama) |
| **Orchestrator** | Görevler, kampanyalar, kanıt, ajanlar; Ayarlar (Brain hafıza erişimi, AI araçlarına MCP kaydı) |

## Güven ve sınırlar

- Hub yalnızca ürünün **kendi imzalı kurucusunu ya da kaldırıcısını** çalıştırır. Genel bir komut satırı arayüzü ya da keyfi kabuk çalıştırma yoktur.
- Katalog ve indirilen paketler imza ve özet ile doğrulanır; doğrulanmayan hiçbir şey kurulmaz.
- Bir adım başarısız olursa ürün kendi değişikliklerini geri alır; sizin değiştirdiğiniz bir kayıt üzerine yazılmaz.
- Hub, ürünlere veri klasörü geçirmez ve yönetilen kopya tutmaz.
- Orchestrator çalışmaları Hub kapansa da sürer; görev durumu ve iptal motorundadır.

Bağlayıcı kurallar: [docs/PRODUCT_CONTRACT.md](docs/PRODUCT_CONTRACT.md), iç yapı: [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md).

## Geliştirme

Node 22.12+, Rust/Cargo ve [Tauri gereksinimleri](https://v2.tauri.app/start/prerequisites/). Uçtan uca testler kardeş ürünleri derler;
üç depo yan yana durmalıdır ([docs/ECOSYSTEM.md](docs/ECOSYSTEM.md)).

```powershell
npm ci
npm run check                                     # tsc + arayüz birim testleri
cargo test --manifest-path src-tauri/Cargo.toml   # Rust birim + uçtan uca (ürünleri geçici köklerde çalıştırır)
python release/tools/acceptance/accept_hub.py     # ikisi + ortak sözleşme; gerçek AI aracı ayarlarının değişmediğini denetler
npm run tauri -- dev                              # geliştirme penceresi
```

`npm run dev` yalnızca tarayıcı önizlemesidir: yerel köprü yoktur, motor işlemleri kapalıdır (taklit edilmez).
`npm run preview:update` tasarım içindir: `tools/preview-update.html`, Brain'in güncellemesi varmış gibi ekranları gösterir; içindeki her veri
uydurmadır ve hiçbir şey kurmaz. Geliştirme sürümü kardeş ürünlerin kaynağını Python ile çalıştırabilir; üretim sürümü yalnızca kurulu ürünleri çalıştırır.

<details>
<summary><b>Gerçek kurulum yapan testler</b></summary>

<br>

Elle çalıştırılır; ikisi de makinede kurulu bir ürün varsa başlamaz ve kurduklarını kaldırır.

```powershell
cargo test --manifest-path src-tauri/Cargo.toml --test installer_e2e -- --ignored --nocapture
npx tauri build --debug --no-bundle
node tests/native/hub_native_e2e.mjs src-tauri/target/debug/respected-hub.exe
```

`installer_e2e` iki ürünü gerçek kurucularıyla `%LOCALAPPDATA%\Respected` altına kurar ve kaldırır. Arayüz sürücüsü derlenmiş Hub'ı
WebView2'nin hata ayıklama portundan tıklayarak sürer (PATH'te Python yokken): iki ürünü kurar, sihirbazı doldurur, ayar değiştirir,
bağlantıları açıp kapatır, not yakalar ve arar, bozulmuş programı onarır, kaldırır. Ürünler sahte bir kullanıcı klasörüne kurulur; gerçek
makinede yalnızca Yüklü uygulamalar girdisi oluşur ve sonunda silinir. Rapor ve ekran görüntüleri `.build-cache/hub-native/` altına yazılır.
`--intent=brain|orchestrator` ürün kurucusunun Hub'ı açtığı yolu dener; `--remote-catalog` indirme yolunu; `--real-semantic` anlamsal aramanın
gerçek indirmesini.

</details>

<details>
<summary><b>Depo düzeni</b></summary>

<br>

| Yol | İçerik |
|---|---|
| `src/shell/` | Uygulama çerçevesi, gezinme, Ana Sayfa, Keşfet, Kütüphane, Ayarlar, kurulum düğmeleri |
| `src/products/brain/`, `src/products/orchestrator/` | Ürün çalışma alanları ve saf modelleri |
| `src/shared/` | RPP istemcisi, yerel köprü, tasarım sistemi |
| `src-tauri/src/hub/` | Kurulu ürünleri bulma, kurucuyu çalıştırma, açılış niyetleri |
| `src-tauri/src/lifecycle/` | Katalog, indirme, paket imzası, işlem günlüğü |
| `src-tauri/src/rpp.rs`, `trust.rs`, `commands/` | RPP köprüsü, imza güveni, Tauri komutları |
| `src-tauri/tests/` | Uçtan uca testler (geçici kökler, sahte kullanıcı klasörü) |
| `tests/` | TypeScript birim testleri; `tests/native/` arayüz sürücüsü |
| `tools/` | Geliştirici araçları: `conformance/`, `measure-rpp.py`, `brand/`, `preview-update.html` |
| `packaging/windows/hub_setup.iss` | Hub kurucusu |
| `release/` | Derleme, imzalama ve yayın ([release/RELEASE.md](release/RELEASE.md)) |

</details>

## Belgeler

| Belge | Konu |
|---|---|
| [PRODUCT_CONTRACT.md](docs/PRODUCT_CONTRACT.md) | Üç ürünün ortak sözleşmesi |
| [ECOSYSTEM.md](docs/ECOSYSTEM.md) | Ekosistem ve geliştirme ortamı |
| [ARCHITECTURE.md](docs/ARCHITECTURE.md) | İç yapı ve bağlayıcı kurallar |
| [ROADMAP.md](docs/ROADMAP.md) | Yol haritası |
| [RELEASE.md](release/RELEASE.md) | Derleme ve yayın |

## Destek

Hata bildirimi ve sorular için [GitHub Issues](https://github.com/respected0/respected-hub/issues).

## Lisans

[MIT lisansı](LICENSE).
