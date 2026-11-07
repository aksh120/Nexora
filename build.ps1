# build.ps1 - Nexora Automated Build Script
param(
    [switch]$Release,
    [switch]$Test,
    [switch]$Run
)

$ErrorActionPreference = "Stop"
$env:CARGO_INCREMENTAL = "0"
$cargo = "$env:USERPROFILE\.cargo\bin\cargo.exe"

if (-not (Test-Path $cargo)) {
    Write-Error "Cargo binary not found at $cargo"
    exit 1
}

if ($Test) {
    Write-Host "Running Nexora Test Suite..." -ForegroundColor Cyan
    & $cargo test
    if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
}

$buildArgs = @("build")
if ($Release) {
    $buildArgs += "--release"
}

Write-Host "Compiling Nexora (Args: $($buildArgs -join ' '))..." -ForegroundColor Cyan
& $cargo @buildArgs
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }

Write-Host "Build completed successfully." -ForegroundColor Green

if ($Run) {
    $targetDir = if ($Release) { "release" } else { "debug" }
    $exe = ".\target\$targetDir\Nexora.exe"
    Write-Host "Launching $exe..." -ForegroundColor Green
    & $exe
}

