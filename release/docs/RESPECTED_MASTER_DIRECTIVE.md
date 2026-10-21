# Respected Ecosystem: ürün vizyonu

Üç bağımsız ürün: **Hub** (masaüstü uygulaması), **SecondBrain** (hafıza motoru), **Orchestrator** (ajan ve proje yürütücüsü).
Mimari değişmez: tek motorda birleştirme yok, dördüncü bir orkestrasyon ürünü yok. Ürünler yalnızca sürümlü protokol (RPP) ile
konuşur; her biri Hub kapalıyken de çalışır. İlk sürüm local-first ve hesapsızdır.

## Hub

Premium masaüstü başlatıcı, kontrol merkezi ve uygulama yöneticisi (Tauri 2 + Rust + React/TypeScript). Teknik bilgisi olmayan
kullanıcı terminal, JSON veya GitHub ile uğraşmadan ürün kurabilmeli, ayarlayabilmeli, güncelleyebilmeli, onarabilmeli ve
kaldırabilmelidir.

* Navigasyon: Home, My Apps, Discover, Connections, Activity, Settings. Home ilk kurulumda karşılama, kullanım arttıkça gerçek
  sistem durumunu (görevler, onaylar, güncellemeler, hatalar) gösterir.
* Görsel dil: koyu graphite/navy, ölçülü mavi ve violet vurgu, net hiyerarşi, anlaşılır Türkçe. İstenmeyenler: generic admin
  paneli, sahte metrik, çalışmayan dekoratif düğme. Her ekranın gerçek bir amacı olur.
* Yaşam döngüsü: kur, güncelle, onar, kaldır; imzalı katalog, doğrulanmış paket, günlük (journal), geri alma. Üretim yalnızca
  imzalı, yüklenmiş kopyaları çalıştırır.
* Safe Integration Layer: AI istemcilerinin (MCP, hook, zamanlama) yapılandırmasını ön izleme, onay, yedek ve geri alma ile yönetir;
  başkasının girdilerine dokunmaz.

## SecondBrain

Tek sorumluluk: kullanıcının hafıza kasası. Notları yakalar, derler, arar ve AI istemcilerine MCP/hook üzerinden sunar. Kurulum
sihirbazı Hub'dan çalışır (kasa, istemci bağlantıları, zamanlama). Kasa ve kullanıcı verisi kullanıcıya aittir; ürün kaldırılsa da
not kasası silinmez. Orchestrator ile bağ yalnızca kullanıcının verdiği, kapsamlı ve geri alınabilir hafıza izniyle kurulur.

## Orchestrator

Birden çok AI aracını (Codex, Claude, Gemini vb.) rol temelli yöneten, uzun süreli otonom geliştirme yapabilen yürütücü:
görev, kampanya, onay, kanıt ve kullanım kaydı. Her model her role atanabilir; açık seçim, Hub varsayılanı, otomatik seçim sırasıyla
önceliklidir. Çalışmalar Hub kapansa da sürer; durum ve iptal motorundadır. MCP sunucusu ve CLI ile Hub dışından da kullanılır.
Hub içindeki çalışma alanı gerçek motor verisini gösterir.

## RPP ve güvenlik

* RPP: izin listeli işlemler, sürüm pazarlığı (1.0–1.1), tipli hatalar, makbuzlar; genel kabuk veya dosya sistemi erişimi yok.
* Güvenlik: Ed25519 imzalı paketler, dosya başına özet, fail-closed kontroller, gizli bilginin günlüklere sızmaması, kullanıcının
  gerçek AI-istemci yapılandırmasına ve verisine test sırasında dokunulmaması.

## Kabul ölçütü

Ürün temiz bir Windows makinesinde kurulur, Hub'dan yönetilir, Hub kapalıyken de çalışır, kaldırıldığında kullanıcı verisini
yok etmeden iz bırakmaz. Hiçbir yetenek kanıtsız "tamam" sayılmaz: imza, yerel kabul ve temiz makine doğrulaması ayrı kapılardır.
