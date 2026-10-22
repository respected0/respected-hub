# Yerel test: urun kurucusu -> "Hub ile devam et" -> Hub'i yerel sunucudan indirir -> Hub urunu yerel katalogdan kurar.
#
#   .\hub-ile-test.ps1 orchestrator      # ya da: brain
#
# Hazir olanlar (benim derledigim):
#   ..\dist\RespectedHub-Windows-Setup.exe     Hub GELISTIRME kurucusu (yerel imza anahtarina guvenir; dagitim icin degil)
#   ..\catalog\                                yerel katalog: iki urunun kendi kurucu paketleri (Hub bunu kendiliginden bulur)
# Gercek konumlara (%LOCALAPPDATA%\Respected\...) kurar. Isin bitince: .\temizle.ps1
param([Parameter(Mandatory = $true)][ValidateSet('orchestrator', 'brain')][string]$Urun)

$ErrorActionPreference = 'Stop'
$kok = Split-Path -Parent (Split-Path -Parent $PSScriptRoot)   # üç deponun durduğu klasör
$hubKurucu = Join-Path $kok 'dist\RespectedHub-Windows-Setup.exe'
$urunKurucu = if ($Urun -eq 'orchestrator') { Join-Path $kok 'respected-orchestrator\dist\RespectedOrchestrator-Windows-Setup.exe' } else { Join-Path $kok 'respected-brain\dist\RespectedBrain-Windows-Setup.exe' }
foreach ($dosya in $hubKurucu, $urunKurucu, (Join-Path $kok 'catalog')) { if (-not (Test-Path $dosya)) { throw "Eksik: $dosya" } }

$hubVar = Test-Path "$env:LOCALAPPDATA\Respected\RespectedHub\app\respected-hub.exe"
if ($hubVar) { Write-Host "Hub zaten kurulu: kurucu dogrudan Hub'i acacak (indirme adimi atlanir)." -ForegroundColor Yellow }

# yerel "GitHub": Hub kurucusunu sunar
$sunucu = Start-Process python -ArgumentList '-m', 'http.server', '8123', '--bind', '127.0.0.1', '--directory', (Join-Path $kok 'dist') -WindowStyle Hidden -PassThru
Start-Sleep -Seconds 1
try {
    Write-Host "Kurucu aciliyor. 'Respected Hub ile devam et' secili halde Ileri'ye bas." -ForegroundColor Cyan
    Start-Process -FilePath $urunKurucu -ArgumentList '/HUBURL=http://127.0.0.1:8123/RespectedHub-Windows-Setup.exe' -Wait
    Write-Host "Kurucu kapandi. Hub acildiysa urunun kurulum sayfasi gorunmeli." -ForegroundColor Cyan
    Read-Host "Hub'da isin bitince Enter'a bas (yerel sunucu kapatilacak)"
}
finally {
    if ($sunucu -and -not $sunucu.HasExited) { Stop-Process -Id $sunucu.Id -Force }
}
