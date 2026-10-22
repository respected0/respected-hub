// What each Respected program puts on the computer, changes and removes. Written from the products' own behaviour
// (respected-brain/docs/USAGE.md §1 and §9, ARCHITECTURE.md §4 and §6; respected-orchestrator/docs/ARCHITECTURE.md §1-3 and §6-7;
// release/RELEASE.md §5). When a product's install, update or removal behaviour changes, change the sentence here in the same commit;
// the product installers show the same text (respected-brain/packaging/windows/brain_info.txt, respected-orchestrator/.../orchestrator_info.txt) and tests/shell/installFacts.test.ts checks it.
export type FactPhase = 'install' | 'use' | 'update' | 'uninstall';
export type FactSubject = 'hub' | 'brain' | 'orchestrator';

export const PHASES: { id: FactPhase; label: string }[] = [
  { id: 'install', label: 'Kurulumda neler olur' },
  { id: 'use', label: 'Kullanırken neler oluşur' },
  { id: 'update', label: 'Güncellemede neler değişir' },
  { id: 'uninstall', label: 'Kaldırınca neler silinir' },
];

const NO_ADMIN = 'Yönetici izni ve Python gerekmez.';

export const FACTS: Record<FactSubject, Record<FactPhase, string[]>> = {
  hub: {
    install: [
      `Program: %LOCALAPPDATA%\\Respected\\RespectedHub\\app. ${NO_ADMIN}`,
      'Başlat menüsü ve masaüstü kısayolu; Windows Ayarlar › Uygulamalar listesinde bir giriş.',
      'Hub\'ın kendi verisi: pencere verisi (%LOCALAPPDATA%\\com.respected.hub, %APPDATA%\\com.respected.hub), katalog önbelleği ve etkinlik günlüğü (%LOCALAPPDATA%\\Respected\\RespectedHub\\managed).',
      'Microsoft Edge WebView2 gerekir (Windows 11\'de hazır gelir); kurucu onu kendiliğinden indirmez, yoksa size söyler.',
    ],
    use: [
      'GitHub\'daki imzalı Respected kataloğunu okur; imzası geçmeyen katalog kullanılmaz.',
      'Ürün kurucuları yalnızca siz onaylayınca indirilir; imza ve özet doğrulanır, sonra ürünün kendi kurucusu çalışır.',
      'Hub ürünler hakkında veri saklamaz: durumu her seferinde ürünlerden okur.',
    ],
    update: [
      'Siz onaylayınca yeni kurucu indirilip doğrulanır; Hub kapanır, yerinde güncellenir ve yeniden açılır.',
      'Ürünlere, notlarınıza ve ayarlarınıza dokunulmaz.',
    ],
    uninstall: [
      'Hub\'ın programı, kısayolları ve kendi klasörleri (pencere verisi, katalog önbelleği, etkinlik günlüğü) silinir.',
      'Kurulu ürünlere (Brain, Orchestrator) dokunulmaz; Hub\'ı yeniden kurunca onları olduğu gibi bulur.',
    ],
  },
  brain: {
    install: [
      `Program: %LOCALAPPDATA%\\Respected\\RespectedBrain\\app. ${NO_ADMIN} Windows Ayarlar › Uygulamalar listesinde bir giriş.`,
      'Veri klasörü: %LOCALAPPDATA%\\Respected\\RespectedBrain\\data (ayarlar, kurulum kaydı, değişiklik yedekleri, arama dizini).',
      'Not kasası: seçtiğiniz klasör. Yeni ve boş bir klasörse şablon (komuta merkezi, şablonlar, Companion) bir kez kopyalanır; dolu bir klasöre şablon yazılmaz.',
      'Seçtiğiniz AI araçlarına: oturum yakalama kancaları, kurallar ve beceriler (Claude Code ve Codex\'te yerel eklenti olarak), MCP sunucu kaydı. Yalnızca Respected\'a ait girdiler yazılır; ayarlarınız ve diğer girdiler korunur.',
      'İsteğe bağlı: Windows Görev Zamanlayıcı\'da günlük sabah özeti görevi, masaüstü kısayolu.',
    ],
    use: [
      'Oturum özetleri kasanızdaki daily/ ve knowledge/ klasörlerine yazılır.',
      'Arama dizini (veri klasöründe) notlarınız değiştikçe güncellenir.',
      'Özet için bilgisayarınızda kurulu AI aracınız (Claude Code, Codex…) çalıştırılır: oturum dökümü önce sır maskelemesinden geçer, sonra o araca gider. Dökümler varsayılan olarak saklanmaz.',
      'Anlamsal arama kapalı başlar; yalnızca siz isterseniz yerel bir model ve sunucusu veri klasörüne indirilir. Notlarınız bilgisayarınızdan çıkmaz.',
      'AI araçlarına yapılan her değişiklik kayıtlıdır; yarıda kalırsa geri alınır.',
    ],
    update: [
      'Programı kullanan süreçler (AI araçlarının açık MCP sunucuları dahil) durdurulur; program klasörü yeni sürümle yenilenir, eski sürümden kalan dosyalar silinir.',
      'Ardından ürün kendini onarır: dosyaları doğrular, AI araçlarındaki ve zamanlayıcıdaki kendi kayıtlarını yeni sürüme göre yeniler.',
      'Ayarlarınız, kasanız ve notlarınız değişmez. Bir adım başarısız olursa ürün kendi değişikliklerini geri alır.',
      'Sizin değiştirdiğiniz bir kayıt üzerine yazılmaz; çakışma olarak bildirilir.',
    ],
    uninstall: [
      'Program klasörü ve Windows Uygulamalar girdisi silinir; kurulum kaydı her zaman kalkar.',
      'AI araçlarındaki Respected girdileri, zamanlanmış görev ve kısayol geri alınır; sizin ve başka araçların girdileri kalır.',
      'Ayarlar ve çalışma verisi (veri klasörü) varsayılan olarak kalır; "çalışma verileri de silinsin" seçeneğiyle silinir.',
      'Not kasanız hiçbir durumda silinmez. Sizin değiştirdiğiniz bir sahipli dosya silinmez, çakışma olarak bildirilir.',
    ],
  },
  orchestrator: {
    install: [
      `Program: %LOCALAPPDATA%\\Respected\\RespectedOrchestrator\\app. ${NO_ADMIN} Windows Ayarlar › Uygulamalar listesinde bir giriş.`,
      'Veri klasörü: %LOCALAPPDATA%\\Respected\\RespectedOrchestrator\\data (ayarlar, işlem makbuzları, görev ve kampanya kayıtları, kanıtları imzalayan anahtar; anahtar yalnızca sizin hesabınıza açıktır).',
      'İsteğe bağlı: seçtiğiniz AI araçlarına MCP kaydı (Claude Code, Codex, Cursor, Gemini, Windsurf) ve Brain hafızasına bağlantı.',
    ],
    use: [
      'Her görev projenizin yalıtılmış bir Git çalışma ağacında çalışır (veri klasöründeki workspaces/ altında). Projenize ancak sizin onayınızla yama uygulanır.',
      'Görev durumu, kanıt ve kampanya kayıtları veri klasöründe birikir.',
      'Görevi, bilgisayarınızda kurulu AI aracı (Codex, Claude, Gemini, agy, Cursor) komut satırından yapar; görev metni o aracın sağlayıcısına gider.',
      'Doğrulama komutları kısıtlı bir Windows belirteciyle çalışır; çalışanın ortamından sır gibi görünen değişkenler çıkarılır.',
    ],
    update: [
      'Çalışan Orchestrator süreçleri durdurulur; program klasörü yeni sürümle yenilenir, eski sürümden kalan dosyalar silinir.',
      'Ardından ürün kendini onarır: dosyaları doğrular, AI araçlarındaki MCP kayıtlarını yeni sürüme göre yeniler.',
      'Ayarlarınız, görev geçmişiniz ve kanıtlarınız değişmez. Veri biçimi değişmişse ürün eski kaydı yerinde taşır ve özgününü .bak olarak saklar.',
    ],
    uninstall: [
      'Program klasörü ve Windows Uygulamalar girdisi silinir; kurulum kaydı her zaman kalkar.',
      'AI araçlarındaki Respected girdileri geri alınır; sizin ve başka araçların girdileri kalır.',
      'Ayarlar, görev geçmişi ve çalışma ağaçları (veri klasörü) varsayılan olarak kalır; "çalışma verileri de silinsin" seçeneğiyle silinir.',
      'Projelerinize ve Git depolarınıza dokunulmaz.',
    ],
  },
};

/** The product action whose facts explain it: a repair rewrites the program files the way an update does. */
export const phaseOf = (action: 'install' | 'update' | 'repair' | 'uninstall'): FactPhase => action === 'repair' ? 'update' : action;
