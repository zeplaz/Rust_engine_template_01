# Mini geometry smoke: example wall job -> validate (no promote) -> witness stub
$ErrorActionPreference = "Stop"
$RepoRoot = (Resolve-Path (Join-Path $PSScriptRoot "..\..\..")).Path
$Python = if ($env:RUST_ENGINE_PYTHON) { $env:RUST_ENGINE_PYTHON } elseif ($env:MCP_PYTHON) { $env:MCP_PYTHON } else { "C:\Users\oz_\AppData\Local\Programs\Python\Python313\python.exe" }
$Cli = "-m", "rust_engine_mcp.cli"
Push-Location (Join-Path $RepoRoot "tools\mcp\python")

$Job = Join-Path $RepoRoot "tools\mcp\schemas\examples\wall_job.example.json"
Write-Host "run-geometry $Job"
& $Python @Cli run-geometry $Job
if ($LASTEXITCODE -ne 0) { throw "run-geometry failed" }

$Status = & $Python @Cli job-status wall_brick_1u_example | ConvertFrom-Json
if ($Status.status -ne "done") { throw "job-status not done: $($Status | ConvertTo-Json -Compress)" }

$Glb = Join-Path $RepoRoot "assets\staging\wall_brick_1u_example\model.glb"
if (-not (Test-Path $Glb)) { throw "missing staging glb: $Glb" }

Write-Host "validate-glb $Glb"
& $Python @Cli validate-glb $Glb
if ($LASTEXITCODE -ne 0) { throw "validate-glb failed" }

$WitnessDir = Join-Path $RepoRoot "debug_runs\art_pipeline"
New-Item -ItemType Directory -Force -Path $WitnessDir | Out-Null
$Mini = @{
    batch_id = "smoke_geometry"
    status   = "pass"
    job_id   = "wall_brick_1u_example"
    valid    = $true
    glb      = "assets/staging/wall_brick_1u_example/model.glb"
} | ConvertTo-Json -Depth 4
$Mini | Set-Content -Encoding utf8 (Join-Path $WitnessDir "smoke_geometry_live.json")
Write-Host "OK smoke_geometry -> debug_runs/art_pipeline/smoke_geometry_live.json"
Pop-Location
