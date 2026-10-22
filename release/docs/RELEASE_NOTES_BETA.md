# Respected Windows Beta 0.1.0-beta.1

**SecondBrain** AI araçlarınızın ortak hafızasıdır; notlar kendi kasanızda durur. **Orchestrator** çok ajanlı bir görev
motorudur. **Respected Hub** ikisini kurar, günceller ve görsel olarak yönetir. Windows 10/11, kullanıcı başına kurulum,
yönetici izni gerekmez.

## İndirme

| Dosya | Ne yapar |
|---|---|
| `RespectedHub-Windows-Setup.exe` | Hub'ı kurar; ürünleri Hub'ın içinden kurarsınız |
| `RespectedSecondBrain-Windows-Setup.exe` | SecondBrain'i kurar; "Respected Hub ile devam et" seçilirse Hub'ı da kurar |
| `RespectedOrchestrator-Windows-Setup.exe` | Orchestrator'ı kurar; aynı seçenekle |

Her ürün Hub olmadan da kurulur, ayarlanır, onarılır ve kaldırılır. Hub'dan kurarken ürünün kurucusu bir kez
internetten indirilir.

## Windows uyaracak (imzasız beta)

Kurucular **Authenticode ile imzalı değil** (ücretli sertifika henüz yok). Windows "Bilinmeyen yayıncı" / SmartScreen
uyarısı gösterir: yalnızca aşağıdaki özet eşleşiyorsa *Ek bilgi › Yine de çalıştır* seçin.
Çalıştırmadan önce: `Get-FileHash <dosya> -Algorithm SHA256` sonucu `SHA256SUMS.txt` içindeki satırla aynı olmalı.
`SHA256SUMS.sig` bu listenin Ed25519 imzasıdır (yayın anahtarı `dabcb5265227bbdf`); ürün kataloğu ve her ürün paketi aynı
anahtarla imzalıdır ve Hub başka hiçbir şeyi kurmaz.

## Güncellemeler

Hub yeni bir Hub ya da ürün sürümü olduğunda söyler ve kurmadan önce sorar; hiçbir şey sessizce güncellenmez.
Hub'sız güncelleme: yeni sürümün kurucusunu çalıştırıp **Onar / güncelle**'yi seçin. Ayarlar, veriler ve kasa korunur;
eski sürüme dönüş desteklenmez.

## Nereye ne gider

Programlar ve verileri `%LOCALAPPDATA%\Respected\` altındadır (`RespectedHub`, `RespectedBrain`, `RespectedOrchestrator`).
Not kasanız seçtiğiniz yerde durur (varsayılan `Belgeler\RespectedOS`; bu betada **OneDrive ile eşitlenen bir klasöre
koymayın**). Her ürün *Windows Ayarlar › Uygulamalar* listesinde ayrı görünür. Bir ürünü kaldırmak programını ve AI
araçlarınıza eklediği kendi girdilerini kaldırır; ayarlar ve veriler isterseniz silinir; **kasa ve notlar hiçbir
durumda silinmez**. Hub'ı kaldırmak ürünlere dokunmaz.

## Bu betanın sınırları

- Yalnızca Windows.
- Kod imzası yok; yukarıdaki SmartScreen uyarısı.
- AI araçları: Claude Code uçtan uca doğrulandı; Codex, Cursor, Gemini ve Antigravity kayıtları testlerle kapsanıyor ama
  gerçek oturumlarla henüz değil (deneysel sayın). Gemini çalışanları Windows'ta kullanılamaz.
- Sorunları Hub'ın *Etkinlik* metni ve Windows sürümünüzle https://github.com/respected0/respected-hub/issues adresine
  bildirin.
