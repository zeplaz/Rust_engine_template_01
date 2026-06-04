# Verify rust-engine-art MCP wiring (Windows)
$ErrorActionPreference = "Stop"

$Repo = if ($env:RUST_ENGINE_REPO) { $env:RUST_ENGINE_REPO } else { "C:\dev\github\Rust_engine_template_01" }
$Py = if ($env:RUST_ENGINE_PYTHON) { $env:RUST_ENGINE_PYTHON } else { "C:\Users\oz_\AppData\Local\Programs\Python\Python313\python.exe" }
$CursorDir = Join-Path $env:USERPROFILE ".cursor"
$McpJson = Join-Path $CursorDir "mcp.json"
$EnvRef = Join-Path $CursorDir "rust_engine_art_mcp.env"

$fail = 0

function Assert-Ok($cond, $msg) {
    if ($cond) { Write-Host "[ok] $msg" -ForegroundColor Green }
    else { Write-Host "[FAIL] $msg" -ForegroundColor Red; $script:fail++ }
}

Write-Host "Rust Engine Art MCP verify" -ForegroundColor Cyan
Write-Host "Repo: $Repo"

Assert-Ok (Test-Path $Py) "Python 3.13: $Py"
Assert-Ok (Test-Path "$Repo\Cargo.toml") "Repo root"
Assert-Ok (Test-Path $McpJson) "~/.cursor/mcp.json"
Assert-Ok (Test-Path $EnvRef) "~/.cursor/rust_engine_art_mcp.env"

if (Test-Path $McpJson) {
    $cfg = Get-Content $McpJson -Raw | ConvertFrom-Json
    Assert-Ok ($null -ne $cfg.mcpServers."rust-engine-art") "mcp.json has rust-engine-art server"
}

Push-Location "$Repo\tools\mcp\python"
try {
    $ping = & $Py -m rust_engine_mcp.cli ping 2>&1
    Assert-Ok ($LASTEXITCODE -eq 0) "CLI ping"
    $blender = & $Py -m rust_engine_mcp.cli locate-blender 2>&1 | ConvertFrom-Json
    Assert-Ok (Test-Path $blender.blender_exe) "Blender exe: $($blender.blender_exe)"
    $pytest = & $Py -m pytest tests/ -q 2>&1
    Assert-Ok ($LASTEXITCODE -eq 0) "pytest tools/mcp/python/tests"
}
finally {
    Pop-Location
}

if ($fail -gt 0) {
    Write-Host "`n$fail check(s) failed. Run: .\tools\mcp\install_designer_mcp.ps1" -ForegroundColor Yellow
    exit 1
}

Write-Host "`nAll checks passed. Restart Cursor if mcp.json changed." -ForegroundColor Green
exit 0
