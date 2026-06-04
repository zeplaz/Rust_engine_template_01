# Run a geometry job in Blender headless (Windows).
# Usage: .\headless_run.ps1 -Job path\to\job.json
param(
    [Parameter(Mandatory = $true)]
    [string]$Job
)

$ErrorActionPreference = "Stop"
$RepoRoot = if ($env:RUST_ENGINE_REPO) { $env:RUST_ENGINE_REPO } else {
    (Resolve-Path (Join-Path $PSScriptRoot "..\..\..")).Path
}
$BlenderExe = $env:BLENDER_EXE
if (-not $BlenderExe) {
    $Defaults = Join-Path $PSScriptRoot "..\config.defaults.json"
    if (Test-Path $Defaults) {
        $cfg = Get-Content $Defaults -Raw | ConvertFrom-Json
        $BlenderExe = $cfg.blender_exe
    }
}
if (-not $BlenderExe) {
    $BlenderExe = "C:\Program Files (x86)\Steam\steamapps\common\Blender\blender.exe"
}
if (-not (Test-Path $BlenderExe)) {
    throw "Blender not found at $BlenderExe — set BLENDER_EXE."
}
$RunScript = Join-Path $PSScriptRoot "scripts\run_job.py"
if (-not (Test-Path $RunScript)) {
    throw "Missing run_job.py at $RunScript"
}
$LogDir = Join-Path $RepoRoot "debug_runs\art_pipeline"
New-Item -ItemType Directory -Force -Path $LogDir | Out-Null
$JobName = [System.IO.Path]::GetFileNameWithoutExtension($Job)
$LogPath = Join-Path $LogDir "$JobName.log"

Push-Location $RepoRoot
try {
    & $BlenderExe --background --python $RunScript -- --job $Job 2>&1 | Tee-Object -FilePath $LogPath
    exit $LASTEXITCODE
} finally {
    Pop-Location
}
