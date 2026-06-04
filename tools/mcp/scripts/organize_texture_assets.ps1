# Safe texture reorganization — MOVE only, never delete.
# Preview: .\organize_texture_assets.ps1 -WhatIf
param(
    [switch]$WhatIf,
    [string]$Repo = "C:\dev\github\Rust_engine_template_01"
)

$ErrorActionPreference = "Stop"
$Repo = (Resolve-Path $Repo).Path
$LogPath = Join-Path $Repo "assets\archive\lod0_tile_pilots_2026-06\MOVED_LOG.json"
$ArchiveDir = Join-Path $Repo "assets\archive\lod0_tile_pilots_2026-06"
$TerrainDir = Join-Path $Repo "assets\textures\terrain"
$ProdDir = Join-Path $Repo "assets\textures\buildings_iso\production"
$LegacyTiles = Join-Path $Repo "assets\textures\tiles"

$moves = @(
    @{
        src = "assets\textures\tiles\bunker_military_pilot_v1_atlas.png"
        dst = "assets\archive\lod0_tile_pilots_2026-06\bunker_military_pilot_v1_atlas.png"
    },
    @{
        src = "assets\textures\tiles\rowhouse_victorian_pilot_v1_atlas.png"
        dst = "assets\archive\lod0_tile_pilots_2026-06\rowhouse_victorian_pilot_v1_atlas.png"
    },
    @{
        src = "assets\textures\tiles\shopfront_colonial_pilot_v1_atlas.png"
        dst = "assets\archive\lod0_tile_pilots_2026-06\shopfront_colonial_pilot_v1_atlas.png"
    },
    @{
        src = "assets\textures\tiles\warehouse_industrial_west_pilot_v1_atlas.png"
        dst = "assets\archive\lod0_tile_pilots_2026-06\warehouse_industrial_west_pilot_v1_atlas.png"
    },
    @{
        src = "assets\textures\tiles\factory_floor_greybox_001_atlas.png"
        dst = "assets\textures\terrain\factory_floor_greybox_001_atlas.png"
    },
    @{
        src = "assets\textures\tiles\factory_floor_greybox_001_atlas_meta.json"
        dst = "assets\textures\terrain\factory_floor_greybox_001_atlas_meta.json"
    }
)

foreach ($dir in @($ArchiveDir, $TerrainDir, $ProdDir)) {
    if (-not (Test-Path $dir)) {
        if ($WhatIf) {
            Write-Host "[WhatIf] mkdir $dir"
        } else {
            New-Item -ItemType Directory -Path $dir -Force | Out-Null
        }
    }
}

$log = @{
    schema_version = 1
    ran_at         = (Get-Date).ToString("o")
    what_if        = [bool]$WhatIf
    moves          = @()
}

foreach ($m in $moves) {
    $srcPath = Join-Path $Repo $m.src
    $dstPath = Join-Path $Repo $m.dst
    $entry = @{
        src    = $m.src
        dst    = $m.dst
        status = "skipped"
    }
    if (-not (Test-Path -LiteralPath $srcPath)) {
        $entry.status = "missing_src"
        $log.moves += $entry
        Write-Host "skip (missing): $($m.src)"
        continue
    }
    if (Test-Path -LiteralPath $dstPath) {
        $entry.status = "dst_exists"
        $log.moves += $entry
        Write-Host "skip (dst exists): $($m.dst)"
        continue
    }
    if ($WhatIf) {
        $entry.status = "would_move"
        Write-Host "[WhatIf] $srcPath -> $dstPath"
    } else {
        $dstParent = Split-Path $dstPath -Parent
        if (-not (Test-Path $dstParent)) {
            New-Item -ItemType Directory -Path $dstParent -Force | Out-Null
        }
        Move-Item -LiteralPath $srcPath -Destination $dstPath
        $entry.status = "moved"
        Write-Host "moved: $($m.src)"
    }
    $log.moves += $entry
}

if (-not $WhatIf) {
    $log | ConvertTo-Json -Depth 6 | Set-Content -Path $LogPath -Encoding UTF8
    Write-Host "`nLog: $LogPath"
    Write-Host "Active tile index: assets/configs/buildings/_tile_atlas_index.ron (pilots in _archive)"
}

Write-Host "`nDone. See assets/README.md and assets/archive/README.md"
