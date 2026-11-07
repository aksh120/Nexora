<#
.SYNOPSIS
    Automated Release Packaging Script for NEXORA v0.1.0
.DESCRIPTION
    Packages the production release build into portable archive (.zip) and installer distribution.
#>

param(
    [string]$Version = "0.1.0"
)

$ErrorActionPreference = "Stop"

$projectRoot = Split-Path -Parent $PSScriptRoot
$releaseExe = Join-Path $projectRoot "target\release\Nexora.exe"
$distDir = Join-Path $projectRoot "dist"
$portableStagingDir = Join-Path $distDir "Nexora-v$Version-portable"
$zipPath = Join-Path $distDir "Nexora-v$Version-windows-x64-portable.zip"

Write-Host "========================================================" -ForegroundColor Cyan
Write-Host " NEXORA RELEASE PACKAGING PIPELINE v$Version " -ForegroundColor Yellow
Write-Host "========================================================" -ForegroundColor Cyan

if (-not (Test-Path $releaseExe)) {
    Write-Error "Release binary not found at '$releaseExe'. Run 'cargo build --release' first."
}

# 1. Inspect binary
$exeItem = Get-Item $releaseExe
$exeSizeMB = [Math]::Round($exeItem.Length / 1MB, 2)
$exeSizeKB = [Math]::Round($exeItem.Length / 1KB, 1)
$sha256 = (Get-FileHash -Path $releaseExe -Algorithm SHA256).Hash

Write-Host "Binary: Nexora.exe" -ForegroundColor Green
Write-Host "Size:   $exeSizeMB MB ($exeSizeKB KB)" -ForegroundColor Green
Write-Host "SHA256: $sha256" -ForegroundColor Green

# 2. Prepare dist directory
if (Test-Path $distDir) {
    Remove-Item -Path $distDir -Recurse -Force
}
New-Item -ItemType Directory -Path $distDir -Force | Out-Null
New-Item -ItemType Directory -Path $portableStagingDir -Force | Out-Null

# 3. Assemble Portable Distribution
Copy-Item -Path $releaseExe -Destination $portableStagingDir -Force
Copy-Item -Path (Join-Path $projectRoot "README.md") -Destination $portableStagingDir -Force -ErrorAction SilentlyContinue
Copy-Item -Path (Join-Path $projectRoot "LICENSE") -Destination $portableStagingDir -Force -ErrorAction SilentlyContinue
Copy-Item -Path (Join-Path $projectRoot "CHANGELOG.md") -Destination $portableStagingDir -Force -ErrorAction SilentlyContinue

$stagingAssets = Join-Path $portableStagingDir "assets"
New-Item -ItemType Directory -Path $stagingAssets -Force | Out-Null
Copy-Item -Path (Join-Path $projectRoot "assets\branding") -Destination $stagingAssets -Recurse -Force

# Create ZIP archive
Compress-Archive -Path "$portableStagingDir\*" -DestinationPath $zipPath -CompressionLevel Optimal
$zipItem = Get-Item $zipPath
$zipSizeMB = [Math]::Round($zipItem.Length / 1MB, 2)
$zipSha256 = (Get-FileHash -Path $zipPath -Algorithm SHA256).Hash

Write-Host "`nCreated Portable ZIP:" -ForegroundColor Yellow
Write-Host "Path:   $zipPath" -ForegroundColor Gray
Write-Host "Size:   $zipSizeMB MB" -ForegroundColor Gray
Write-Host "SHA256: $zipSha256" -ForegroundColor Gray

# 4. Check for Inno Setup Compiler (ISCC)
$isccPaths = @(
    "C:\Program Files (x86)\Inno Setup 6\ISCC.exe",
    "C:\Program Files\Inno Setup 6\ISCC.exe",
    (Get-Command iscc.exe -ErrorAction SilentlyContinue | Select-Object -ExpandProperty Source)
)

$iscc = $isccPaths | Where-Object { $_ -and (Test-Path $_) } | Select-Object -First 1

if ($iscc) {
    Write-Host "`nInno Setup Compiler found: $iscc" -ForegroundColor Green
    $issFile = Join-Path $projectRoot "installer\nexora_setup.iss"
    Write-Host "Compiling installer from: $issFile" -ForegroundColor Cyan
    & $iscc $issFile
} else {
    Write-Host "`nNote: Inno Setup (ISCC.exe) not found on PATH. The installer script is authored at installer/nexora_setup.iss." -ForegroundColor Gray
}

# 5. Generate Release Manifest JSON
$manifest = [PSCustomObject]@{
    Version         = $Version
    BuildDate       = (Get-Date).ToString("o")
    Target          = "x86_64-pc-windows-gnu"
    Binary = [PSCustomObject]@{
        Name        = "Nexora.exe"
        SizeBytes   = $exeItem.Length
        SizeMB      = $exeSizeMB
        SHA256      = $sha256
    }
    PortableArchive = [PSCustomObject]@{
        Name        = [System.IO.Path]::GetFileName($zipPath)
        SizeBytes   = $zipItem.Length
        SizeMB      = $zipSizeMB
        SHA256      = $zipSha256
    }
    InstallerScript = "installer/nexora_setup.iss"
    Optimizations   = @("opt-level=3", "lto=thin", "panic=abort", "strip=true", "codegen-units=1")
}

$manifestPath = Join-Path $distDir "release_manifest.json"
$manifest | ConvertTo-Json -Depth 4 | Set-Content -Path $manifestPath -Encoding UTF8
Write-Host "`nRelease Manifest written to: $manifestPath" -ForegroundColor Green
Write-Host "Release packaging pipeline completed successfully." -ForegroundColor Green
