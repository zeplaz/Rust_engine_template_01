# Write ~/.cursor/mcp.json + rust_engine_art_mcp.env from repo template
param(
    [string]$Repo = "C:\dev\github\Rust_engine_template_01",
    [string]$Python = "C:\Users\oz_\AppData\Local\Programs\Python\Python313\python.exe",
    [string]$Blender = "C:\Program Files (x86)\Steam\steamapps\common\Blender\blender.exe",
    [switch]$Merge
)

$ErrorActionPreference = "Stop"
$CursorDir = Join-Path $env:USERPROFILE ".cursor"
$McpJsonPath = Join-Path $CursorDir "mcp.json"
$EnvPath = Join-Path $CursorDir "rust_engine_art_mcp.env"
$EnvMdPath = Join-Path $CursorDir "rust_engine_art_mcp.env.md"

if (-not (Test-Path $CursorDir)) {
    New-Item -ItemType Directory -Path $CursorDir | Out-Null
}

$Launcher = Join-Path $Repo "tools\mcp\scripts\run_rust_engine_mcp.cmd"

$server = @{
    command = "cmd"
    args    = @("/c", $Launcher)
    env     = @{
        RUST_ENGINE_REPO   = $Repo
        BLENDER_EXE        = $Blender
        PYTHONIOENCODING   = "utf-8"
        PYTHONUTF8         = "1"
        RUST_ENGINE_PYTHON = $Python
    }
}

if ($Merge -and (Test-Path $McpJsonPath)) {
    $existing = Get-Content $McpJsonPath -Raw | ConvertFrom-Json
    if (-not $existing.mcpServers) {
        $existing | Add-Member -NotePropertyName mcpServers -NotePropertyValue ([pscustomobject]@{})
    }
    $existing.mcpServers | Add-Member -NotePropertyName "rust-engine-art" -NotePropertyValue $server -Force
    $out = @{ mcpServers = $existing.mcpServers }
} else {
    $out = @{ mcpServers = @{ "rust-engine-art" = $server } }
}

[System.IO.File]::WriteAllText($McpJsonPath, ($out | ConvertTo-Json -Depth 6), [System.Text.UTF8Encoding]::new($false))

$envLines = @(
    "# Rust Engine Art MCP machine reference (dotenv-style)"
    "# Active runtime env is in ~/.cursor/mcp.json under rust-engine-art."
    "# Re-sync: .\tools\mcp\install_designer_mcp.ps1"
    ""
    "RUST_ENGINE_REPO=$Repo"
    "BLENDER_EXE=$Blender"
    "RUST_ENGINE_PYTHON=$Python"
    ""
    "# Micro CLI (same code path as MCP):"
    "# cd $Repo\tools\mcp\python"
    "# $Python -m rust_engine_mcp.cli ping"
    ""
)
Set-Content -Path $EnvPath -Value $envLines -Encoding utf8

$mdLines = @(
    "# Rust Engine Art MCP - Cursor user config reference"
    ""
    "- Active wiring: ``~/.cursor/mcp.json``"
    "- Path reference: ``~/.cursor/rust_engine_art_mcp.env``"
    ""
    "Re-sync from repo:"
    ""
    "    .\tools\mcp\install_designer_mcp.ps1"
    "    .\tools\mcp\scripts\verify_mcp_setup.ps1"
    ""
    "Then restart Cursor and confirm rust-engine-art is green in Settings - MCP."
    ""
    "Docs: tools/mcp/README.md"
    ""
)
Set-Content -Path $EnvMdPath -Value $mdLines -Encoding utf8

Write-Host "Wrote $McpJsonPath"
Write-Host "Wrote $EnvPath"
Write-Host "Wrote $EnvMdPath"
