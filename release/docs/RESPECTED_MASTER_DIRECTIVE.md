# RESPECTED ECOSYSTEM — ULTIMATE MASTER PRODUCT DIRECTIVE

## 0. GÖREVİN VE TEMEL KURAL

Sen **Claude Master Architect & Product Director** rolündesin.

Geliştirilecek ürünün vizyonu aşağıda açıkça tanımlanmıştır. Bu vizyonu sıfırdan sen icat etmeyeceksin. Mevcut repoları incelemenin amacı, bu vizyonun hangi parçalarının çalıştığını ve hangilerinin eksik olduğunu belirlemektir.

**HEDEF:** Respected Hub, Respected SecondBrain ve Respected Orchestrator'ı birbirleriyle uyumlu, güvenilir, kolay kullanılabilir, profesyonel şekilde dağıtılabilir bir ürün ekosistemi hâline getirmek.

Bu görev yalnızca test yazmak, dashboard oluşturmak veya mevcut API'leri birbirine bağlamak değildir. Son kullanıcıya gerçek anlamda kullanılabilir ürünler teslim etmektir.

Çalışma dizini:

`C:\Users\Furkan\Documents\respected-ecosystem`

Üç bağımsız ürün/repo vardır:

- `respected-hub`
- `respected-secondbrain`
- `respected-orchestrator`

Mimariyi değiştirme, bu üç ürünü tek motorda birleştirme, dördüncü bir orchestration ürünü oluşturma.

---

# 1. RESPECTED HUB — ÜRÜN VİZYONU

## 1.1 Hub nedir?

**Respected Hub = Premium Desktop Launcher + Control Center + App Manager.**

Respected ekosisteminin zorunlu kullanıcı arayüzüdür.

Teknik bilgi sahibi olmayan birisi bile Hub üzerinden ürün kurabilmeli, yapılandırabilmeli, kullanabilmeli, güncelleyebilmeli, onarabilmeli ve kaldırabilmelidir.

Son kullanıcıdan terminal komutu çalıştırması, JSON düzenlemesi, Markdown yapılandırması yazması veya GitHub deposuyla uğraşması istenmeyecek.

Ürün motorları kendi başlarına çalışabilir. Ancak kullanıcıya yönelik kurulum, ayar ve yönetim işlemlerinin merkezi Hub'dır.

**Teknoloji:** Tauri 2 + Rust + React/TypeScript.

**Hedef platformlar:** Windows, macOS, Linux.

İlk sürüm local-first ve hesapsızdır. Hesap açma zorunluluğu olmayacak. Henüz kullanıcı tabanı bulunmadığından eski sürümlerden migration yapmak başlangıç kapsamına dâhil değildir.

Hub normalde sürekli arka planda çalışmak zorunda değildir. Hub kapatıldığında kurulu ürünlerin bağımsız servisleri, zamanlanmış görevleri, MCP entegrasyonları veya gerekli motorları çalışmaya devam edebilmelidir.

## 1.2 Görsel tasarım

Hedefimiz sıradan bir yazılım dashboard'u değildir.

Kalite referansları:

- Riot Client
- Epic Games Launcher

Bunların kopyasını yapma. Aynı düzeyde profesyonellik ve görsel bütünlük hedefle.

Tasarım dili:

- Koyu graphite ve navy tonları
- Ölçülü elektrik mavisi ve violet vurgular
- Premium tipografi, boşluk ve kart düzeni
- Sinematik ama abartısız derinlik
- Akıcı ve amaçlı mikro animasyonlar
- Tutarlı ikonografi
- Net görsel hiyerarşi
- Erişilebilir, okunabilir arayüz
- Teknik olmayan kullanıcılar için anlaşılır Türkçe metinler

**Kesinlikle istemediklerimiz:** Generic admin dashboard, AI-slop görünüm, aşırı neon, rastgele gradyanlar, sahte kullanım verileri, dekoratif ama çalışmayan butonlar.

Her ekranın gerçek bir amacı ve çalışan davranışı olmalı.

## 1.3 Ana navigasyon

Hub'ın ana omurgası:

**Home / My Apps / Discover / Connections / Activity / Settings**

### Home

Home dinamik olacak.

İlk kurulumda premium bir launcher hissi vermeli:

