# ADR: Üçüncü taraf entegrasyon sahipliği

- **Durum:** Kabul edildi (M1/M2 salt okunur; M3–M5 motoru testli, üretim yazma IPC'si kapalı)
- **Tarih:** 2026-11-04
- **Bağlam:** Hub'ın MCP, Skill ve Plugin envanteri ile ileride bunları yönetmesi

## Karar

Hub, üçüncü taraf AI istemcilerinin (Claude Code, Codex, Cursor, Gemini CLI, OpenCode) yapılandırma
dosyalarını **yalnızca okur** ve envanteri gösterir. M1 ve M2 aşamalarında hiçbir istemci
yapılandırmasına, Skill klasörüne veya Plugin dosyasına yazmaz.

Brain ve Orchestrator'ın kendi MCP kayıtları ve kurulumları o ürünlere aittir. Hub bu kayıtları
doğrudan değiştirmez; ürünün kendi arayüzü veya RPP komutu kullanılır.

Yazma (M3 ve sonrası) ancak bu ADR'nin yeni bir sürümü, ürün sahibinin açık onayı ve
`ARCHITECTURE.md`, `FOOTPRINT.md`, `PRIVACY.md` belgelerinin aynı değişiklikte güncellenmesiyle açılır.
Yazma açıldığında bile plan → kullanıcı onayı → işlem → geri okuma → geri alma zinciri zorunludur.

## Sahiplik matrisi

| Kaynak | Hub erişimi | Sahip | Not |
|---|---|---|---|
| Brain/Orchestrator MCP girdileri | Okunabilir | İlgili ürün | Yazma yalnızca ürünün RPP komutuyla |
| Üçüncü taraf MCP girdileri (istemci config) | M1/M2: okunabilir; M3+: yazılabilir | Kullanıcı | Yazma kapısı bu ADR'de tanımlı |
| Skill klasörleri ve `SKILL.md` | M1/M2: okunabilir; M3+: yazılabilir | Kullanıcı | Symlink hedefi ve döngü denetlenir |
| Plugin manifestleri ve alt bileşenleri | M1/M2: okunabilir; M3+: yazılabilir | Kullanıcı | Parent-child ilişkisi korunur |
| Üçüncü taraf katalog kaydı | Okunabilir | Hub kataloğu | İmzalı ürün kataloğundan ayrıdır |
| Brain/Orchestrator kurulum dosyaları | Okunabilir | İlgili ürün | Hub kurucu/kaldırıcı çalıştırır |
| Proje dosyaları | Yalnızca kullanıcının açıkça seçtiği proje | Kullanıcı | Otomatik disk taraması yok |
| Kimlik bilgileri, token, parola | Okunmaz/gösterilmez | Kullanıcı | Tespit edilirse redakte edilir |
| Rastgele kabuk komutu veya URL | Yasak | — | Genel exec/ağ API'si yok |

## İzin seviyeleri

- **Okunabilir:** Yalnızca bilinen, sabit istemci konumları ve sınırlı boyutta dosyalar.
- **Yazılabilir:** Yalnızca M3 kapısı açıldıktan sonra, plan ve geri alma ile.
- **Ürüne devredilir:** Hub yazmaz; ürünün kendi komutunu çağırır.
- **Yasak:** Genel kabuk, keyfî yol, keyfî URL, üçüncü taraf kod çalıştırma, ürün kaydını doğrudan ezme.

## Durum modeli

`bilinmiyor`, `kurulu`, `yapılandırılmış`, `etkin`, `bağlantı doğrulandı` ve `kullanıldığı kanıtlandı`
ayrı alanlardır. Klasörün veya dosyanın varlığı tek başına "etkin" veya "kullanılıyor" anlamına gelmez;
belirsiz durumda `unknown` döner.

## Güvenlik sınırları

- Tarama sırasında hiçbir üçüncü taraf kodu çalıştırılmaz.
- Symlink'ler hedef kökün dışına çıkamaz; döngüler sınırlı derinlikte kesilir.
- Dosya boyutu ve tarama derinliği sınırlıdır; aşım kontrollü hata üretir.
- Yalnızca `https://github.com/...` katalog bağlantısı açılır; MVP kurulum yapmaz.
- Testler geçici kök ve sahte kullanıcı klasörü kullanır; gerçek AI ayarlarına yazmaz.

## Tehdit modeli

**Varlıklar:** Kullanıcının AI istemci yapılandırmaları, Skill/Plugin kaynak dosyaları, config içindeki sırlar,
journal ve geri alma yedekleri, katalog imza anahtarı ve sequence durumu.

**Tehditler:** Kötü niyetli config/katalog içeriği, sır ifşası, symlink veya yol kaçışı ve döngü, plan ile yazma
arasında TOCTOU/hash kayması, ürün/katalog sahipliğinin ezilmesi, aşırı büyük veya bozuk dosya, imza
tekrarı/düşürme/alan karışması, çökme veya yarım yazma.

**Kontroller:** Tipli ve kapalı IPC (keyfî yol, komut veya URL yok), plan → kullanıcı onayı → atomik yazma →
geri okuma → undo zinciri, kanonik yol ve sabit istemci allowlist'i, sınırlı okuma/tarama, sır redaksiyonu,
sahiplik koruması (`OWNER_PROTECTED`), alan ayrımlı Ed25519 imza ve sequence doğrulaması.
Kanıt: `integrations/security.rs`, planner/apply testleri, adapter E2E sır ve symlink vakaları, `catalog_e2e`.

**Kalan risk / kapsam dışı:** Aynı kullanıcı hesabında çalışan kötü niyetli yerel yazılım, kullanıcının kendi
bozduğu istemci yapılandırması ve imzalı metadata'nın bağlı kaynak kodun güvenli olduğunu garanti etmemesi.
Üretim yazma IPC'si bu ADR'nin kapısı açılana kadar kapalıdır; kurma/kaldırma yoktur.

## Sonuçlar

- M1/M2'de kullanıcı gerçek envanteri görür; yanlış "yönetiyor" beklentisi oluşmaz.
- Brain/Orchestrator sahipliği korunur.
- Yazma özelliği, ayrı bir karar ve test kanıtı olmadan üretime çıkamaz.
