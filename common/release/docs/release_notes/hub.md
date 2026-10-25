# Respected Hub {version}

Respected Hub, Respected ürünlerini kuran, güncelleyen ve görsel olarak yöneten masaüstü uygulamasıdır. Windows 10/11,
kullanıcı başına kurulum, yönetici izni gerekmez.

## Yenilikler

{notes}

## İndirme

`Respected-Hub-<sürüm>-Windows-x64-Setup.exe`: Hub'ı kurar; ürünleri Hub'ın içinden kurarsınız.

## Windows uyaracak (imzasız beta)

Kurucu **Authenticode ile imzalı değil** (ücretli sertifika henüz yok). Windows "Bilinmeyen yayıncı" / SmartScreen uyarısı
gösterir: yalnızca özet eşleşiyorsa *Ek bilgi › Yine de çalıştır* seçin. Çalıştırmadan önce `Get-FileHash <dosya> -Algorithm SHA256`
sonucu, bu sayfada dosyanın yanında GitHub'ın gösterdiği `sha256:` değeriyle aynı olmalı.

## Güncelleme

Hub yeni bir sürüm olduğunda söyler ve kurmadan önce sorar; hiçbir şey sessizce güncellenmez. Ürün kataloğu ve paketleri
yayın anahtarıyla (`dabcb5265227bbdf`) imzalıdır; Hub imzası doğrulanmayan hiçbir şeyi kurmaz. Hub'ı kaldırmak, kurduğu
ürünlere dokunmaz.

## Gizlilik

Hesap ve telemetri yoktur. Hangi verinin nerede durduğu ve neyin dışarı çıktığı: https://github.com/respected0/respected-hub/blob/main/PRIVACY.md

## Bu betanın sınırları

- Yalnızca Windows; kod imzası yok.
- Sorunları Hub'ın *Etkinlik* metni ve Windows sürümünüzle https://github.com/respected0/respected-hub/issues adresine bildirin.
