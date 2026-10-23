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

## Nereye ne gider

Program ve veriler `%LOCALAPPDATA%\Respected\RespectedOrchestrator\` altındadır. Orchestrator'ı kaldırmak programını ve AI
araçlarınıza eklediği girdileri kaldırır; projelerinize dokunmaz.

## Bu betanın sınırları

- Yalnızca Windows; kod imzası yok.
- Gemini çalışanları Windows'ta kullanılamaz.
- Sorunları https://github.com/respected0/respected-orchestrator/issues adresine bildirin.