- Respected ekosistemine giriş
- Kullanılabilir ürünler
- Kurulum durumları
- Continue Setup
- Ürün tanıtımları
- Kurulum ve güncelleme aksiyonları

Kullanım arttıkça aynı ekran canlı bir kontrol merkezine dönüşmeli:

- Kurulu ürünler ve sağlık durumları
- Çalışan görevler ve kampanyalar
- Bekleyen onaylar
- Güncellemeler
- Hatalar ve onarım seçenekleri
- Son aktiviteler
- Gerçek sistem durumu

Sahte metrik gösterilmeyecek.

### My Apps

Kurulu Respected ürünlerinin yönetimi.

Her ürün için uygun olduğunda:

- Kur / kaldır
- Aç / yönet
- Yapılandır
- Özellikleri aç / kapat
- Güncelle
- Onar
- Sağlık durumunu incele
- Sürüm ve uyumluluk bilgilerini görüntüle

Her uygulamanın Hub içerisinde kendine ait çalışma alanı olacak.

### Discover

Genişletilebilir uygulama ve entegrasyon keşif merkezi.

Kapsam:

- Respected ürünleri
- Gelecekteki Respected uygulamaları
- Uyumluluğu doğrulanabilen MCP sunucuları
- Desteklenen GitHub kaynaklı entegrasyonlar
- Üçüncü taraf AI ekosistemi araçları

Keşfedilen bir bileşen otomatik olarak güvenilir veya kurulu kabul edilmez.

Kurulum öncesi güvenilirlik, uyumluluk, izinler, sürüm ve geri alma imkânı incelenecek.

Katalogda doğrulanmamış öğeler, kurulabilir ve güvenli ürünler gibi gösterilmeyecek.

### Connections

AI araçları ve üçüncü taraf entegrasyonları buradan yönetilecek.

Bağlantı durumları, izinler, sahiplik, uyumluluk ve kaldırma işlemleri kullanıcıya anlaşılır biçimde sunulacak.

### Activity

Gerçek faaliyet geçmişi:

- Kurulum ve güncelleme işlemleri
- Görev ve kampanya durumları
- Onaylar
- Hatalar
- Kurtarma işlemleri
- İlgili denetim kayıtları

### Settings

Hub ayarları, ürün ayarları, entegrasyon izinleri ve bakım seçenekleri.

Ayarların temel davranışı **“Kaydet ve Uygula”** olacak.

Kaydedilen ayarın gerçekten uygulandığı, doğrulandığı veya neden uygulanamadığı açıkça gösterilecek.

## 1.4 Kurulum ve yaşam döngüsü

Hub aracılığıyla ürünler ihtiyaç duyulduğunda indirilebilmeli. Başlangıçta bütün motorların zorla kurulması gerekmiyor.

Gerçek kullanıcı akışları:

**Discover → Install → Configure → Use → Update → Repair → Uninstall**

Bütün yaşam döngüsü işlemleri:

- Güvenli ve doğrulanabilir olmalı
- İşlem durumu kalıcı saklanmalı
- Yarım kalan işlemler toparlanabilmeli
- Başarısızlıkta veri kaybı önlenmeli
- Uygun işlemlerde rollback sağlanmalı
- Tekrarlanan işlemler güvenli şekilde ele alınmalı
- Kullanıcıdan terminal müdahalesi beklenmemeli

Bilgisayar aniden kapanırsa, kurulum yarıda kalırsa, işlem kilitlenirse veya Hub kapanırsa sistem bir sonraki fırsatta güvenli şekilde toparlanabilmeli.

Gerekli sınırlı yaşam döngüsü yardımcıları Hub kapandıktan sonra işlemi tamamlayabilir; bunun için sürekli çalışan gereksiz bir Hub daemon'ı oluşturma.

---

# 2. RESPECTED SECONDBRAIN — ÜRÜN VİZYONU

## 2.1 SecondBrain'in tek sorumluluğu

**Respected SecondBrain = Bağımsız AI Memory Engine.**

Bu ürün zaten önemli ölçüde geliştirilmiş durumda. Sıfırdan yeniden yazılmayacak.

Mevcut yetenekleri koru ve gerektiği yerde tamamla:

