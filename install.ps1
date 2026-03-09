# Vault Secret Scanner (vss-can) - Windows Installer
# Usage: irm https://raw.githubusercontent.com/markush0f/vault-secret-scanner/main/install.ps1 | iex

$ErrorActionPreference = "Stop"

Write-Host "======================================" -ForegroundColor Cyan
Write-Host "   Vault Secret Scanner (vss-can) Installer" -ForegroundColor Green
Write-Host "======================================" -ForegroundColor Cyan
Write-Host ""

$repo   = "markush0f/vault-secret-scanner"
$asset  = "vss-can-x86_64-windows.zip"
$url    = "https://github.com/$repo/releases/latest/download/$asset"

# Determine install location inside the user profile so that no elevation is required.
$installDir = Join-Path $env:LOCALAPPDATA "vss-can\bin"
if (-not (Test-Path $installDir)) {
    New-Item -ItemType Directory -Path $installDir | Out-Null
}

$tmpDir   = Join-Path ([System.IO.Path]::GetTempPath()) ([System.IO.Path]::GetRandomFileName())
New-Item -ItemType Directory -Path $tmpDir | Out-Null
$zipPath  = Join-Path $tmpDir $asset
$exeDest  = Join-Path $installDir "vss-can.exe"

try {
    Write-Host "1> Detecting system: Windows x86_64" -ForegroundColor Cyan
    Write-Host "2> Downloading $asset from GitHub Releases..." -ForegroundColor Cyan

    # Download the release asset.
    Invoke-WebRequest -Uri $url -OutFile $zipPath -UseBasicParsing

    Write-Host "3> Extracting executable..." -ForegroundColor Cyan
    Expand-Archive -Path $zipPath -DestinationPath $tmpDir -Force

    Write-Host "4> Installing 'vss-can' to $installDir..." -ForegroundColor Cyan
    Copy-Item -Path (Join-Path $tmpDir "vss-can.exe") -Destination $exeDest -Force
} finally {
    Remove-Item -Recurse -Force $tmpDir -ErrorAction SilentlyContinue
}

Write-Host ""
Write-Host "SUCCESS! Agent Key Detector has been installed." -ForegroundColor Green
Write-Host "The executable is located at: $exeDest" -ForegroundColor Cyan
Write-Host ""

# Add the install directory to the user PATH if it is not already present.
$userPath = [System.Environment]::GetEnvironmentVariable("PATH", "User")
if ($userPath -notlike "*$installDir*") {
    Write-Host "Adding $installDir to your user PATH..." -ForegroundColor Cyan
    [System.Environment]::SetEnvironmentVariable(
        "PATH",
        "$installDir;$userPath",
        "User"
    )
    Write-Host "PATH updated. Please open a new terminal window for the change to take effect." -ForegroundColor Green
} else {
    Write-Host "$installDir is already in your PATH." -ForegroundColor Green
}

Write-Host ""
Write-Host "You can now run the detector by typing:" -ForegroundColor White
Write-Host "  vss-can" -ForegroundColor Cyan
Write-Host "(Restart your terminal first if this is a fresh install.)" -ForegroundColor White
