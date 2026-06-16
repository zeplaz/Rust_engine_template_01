@echo off
REM Post-Blender finish — calls PowerShell with Bypass (or use manual commands in runbook)
setlocal
cd /d "%~dp0..\..\.."
powershell -NoProfile -ExecutionPolicy Bypass -File "%~dp0operator_warehouse_keyframe_finish.ps1" %*
exit /b %ERRORLEVEL%
