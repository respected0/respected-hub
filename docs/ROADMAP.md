# Hub: gelecekte yapılacaklar

Ürün beklentisiyle bugünkü durum arasındaki açıklar (2026-10-23 itibarıyla). Yapılan madde silinir.

## Dış eklenti ve MCP keşfi

- [ ] Keşfet sayfasında GitHub repoları, eklentiler ve MCP sunucularını listelemek (bugün yalnızca Brain ve
  Orchestrator; `src/shell/views/Discover.tsx`).
- [ ] Bunları Hub'dan kurmak (örnek: ponytail) ve kaldırmak; ürünlerimizdeki gibi yalnızca kendi girdisini yazıp silerek.
- [ ] "Hangi AI aracında kurulu" görünümü: her MCP/eklentinin Claude Code, Codex, Cursor, Gemini… ayarlarında olup
  olmadığını göstermek.
- [ ] Dış eklentiler ve MCP sunucuları için de imzalı katalog: ürünlerimizin kataloğu GitHub'dan imzalı gelir (Hub imzayı
  ve dosya özetlerini doğrular); dış eklentiler için böyle bir katalog ve güven modeli henüz yok.

## Yayın

- [ ] Temiz bir makinede ilk kurulum henüz denenmedi (`tests/native/hub_native_e2e.mjs` kurulu ürün olan makinede başlamaz).
  Hub'ın katalogdan kendini güncellemesi (Ayarlar > Güncellemeler > Hub'ı güncelle) ilk sürümde kapalı; bir sonraki sürüm yayınlanınca gerçek yeni sürümle denenecek;
  `npm run preview:update` yalnızca ekranları sahte veriyle gösterir.
- [ ] Kurucuyu Authenticode ile imzalamak (SmartScreen uyarısı). Kod imzalama sertifikası gerekir; yok.
- [ ] macOS / Linux: doğrulanmadı. Bu geliştirme makinesi Windows.
- [ ] Güncelleme yolu karara bağlandı: her sürümden en yeniye doğrudan (sözleşme §4: yedekli güncelleme, silinmeyen `STEPS`,
  sürüm başına gerçek veri fixture'ları). Kalan: yayınlanmış eski kurucuları gerçek Windows'ta yenisinin üzerine kuran CI işi
  (beta.2 yayınlanınca beta.1 ile başlar). Zincir gerçekten uzarsa `min_from` ile basamak sürüm istenir.
- [ ] Hub'sız kullanıcı yeni sürümden haberdar olmuyor: ürünlerde sürüm denetimi yok.
- [ ] Aynı sürüm numarasıyla düzeltilmiş yayın: Hub önbellekteki eski kurucuyla onarabilir ve "Güncelleme var" göstermez;
  kurmuş olanlar düzeltmeyi kurucuyu elle çalıştırarak alır.