- Memory capture
- Memory recall
- Arama ve anlamsal arama
- Embedding tabanlı bilgi erişimi
- Kalıcı bilgi organizasyonu
- Knowledge Compiler
- Companion hafızası
- Daily / briefing mekanizmaları
- Model destekli özetleme
- MCP entegrasyonları
- Hooks
- Markdown ve Obsidian vault desteği
- Zamanlanmış hafıza işleri
- Hafıza verilerinin güvenli yönetimi ve yedeklenmesi

Mevcut fonksiyonların gerçekten nasıl çalıştığını repodan doğrula. Var olan başarılı mimariyi gereksiz yere değiştirme.

## 2.2 Sorumluluk Ayrımı ve Eski Kodların Temizlenmesi

**ÖNEMLİ: Dashboard ve orkestrasyon sorumluluklarının Hub ve Orchestrator'a taşınması zaten gerçekleştirilmiştir.**

Hatta taşınan özellikler ilgili ürünlerde bağımsız olarak geliştirilmeye devam edilmiş ve orijinal SecondBrain implementasyonlarının ötesine geçmiştir.

Ancak geçmiş mimariden kalan dashboard, orchestration ve bunlarla ilişkili bazı eski kodlar hâlâ SecondBrain reposunda bulunmaktadır.

**Görevin bu özellikleri yeniden taşımak veya sıfırdan geliştirmek DEĞİLDİR.**

Yapılması gereken:

1. SecondBrain reposundaki eski dashboard, orchestration ve bunlara bağlı kalıntıları tespit et.
2. Bunların güncel Hub ve Orchestrator karşılıklarını incele.
3. Güncel ürünlerde zaten bulunan özellikleri yeniden geliştirme, yeniden taşıma veya eski implementasyonlarla değiştirme.
4. Artık kullanılmayan ve başka işlevlerin ihtiyaç duymadığı eski kodları, bağımlılıklarını ve gereksiz yapılandırmalarını kontrollü biçimde temizle.
5. Hâlâ kullanılan ortak kodları veya hafıza motorunun gerçek bağımlılıklarını belirle. Gerekiyorsa sorumluluklarına uygun biçimde yeniden düzenle.
6. Temizlik sırasında mevcut SecondBrain hafıza özelliklerini, vault verilerini, MCP entegrasyonlarını, hook'ları, capture/recall sistemini, hafızaya özel zamanlanmış işleri ve dış entegrasyon sözleşmelerini koru.
7. Silme veya değiştirme işlemlerinden önce gerçek kullanım ve bağımlılık analizi yap. Ardından regresyon ve entegrasyon testleriyle davranışların korunmasını doğrula.

### Nihai mimari

- **Respected Hub:** Kullanıcı arayüzü, ürün kurulumu, yapılandırma, yönetim ve yaşam döngüsü.
- **Respected SecondBrain:** Bağımsız hafıza motoru ve hafızayla doğrudan ilişkili işlemler.
- **Respected Orchestrator:** Agent yönetimi, görev yürütme, kampanyalar, projeler, workflow ve orkestrasyon.

Bu sorumluluklar arasında gereksiz tekrar veya çakışma bulunmamalıdır.

**Temel prensip: Mevcut başarılı ayrışmayı koru, güncel implementasyonları esas al ve yalnızca SecondBrain'de kalmış eski mimari kalıntılarını güvenli biçimde temizle.**

Temizlik işlemi yalnızca kod azaltmak için yapılmamalı; doğrulanmış kullanım, bağımlılık ve regresyon kanıtlarına dayanmalıdır.
## 2.3 SecondBrain kurulum sihirbazı

Kullanıcı SecondBrain'i Hub üzerinden kurduğunda bir setup wizard görecek.

Örneğin:

**Identity → Memory → Sources → Permissions → Finish**

Kullanıcı:

- Asistanına isim verebilecek; örneğin Jarvis
- Hafızanın saklanacağı yeri seçebilecek
- Vault konumunu yönetebilecek
- Hafıza yakalama tercihlerini belirleyebilecek
- Otomatik kaydetme veya önce izin isteme davranışını yapılandırabilecek
- Hafıza kaynaklarını seçebilecek
- Desteklenen AI istemcileriyle bağlantı kurabilecek
- İzinleri görüntüleyip değiştirebilecek

Bir ayar motor tarafından desteklenmiyorsa Hub bunu çalışıyormuş gibi göstermeyecek.

Örneğin asistan adı yalnızca Hub seviyesinde bir görünen isimse bunu hafıza motorunun gerçek bir özelliği gibi sunma.

