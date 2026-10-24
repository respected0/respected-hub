# Respected Orchestrator {version}

Respected Orchestrator, kodlama ajanlarını yalıtılmış Git çalışma ağaçlarında çalıştıran ve sonucu denetleyen yerel görev
motorudur. Windows 10/11, kullanıcı başına kurulum, yönetici izni ve Python gerekmez; Git gerekir.

## Yenilikler

{notes}

## İndirme

`Respected-Orchestrator-<sürüm>-Windows-x64-Setup.exe`: Orchestrator'ı tek başına kurar; isterseniz kurucuda "Respected Hub
ile devam et" seçeneğiyle Hub da kurulur. Orchestrator Hub olmadan da kurulur, ayarlanır, onarılır ve kaldırılır.

## Windows uyaracak (imzasız beta)

Kurucu **Authenticode ile imzalı değil** (ücretli sertifika henüz yok). Windows "Bilinmeyen yayıncı" / SmartScreen uyarısı
gösterir: yalnızca özet eşleşiyorsa *Ek bilgi › Yine de çalıştır* seçin. Çalıştırmadan önce `Get-FileHash <dosya> -Algorithm SHA256`
sonucu `SHA256SUMS.txt` içindeki satırla aynı olmalı.

## Güncelleme

Yeni sürümün kurucusunu çalıştırıp **Onar / Güncelle**'yi seçin (Hub kuruluysa Hub de önerir). Ayarlar ve veriler korunur;
eski sürüme dönüş desteklenmez.

## Kurulumdan sonra

Kullandığınız AI aracını yeniden başlatın ve aracın ilgili yerinden (eklentiler, MCP) Orchestrator'a izin verin. Yoksa araç Orchestrator'ı görmez.

## Nereye ne gider

Program ve veriler `%LOCALAPPDATA%\Respected\RespectedOrchestrator\` altındadır. Orchestrator'ı kaldırmak programını ve AI
araçlarınıza eklediği girdileri kaldırır; projelerinize dokunmaz.

## Gizlilik

Hesap ve telemetri yoktur. Hangi verinin nerede durduğu ve neyin dışarı çıktığı: https://github.com/respected0/respected-orchestrator/blob/main/PRIVACY.md

## Bu betanın sınırları

- Yalnızca Windows; kod imzası yok.
- Gemini çalışanı, npm'in `.cmd` kabuğuyla kurulmuşsa Windows'ta kullanılamaz.
- Çalışan araç bilgisayarınızda sıradan bir süreçtir: çalışma ağacının dışına yazarsa Orchestrator bunu yakalamaz.
- Sorunları https://github.com/respected0/respected-orchestrator/issues adresine bildirin.
