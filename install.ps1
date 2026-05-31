<#
.SYNOPSIS
    Installs WinFolSize on Windows.

.DESCRIPTION
    Downloads the latest WinFolSize release from GitHub, extracts it to
    %LOCALAPPDATA%\Programs\winfolsize, and adds that directory to the
    current user's PATH.

.PARAMETER Version
    Specific release tag to install (e.g. v0.1.0). Defaults to the latest release.

.PARAMETER InstallDir
    Override the install directory. Defaults to %LOCALAPPDATA%\Programs\winfolsize.

.PARAMETER Repo
    GitHub repository in 'owner/name' form. Defaults to wictorwilen/winfolsize.

.EXAMPLE
    irm https://raw.githubusercontent.com/wictorwilen/winfolsize/main/install.ps1 | iex
#>
[CmdletBinding()]
param(
    [string]$Version,
    [string]$InstallDir = (Join-Path $env:LOCALAPPDATA 'Programs\winfolsize'),
    [string]$Repo = 'wictorwilen/winfolsize'
)

$ErrorActionPreference = 'Stop'
$ProgressPreference = 'SilentlyContinue'

function Write-Info($msg) { Write-Host "==> $msg" -ForegroundColor Cyan }
function Write-Warn($msg) { Write-Host "warn: $msg" -ForegroundColor Yellow }

# Detect architecture
$archMap = @{
    'AMD64' = 'x86_64'
    'ARM64' = 'aarch64'
    'x86'   = 'x86_64'   # 32-bit host: fall back to x64
}
$arch = $archMap[$env:PROCESSOR_ARCHITECTURE]
if (-not $arch) { throw "Unsupported architecture: $env:PROCESSOR_ARCHITECTURE" }

# Resolve version
if (-not $Version) {
    Write-Info "Looking up latest release of $Repo…"
    $rel = Invoke-RestMethod "https://api.github.com/repos/$Repo/releases/latest"
    $Version = $rel.tag_name
}
$ver = $Version -replace '^[vV]', ''

$asset = "winfolsize-$ver-windows-$arch.zip"
$url   = "https://github.com/$Repo/releases/download/$Version/$asset"

Write-Info "Downloading $asset"
$tmp = New-Item -ItemType Directory -Path (Join-Path $env:TEMP "winfolsize-install-$([guid]::NewGuid())")
try {
    $zip = Join-Path $tmp.FullName $asset
    Invoke-WebRequest -Uri $url -OutFile $zip -UseBasicParsing

    Write-Info "Extracting to $InstallDir"
    if (-not (Test-Path $InstallDir)) { New-Item -ItemType Directory -Path $InstallDir | Out-Null }
    Expand-Archive -Path $zip -DestinationPath $InstallDir -Force
}
finally {
    Remove-Item $tmp.FullName -Recurse -Force -ErrorAction SilentlyContinue
}

$exe = Join-Path $InstallDir 'winfolsize.exe'
$cliExe = Join-Path $InstallDir 'winfolsizec.exe'
if (-not (Test-Path $exe)) { throw "winfolsize.exe not found in archive" }
if (-not (Test-Path $cliExe)) { Write-Warn "winfolsizec.exe not found — CLI output in cmd/PowerShell may appear after the prompt." }

# Add to user PATH if missing
$userPath = [Environment]::GetEnvironmentVariable('Path', 'User')
$pathParts = if ($userPath) { $userPath -split ';' } else { @() }
if ($pathParts -notcontains $InstallDir) {
    Write-Info "Adding $InstallDir to user PATH"
    $newPath = (($pathParts + $InstallDir) | Where-Object { $_ } ) -join ';'
    [Environment]::SetEnvironmentVariable('Path', $newPath, 'User')
    $env:Path = "$env:Path;$InstallDir"
    Write-Warn "Open a new terminal for PATH changes to take effect."
}

& (if (Test-Path $cliExe) { $cliExe } else { $exe }) --version
Write-Info "Done. Try: winfolsizec --help  (or just winfolsize for the GUI)"