Sihirbaz kontrollü, anlaşılır, tercihen geri alınabilir ve kullanıcının verileri üzerinde söz sahibi olduğu bir deneyim olmalı.

## 2.4 Orchestrator entegrasyonu

SecondBrain, Orchestrator için **isteğe bağlı hafıza sağlayıcısı** olabilir.

Ancak:

- Orchestrator çalışmak için SecondBrain'e bağımlı olmayacak.
- Görev geçmişinin tamamı otomatik olarak hafızaya aktarılmayacak.
- Task, attempt, log ve evidence geçmişi Orchestrator'ın kendi sisteminde kalacak.
- SecondBrain'e yalnızca kalıcı ve yeniden kullanılabilir bilgiler, ilgili politika ve izinler doğrultusunda aktarılacak.

---

# 3. RESPECTED ORCHESTRATOR — ÜRÜN VİZYONU

## 3.1 Orchestrator nedir?

**Respected Orchestrator = Provider-agnostic AI Agent Orchestration Engine.**

Amacı farklı AI modellerini ve agent araçlarını, belirlenen hedefler doğrultusunda güvenli, verimli, kaliteli ve mümkün olduğunca otonom çalıştırmak.

Yalnızca birkaç AI'ı sırayla çağıran basit bir script olmayacak.

Microsoft Agent Framework tabanı korunabilir; Respected'a özgü workflow, policy, adapter, verification ve lifecycle katmanları bunun üzerinde yer alacak.

Framework'ü gereksiz yere yeniden yazma veya bütün ürünü tek framework'ün iç detaylarına kilitleme.

## 3.2 Temel kabiliyetler

Orchestrator şu yeteneklere sahip olmalı:

**Agent ve model yönetimi**

- Farklı AI sağlayıcıları
- Farklı CLI agent araçları
- Provider/model profilleri
- Master, worker, reviewer gibi roller
- Roller ile modellerin birbirinden bağımsız tanımlanması
- Model değiştirme ve fallback
- Capability-aware seçim
- Kullanıcı tarafından yapılandırılabilir roller ve bağlantılar

**Görev orkestrasyonu**

- Tekil görevler
- Çok adımlı workflow'lar
- Kalıcı projeler
- Kampanyalar
- Bağımlı fazlar
- Paralel yürütme
- Delegation / handoff
- Otomatik görev dağıtımı
- Görev sonucuna göre sonraki adımı seçme

**Kalite kontrol**

- Worker self-check
- Bağımsız reviewer
- Yapılandırılmış değerlendirme
- Correction loop
- Deterministic acceptance tests
- Gerçek kaynak üzerinde doğrulama
- Review evidence
- İnsan onayı gereken kararların ayrıştırılması

**Dayanıklılık**

- Kalıcı görev ve çalışma durumu
- Checkpoint
- Resume
- Crash recovery
- İptal
- Timeout
- Retry sınırları
- Stale snapshot önleme
- Duplicate execution önleme
- Güvenli apply ve gerektiğinde rollback

**Kaynak yönetimi**

- Token kullanımı
- API/provider maliyetleri
- Zaman bütçeleri
- Paralellik limitleri
- Model bazlı kullanım sınırları
- Gereksiz pahalı model çağrılarını önleyen routing

**Güvenlik**

- İzole çalışma alanları
- Dosya ve işlem kapsamı
- Yetki sınırları
- Değişiklik sahipliği
- Denetlenebilir işlemler
- Güvenilmeyen çıktılara karşı kontroller
- Açık yetki olmadan hassas veya kapsam dışı değişiklik yapmama

## 3.3 Uzun süreli otonom geliştirme

Bu özellik özellikle kritik.

Kullanıcı tek bir büyük ürün talimatı verdiğinde:

1. Claude Master ürün hedefini ve mimariyi belirler.
2. İşleri gerçek bağımlılıklarına göre fazlara böler.
3. Orchestrator kalıcı bir execution DAG oluşturur.
4. Bağımsız işler güvenli biçimde paralel başlar.
5. Worker'lar uygulama yapar.
6. Bağımsız reviewer kontrol eder.
7. Kabul testleri çalışır.
8. Önceden yetkilendirilmiş değişiklikler güvenli biçimde uygulanır.
9. Gerçek kaynak doğrulanır.
10. Bağımlılığı çözülen sonraki fazlar otomatik başlar.
11. Son aşamada taze Claude Master bağımsız ürün denetimi yapar.
12. Eksik varsa yeni düzeltme görevleri otomatik oluşturulur.

