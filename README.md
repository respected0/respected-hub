<p align="center"><img src="assets/brand/hub.svg" width="128" alt="Respected Hub logosu"></p>

# Respected Hub

Respected ekosisteminin masaüstü uygulaması (Tauri 2 + Rust + React/TypeScript). İmzalı katalogdan **Brain** ve
**Orchestrator**'ın kendi kurucularını indirir ve çalıştırır (kur, güncelle, onar, kaldır); ürünlerin ayarlarını,
AI araçlarıyla bağlantılarını ve birbirleriyle bağlantısını görsel olarak yönetir; her ürüne bir çalışma alanı verir
(Orchestrator görevleri ve kanıtları, Brain hafıza sayfası).

Hub yalnızca kolaylıktır. Yaptığı her işin Hub'sız bir karşılığı vardır (kurucuyu çalıştırmak ya da ürünün komutu) ve
Hub o karşılığı çağırır. Ürün hakkında hiçbir şey saklamaz: kurulu olanı, sürümü, ayarları ve sağlığı her seferinde
ürünün kendisinden okur. Hub kaldırılınca ürünler olduğu gibi çalışmaya devam eder. Kurallar:
[docs/PRODUCT_CONTRACT.md](docs/PRODUCT_CONTRACT.md); ekosistem ve geliştirme ortamı: [docs/ECOSYSTEM.md](docs/ECOSYSTEM.md). İç yapı: [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md).

## Kurulum ve kaldırma

