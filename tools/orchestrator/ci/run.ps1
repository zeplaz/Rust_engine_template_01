# CI entry: orchestrate after check (fast path).
$ErrorActionPreference = "Stop"
$RepoRoot = Resolve-Path (Join-Path $PSScriptRoot "..\..\..")
Set-Location $RepoRoot
cargo check -p proc_A_dine01
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
& (Join-Path $RepoRoot "tools\orchestrator\scripts\check_live_proof_containment.ps1") -HardFail
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
& (Join-Path $RepoRoot "tools\orchestrator\scripts\check_visual_runbook_no_raster_env.ps1")
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
cargo rustc -p proc_A_dine01 --lib -- -D warnings
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
cargo test -p proc_A_dine01 --lib stage5 -- --nocapture
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
cargo run --manifest-path tools/orchestrator/Cargo.toml -- --skip-clippy --skip-test
exit $LASTEXITCODE