Kullanıcıdan her faz için yeniden prompt beklenmeyecek.

Context şişmesini önlemek için her agent yalnızca ilgili görev bağlamını alacak.

Sistem süresiz, kontrolsüz veya yetkisiz çalışmayacak.

## 3.4 Geliştirmede kullanılacak roller

Bu geliştirme programında:

- **Claude:** Master Architect, başlangıç planı, kritik mimari denetimler ve final ürün kontrolü
- **DeepSeek:** Birincil implementer ve self-check
- **GLM:** Bağımsız reviewer; gerektiğinde implementer
- **GLM implementer olduğunda:** DeepSeek bağımsız reviewer
- **MiMo:** Kullanılmayacak

Claude doğrudan sıradan ürün kodlarını yazmayacak.

Bu rol dağılımı ürünün gelecekte yalnızca bu modellere destek vereceği anlamına gelmez. Orchestrator farklı modeller ve araçlarla çalışabilecek genel bir ürün olarak kalmalıdır.

## 3.5 Hub içindeki Orchestrator workspace

Hub üzerinden şu çalışma alanları yönetilebilmeli:

- Overview
- Tasks
- Campaigns
- Agents & Models
- Workflows / Projects
- Review & Evidence
- Settings

Kullanıcı görev başlatabilmeli, gerçek ilerlemeyi görebilmeli, sonuçları inceleyebilmeli, uygun onayları verebilmeli, hata ve kurtarma durumlarını yönetebilmeli.

Agent rolü ile model profili birbirine karıştırılmayacak.

Durum, süre, token, review ve maliyet verileri gerçek execution kayıtlarından üretilecek. Sahte demo bilgileri kullanılmayacak.

---

# 4. RPP — ÜRÜNLER ARASI İLETİŞİM

Hub, SecondBrain ve Orchestrator ayrı ürünler olmaya devam ederken, **versioned Respected Product Protocol (RPP)** üzerinden iletişim kuracak.

RPP şu kavramları tanımlamalı:

- Product identity
- Protocol compatibility
- Manifest
- Capabilities
- Health/status
- Settings schema
- Validation
- Apply/readback
- Lifecycle operations
- Typed errors
- Operation IDs
- Durable operation receipts

Hub desteklenmeyen bir motor özelliğini varmış gibi göstermeyecek.

RPP genel amaçlı sınırsız shell veya filesystem erişim arayüzüne dönüşmeyecek.

Hub'ın kullanıcı arayüzüyle ürün motorları arasındaki sözleşmeler açık, test edilebilir ve sürümlenebilir olacak.

---

# 5. SIL — SAFE INTEGRATION LAYER

Bu ürün vizyonunun en önemli parçalarından biri **üçüncü taraf AI ekosistemiyle çakışmadan birlikte çalışmak**.

Uyumluluk yalnızca Claude Code, Codex, Antigravity, Ponytail veya 9Router için hazırlanmayacak.

Bunlar yalnızca örnektir.

Genel çözüm şu tür mevcut ve gelecekteki araçlara genişletilebilir olmalı:

- AI istemcileri
- Agent CLI araçları
- Model router'ları
- MCP sunucuları
- Hooks
- Skills
- Plugins
- IDE entegrasyonları
- Otomasyon araçları
- Desteklenebilen üçüncü taraf bileşenler

**Ana kural:** Respected başka yazılımların mevcut ayarlarını bozmamalı.

SIL'in işlemleri:

**Discover → Inspect → Compatibility Check → Plan → Explain → Approve → Apply → Verify → Record Ownership → Repair/Rollback**

SIL:

- Mevcut yapılandırmaları inceleyecek
- Bilinmeyen yapılandırmaları tahminle değiştirmeyecek
- Yalnızca Respected'ın sahip olduğu kayıtları yönetecek
- Kullanıcının ve başka araçların kayıtlarını koruyacak
- Model, proxy veya API anahtarlarını keyfî değiştirmeyecek
- Sahiplik kayıtları tutacak
- Değişiklikleri doğrulayacak
- Çakışmaları algılayacak
- Güvenli durumlarda geri alabilecek
- Gereksiz yeniden kurulum veya zorunlu yükseltme yapmayacak

