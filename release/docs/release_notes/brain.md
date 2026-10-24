# Respected Brain {version}

Respected Brain, AI araçlarınızın ortak hafızasıdır: oturumları özetler, günlüklerden bağlantılı bilgi notları derler ve yeni
oturumda bağlamı aracınıza verir. Notlar kendi kasanızda, düz Markdown olarak durur. Windows 10/11, kullanıcı başına
kurulum, yönetici izni ve Python gerekmez.

## Yenilikler

{notes}

## İndirme

`Respected-Brain-<sürüm>-Windows-x64-Setup.exe`: Brain'i tek başına kurar; isterseniz kurucuda "Respected Hub ile devam et"
seçeneğiyle Hub da kurulur. Brain Hub olmadan da kurulur, ayarlanır, onarılır ve kaldırılır.

## Windows uyaracak (imzasız beta)

Kurucu **Authenticode ile imzalı değil** (ücretli sertifika henüz yok). Windows "Bilinmeyen yayıncı" / SmartScreen uyarısı
gösterir: yalnızca özet eşleşiyorsa *Ek bilgi › Yine de çalıştır* seçin. Çalıştırmadan önce `Get-FileHash <dosya> -Algorithm SHA256`
sonucu `SHA256SUMS.txt` içindeki satırla aynı olmalı.

## Güncelleme

Yeni sürümün kurucusunu çalıştırıp **Onar / Güncelle**'yi seçin (Hub kuruluysa Hub de önerir). Ayarlar, veriler ve kasa
korunur; eski sürüme dönüş desteklenmez.

## Kurulumdan sonra

Kullandığın AI aracını yeniden başlat ve aracın ilgili yerinden (eklentiler, hook'lar) Brain'in istediği onayları ver. Onaylamazsan hafıza yazılmaz.

## Nereye ne gider

Program ve veriler `%LOCALAPPDATA%\Respected\RespectedBrain\` altındadır. Not kasanız seçtiğiniz yerde durur (varsayılan
`Belgeler\RespectedOS`; bu betada **OneDrive ile eşitlenen bir klasöre koymayın**). Brain'i kaldırmak programını ve AI
araçlarınıza eklediği girdileri kaldırır; **kasa ve notlar hiçbir durumda silinmez**.

## Gizlilik

Hesap ve telemetri yoktur. Hangi verinin nerede durduğu ve neyin dışarı çıktığı: https://github.com/respected0/respected-brain/blob/main/PRIVACY.md

## Bu betanın sınırları

- Yalnızca Windows; kod imzası yok.
- Claude Code uçtan uca doğrulandı; Codex, Cursor, Gemini ve Antigravity kayıtları testlerle kapsanıyor ama gerçek oturumlarla
  henüz değil (deneysel sayın).
- Sorunları https://github.com/respected0/respected-brain/issues adresine bildirin.
