# Derleme, imzalama ve yayın

Windows betasını derleyen, imzalayan ve yayınlayan her şey bu klasördedir. Betikler üç deponun yan yana durduğunu varsayar
(`release/tools/common/ecosystem_root.py` üst klasörü bulur; ya da `RESPECTED_ECOSYSTEM_ROOT`). Çıktı `<ekosistem>/dist`,
önbellek `<ekosistem>/.build-cache`. Geliştirme ortamının kurulumu: [../docs/ECOSYSTEM.md](../docs/ECOSYSTEM.md).

## 1. Sıfırdan başlamak

```text
git clone https://github.com/respected0/respected-hub.git
git clone https://github.com/respected0/respected-brain.git
git clone https://github.com/respected0/respected-orchestrator.git
```

Üçünü aynı klasöre koyun. Depoya girmeyen ve yeniden üretilen klasörler: `dist\` (kurucular), `.build-cache\`,
`.cargo-target-hub\` (Rust çıktısı, birkaç GB; `CARGO_TARGET_DIR` ile seçilir), isteğe bağlı `catalog\` (yerel katalog, yalnızca geliştirme).

**Yalnızca bunu yedekleyin:** `%USERPROFILE%\.respected-release\respected-release.vault` ve parolası. Katalogu ve her
paketi imzalayan yayın anahtarı odur; kaybolursa kurulu Hub'lar artık güncelleme alamaz (yalnızca bu anahtara güvenirler)
ve insanlar yeni Hub'ı elle indirmek zorunda kalır. Anahtar kimliği `dabcb5265227bbdf`, açık satırı
`src-tauri/release-keys.txt`. Depoda anahtar malzemesi yoktur.

## 2. Araçlar

| Betik | Ne yapar |
|---|---|
| `tools/build/build_bundle.py orchestrator\|brain [--out DİZİN] [--key DOSYA] [--force]` | Ürünün kendi `packaging/build_installer.py`'si ile `<Ürün>-Windows-Setup.exe` üretir (kaynak değişmediyse yeniden kullanır) ve onu imzalı katalog paketine sarar: `<ürün>-<sürüm>/bundle.json` + `payload/<Ürün>-Windows-Setup.exe`. Anahtar verilmezse yerel geliştirme anahtarıyla imzalar. |
| `tools/build/build_hub_installer.py [--skip-app] [--release-key DOSYA]` | Hub kurucusu. Varsayılan GELİŞTİRME kurucusudur (yerel geliştirme anahtarına güvenir, dağıtılmaz); `--release-key` ile ÜRETİM kurucusu (dev-trust yok, geliştirme anahtarı yok). |
| `tools/build/publish_release.py build\|catalog\|upload\|local-catalog` | Yayın (aşağıda); `local-catalog` testler için file:// adresli, geliştirme imzalı katalog. |
| `tools/build/release_package.py build\|verify` | İndirme klasörü: kurucular + `SHA256SUMS.txt` + `SHA256SUMS.sig` + `BETA_NOTICE.txt`. |
| `tools/acceptance/accept_hub.py`, `real_env_guard.py` | Yerel kabul; gerçek AI aracı ayarlarının önce/sonra aynı olduğunu denetler. |
| `tools/signing/` | `bundle_signing.py`, `release_keys.py`, `release_vault.py`: anahtar saklama ve imzalama. |
| `tests/test_release_tools.py` | Araçların kendi testi (CI'da çalışır). |
| `BETA_NOTICE.txt` | Hub kurucusunun kurmadan önce gösterdiği sayfa. |
| `docs/RELEASE_NOTES_BETA.md` | GitHub sürüm sayfalarının metni (`upload` kullanır). |

## 3. Sürüm

Her şey için tek sürüm (`0.1.0-beta.1`). Değiştirilecek yerler: `respected-hub/src-tauri/tauri.conf.json`, `Cargo.toml`,
`package.json`, `packaging/windows/hub_setup.iss` varsayılanı; `respected-brain/pyproject.toml`;
`respected-orchestrator/pyproject.toml`; ayrıca iki ürünün `__init__.py` dosyasındaki `__version__` (PEP 440 yazımı, ör. `0.1.0b2`; paket bilgisinden okunmaz çünkü her çağrıda ~200 ms tutuyordu, iki depodaki sürüm testi uyuşmazlığı yakalar). Sonra iki ürünü düzenlenebilir olarak
yeniden kurun (`pip install -e`). Sürümler SemVer'dir; "beta" sürümün parçasıdır (`0.1.0-beta.2`, `0.1.0-rc.1`, `0.1.0`).
Bir yayın her zaman iki ürünü güncel kaynaktan yeniden dondurur.

## 4. Yayınlamak

```powershell
$k = "$env:USERPROFILE\.respected-release\respected-release.vault"
$s = Read-Host "Kasa parolası" -AsSecureString
$env:RESPECTED_VAULT_PASSPHRASE = [Runtime.InteropServices.Marshal]::PtrToStringBSTR([Runtime.InteropServices.Marshal]::SecureStringToBSTR($s))
python respected-hub\release\tools\build\publish_release.py build --key $k       # dist\release: kurucular, imzalı arşivler
python respected-hub\release\tools\build\publish_release.py catalog --key $k     # yayındaki katalogla birleştirir, imzalar
python respected-hub\release\tools\build\release_package.py build --key $k --dir dist\release
python respected-hub\release\tools\build\publish_release.py upload               # gh komutlarını yazar; yayınlamak için --yes
Remove-Item Env:\RESPECTED_VAULT_PASSPHRASE
```

- `build --only brain` yalnızca bir ürünü yayınlar; katalog diğerlerini yayındaki haliyle tutar.
- `catalog --channel stable` kararlı kanala yayınlar; `--min-hub X` eski Hub'ların önce kendini güncellemesini ister;
  `--notes "metin"` güncelleme önerilirken gösterilen kısa sürüm notudur (en çok 600 karakter, katalog imzasına dahildir).
- **Yayınlanmış bir sürüm numarası bir daha kullanılmaz.** Düzeltme gerekiyorsa yeni numara çıkar (`0.1.0-beta.3`):
  Hub aynı numarayı "daha yeni" saymaz, kurmuş olanlar düzeltmeyi almaz. Yalnızca hiç yayınlanmamış numara yerinde yeniden derlenir.
- Yükleme sırası: önce ürünler, en son katalog (katalog hiçbir zaman olmayan bir dosyayı göstermez).
- Anahtarsız derleme GELİŞTİRMEdir ve asla yayınlanmaz.

GitHub'daki düzen (Hub yalnızca katalog adresini bilir; gerisini imzalı katalog söyler):

| Depo | Etiket | Dosyalar |
|---|---|---|
| `respected0/respected-hub` | `v<sürüm>` | `Respected-Hub-<sürüm>-Windows-x64-Setup.exe` (Brain ve Orchestrator kurucuları Hub'ı `releases/latest` listesinden bulup indirir) |
| `respected0/respected-brain` | `v<sürüm>` | `Respected-Brain-<sürüm>-Windows-x64-Package.zip` (katalog paketi), `Respected-Brain-<sürüm>-Windows-x64-Setup.exe` (Hub'sız kuranlar için) |
| `respected0/respected-orchestrator` | `v<sürüm>` | `Respected-Orchestrator-<sürüm>-Windows-x64-Package.zip`, `Respected-Orchestrator-<sürüm>-Windows-x64-Setup.exe` |
| `respected0/respected-hub` | `catalog` | `catalog.json` + `catalog.json.sig` (her yayında değişir) |

## 5. Neyi ne kurar, neyi ne siler

Her şey kullanıcı başınadır, yönetici izni gerekmez. `%LOCALAPPDATA%` = `C:\Users\<siz>\AppData\Local`.

| Adım | Oluşan | Silinen |
|---|---|---|
| Hub kurucusu | `Respected\RespectedHub\app\` (`respected-hub.exe`, kaldırıcı), Başlat ve masaüstü kısayolu, Yüklü uygulamalar: Respected Hub | — |
| Hub'ın ilk açılışı | `%LOCALAPPDATA%\com.respected.hub\` (WebView2, pencere), `%APPDATA%\com.respected.hub\`, `RespectedHub\managed\catalog-index.json` (+ `.sig`) | — |
| Ürün kurulumu (kurucuyla ya da Hub'dan) | `Respected\<Ürün>\app\` (program, `distribution.json`, kaldırıcı), `Respected\<Ürün>\data\`, Yüklü uygulamalar girdisi; Brain'de seçilen kasa ve yalnızca seçilen AI araçlarındaki girdiler, isteğe bağlı zamanlanmış görev ve kısayol; Hub'dan kurulduysa `managed\catalog\<ürün>-<sürüm>\` (çevrimdışı onarım için) | — |
| Ürün güncellemesi | yeni kurucu programı yerinde değiştirir, ürün `repair` çalıştırır | eski programın dosyaları, eski kurucu önbelleği |
| Ürünü kaldırma | — | program, Yüklü uygulamalar girdisi, ürünün AI araçlarındaki ve zamanlayıcıdaki kendi girdileri, kurulum kaydı; "Evet"/`/PURGE=1` ile ayarlar ve çalışma verisi. **Kasa hiçbir zaman.** |
| Hub'ı kaldırma | — | Hub'ın programı ve kendi klasörleri. **Ürünlere dokunulmaz;** Hub'ı yeniden kurmak onları olduğu gibi bulur. |

Hiçbir adım PATH'i değiştirmez, servis kurmaz, sistem Python'u istemez.

## 6. Açık kapılar

Kanıt gerektirenler: temiz bir Windows makinesinde ilk kurulum, Hub'ın kendini
güncellemesi, Yüklü uygulamalar'dan kaldırma ve değiştirme. Authenticode sertifikası yok. macOS/Linux doğrulanmadı.
Yükleme yalnızca kullanıcının onayıyla yapılır.
