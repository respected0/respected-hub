# Draws the installer wizard image (the welcome/finish side image) from the brand PNGs.
#   powershell -File wizard_images.ps1 -Product hub|brain|orchestrator -OutDir <folder>
# Output: side-<w>x<h>.bmp (four DPI sizes). Inno Setup reads BMP only.
param([Parameter(Mandatory)][ValidateSet('hub', 'brain', 'orchestrator')][string]$Product, [Parameter(Mandatory)][string]$OutDir)
Add-Type -AssemblyName System.Drawing
$brand = Join-Path $PSScriptRoot '..\..\..\assets\brand'
$logo = [System.Drawing.Image]::FromFile((Resolve-Path (Join-Path $brand "$Product-1024.png")))
$names = @{ hub = 'Hub'; brain = 'Brain'; orchestrator = 'Orchestrator' }
New-Item -ItemType Directory -Force $OutDir | Out-Null

function New-Canvas($w, $h, $color) {
    $bmp = New-Object System.Drawing.Bitmap $w, $h, ([System.Drawing.Imaging.PixelFormat]::Format24bppRgb)
    $g = [System.Drawing.Graphics]::FromImage($bmp)
    $g.SmoothingMode = 'AntiAlias'; $g.InterpolationMode = 'HighQualityBicubic'; $g.TextRenderingHint = 'AntiAliasGridFit'; $g.PixelOffsetMode = 'HighQuality'
    $g.Clear($color)
    return @{ Bitmap = $bmp; Graphics = $g }
}

function Save-Bmp($canvas, $path) { $canvas.Graphics.Dispose(); $canvas.Bitmap.Save($path, [System.Drawing.Imaging.ImageFormat]::Bmp); $canvas.Bitmap.Dispose() }

$paper = [System.Drawing.Color]::FromArgb(243, 246, 250)
$ink = [System.Drawing.Color]::FromArgb(17, 22, 29)
$muted = [System.Drawing.Color]::FromArgb(110, 124, 142)
$line = [System.Drawing.Color]::FromArgb(222, 228, 236)

foreach ($size in @(@(164, 314), @(192, 366), @(246, 470), @(328, 628))) {
    $w = $size[0]; $h = $size[1]; $k = $w / 164.0
    $c = New-Canvas $w $h $paper
    $g = $c.Graphics
    $edge = New-Object System.Drawing.Pen $line, ([Math]::Max(1, [int]$k))
    $g.DrawLine($edge, $w - 1, 0, $w - 1, $h)
    $logoSize = [int](92 * $k)
    $g.DrawImage($logo, [int](($w - $logoSize) / 2), [int](96 * $k), $logoSize, $logoSize)
    $center = New-Object System.Drawing.StringFormat; $center.Alignment = 'Center'
    $small = New-Object System.Drawing.Font 'Segoe UI', ([single](8.5 * $k)), ([System.Drawing.FontStyle]::Regular), ([System.Drawing.GraphicsUnit]::Pixel)
    $big = New-Object System.Drawing.Font 'Segoe UI Semibold', ([single](17 * $k)), ([System.Drawing.FontStyle]::Regular), ([System.Drawing.GraphicsUnit]::Pixel)
    $g.DrawString('RESPECTED', $small, (New-Object System.Drawing.SolidBrush $muted), (New-Object System.Drawing.RectangleF 0, (206 * $k), $w, (14 * $k)), $center)
    $g.DrawString($names[$Product], $big, (New-Object System.Drawing.SolidBrush $ink), (New-Object System.Drawing.RectangleF 0, (222 * $k), $w, (26 * $k)), $center)
    Save-Bmp $c (Join-Path $OutDir "side-${w}x${h}.bmp")
}

$logo.Dispose()
