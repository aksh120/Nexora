# Generates multi-resolution .ico and .png icons for Nexora
Add-Type -AssemblyName System.Drawing

function Draw-QuantumPrismBitmap([int]$size) {
    $bmp = New-Object System.Drawing.Bitmap($size, $size, [System.Drawing.Imaging.PixelFormat]::Format32bppArgb)
    $g = [System.Drawing.Graphics]::FromImage($bmp)
    $g.SmoothingMode = [System.Drawing.Drawing2D.SmoothingMode]::HighQuality
    $g.InterpolationMode = [System.Drawing.Drawing2D.InterpolationMode]::HighQualityBicubic
    $g.PixelOffsetMode = [System.Drawing.Drawing2D.PixelOffsetMode]::HighQuality
    $g.Clear([System.Drawing.Color]::Transparent)

    $scale = $size / 256.0

    # 1. Base rounded rectangle (AMOLED Black with Crimson border)
    $pad = 8.0 * $scale
    $rectSize = 240.0 * $scale
    $radius = 54.0 * $scale
    $diameter = $radius * 2.0

    $basePath = New-Object System.Drawing.Drawing2D.GraphicsPath
    $basePath.AddArc($pad, $pad, $diameter, $diameter, 180, 90)
    $basePath.AddArc($pad + $rectSize - $diameter, $pad, $diameter, $diameter, 270, 90)
    $basePath.AddArc($pad + $rectSize - $diameter, $pad + $rectSize - $diameter, $diameter, $diameter, 0, 90)
    $basePath.AddArc($pad, $pad + $rectSize - $diameter, $diameter, $diameter, 90, 90)
    $basePath.CloseFigure()

    $blackBrush = New-Object System.Drawing.SolidBrush([System.Drawing.Color]::FromArgb(255, 0, 0, 0))
    $g.FillPath($blackBrush, $basePath)
    $blackBrush.Dispose()

    $borderPen = New-Object System.Drawing.Pen([System.Drawing.Color]::FromArgb(255, 41, 21, 24), [Math]::Max(1.0, 4.0 * $scale))
    $g.DrawPath($borderPen, $basePath)
    $borderPen.Dispose()
    $basePath.Dispose()

    # Monogram transform: translate(32, 32) scale(1.5 * scale)
    $tx = 32.0 * $scale
    $ty = 32.0 * $scale
    $mScale = 1.5 * $scale

    function M-Pt([double]$x, [double]$y) {
        return New-Object System.Drawing.PointF(($tx + $x * $mScale), ($ty + $y * $mScale))
    }

    # Left Wing main facet: M22 20 L58 20 L38 88 L18 88 Z
    $pLeft1 = @( (M-Pt 22 20), (M-Pt 58 20), (M-Pt 38 88), (M-Pt 18 88) )
    $gradLeft1 = New-Object System.Drawing.Drawing2D.LinearGradientBrush(
        (M-Pt 16 16), (M-Pt 72 96),
        [System.Drawing.Color]::FromArgb(255, 255, 56, 92),
        [System.Drawing.Color]::FromArgb(255, 225, 29, 72)
    )
    $g.FillPolygon($gradLeft1, $pLeft1)
    $gradLeft1.Dispose()

    # Left Wing secondary facet: M58 20 L78 68 L58 68 L38 20 Z
    $pLeft2 = @( (M-Pt 58 20), (M-Pt 78 68), (M-Pt 58 68), (M-Pt 38 20) )
    $brushLeft2 = New-Object System.Drawing.SolidBrush([System.Drawing.Color]::FromArgb(217, 253, 164, 175))
    $g.FillPolygon($brushLeft2, $pLeft2)
    $brushLeft2.Dispose()

    # Right Wing main facet: M106 108 L70 108 L90 40 L110 40 Z
    $pRight1 = @( (M-Pt 106 108), (M-Pt 70 108), (M-Pt 90 40), (M-Pt 110 40) )
    $gradRight1 = New-Object System.Drawing.Drawing2D.LinearGradientBrush(
        (M-Pt 112 112), (M-Pt 56 32),
        [System.Drawing.Color]::FromArgb(255, 251, 113, 133),
        [System.Drawing.Color]::FromArgb(255, 159, 18, 57)
    )
    $g.FillPolygon($gradRight1, $pRight1)
    $gradRight1.Dispose()

    # Right Wing secondary facet: M70 108 L50 60 L70 60 L90 108 Z
    $pRight2 = @( (M-Pt 70 108), (M-Pt 50 60), (M-Pt 70 60), (M-Pt 90 108) )
    $brushRight2 = New-Object System.Drawing.SolidBrush([System.Drawing.Color]::FromArgb(217, 244, 63, 94))
    $g.FillPolygon($brushRight2, $pRight2)
    $brushRight2.Dispose()

    # Center Negative-Space Facet: points="50,60 78,68 64,104"
    $pCenter = @( (M-Pt 50 60), (M-Pt 78 68), (M-Pt 64 104) )
    $brushCenter = New-Object System.Drawing.SolidBrush([System.Drawing.Color]::FromArgb(153, 159, 18, 57))
    $g.FillPolygon($brushCenter, $pCenter)
    $brushCenter.Dispose()

    # Center Node: cx=64, cy=64, r=3.5
    $cNode = M-Pt 64 64
    $rNode = [Math]::Max(1.0, 3.5 * $mScale)
    $whiteBrush = New-Object System.Drawing.SolidBrush([System.Drawing.Color]::FromArgb(255, 255, 255, 255))
    $g.FillEllipse($whiteBrush, [float]($cNode.X - $rNode), [float]($cNode.Y - $rNode), [float]($rNode * 2.0), [float]($rNode * 2.0))
    $whiteBrush.Dispose()

    $g.Dispose()
    return $bmp
}