`Respected-Hub-<sürüm>-Windows-x64-Setup.exe` kullanıcı başına kurulur (`%LOCALAPPDATA%\Respected\RespectedHub\app`), yönetici izni
istemez, WebView2 gerektirir (Windows 11'de hazır). Bir ürünün kurucusunda **Respected Hub ile devam et** seçilirse kurucu
Hub'ı indirip kurar ve ürünü Hub'da açar (`--install=<ürün>`). Hub'ın sakladığı her şey kendisinindir ve kaldırınca
silinir: pencere verisi (`%LOCALAPPDATA%\com.respected.hub`), ayarlar (`%APPDATA%\com.respected.hub`), indirilen
kurucular ve işlem günlüğü (`RespectedHub\managed`). Ürünlere dokunulmaz.

Hub açılışta kataloğu denetler; yeni bir Hub ya da ürün sürümü **önerilir**, sessizce kurulmaz. Ürün güncellemesi, yeni
sürümün kurucusunu çalıştırmaktır; Hub güncellemesi doğrulanmış Hub kurucusunu `/SILENT /RELAUNCH=1` ile çalıştırır.

## Ekranlar

| Ekran | Ne var |
|---|---|
| Ana Sayfa | İlk açılışta karşılama; sonra ürünlerin gerçek durumu |
| Keşfet | Katalogdaki uygulamalar: ad, kısa tanıtım, sürüm; Kur ya da kuruluysa Kütüphanede |
| Kütüphane | Solda kurulu uygulamalar; sağda seçilenin kurulum durumu ve Güncelle / Onar / Kaldır düğmeleri, altında kendi sayfaları (Genel bakış, ürün sağlığı, neler kurulur dahil) |
| Ayarlar | İki sekme: **Genel** (kanal, güncellemeleri denetle, Hub'ın neler kurduğu; geliştirme sürümünde kaynak yolları) ve **Etkinlik** (Hub'dan yapılan kurulum işlemleri, ürünlerin son kurulum kaydı, görev geçmişi) |
| Brain | Genel bakış (ürün sağlığı burada, başka yerde değil), Hafıza (arama, okuma, hızlı not, sabah özeti), Kurulum sihirbazı, Ayarlar (şemadan form, AI aracı bağlantıları, anlamsal arama) |
| Orchestrator | Görevler, kampanyalar, kanıt, ajanlar; Ayarlar (şemadan form, Brain hafıza erişimi, AI araçlarına MCP kaydı) |

## Geliştirme

Node 22.12+, Rust/Cargo ve [Tauri gereksinimleri](https://v2.tauri.app/start/prerequisites/). Uçtan uca testler kardeş
ürünleri derler; üç depo yan yana durmalıdır ([docs/ECOSYSTEM.md](docs/ECOSYSTEM.md)).

```powershell
npm ci
npm run check                                     # tsc + arayüz birim testleri
cargo test --manifest-path src-tauri/Cargo.toml   # Rust birim + uçtan uca (ürünleri geçici köklerde çalıştırır)
python release/tools/acceptance/accept_hub.py     # ikisi + ortak sözleşme; gerçek AI aracı ayarlarının değişmediğini denetler
npm run tauri -- dev                              # geliştirme penceresi
```

`npm run dev` yalnızca tarayıcı önizlemesidir: yerel köprü yoktur, motor işlemleri kapalıdır (taklit edilmez).
`npm run preview:update` tasarım içindir: uygulamaya girmeyen `dev/preview-update.html` sayfası, Brain'in güncellemesi varmış gibi ekranları gösterir; içindeki her veri uydurmadır ve hiçbir şey kurmaz.
Geliştirme sürümü kardeş ürünlerin kaynağını Python ile çalıştırabilir (Ayarlar › Ürün durumu); üretim sürümü yalnızca
kurulu ürünleri çalıştırır.

**Gerçek kurulum yapan testler** (elle; ikisi de makinede kurulu bir ürün varsa başlamaz ve kurduklarını kaldırır):

```powershell
cargo test --manifest-path src-tauri/Cargo.toml --test installer_e2e -- --ignored --nocapture
npx tauri build --debug --no-bundle
node tests/native/hub_native_e2e.mjs src-tauri/target/debug/respected-hub.exe
```

`installer_e2e` iki ürünü gerçek kurucularıyla `%LOCALAPPDATA%\Respected` altına kurar ve kaldırır. Arayüz sürücüsü
derlenmiş Hub'ı WebView2'nin hata ayıklama portundan tıklayarak sürer (PATH'te Python yokken): iki ürünü kurar, sihirbazı
doldurur, ayar değiştirir, bağlantıları açıp kapatır, not yakalar ve arar, bozulmuş programı onarır, kaldırır. Ürünler
sahte bir kullanıcı klasörüne kurulur; gerçek makinede yalnızca Yüklü uygulamalar girdisi oluşur ve sonunda silinir.
Rapor ve ekran görüntüleri `.build-cache/hub-native/` altına yazılır. `--intent=brain|orchestrator` ürün kurucusunun
Hub'ı açtığı yolu dener; `--remote-catalog` indirme yolunu; `--real-semantic` anlamsal aramanın gerçek indirmesini.

## Düzen

| Yol | İçerik |
|---|---|
| `src/shell/` | Uygulama çerçevesi, gezinme, Ana Sayfa, Keşfet, Kütüphane, Ayarlar (Genel, Ürün durumu, Etkinlik sekmeleri), kurulum düğmeleri |
| `src/products/brain/`, `src/products/orchestrator/` | Ürün çalışma alanları ve saf modelleri |
| `src/shared/` | RPP istemcisi, yerel köprü, tasarım sistemi |
| `src-tauri/src/hub/` | Kurulu ürünleri bulma, kurucuyu çalıştırma, açılış niyetleri |
| `src-tauri/src/lifecycle/` | Katalog, indirme, paket imzası, işlem günlüğü |
| `src-tauri/src/rpp.rs`, `trust.rs`, `commands/` | RPP köprüsü, imza güveni, Tauri komutları |
| `src-tauri/tests/` | Uçtan uca testler (geçici kökler, sahte kullanıcı klasörü) |
| `tests/` | TypeScript birim testleri; `tests/native/` arayüz sürücüsü |
| `packaging/windows/hub_setup.iss` | Hub kurucusu |
| `release/` | Derleme, imzalama ve yayın ([release/RELEASE.md](release/RELEASE.md)) |

## Lisans

[MIT lisansı](LICENSE).
