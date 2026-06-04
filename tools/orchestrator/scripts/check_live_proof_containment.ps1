# DEV-ARTIFACT-CONTAINMENT-001 — warn on *live_proof*.rs outside runtime_witness root.
# Slice B: warning only. Slice D: set $HardFail = $true (or -HardFail switch).
param(
    [switch]$HardFail
)

$ErrorActionPreference = "Stop"
$RepoRoot = Resolve-Path (Join-Path $PSScriptRoot "..\..\..")
$ContainmentRoot = Join-Path $RepoRoot "src\dev\runtime_witness"
$ExceptionsManifest = Join-Path $ContainmentRoot "exceptions_manifest.json"

$allowedShims = @()
if (Test-Path $ExceptionsManifest) {
    $manifest = Get-Content $ExceptionsManifest -Raw | ConvertFrom-Json
    if ($manifest.allowed_shim_paths) {
        $allowedShims = @($manifest.allowed_shim_paths | ForEach-Object { $_.Replace('/', '\') })
    }
}

$violations = @()
Get-ChildItem -Path (Join-Path $RepoRoot "src") -Recurse -Filter "*live_proof*.rs" | ForEach-Object {
    $rel = $_.FullName.Substring($RepoRoot.Path.Length + 1)
    if ($rel.StartsWith("src\dev\runtime_witness\", [StringComparison]::OrdinalIgnoreCase)) {
        return
    }
    $normalized = $rel.Replace('/', '\')
    if ($allowedShims -contains $normalized) {
        return
    }
    $violations += $rel
}

if ($violations.Count -eq 0) {
    Write-Host "live_proof containment: OK (no out-of-root writers outside manifest)"
    exit 0
}

Write-Host "live_proof containment: $($violations.Count) path(s) outside src/dev/runtime_witness/"
foreach ($v in $violations) {
    Write-Host "  WARN: $v"
}
Write-Host "Policy: src/dev/dev_artifact_containment_policy_v1.md"
Write-Host "Shims: src/dev/runtime_witness/exceptions_manifest.json"

if ($HardFail) {
    Write-Error "Containment regression - migrate writers or add timed shim to exceptions_manifest.json"
    exit 1
}
Write-Host "(warning mode - CI hard-fails when -HardFail is set)"
exit 0
