# Runs ToneSnap in dev mode from any folder or terminal.
#
# Needs only Node (machine-wide in Program Files) and Rust (~\.cargo). It deliberately avoids
# pnpm: pnpm lives in AppData\Roaming\npm, which terminals hosted by packaged (MSIX) apps may
# not see. Tauri's own CLI comes from node_modules and Vite is started with `npm run`.
# Use -Check to only verify the toolchain.
param([switch]$Check)

foreach ($pair in @(
        @('APPDATA', 'ApplicationData'),
        @('LOCALAPPDATA', 'LocalApplicationData'),
        @('USERPROFILE', 'UserProfile'))) {
    if (-not [Environment]::GetEnvironmentVariable($pair[0])) {
        [Environment]::SetEnvironmentVariable($pair[0], [Environment]::GetFolderPath($pair[1]))
    }
}

$env:Path = @(
    (Join-Path $env:USERPROFILE '.cargo\bin'),
    (Join-Path $env:ProgramFiles 'nodejs'),
    [Environment]::GetEnvironmentVariable('Path', 'User'),
    [Environment]::GetEnvironmentVariable('Path', 'Machine'),
    $env:Path
) -join ';'

$tauri = Join-Path $PSScriptRoot 'node_modules\.bin\tauri.cmd'
$missing = @()
foreach ($tool in 'node', 'npm', 'cargo') {
    if (-not (Get-Command $tool -ErrorAction SilentlyContinue)) { $missing += $tool }
}
if (-not (Test-Path $tauri)) { $missing += 'node_modules (run "pnpm install" in the project folder)' }
if ($missing) {
    Write-Host "Missing: $($missing -join ', ')" -ForegroundColor Red
    Write-Host "USERPROFILE=$env:USERPROFILE"
    Write-Host "PATH=$env:Path"
    exit 1
}

if ($Check) {
    "node  $(node --version)"
    "npm   $(npm --version)"
    "cargo $(cargo --version)"
    "tauri $(& $tauri --version)"
    exit 0
}

Set-Location $PSScriptRoot
& $tauri dev
