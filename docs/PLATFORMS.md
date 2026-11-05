# Platform yetenek matrisi

`PORT-001`: bir platform ancak ayni kontrat testleri gercek bir kosucuda gecerse "destekleniyor" sayilir. Kanit: 2026-11-05 tarihli `ci` kosusu [41958104756](https://github.com/respected0/respected-hub/actions/runs/41958104756).

| Yetenek | Windows | macOS | Linux |
|---|---|---|---|
| 5 istemcinin kesfi (`integrations::discovery`) | Evet, CI | Evet, CI | Evet, CI |
| Salt okunur envanter, cozumleme, drift | Evet, CI | Evet, CI | Evet, CI |
| Entegrasyon plan/apply/undo ve adapter kontratlari | Evet, CI | Evet, CI | Evet, CI |
| TypeScript kontrolu ve birim testleri | Evet, CI | Evet, CI | Evet, CI |
| Uretim guven kumesi (`production_build`) | Evet, CI | Evet, CI | Evet, CI |
| Urun kuruculari (Brain/Orchestrator Setup) | Evet, CI | Desteklenmiyor | Desteklenmiyor |
| Urun motoru (RPP, kampanya) | Evet, CI | Dogrulanmadi | Dogrulanmadi |
| Yayin paketleme (`build_installer.py`) | Evet, CI | Desteklenmiyor | Desteklenmiyor |
| Masaustu paketleme/imzalama | Evet, CI | Dogrulanmadi | Dogrulanmadi |

Kontrat sayilari: her iki platformda 149 birim + 34 entegrasyon + 1 uretim guven testi; `npm run check` 132 test. Windows isi tam `cargo test --no-fail-fast` paketini calistirmaya devam eder.

macOS/Linux'ta urun kurulumu ve urun motoru testleri bilerek kosulmaz: kurucu dosyalari Windows Setup bicimindedir. Bu yetenekler icin "destekleniyor" etiketi ancak hedef OS'te gercek bir kurulum/motor kosusu yesil oldugunda eklenir.
