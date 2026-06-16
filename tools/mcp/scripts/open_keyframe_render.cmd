@echo off
REM Launch Blender with legacy keyframe_render.py addon (warehouse 24 stills).
cd /d "%~dp0..\..\.."
set REPO=%CD%
echo Repo: %REPO%
echo.
echo In Blender after launch:
echo   1. File -^> Open -^> assets\staging\assemblies\industrial_west_7x5_s39_9fa1.blend
echo   2. File -^> Append -^> utils\Tile_iso_rig_v1.blend -^> TILE_ISO_RIG
echo   3. Properties -^> Output -^> Keyframes (legacy export)
echo   4. Expand TILE_ISO_RIG -^> pick IsoCamera -^> Refresh (or Add current frame)
echo   See utils/KEYFRAME_RENDER_README.md
echo.
pushd tools\mcp\python
python -c "from rust_engine_mcp.paths import blender_exe, repo_root; import subprocess; s=repo_root()/'utils'/'keyframe_render.py'; subprocess.Popen([str(blender_exe()), '--python', str(s)], cwd=str(repo_root()))"
popd
echo Launched Blender with keyframe_render.py
