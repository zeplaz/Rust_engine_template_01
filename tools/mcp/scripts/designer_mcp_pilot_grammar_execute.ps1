# MCP-PILOT-GRAMMAR-001 — Phases 3–6 (iso-rig keyframe + G4 witness)
# NOT tile_compile_minimum_bake.py — uses designer_mcp_pilot_grammar_keyframe.py

$ErrorActionPreference = "Stop"
$RepoRoot = (Resolve-Path (Join-Path $PSScriptRoot '..\..\..')).Path
Set-Location $RepoRoot

Write-Host "=== Phase 1–2 prep ==="
powershell -File tools/mcp/scripts/designer_mcp_pilot_grammar_prep.ps1
if ($LASTEXITCODE -ne 0) { exit 1 }

Write-Host "=== Phases 3–6: iso-rig keyframe 24 cells + pack + G4 ==="
python tools/mcp/scripts/designer_mcp_pilot_grammar_keyframe.py --force-bake
exit $LASTEXITCODE