# Sizes for high-DPI Windows icon: 16, 24, 32, 48, 64, 128, 256
$sizes = @(16, 24, 32, 48, 64, 128, 256)
$pngBytesList = @()

foreach ($s in $sizes) {
    $bmp = Draw-QuantumPrismBitmap -size $s
    $ms = New-Object System.IO.MemoryStream
    $bmp.Save($ms, [System.Drawing.Imaging.ImageFormat]::Png)
    $pngBytesList += ,@($s, $ms.ToArray())
    $ms.Dispose()

    if ($s -eq 256) {
        $bmp.Save("assets/branding/icon_256.png", [System.Drawing.Imaging.ImageFormat]::Png)
        $bmp.Save("assets/branding/app_icon.png", [System.Drawing.Imaging.ImageFormat]::Png)
    }
    $bmp.Dispose()
}

# Construct ICO binary stream
$icoMs = New-Object System.IO.MemoryStream
$bw = New-Object System.IO.BinaryWriter($icoMs)

# ICONDIR header
$bw.Write([uint16]0) # Reserved
$bw.Write([uint16]1) # Type: 1 = ICO
$bw.Write([uint16]$pngBytesList.Count) # Count of images

$headerSize = 6
$entrySize = 16
$offset = $headerSize + ($pngBytesList.Count * $entrySize)

# Write ICONDIRENTRY for each
foreach ($item in $pngBytesList) {
    $size = $item[0]
    $bytes = $item[1]

    $w = if ($size -ge 256) { [byte]0 } else { [byte]$size }
    $h = if ($size -ge 256) { [byte]0 } else { [byte]$size }

    $bw.Write($w) # Width
    $bw.Write($h) # Height
    $bw.Write([byte]0) # Color count
    $bw.Write([byte]0) # Reserved
    $bw.Write([uint16]1) # Color planes
    $bw.Write([uint16]32) # Bits per pixel
    $bw.Write([uint32]$bytes.Length) # Image size in bytes
    $bw.Write([uint32]$offset) # Image offset

    $offset += $bytes.Length
}

# Write PNG bytes
foreach ($item in $pngBytesList) {
    $bw.Write($item[1])
}

$bw.Flush()
$icoBytes = $icoMs.ToArray()
$bw.Dispose()
$icoMs.Dispose()

[System.IO.File]::WriteAllBytes("assets/branding/icon.ico", $icoBytes)
[System.IO.File]::WriteAllBytes("assets/icons/nexora.ico", $icoBytes)

Write-Host "Successfully generated Nexora official .ico files ($($icoBytes.Length) bytes, 7 resolutions: 16 to 256px)"

