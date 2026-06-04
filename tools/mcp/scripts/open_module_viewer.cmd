@echo off
REM Module Kit Viewer — no PowerShell execution policy required.
setlocal EnableExtensions

set "REPO=%~dp0..\..\.."
for %%I in ("%REPO%") do set "REPO=%%~fI"

set "PY=C:\Users\oz_\AppData\Local\Programs\Python\Python313\python.exe"
if not exist "%PY%" set "PY=python"

set "RUST_ENGINE_REPO=%REPO%"

pushd "%REPO%\tools\mcp\python"
"%PY%" -m pip install -q -r ..\requirements.txt
if errorlevel 1 goto :fail
"%PY%" -m pip install -q -e .
if errorlevel 1 goto :fail
popd

"%PY%" -m pip install -q -r "%REPO%\tools\mcp\module_viewer\requirements.txt"
if errorlevel 1 goto :fail

"%PY%" "%REPO%\tools\mcp\module_viewer\run.py"
set "EC=%ERRORLEVEL%"
endlocal & exit /b %EC%

:fail
echo Module viewer setup failed. Check Python 3.13 and pip.
popd 2>nul
exit /b 1
