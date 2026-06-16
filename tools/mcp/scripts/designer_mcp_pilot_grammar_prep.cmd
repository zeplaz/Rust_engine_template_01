@echo off
REM MCP-PILOT-GRAMMAR-001 prep — same as designer_mcp_pilot_grammar_prep.ps1 (no PS execution policy)
setlocal
cd /d "%~dp0..\..\.."
if not exist "assets\staging\assemblies\industrial_west_7x5_s39_9fa1.json" (
  echo ERROR: Missing assets\staging\assemblies\industrial_west_7x5_s39_9fa1.json
  exit /b 1
)
cd tools\mcp\python
echo === Generate material textures ===
python -m rust_engine_mcp.cli generate-material-textures --profile steel_panel_01
if errorlevel 1 exit /b 1
python -m rust_engine_mcp.cli generate-material-textures --profile roof_metal_01
if errorlevel 1 exit /b 1
python -m rust_engine_mcp.cli generate-material-textures --profile brick_red_01
if errorlevel 1 exit /b 1
python -m rust_engine_mcp.cli generate-material-textures --profile wood_plank_01
if errorlevel 1 exit /b 1
echo === assembly-build-run ===
python -m rust_engine_mcp.cli assembly-build-run "%CD%\..\..\..\assets\staging\assemblies\industrial_west_7x5_s39_9fa1.json"
if errorlevel 1 exit /b 1
cd ..\..\..
findstr /C:"ASSEMBLY_MATERIAL_WARN" debug_runs\art_pipeline\asm_industrial_west_7x5_s39_9fa1.log >nul 2>&1
if not errorlevel 1 (
  echo ERROR: Materials NOT applied in Blender — see debug_runs\art_pipeline\asm_industrial_west_7x5_s39_9fa1.log
  echo Re-run this script after assembly_import fix, or run prep.ps1
  exit /b 1
)
echo Materials OK in build log.
echo === Prep complete ===
echo Blend: assets\staging\assemblies\industrial_west_7x5_s39_9fa1.blend
echo Part 2 keyframe: tools\mcp\scripts\open_keyframe_render.cmd
echo Runbook: src\dev\pilot_grammar_operator_runbook_v1.md
exit /b 0