Desteklenmeyen veya bilinmeyen bir entegrasyon için güvenli davranış, kullanıcıya açıkça sınırlamayı bildirmektir; uydurma uyumluluk değildir.

---

# 6. GÜVENİLİRLİK VE ÜRÜN KALİTESİ

Respected ekosistemi yalnızca mutlu senaryoda çalışan bir prototip olmamalıdır.

Özellikle şu koşullar düşünülmelidir:

- Bilgisayar beklenmedik şekilde kapanması
- İşlem ortasında uygulamanın sonlandırılması
- Ağ/API kesintisi
- Yetkisiz dosya erişimi
- Bozuk veya eksik kurulum
- Başarısız güncelleme
- Çakışan yapılandırmalar
- Kilitli dosyalar
- Yarım kalmış işlemler
- Tekrarlanan komutlar
- Eski veya uyumsuz ürün sürümleri
- Disk ve izin problemleri

Güvenli journal/checkpoint, idempotency, recovery, verification ve rollback mekanizmaları gerektiği yerde kullanılmalı.

Özellikle lifecycle işlemlerinde geri döndürülemeyen değişiklikler açıkça tanımlanmalı.

Platform desteği Windows/macOS/Linux hedeflerini kapsamalı; ancak test edilmemiş platform destekleniyor diye ilan edilmemeli.

---

# 7. ŞİMDİ MEVCUT REPOYU NASIL İNCELEYECEKSİN?

Artık ürün hedeflerini biliyorsun.

Bu yüzden repoları **vizyon üretmek için değil, yukarıdaki somut hedeflere karşı gap analysis yapmak için** incele.

Başlangıç kaynakları:

- AGENTS.md
- Architecture/decision belgeleri
- Repository maps
- RPP/SIL dokümanları
- Respected Orchestrator Golden Standard
- Mevcut manifest ve API sözleşmeleri
- Testler
- Önceki campaign/project sonuçları
- Gerçek kaynak kod

Belgelerde yazıyor olması bir özelliğin tamamlandığını kanıtlamaz.

Her özelliği şu kategorilerde sınıflandır:

**Verified / Implemented but Unverified / Partial / Missing / Broken**

Mevcut ve iyi çalışan parçaları yeniden yazma.

Kapsam dışı mevcut özellikler bulursan onları ayrı raporla; yeni ürün gereksinimi olarak kendiliğinden ekleme.

Bağlamı verimli kullan. Bütün repoları gelişigüzel okumak yerine hedefli inceleme yap.

---

# 8. MASTER EXECUTION PLAN

Yukarıdaki ürün şartnamesini temel alarak tek bir kalıcı geliştirme programı oluştur.

Plan sayısı sabit değil. Gerçek işe göre 15, 20, 30 veya daha fazla faz olabilir.

Planın yapısı:

**Product Contract → Milestones → Phase DAG → Tasks → Implementation → Review → Acceptance → Apply → Verification**

Her fazın:

- Somut hedefi
- Ürün/repo kapsamı
- Bağımlılıkları
- Paralellik koşulları
- İzin verilen dosya kapsamı
- Worker/reviewer rolleri
- Acceptance kriterleri
- Test komutları
- Bütçesi
- Retry sınırları
- Kanıt gereksinimleri

açık olmalı.

Bağımsız fazları paralel çalıştır. Ortak dosya veya kaynak çakışmalarını engelle. Güncellenmemiş kaynak snapshot'larıyla bağımlı faz başlatma.

Her tamamlanan faz, gerçek çalışma ağacında doğrulanmalı.

Mevcut Orchestrator'ın `project-start`, `project-resume`, `project-status`, `project-cancel` yeteneklerini kullan ve gerçekten eksik olan mekanizmaları güvenli şekilde tamamla.

İş akışı mümkün olduğunca kesintisiz ilerlemeli.

---

# 9. CLAUDE MASTER ÇALIŞMA KURALLARI

**Claude Master kodlama işçisi değildir.**

Başlangıçta sen:

1. Mevcut mimariyi ve kod durumunu incelersin.
2. Bu prompttaki sabit ürün hedeflerini Product Contract'a dönüştürürsün.
3. Eksik özellikleri belirlersin.
4. Faz grafiğini oluşturursun.
5. Bütçe, paralellik ve yetki sınırlarını belirlersin.
6. Bana kısa bir yönetici özeti sunarsın.

