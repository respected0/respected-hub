# Handoff: where things stand

Üç depo yan yana: `respected-hub`, `respected-secondbrain`, `respected-orchestrator`. Vizyon: `RESPECTED_MASTER_DIRECTIVE.md`.
Kurulum düzeni: `INSTALL_LAYOUT.md`. Sıfırdan derleme: `../BUILD_FROM_SCRATCH.md`. İndirme metni: `RELEASE_NOTES_BETA.md`.

## Durum

| Ürün | Dağıtım | Testler |
|---|---|---|
| Orchestrator | `tools/build_dist.py` ile PyInstaller onedir (`respected-orchestrator.exe`, Python gerekmez) | `python -m unittest discover -s tests -t .` |
| SecondBrain | `tools/build_dist.py` ile dondurulmuş dağıtım klasörü (`respectedbrain.exe`) | `python -m unittest discover -s tests -t .` |
| Hub | Tauri uygulaması; iki ürünü imzalı katalogdan kurar (yalnızca "distribution" paketleri) | `npm run check`, `cargo test --manifest-path src-tauri/Cargo.toml`, `python release/tools/accept_hub.py` |

Üç depoda GitHub Actions CI vardır (windows-latest). Hub CI'ı kardeş depoları yan yana çeker.

## Bilinçli kararlar

* Ürünler yalnızca RPP/MCP ile konuşur; birbirinin iç modülünü içe aktarmaz.
* Hub'da Python çalışma zamanı bileşeni yoktur; Orchestrator paketli exe olarak kurulur, MCP başlatıcısı exe'yi çağırır.
* Kaynak (Python) çalıştırma yalnızca geliştirme modundadır (`dev_enabled`); üretim imzalı kopyaları kullanır.
* Dosya atlası/`repository_map`, `docs/records` ve tarihsel kampanya kayıtları kaldırıldı; kararlar `docs/decisions/` içinde.

## Kapanmamış kapılar (kanıt gerekir)

* Üretim anahtarı ile imzalama, temiz Windows makinesinde ilk kurulum, Hub kendi kendine güncelleme, Installed-apps
  kaldır/değiştir. Anahtar kasası ve parola kullanıcıdadır; depoda anahtar malzemesi yoktur.
* macOS/Linux doğrulanmadı.
* Yerel yayın sırası: `release/tools/publish_release.py build|catalog|upload` (yükleme yalnızca kullanıcı onayıyla).

## Çalışma kuralları

* Gerçek AI-istemci yapılandırmasına, AppData kurulumlarına ve canlı kasaya test sırasında dokunulmaz (`real_env_guard.py`).
* Ağır paketler seri çalışır; push yalnızca kullanıcı isteğiyle.
