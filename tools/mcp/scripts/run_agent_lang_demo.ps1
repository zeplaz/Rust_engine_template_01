# Launch AGENT-LANG workflow demo UI (tkinter)
$ErrorActionPreference = "Stop"
$RepoRoot = (Resolve-Path (Join-Path $PSScriptRoot "..\..\..")).Path
Set-Location (Join-Path $RepoRoot "tools\mcp\python")
python -m rust_engine_mcp.cli agent-lang-demo @args
