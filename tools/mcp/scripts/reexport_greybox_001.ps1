# Re-export kit_greybox modules with greybox PBR materials (after export_glb fix)
$ErrorActionPreference = "Stop"
$Repo = "C:\dev\github\Rust_engine_template_01"
$Py = "C:\Users\oz_\AppData\Local\Programs\Python\Python313\python.exe"
$Jobs = @(
    "wall_concrete_2u_run001.json",
    "roof_flat_2u_run001.json",
    "door_industrial_1u_run001.json",
    "wall_wood_1u_run001.json",
    "wall_steel_1u_run001.json",
    "wall_glass_1u_run001.json",
    "roof_pitched_2u_run001.json",
    "door_warehouse_2u_run001.json",
    "door_shop_1u_run001.json",
    "roof_industrial_shed_2u_run001.json"
)
$Examples = Join-Path $Repo "tools\mcp\schemas\examples"
Push-Location (Join-Path $Repo "tools\mcp\python")
foreach ($j in $Jobs) {
    $path = Join-Path $Examples $j
    if (-not (Test-Path $path)) { Write-Warning "Skip missing $j"; continue }
    Write-Host "Re-export $j" -ForegroundColor Cyan
    & $Py -m rust_engine_mcp.cli run-geometry $path
    if ($LASTEXITCODE -ne 0) { throw "Failed: $j" }
}
Pop-Location
Write-Host "Re-export done. Re-promote if needed: python -m rust_engine_mcp.cli promote <job_id>" -ForegroundColor Green
