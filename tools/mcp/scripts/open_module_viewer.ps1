# Launch Module Kit Viewer (browse promoted GLBs + edit metadata)
# ASCII-only: avoids PowerShell parse errors on some Windows locales.
$ErrorActionPreference = "Stop"

$Repo = (Resolve-Path (Join-Path $PSScriptRoot "..\..\..")).Path
$Py = "C:\Users\oz_\AppData\Local\Programs\Python\Python313\python.exe"
if (-not (Test-Path -LiteralPath $Py)) {
    Write-Warning "Python 3.13 not found - falling back to python (may lack jsonschema)."
    $Py = "python"
}

$env:RUST_ENGINE_REPO = $Repo

Push-Location (Join-Path $Repo "tools\mcp\python")
& $Py -m pip install -q -r "..\requirements.txt"
if ($LASTEXITCODE -ne 0) { Pop-Location; exit $LASTEXITCODE }
& $Py -m pip install -q -e .
if ($LASTEXITCODE -ne 0) { Pop-Location; exit $LASTEXITCODE }
Pop-Location

$viewerReq = Join-Path $Repo "tools\mcp\module_viewer\requirements.txt"
& $Py -m pip install -q -r $viewerReq
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }

$RunPy = Join-Path $Repo "tools\mcp\module_viewer\run.py"
& $Py $RunPy
exit $LASTEXITCODE
