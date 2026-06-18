# Designer grammar iterate — CLI loop (token-efficient)
param(
    [ValidateSet("fast", "full")]
    [string]$Mode = "fast",
    [switch]$WriteWitness
)

$ErrorActionPreference = "Stop"
$pyRoot = Join-Path $PSScriptRoot "..\python"
Set-Location $pyRoot

$args = @("designer-grammar-quality-loop")
if ($Mode -eq "full") { $args += "--full" }
if ($WriteWitness) { $args += "--write-witness" }

python -m rust_engine_mcp.cli @args
