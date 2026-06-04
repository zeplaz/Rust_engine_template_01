# One-shot setup for designer art MCP (Windows)
# Installs Python package, writes ~/.cursor/mcp.json + rust_engine_art_mcp.env, runs smoke.
param(
    [string]$Repo = "C:\dev\github\Rust_engine_template_01",
    [string]$Py = "C:\Users\oz_\AppData\Local\Programs\Python\Python313\python.exe"
)

$ErrorActionPreference = "Stop"

if (-not (Test-Path $Py)) {
    Write-Warning "Python 3.13 not found at $Py — falling back to 'python' (may be 3.14 without deps)."
    $Py = "python"
}

Write-Host "Installing rust_engine_mcp..." -ForegroundColor Cyan
Push-Location "$Repo\tools\mcp\python"
& $Py -m pip install -r ..\requirements.txt
& $Py -m pip install -e .
Pop-Location

Write-Host "Writing Cursor MCP config (~/.cursor)..." -ForegroundColor Cyan
& "$Repo\tools\mcp\scripts\write_cursor_mcp_config.ps1" -Repo $Repo -Python $Py -Merge

Write-Host "Smoke: ping" -ForegroundColor Cyan
& $Py -m rust_engine_mcp.cli ping

Write-Host "Smoke: locate-blender" -ForegroundColor Cyan
& $Py -m rust_engine_mcp.cli locate-blender

Write-Host "Smoke: validate example spec + job" -ForegroundColor Cyan
& $Py -m rust_engine_mcp.cli validate-spec "$Repo\assets\staging\specs\wall_brick_1u.example.json"
& $Py -m rust_engine_mcp.cli run-geometry "$Repo\tools\mcp\schemas\examples\wall_job.example.json"

Write-Host "Verify full setup..." -ForegroundColor Cyan
$env:RUST_ENGINE_REPO = $Repo
$env:RUST_ENGINE_PYTHON = $Py
& "$Repo\tools\mcp\scripts\verify_mcp_setup.ps1"

Write-Host "`nDone. Restart Cursor to load rust-engine-art MCP server." -ForegroundColor Green
