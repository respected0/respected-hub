# Yerel testin biraktigi her seyi kaldirir: urunler (kendi kaldiricilariyla), Hub ve Hub'in kendi klasorleri.
# Not kasalarina HIC dokunmaz. Urun ayarlari ve calisma verileri varsayilan olarak KALIR; onlari da silmek icin -VeriyiSil ver
# (urunlerin kaldiricisina /PURGE=1 gecer).
#   .\temizle.ps1              # programlar ve kurulum kayitlari
#   .\temizle.ps1 -VeriyiSil   # + urunlerin ayarlari ve calisma verileri
param([switch]$VeriyiSil)
$ErrorActionPreference = 'Continue'
$yerel = Join-Path $env:LOCALAPPDATA 'Respected'

function Kaldir($ad, $kaldirici, [string[]]$ek) {
    if (Test-Path $kaldirici) {
        Write-Host "$ad kaldiriliyor..." -ForegroundColor Cyan
        Start-Process -FilePath $kaldirici -ArgumentList (@('/VERYSILENT', '/SUPPRESSMSGBOXES', '/NORESTART') + $ek) -Wait
    } else { Write-Host "${ad}: kurulu degil." }
}

Get-Process respected-hub, respected-brain, respected-orchestrator -ErrorAction SilentlyContinue | Stop-Process -Force
$purge = if ($VeriyiSil) { @('/PURGE=1') } else { @() }
Kaldir 'Orchestrator' "$yerel\RespectedOrchestrator\app\unins000.exe" $purge
Kaldir 'Brain' "$yerel\RespectedBrain\app\unins000.exe" $purge
Kaldir 'Hub' "$yerel\RespectedHub\app\unins000.exe" @()
Start-Sleep -Seconds 5   # Inno'nun kaldirici kopyasi bir an sonra biter

# eski gelistirme Hub'inin biraktigi klasorler (bugunku Hub bunlari yazmaz)
foreach ($yol in "$env:LOCALAPPDATA\RespectedOrchestratorDevelopment", "$yerel\RespectedHub\managed") {
    if (Test-Path $yol) { Write-Host "Siliniyor: $yol" -ForegroundColor Yellow; Remove-Item $yol -Recurse -Force -ErrorAction SilentlyContinue }
}

# Icinde hic dosya kalmayan Respected klasorlerini kaldir
if (Test-Path $yerel) {
    Get-ChildItem $yerel -Directory | Where-Object { -not (Get-ChildItem $_.FullName -Recurse -File -ErrorAction SilentlyContinue) } | Remove-Item -Recurse -Force -ErrorAction SilentlyContinue
    if (-not (Get-ChildItem $yerel -Recurse -File -ErrorAction SilentlyContinue)) { Remove-Item $yerel -Recurse -Force -ErrorAction SilentlyContinue }
}

Write-Host ''
Write-Host 'Kalanlar:' -ForegroundColor Yellow
if (Test-Path $yerel) { Get-ChildItem $yerel -Recurse -Depth 2 -ErrorAction SilentlyContinue | Select-Object -ExpandProperty FullName } else { Write-Host '(Respected klasoru kalmadi)' }
$girdi = Get-ChildItem 'HKCU:\Software\Microsoft\Windows\CurrentVersion\Uninstall' -ErrorAction SilentlyContinue | Get-ItemProperty | Where-Object { $_.DisplayName -like 'Respected*' }
if ($girdi) { Write-Host 'Yuklu uygulamalar girdileri:' -ForegroundColor Yellow; $girdi | Select-Object DisplayName, DisplayVersion } else { Write-Host 'Yuklu uygulamalar girdisi kalmadi.' -ForegroundColor Green }
