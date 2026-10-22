# Respected Hub deposu: çalışma kuralları

Bu, Respected Hub'dır: Tauri 2, Rust, React ve TypeScript. Belgeler: `README.md` (giriş, testler), `docs/ARCHITECTURE.md`
(bağlayıcı kurallar ve güven), `release/RELEASE.md` (derleme ve yayın). Üç ürünün ortak sözleşmesi ekosistem kökündeki
`docs/PRODUCT_CONTRACT.md` dosyasındadır.

## Yasaklar

- Global araç kurma, kişinin AI araçlarının gerçek ayarlarını ya da gerçek SecondBrain verisini değiştirme.
- Kullanıcı istemedikçe commit, push ya da paralel ajan yok.
- Genel bir komut satırı arayüzü ekleme, görsel giriş noktasını değiştirme, IPC'yi sahte başarılı görevlerle değiştirme,
  genel kabuk çalıştırma. Yalnızca açıkça izin verilen motor işlemleri sunulur.
- Geçiş dönemi, eski sürüm desteği ya da uyumluluk katmanı ekleme.

## Sınırlar

- Hub ürün hakkında hiçbir şey saklamaz: kurulu olanı ürünlerden okur; kurar, günceller, onarır ve kaldırırken yalnızca
  ürünün kendi imzalı kurucusunu ya da kaldırıcısını çalıştırır (`hub/installer.rs`, `kind: installer` katalog paketleri);
  ayarları ürünün şemasıyla değiştirir; Orchestrator'ı SecondBrain'e Orchestrator'ın `memory.connect`'i ile bağlar.
  Yönetilen kopya, Hub'ın yazdığı Yüklü uygulamalar girdisi, ürünlere geçirilen veri klasörü yok. RPP yalnızca `"1"`.
- Orchestrator çalışmaları Hub kapansa da sürer; görev durumu ve iptal motorundadır.
- Kardeş depoların kaynağını bulmak yalnızca geliştirme sürümündedir. Yayın imzası ve gerçek makinede kabul ayrı
  kapılardır; kanıtsız iddia edilmez.

## Değişiklik disiplini

- Testler geçici kökler ve sahte bir kullanıcı klasörü kullanır. Gerçek kurulum yapanlar `installer_e2e` (ignored) ve arayüz
  sürücüsüdür; ikisi de makinede kurulu bir ürün varsa başlamaz ve kurduklarını kaldırır.
- Tamamlandı demeden önce: `npm run check` ve `cargo test` (ya da `python release/tools/acceptance/accept_hub.py`).
  Arayüz ya da kurulum değişikliğinde ayrıca derlenmiş exe'ye karşı arayüz sürücüsü:
  `npx tauri build --debug --no-bundle`, sonra `node tests/native/hub_native_e2e.mjs src-tauri/target/debug/respected-hub.exe`.
- `docs/ARCHITECTURE.md` koda bağlayıcıdır; aynı değişiklikte doğru tut. Belgeler Türkçedir; belge yolları çalışma zamanı
  bağımlılığı olmaz, testler belge cümlelerine bağlanmaz.
