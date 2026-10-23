# Respected ekosistemi

Bu belge Hub deposunda durur ama üç ürün için geçerlidir. Her depo kendi başına yeterlidir; ortak sözleşme üçünde de aynı kopyadır.

Üç bağımsız ürün, üç ayrı depo:

| Ürün | Ne yapar | Depo |
|---|---|---|
| **Brain** (`respected-brain.exe`) | AI araçlarının ortak hafızası: oturumları özetler, notları derler, arar ve MCP/hook ile AI araçlarına sunar. Notlar kişinin kendi Markdown kasasındadır. | `respected-brain` |
| **Orchestrator** (`respected-orchestrator.exe`) | Kodlama ajanlarını (Codex, Claude, Gemini, Antigravity, Cursor) yalıtılmış Git çalışma ağaçlarında çalıştırır, sonucu doğrular, onaysız hiçbir şeyi projeye uygulamaz. | `respected-orchestrator` |
| **Hub** (`respected-hub.exe`) | Masaüstü uygulaması: katalogdan ürünlerin kurucusunu indirir ve çalıştırır, ayarları ve bağlantıları görsel olarak yönetir. | `respected-hub` |

Her ürün tek başına kurulur, ayarlanır, onarılır ve kaldırılır. Hub yalnızca kolaylıktır; Hub kapalıyken ya da hiç
kurulmamışken de ürünler çalışır. Ürünler birbirinin kodunu içe aktarmaz, yalnızca protokolle (RPP) ve MCP ile konuşur.

## Belgeler

| Belge | İçerik |
|---|---|
| [PRODUCT_CONTRACT.md](PRODUCT_CONTRACT.md) | Üç ürünün uyduğu ortak sözleşme: klasörler, kurucu, komutlar, ayarlar, protokol, Hub'ın rolü |
| `respected-brain/README.md` | Brain'e giriş; `docs/USAGE.md`, `docs/COMMANDS.md`, `docs/ARCHITECTURE.md` |
| `respected-orchestrator/README.md` | Orchestrator'a giriş; `docs/USAGE.md`, `docs/COMMANDS.md`, `docs/ARCHITECTURE.md` |
| `README.md` (bu depo) | Hub'a giriş; `docs/ARCHITECTURE.md`; yayın: `release/RELEASE.md` |
| Her deponun `AGENTS.md` dosyası | O depoda çalışan AI ajanları için kurallar |

## Klasör düzeni

```
respected-ecosystem\
  respected-hub\  respected-brain\  respected-orchestrator\   üç depo, yan yana (araçlar birbirini böyle bulur)
    respected-hub\tools\                                       sözleşmeye uyum testi (conformance/, üç depoyu birlikte ölçer), ölçüm, marka, önizleme
  dist\  .build-cache\  .cargo-target-hub\                     üretilen dosyalar, depoya girmez, silinirse yeniden üretilir
```

## Geliştirme ortamı (Windows 10/11, bir kez)

| Araç | Ne için |
|---|---|
| Python 3.13 (python.org kurulumu, Store sürümü değil) | ürünlerin kaynağı, testler, derleme betikleri, PyInstaller |
| Node.js 22.12+ ve npm | Hub arayüzü |
| Rust (MSVC) + Visual Studio Build Tools (C++) | Hub (Tauri 2) |
| Inno Setup 6.7 | kurucular (`ISCC.exe`, ya da `INNO_COMPILER`) |
| WebView2 | Hub'ı çalıştırmak (Windows 11'de hazır) |

```powershell
pip install cryptography
pip install -e "respected-brain[dev]" -e "respected-orchestrator[dev]"   # ekosistem klasöründen
cd respected-hub; npm ci
```

## Günlük kontroller

```powershell
cd respected-hub; python -m unittest discover -s tools/conformance               # ortak sözleşme (kaynakta)
cd ..\respected-brain; python -m unittest discover -s tests -t .
cd ..\respected-orchestrator; python -m unittest discover -s tests -t .
cd ..\respected-hub; npm run check; cargo test --manifest-path src-tauri/Cargo.toml
```

Ürün test takımlarını aynı anda çalıştırmayın (ikisi de derleme önbelleğini kullanır). Testler geçici klasörlerde ve
sahte bir kullanıcı klasöründe çalışır; gerçek AI araçlarının ayarlarına ve gerçek kasaya dokunmaz. Gerçek kurulum yapan
testler ayrıdır ve elle çalıştırılır: Hub'ın `installer_e2e` testi ve arayüz sürücüsü (`respected-hub/README.md`).

## Durum

- Windows betası. macOS/Linux kodu ve kurulum betikleri var ama doğrulanmadı.
- Kurucular Authenticode ile imzalı değil (sertifika yok); Windows SmartScreen uyarır. Ürün paketleri ve katalog Ed25519
  ile imzalıdır.
- Temiz bir makinede ilk kurulum ve Hub'ın kendini güncellemesi henüz bir kişi
  tarafından denenmedi.
