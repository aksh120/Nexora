# scripts/create_icon.ps1
# Generates a clean 48x48 PNG-in-ICO file with Nexora electric cyan visual identity.

Add-Type -AssemblyName System.Drawing

$size = 48
$bmp = New-Object System.Drawing.Bitmap($size, $size)
$gfx = [System.Drawing.Graphics]::FromImage($bmp)
$gfx.SmoothingMode = [System.Drawing.Drawing2D.SmoothingMode]::AntiAlias

# Background dark slate rounded rect
$brushBg = New-Object System.Drawing.SolidBrush([System.Drawing.Color]::FromArgb(255, 11, 14, 20))
$gfx.FillRectangle($brushBg, 0, 0, $size, $size)

# Foreground electric cyan chevron / lightning symbol
$penCyan = New-Object System.Drawing.Pen([System.Drawing.Color]::FromArgb(255, 0, 229, 255), 4)
$points = @(
    [System.Drawing.Point]::new(12, 12),
    [System.Drawing.Point]::new(26, 24),
    [System.Drawing.Point]::new(20, 26),
    [System.Drawing.Point]::new(36, 36)
)
$gfx.DrawLines($penCyan, $points)

# Highlight dot
$brushWhite = New-Object System.Drawing.SolidBrush([System.Drawing.Color]::FromArgb(255, 255, 255, 255))
$gfx.FillEllipse($brushWhite, 34, 34, 4, 4)

$pngStream = New-Object System.IO.MemoryStream
$bmp.Save($pngStream, [System.Drawing.Imaging.ImageFormat]::Png)
$pngBytes = $pngStream.ToArray()

# Build ICO with embedded PNG
$icoStream = New-Object System.IO.MemoryStream
$writer = New-Object System.IO.BinaryWriter($icoStream)

# ICONDIR
$writer.Write([uint16]0) # Reserved
$writer.Write([uint16]1) # Type (1 = ICO)
$writer.Write([uint16]1) # 1 image

# ICONDIRENTRY
$writer.Write([byte]$size)        # Width
$writer.Write([byte]$size)        # Height
$writer.Write([byte]0)            # Color count
$writer.Write([byte]0)            # Reserved
$writer.Write([uint16]1)          # Color planes
$writer.Write([uint16]32)         # Bits per pixel
$writer.Write([uint32]$pngBytes.Length) # Size of image data
$writer.Write([uint32]22)         # Offset of image data (6 + 16 = 22)

# Image data (PNG bytes)
$writer.Write($pngBytes)
$writer.Flush()

$outPath = Join-Path $PSScriptRoot "..\assets\branding\icon.ico"
[System.IO.File]::WriteAllBytes($outPath, $icoStream.ToArray())

$gfx.Dispose()
$bmp.Dispose()
$pngStream.Dispose()
$icoStream.Dispose()

Write-Host "Generated icon.ico at: $outPath ($($icoStream.Length) bytes)"
