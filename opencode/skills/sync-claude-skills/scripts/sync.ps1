# Sync .claude/skills to .cursor/skills AND .claude/agents to .cursor/agents (project base).
# Overlays source files onto destination; never deletes cursor-only extras.
param(
    [switch]$Force,
    [switch]$WhatIf
)

$ErrorActionPreference = 'Stop'
$repoRoot = Resolve-Path (Join-Path $PSScriptRoot '..\..\..\..')
$sourceRoot = Join-Path $repoRoot '.claude\skills'
$destRoot = Join-Path $repoRoot '.cursor\skills'

if (-not (Test-Path $sourceRoot)) {
    Write-Error "Source not found: $sourceRoot"
}

$report = [ordered]@{
    added   = @()
    updated = @()
    skipped = @()
}

Get-ChildItem $sourceRoot -Directory | ForEach-Object {
    $name = $_.Name
    $srcDir = $_.FullName
    $dstDir = Join-Path $destRoot $name
    $srcSkill = Join-Path $srcDir 'SKILL.md'
    $dstSkill = Join-Path $dstDir 'SKILL.md'

    if (-not (Test-Path $srcSkill)) {
        Write-Warning "Skip $name - no SKILL.md in .claude/skills"
        $report.skipped += $name
        return
    }

    $needsBootstrap = -not (Test-Path $dstDir)
    $needsSkill = $needsBootstrap -or -not (Test-Path $dstSkill) -or ((Get-Item $dstSkill).Length -eq 0)

    if ($needsBootstrap) {
        if ($WhatIf) {
            Write-Output "[WhatIf] Would create $dstDir"
            $report.added += $name
            return
        }
        New-Item -ItemType Directory -Path $dstDir -Force | Out-Null
    }

    $changed = $false
    Get-ChildItem $srcDir -Recurse -File | ForEach-Object {
        $rel = $_.FullName.Substring($srcDir.Length).TrimStart('\', '/')
        $target = Join-Path $dstDir $rel
        $targetParent = Split-Path $target -Parent
        $shouldCopy = $Force -or $needsSkill -or -not (Test-Path $target)

        if (-not $shouldCopy -and (Test-Path $target)) {
            $shouldCopy = $_.LastWriteTimeUtc -gt (Get-Item $target).LastWriteTimeUtc
        }

        if (-not $shouldCopy) { return }

        if ($WhatIf) {
            Write-Output "[WhatIf] Would copy $($_.FullName) -> $target"
            $changed = $true
            return
        }

        if (-not (Test-Path $targetParent)) {
            New-Item -ItemType Directory -Path $targetParent -Force | Out-Null
        }
        Copy-Item $_.FullName $target -Force
        $changed = $true
    }

    if ($changed) {
        if ($needsBootstrap) { $report.added += $name }
        else { $report.updated += $name }
    } else {
        $report.skipped += $name
    }
}

# --- agents: flat .md mirror (.claude/agents -> .cursor/agents) ---
$agentSrc = Join-Path $repoRoot '.claude\agents'
$agentDst = Join-Path $repoRoot '.cursor\agents'
if (Test-Path $agentSrc) {
    if (-not (Test-Path $agentDst) -and -not $WhatIf) { New-Item -ItemType Directory -Path $agentDst -Force | Out-Null }
    Get-ChildItem $agentSrc -File -Filter *.md | ForEach-Object {
        $target = Join-Path $agentDst $_.Name
        $shouldCopy = $Force -or -not (Test-Path $target)
        if (-not $shouldCopy -and (Test-Path $target)) {
            $shouldCopy = $_.LastWriteTimeUtc -gt (Get-Item $target).LastWriteTimeUtc
        }
        if (-not $shouldCopy) { return }
        if ($WhatIf) { Write-Output "[WhatIf] Would copy agent $($_.Name)"; $report.updated += "agent:$($_.BaseName)"; return }
        Copy-Item $_.FullName $target -Force
        $report.updated += "agent:$($_.BaseName)"
    }
}

Write-Output ''
Write-Output 'sync-claude-skills report'
Write-Output "  added:   $($report.added -join ', ')"
Write-Output "  updated: $($report.updated -join ', ')"
Write-Output "  skipped: $($report.skipped -join ', ')"
