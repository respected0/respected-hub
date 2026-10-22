# Hub: gelecekte yapılacaklar

Ürün beklentisiyle bugünkü durum arasındaki açıklar (2026-10-22 değerlendirmesi, aynı gün güncellendi). Yapılan madde silinir.

## Dış eklenti ve MCP keşfi

- [ ] Keşfet sayfasında GitHub repoları, eklentiler ve MCP sunucularını listelemek (bugün yalnızca Brain ve
  Orchestrator; `src/shell/views/Discover.tsx`).
- [ ] Bunları Hub'dan kurmak (örnek: ponytail) ve kaldırmak; ürünlerimizdeki gibi yalnızca kendi girdisini yazıp silerek.
- [ ] "Hangi AI aracında kurulu" görünümü: her MCP/eklentinin Claude Code, Codex, Cursor, Gemini… ayarlarında olup
  olmadığını göstermek.
- [ ] Uzak, imzalı katalog: bugün katalog yereldir, internetten ürün önerilmez veya indirilmez.

## Yayın

- [ ] Temiz bir makinede ilk kurulum ve Hub'ın kendini güncellemesi henüz bir kişi
  tarafından denenmedi.
- [ ] Kurucuyu Authenticode ile imzalamak (SmartScreen uyarısı). Kod imzalama sertifikası gerekir; yok.
- [ ] macOS / Linux: doğrulanmadı. Bu geliştirme makinesi Windows.
- [ ] Hub'sız kullanıcı yeni sürümden haberdar olmuyor: ürünlerde sürüm denetimi yok.
- [ ] Aynı sürüm numarasıyla düzeltilmiş yayın: Hub önbellekteki eski kurucuyla onarabilir ve "Güncelleme var" göstermez;
  kurmuş olanlar düzeltmeyi kurucuyu elle çalıştırarak alır.
