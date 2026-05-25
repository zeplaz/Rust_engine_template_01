# Writes tools/orchestrator/queues/HANDOFF.md from template (session continuity when Task subagents fail).
# Usage: ./tools/orchestrator/invoke_handoff.ps1 -Goal "Close LOG-A" -Lane LOG

param(
    [Parameter(Mandatory = $true)]
    [string] $Goal,
    [Parameter(Mandatory = $true)]
    [ValidateSet("Stage5", "Construction", "LOG", "VM", "Industrial", "Other")]
    [string] $Lane,
    [string] $Owner = "parent Auto",
    [string] $Witness = "",
    [string] $NextAction = ""
)

$ErrorActionPreference = "Stop"
$root = Split-Path (Split-Path $PSScriptRoot -Parent) -Parent
$template = Join-Path $PSScriptRoot "queues/HANDOFF.template.md"
$out = Join-Path $PSScriptRoot "queues/HANDOFF.md"
$date = Get-Date -Format "yyyy-MM-dd"

if (-not (Test-Path $template)) {
    Write-Error "Missing template: $template"
}

$content = Get-Content $template -Raw
$content = $content -replace "YYYY-MM-DD", $date
$content = $content -replace "@coder \| @designer \| parent Auto", $Owner
$content = $content -replace "Stage5 \| Construction \| LOG \| VM \| Industrial", $Lane
$content = $content -replace "One sentence: what .done. looks like for this slice\.", $Goal

if ($Witness) {
    $content = $content -replace "debug_runs/… — key fields:", "``$Witness`` — key fields:"
}
if ($NextAction) {
    $content = $content -replace "e.g. .Implement VM-C C1: route map camera input through ViewProjectionAuthority::commit_pose.", $NextAction
}

Set-Content -Path $out -Value $content -Encoding utf8
Write-Host "Wrote $out"
Write-Host "Continue in chat: @coder Continue HANDOFF.md (see subagent_continuity_playbook_v1.md)"