**Bu noktada bir defa onayımı beklersin.**

Ben onay verdikten sonra:

- Onaylanmış kapsamı kalıcı olarak kaydet.
- Orchestrator üzerinde otonom programı başlat.
- Gerçek supervisor'a devret.
- İlk Master oturumunu bitir.
- Sıradaki fazlar için tekrar kullanıcı promptu bekleme.

Yeni Claude Master oturumları yalnızca kritik mimari denetimlerde, ciddi engellerde veya final ürün değerlendirmesinde kullanılmalı.

DeepSeek ve GLM'ye görevleri Orchestrator dağıtmalı.

Sistem kendi içinde kontrollü düzeltmeler ve yeniden planlama yapabilmeli.

Kritik bütçe aşımı, yeni yetki veya ürün kapsamı değişikliği olmadan sıradan teknik işler için beni rahatsız etme.

## Güvenlik sınırları

- Kullanıcının gerçek repolarında Git commit/push/reset/branch yapma.
- Mevcut uncommitted değişiklikleri koru.
- Yetki reddini atlatma.
- Önceden yetkilendirilmemiş değişiklikleri otomatik uygulama.
- Gerçek test olmadan tamamlandı deme.
- Sonsuz retry yapma.
- Kullanıcının verilerini silme.
- Makine başlangıcına otomatik görev veya servis kurmak için ayrıca açık yetki al.
- Kapsam ve bütçe sınırlarını koru.

---

# 10. SON ÜRÜN KABULÜ

Ürün, yalnızca fazların tamamlanması veya birim testlerin geçmesiyle bitmiş sayılmayacak.

Taze Claude Master, başlangıçta onaylanmış Product Contract üzerinden bağımsız final incelemesi gerçekleştirecek.

Şunlar değerlendirilecek:

- Gerçek kullanıcı akışları
- Eksiksiz kurulum/yapılandırma
- Kullanılabilir ve premium Hub deneyimi
- SecondBrain hafıza fonksiyonlarının korunması
- Orchestrator'ın gerçek görev yürütmesi
- RPP uyumluluğu
- SIL güvenliği
- Güncelleme/onarım/kaldırma
- Veri bütünlüğü
- Crash/recovery
- Native desktop smoke testleri
- Uygun entegrasyon ve güvenlik testleri
- Platform doğrulaması
- Release readiness

Final sonuç:

**APPROVED:** Kabul kriterleri gerçekten karşılanıyor.

**CHANGES REQUIRED:** Eksikler için yeni düzeltme fazları oluşturulur; Orchestrator otomatik devam eder.

**BLOCKED:** Gerçekten kullanıcı kararı, yeni yetki veya bütçe gerekiyor.

Sonuç kanıta dayanmalı.

---

# 11. ŞİMDİKİ İLK GÖREVİN

**Henüz kod yazma ve yeni büyük geliştirme kampanyaları başlatma.**

Önce şu somut ürün vizyonuna göre mevcut durumu incele.

Bana yalnızca okunabilir bir özet sun:

1. **Hub:** Bu hedeflerin hangileri hazır, hangileri eksik?
2. **SecondBrain:** Hangi mevcut özellikler korunacak, entegrasyon için ne gerekiyor?
3. **Orchestrator:** Golden Standard ve otonom yürütme açısından ne eksik?
4. **RPP / SIL / Lifecycle:** Hangi parçalar eksik?
5. **Önerdiğin faz grafiği:** Kaç faz, hangileri paralel, ana bağımlılıklar ne?
6. **Tahmini maliyet/kaynak:** Süre, token, concurrency ve retry sınırları.
7. **Yetkiler:** Otomatik apply dahil hangi işlemleri onaylamam gerekiyor?

Ayrıntılı Product Contract ve phase DAG dosyalarda saklansın; sohbete binlerce satır dökme.

**Bana bir kere planı ve yetki sınırlarını onaylat. Ardından tüm yetkilendirilmiş geliştirme programını Respected Orchestrator'a teslim et.**

Hedefim her iki saatte bir yeni Master promptu yazmak değil.

**Hedefim: Respected ekosisteminin baştan kararlaştırılmış ürün vizyonuna göre, otonom ve doğrulanabilir şekilde tamamlanması.**

BAŞLA.