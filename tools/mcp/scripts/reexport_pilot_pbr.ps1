# Pilot PBR re-export — modules with generated tileable texture profiles
$ErrorActionPreference = "Stop"
$Repo = "C:\dev\github\Rust_engine_template_01"
$Py = "C:\Users\oz_\AppData\Local\Programs\Python\Python313\python.exe"
$Examples = Join-Path $Repo "tools\mcp\schemas\examples"

Push-Location (Join-Path $Repo "tools\mcp\python")

Write-Host "Generate pilot material texture sets..." -ForegroundColor Cyan
& $Py -m pip install Pillow -q
& $Py -m rust_engine_mcp.cli generate-material-textures --all-pilot
if ($LASTEXITCODE -ne 0) { throw "Texture generation failed" }

$Jobs = @(
    "wall_concrete_2u_run001.json",
    "wall_wood_1u_run001.json",
    "door_industrial_1u_run001.json",
    "wall_job.example.json"
)
foreach ($j in $Jobs) {
    $path = Join-Path $Examples $j
    if (-not (Test-Path $path)) { Write-Warning "Skip missing $j"; continue }
    Write-Host "Re-export PBR $j" -ForegroundColor Cyan
    & $Py -m rust_engine_mcp.cli run-geometry $path
    if ($LASTEXITCODE -ne 0) { throw "Failed: $j" }
    $jobMeta = & $Py -c "import json; print(json.load(open(r'$path', encoding='utf-8'))['job_id'])"
    $jobId = $jobMeta.Trim()
    & $Py -m rust_engine_mcp.cli validate-glb (Join-Path $Repo "assets\staging\$jobId\model.glb")
    if ($LASTEXITCODE -ne 0) { throw "Validate failed: $jobId" }
    & $Py -m rust_engine_mcp.cli promote $jobId
    if ($LASTEXITCODE -ne 0) { throw "Promote failed: $jobId" }
}

& $Py -m rust_engine_mcp.cli library-register --rebuild-all
Pop-Location
Write-Host "Pilot PBR done - 4 modules with tileable albedo/normal/roughness" -ForegroundColor Green
