# Writes tools/orchestrator/queues/HANDOFF.md from template (session continuity when Task subagents fail).
# Usage: ./tools/orchestrator/invoke_handoff.ps1 -Goal "Close LOG-A" -Lane LOG

param(
    [Parameter(Mandatory = $true)]
    [string] $Goal,
    [Parameter(Mandatory = $true)]
    [ValidateSet("Stage5", "Construction", "LOG", "VM", "Industrial", "MCP", "Other")]
    [string] $Lane,
    [string] $Owner = "parent Auto",
    [string] $Witness = "",
    [string] $NextAction = "",
    [string] $TaskId = "",
    [string] $Track = "",
    [ValidateSet("", "success", "blocked", "failed", "partial")]
    [string] $OpsStatus = "",
    [switch] $OpsScan,
    [switch] $OpsEvent
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

if ($OpsScan) {
    $scan = Join-Path $PSScriptRoot "scripts\ops_intelligence_scan.ps1"
    if (Test-Path $scan) {
        & $scan
    }
}

if ($OpsEvent -and $TaskId) {
    $eventsDir = Join-Path $root "debug_runs\agent_ops"
    New-Item -ItemType Directory -Force -Path $eventsDir | Out-Null
    $eventsFile = Join-Path $eventsDir "events.jsonl"
    $epoch = [int][double]::Parse((Get-Date -UFormat %s))
    $event = @{
        schema = "agent_run_event_v1"
        run_id = [guid]::NewGuid().ToString()
        agent = $Owner
        lane = $Lane
        program_id = $Track
        track = $Track
        task_id = $TaskId
        status = if ($OpsStatus) { $OpsStatus } else { "partial" }
        witness_paths = @($Witness) | Where-Object { $_ }
        written_at_epoch_secs = $epoch
    }
    ($event | ConvertTo-Json -Compress) | Add-Content -Path $eventsFile -Encoding utf8
    Write-Host "Appended agent_run_event to $eventsFile"
}

Write-Host "Continue in chat: @coder Continue HANDOFF.md (see subagent_continuity_playbook_v1.md)"
Write-Host "OPS: powershell -File tools/orchestrator/scripts/ops_intelligence_scan.ps1"
