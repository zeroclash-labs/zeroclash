#!/usr/bin/env pwsh
# Build zeroclash release and produce a Windows NSIS installer.
# Usage: .\scripts\package.ps1
# Requires: NSIS (choco install nsis)
#
# Inspired by Zed's script/bundle-windows.ps1 — uses /D definitions
# passed to makensis instead of sed placeholder substitution.

$ErrorActionPreference = 'Stop'
$ProjectRoot = Split-Path -Parent (Split-Path -Parent $PSCommandPath)
$DistDir = Join-Path $ProjectRoot 'dist'
$AppDir = Join-Path $ProjectRoot 'apps' 'zeroclash'

# ── Version (cargo metadata approach, inspired by Zed's script/get-crate-version) ──
if ($env:APP_VERSION) {
    $Version = $env:APP_VERSION
} else {
    $gitTag = & git describe --tags --abbrev=0 2>$null
    if ($gitTag -and ($gitTag -match '^v(\d+\.\d+\.\d+(?:-[a-zA-Z0-9.]+)?)$')) {
        $Version = $Matches[1]
    } else {
        $metadata = & cargo metadata --no-deps --format-version=1 | ConvertFrom-Json
        $pkg = $metadata.packages | Where-Object { $_.name -eq 'zeroclash' } | Select-Object -First 1
        $Version = $pkg.version
        Write-Warning "No git version tag found. Using Cargo.toml version: $Version (local/dev build)"
    }
}

# ── Target triple ──
$Target = & rustc -vV | Select-String 'host:' | ForEach-Object { $_.ToString().Split(':', 2)[1].Trim() }
Write-Host "[package] zeroclash v$Version for $Target"

# ── Step 1: Build ──
Write-Host '[package] Building zeroclash --release...'
& cargo build --release
if ($LASTEXITCODE -ne 0) { throw 'Build failed' }

# ── Step 2: Verify required files ──
$binDir = Join-Path $ProjectRoot 'target' $Target 'release'
$required = @('zeroclash.exe', 'zeroclash-cli.exe', 'mihomo.exe')
foreach ($file in $required) {
    $path = Join-Path $binDir $file
    if (-not (Test-Path $path)) {
        Write-Warning "Missing: $path — installer may be incomplete"
    }
}

# ── Step 3: Check for NSIS ──
$makensis = Get-Command 'makensis' -ErrorAction SilentlyContinue
if (-not $makensis) {
    Write-Error @'
makensis (NSIS) not found in PATH.

Install it via:  choco install nsis
Or download from: https://nsis.sourceforge.io/Download
'@
    exit 1
}

# ── Step 4: Generate NSIS installer (use /D definitions, inspired by Zed's Inno Setup /d flags) ──
$nsiSource = Join-Path $AppDir 'installer.nsi'

# NSIS license page expects LICENSE.txt next to the .nsi
$licenseSrc = Join-Path $ProjectRoot 'LICENSE.md'
$licenseDst = Join-Path $ProjectRoot 'LICENSE.txt'
Copy-Item $licenseSrc $licenseDst -Force

Write-Host '[package] Running NSIS...'
$nsisArgs = @(
    "/DPRODUCT_VERSION=$Version",
    "/DTARGET_TRIPLE=$Target",
    $nsiSource
)
& $makensis.FullName $nsisArgs
if ($LASTEXITCODE -ne 0) { throw 'NSIS installer build failed' }

# ── Step 5: Collect output ──
if (-not (Test-Path $DistDir)) { New-Item -ItemType Directory -Path $DistDir | Out-Null }

$installerName = "ZeroClash-${Version}-setup.exe"
$installerPath = Join-Path $ProjectRoot $installerName
if (Test-Path $installerPath) {
    Move-Item -Path $installerPath -Destination (Join-Path $DistDir $installerName) -Force
}

# ── Cleanup ──
Remove-Item $licenseDst -Force -ErrorAction SilentlyContinue

Write-Host "[package] Done. Installer: $DistDir\$installerName"
