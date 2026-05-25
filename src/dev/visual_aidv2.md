# Visual Aid v2 (design + program status)

> **Orchestrator:** [`prompts/guides/visual_aidv2_runbook_v1.md`](../../prompts/guides/visual_aidv2_runbook_v1.md)  
> **Step packs:** [`prompts/matrix/experience/runbook/`](../../prompts/matrix/experience/runbook/README.md)  
> **Live board:** [`visual_aidv2_live_todos.rs`](visual_aidv2_live_todos.rs) (`VISUAL-AID-V2-01`…`06`)  
> **Proof JSON:** `debug_runs/visual_aidv2_live.json` (from `--test visual` when wired)

| Lane | Rows | Not Stage 5 exit |
|:---|:---|:---|
| VA1 HUD | VISUAL-AID-V2-01 | `HudPanelState` |
| VA2 Footprint | VISUAL-AID-V2-02 | GPU `TileDebugInstanceMap` |
| VA3 Readability | VISUAL-AID-V2-03 | `TileReadabilityConfig` |
| VA4–VA6 | 04–06 | `RepresentationResult` / icons |

---

HUD PANEL EXPANDS BUT HAS NO COLLAPSE STATE
Current behavior sounds like:

collapsed → expanded

but missing:

expanded → pinned
expanded → auto-hide
expanded → collapsed

You need a proper panel state machine.

Recommended HUD State Model

Do NOT use a bool.

Bad:

pub expanded: bool

Use:

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HudPanelState {
    Collapsed,
    Peek,
    Expanded,
    Pinned,
}

Then:

#[derive(Component)]
pub struct HudPanel {
    pub state: HudPanelState,
    pub width_collapsed: f32,
    pub width_peek: f32,
    pub width_expanded: f32,
    pub anim_t: f32,
}
Recommended UX Behavior
State	Width	Behavior
Collapsed	36px	icon strip only
Peek	180px	hover temporary
Expanded	320px	click open
Pinned	320px	locked open
Suggested Interaction
hover edge
    -> Peek

mouse leave
    -> Collapsed

click panel
    -> Expanded

click pin icon
    -> Pinned

ESC
    -> Collapsed

click outside
    -> Collapsed
Bevy Egui Example
pub fn hud_panel_system(
    mut contexts: EguiContexts,
    mut query: Query<&mut HudPanel>,
) {
    let ctx = contexts.ctx_mut();

    for mut panel in &mut query {
        let target_width = match panel.state {
            HudPanelState::Collapsed => 36.0,
            HudPanelState::Peek => 180.0,
            HudPanelState::Expanded => 320.0,
            HudPanelState::Pinned => 320.0,
        };

        egui::SidePanel::left("hud_panel")
            .exact_width(target_width)
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    if ui.button("×").clicked() {
                        panel.state = HudPanelState::Collapsed;
                    }

                    if ui.button("📌").clicked() {
                        panel.state = if panel.state == HudPanelState::Pinned {
                            HudPanelState::Expanded
                        } else {
                            HudPanelState::Pinned
                        };
                    }
                });

                ui.separator();

                ui.label("Logistics");
                ui.label("Construction");
                ui.label("Power");
            });
    }
}
2. BUILDING GHOST FOOTPRINT NOT CLEAR

This is a major RTS/city-builder readability issue.

Right now you likely only render:

mesh ghost

without:

tile occupancy
blocked cells
footprint extents
snap orientation
clearance bounds

You need a dedicated placement overlay pipeline.

Recommended Placement Visualization

Render ALL of:

Layer	Purpose
footprint fill	occupied tiles
border outline	exact bounds
blocked cells	red invalid cells
entrance arrows	access direction
terrain conformity	slope warning
clearance ring	required spacing
transport links	nearby road attach
Required Components
#[derive(Component)]
pub struct PlacementGhost {
    pub footprint: IVec2,
    pub rotation: u8,
    pub valid: bool,
}
Tile Overlay Mesh

DO NOT use sprites for footprint highlighting.

Use a batched mesh.

Example:

pub struct PlacementOverlayVertex {
    pub pos: Vec3,
    pub color: [f32; 4],
}

Then generate:

GREEN = valid
RED = blocked
YELLOW = partial support
BLUE = road connection
Example Ghost Grid
+---+---+---+
| G | G | G |
+---+---+---+
| G | G | G |
+---+---+---+

G = occupied valid
R = blocked
Recommended Footprint Rendering
pub fn update_placement_overlay(
    ghost: Res<CurrentPlacementGhost>,
    terrain: Res<TerrainGrid>,
    mut gizmos: Gizmos,
) {
    let base = ghost.tile;

    for z in 0..ghost.size.y {
        for x in 0..ghost.size.x {
            let tile = base + IVec2::new(x, z);

            let valid = terrain.can_build(tile);

            let color = if valid {
                Color::srgba(0.1, 1.0, 0.1, 0.25)
            } else {
                Color::srgba(1.0, 0.1, 0.1, 0.4)
            };

            gizmos.rect(
                Vec3::new(tile.x as f32, 0.05, tile.y as f32),
                Quat::IDENTITY,
                Vec2::ONE,
                color,
            );
        }
    }
}
3. WORLD SCALE / BUILDING SCALE FEELS WRONG DURING ZOOM

This is the biggest issue.

You are hitting the classic mismatch between:

world-space scale
vs
screen-space readability

Most successful builders/RTS games cheat aggressively here.

You should too.

Your Current Problem

Likely:

true perspective camera
+
uniform world scaling
+
single LOD

This causes:

buildings shrink too aggressively
tile readability disappears
silhouettes collapse
ghost placement becomes ambiguous
What Major Games Actually Do
Supreme Commander

Uses:

strategic zoom
icon substitution
nonlinear scale perception

Units become:

mesh → simplified mesh → icon

NOT true perspective continuity.

Cities Skylines

Uses:

exaggerated vertical scale
aggressive LOD swaps
billboard transitions
UI overlays detached from world scale
Factorio

Uses:

orthographic-ish readability
tile-first rendering
strict scale discipline

Everything obeys tile readability.

Anno / Frostpunk

Uses:

dynamic detail emphasis
selective scale exaggeration
fog + contrast manipulation
camera-dependent LOD bias
Recommended Architecture For Your Engine

You already have:

Simulation rect
Camera latch
Render hole
Scissor

Good.

Now add:

A. Simulation Scale Layer
B. Visual Scale Layer
C. LOD Bias Layer

Separately.

A. SIMULATION SCALE

Never changes.

1 tile = 1 logical world unit

Always.

Simulation must remain stable.

B. VISUAL SCALE CURVE

Buildings should NOT visually scale linearly with zoom.

Add:

pub struct ZoomVisualBias {
    pub min_scale: f32,
    pub max_scale: f32,
    pub curve_exp: f32,
}

Then:

let visual_scale =
    zoom.powf(curve_exp)
        .clamp(min_scale, max_scale);

This preserves readability.

C. LOD TIERS

You need discrete rendering tiers.

Recommended World LOD
pub enum WorldLod {
    Macro,
    Strategic,
    Operational,
    Detail,
}
Example Thresholds
Zoom	LOD
far	Macro
medium	Strategic
near	Operational
close	Detail
Rendering Behavior Per LOD
LOD	Buildings	Roads	Trees
Macro	colored blocks	splines	none
Strategic	simplified meshes	width-only	billboards
Operational	normal meshes	full	impostors
Detail	full geometry	decals	full
CRITICAL: TILE READABILITY LOCK

You need a minimum on-screen tile size.

Example:

pub struct TileReadabilityConfig {
    pub min_pixels_per_tile: f32,
}

Then dynamically bias:

camera zoom speed
LOD
mesh simplification
icon overlays

based on:

screen_pixels_per_tile

NOT world zoom alone.

Strong Recommendation

You should move toward:

Hybrid Orthographic Perspective

NOT pure perspective.

Very common RTS trick.

Recommended Camera
OrthographicProjection

with:

scale adjusted by zoom

PLUS:

fake perspective tilt

through camera angle only.

This massively improves:

tile readability
placement precision
UI alignment
overlay stability
strategic zoom
Additional Important Fix

Your buildings likely scale badly because:

world meshes authored at inconsistent dimensions

You need strict footprint normalization.

Every building should declare:

pub struct BuildingFootprint {
    pub tiles: UVec2,
    pub mesh_scale: Vec3,
    pub visual_padding: f32,
}

Then auto-fit meshes to tile dimensions.

Final Recommended Priority
Immediate
1. HUD collapse state machine

Small patch, huge UX improvement.

2. Tile footprint overlay

Absolutely required.

3. Add minimum tile readability clamp

This will instantly improve zoom feel.

Next
4. Introduce LOD tiers

Especially strategic vs operational.

5. Hybrid ortho camera

This will solve many scaling problems simultaneously.

Long-Term
6. Nonlinear visual scale bias

AAA RTS trick.

7. Strategic icon rendering

For far zoom.

Most Important Insight

Do NOT try to maintain physically correct scale perception.

RTS/city-builder readability always wins over realism.

Your simulation already has strong authority separation.

Your renderer now needs:

perceptual scaling

instead of:

literal scaling


New Target Files

Your current architecture still has:

GUI owns viewport semantics
Render owns committed extents
MapView owns presentation
ViewAuthority owns camera identity

This is better than before, but still fragmented.

The biggest remaining issue:

View lifecycle != View rendering != View authority

You need a true unified view spine.

The two highest-value new files are:

1. src/view/view_context.rs

This becomes the canonical immutable per-frame view snapshot.

Right now:

ViewManager
ResolvedViewports
MapViewInstances
RepresentationResult
SimulationMapViewport

all partially overlap.

This file collapses that fragmentation.

2. src/viewport/viewport_resolver.rs

This becomes the ONLY viewport authority pipeline.

Right now:

semantic viewport
authoritative viewport
resolved viewport
minimap shell viewport
preview viewport
rescue floor
frozen rects

are spread across too many systems.

This file centralizes:

request intake
arbitration
semantic solve
commit
freeze/heal policy
diagnostics

into one spine.

FILE 1
src/view/view_context.rs
use bevy::prelude::*;
use bevy::render::camera::Viewport;
use std::collections::HashMap;

use crate::gui::view_authority::ViewId;
use crate::gui::map_view::presentation::OverlayMask;
use crate::render::viewport_pipeline::ResolvedViewport;
use crate::systems::transport::ChunkCellKey;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ViewSurfaceKind {
    Window,
    OffscreenGpu,
    CpuRaster,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ViewLodBand {
    Macro,
    Strategic,
    Operational,
    Detail,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ViewInputPolicy {
    Passive,
    Interactive,
    Exclusive,
}

#[derive(Debug, Clone)]
pub struct ViewProjectionState {
    pub logical_size: Vec2,
    pub world_camera_pos: Vec3,
    pub zoom: f32,
    pub ortho_scale: f32,
    pub visible_world_rect: Rect,
}

#[derive(Debug, Clone)]
pub struct ViewSurfaceState {
    pub kind: ViewSurfaceKind,
    pub viewport: Viewport,
    pub resolved: ResolvedViewport,
    pub texture_id: Option<u64>,
    pub physical_size: UVec2,
}

#[derive(Debug, Clone)]
pub struct ViewOverlayState {
    pub overlays: OverlayMask,
    pub overlay_revision: u64,
}

#[derive(Debug, Clone)]
pub struct ViewVisibilityState {
    pub visible_chunks: Vec<ChunkCellKey>,
    pub culling_revision: u64,
}

#[derive(Debug, Clone)]
pub struct ViewIsolationState {
    pub shares_camera_with: Option<ViewId>,
    pub shares_projection_with: Option<ViewId>,
    pub input_policy: ViewInputPolicy,
    pub focus_locked: bool,
}

#[derive(Debug, Clone)]
pub struct ViewContext {
    pub id: ViewId,
    pub revision: u64,

    pub projection: ViewProjectionState,
    pub surface: ViewSurfaceState,
    pub overlays: ViewOverlayState,
    pub visibility: ViewVisibilityState,
    pub isolation: ViewIsolationState,

    pub lod_band: ViewLodBand,

    pub is_authoritative: bool,
    pub is_simulation_surface: bool,
}

#[derive(Resource, Default)]
pub struct ViewContextRegistry {
    pub contexts: HashMap<ViewId, ViewContext>,
    pub frame_revision: u64,
}

impl ViewContextRegistry {
    #[inline]
    pub fn get(&self, id: ViewId) -> Option<&ViewContext> {
        self.contexts.get(&id)
    }

    #[inline]
    pub fn world_main(&self) -> Option<&ViewContext> {
        self.get(ViewId::WorldMain)
    }

    #[inline]
    pub fn minimap(&self) -> Option<&ViewContext> {
        self.get(ViewId::Minimap)
    }
}
Why This File Matters

This removes the current split between:

ViewManager
ResolvedViewports
MapViewInstances
RepresentationResult
SimulationMapViewport

Everything becomes:

ViewContext

read-only.

That is the missing architecture layer.

New Rule

ALL rendering systems read:

Res<ViewContextRegistry>

instead of:

raw MapCameraDesired
raw ResolvedViewports
raw minimap state
raw preview state
FILE 2
src/viewport/viewport_resolver.rs

This becomes the ONLY viewport authority path.

use bevy::prelude::*;
use std::collections::VecDeque;

use crate::gui::view_authority::ViewId;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ViewportRequestSource {
    Hud,
    Minimap,
    Preview,
    CameraLatch,
    Boot,
    Recovery,
}

#[derive(Debug, Clone)]
pub struct ViewportRequest {
    pub view_id: ViewId,
    pub logical_rect: Rect,
    pub priority: i32,
    pub source: ViewportRequestSource,
}

#[derive(Debug, Clone)]
pub struct SemanticViewport {
    pub rect: Rect,
    pub revision: u64,
}

#[derive(Debug, Clone)]
pub struct CommittedViewport {
    pub rect: Rect,
    pub physical_size: UVec2,
    pub revision: u64,
}

#[derive(Debug, Clone)]
pub struct ViewportDriftMetrics {
    pub semantic_to_committed: Vec2,
    pub committed_to_rendered: Vec2,
}

#[derive(Debug, Clone)]
pub struct ViewportFreezeState {
    pub frozen: bool,
    pub freeze_streak: u32,
    pub hold_reason: Option<&'static str>,
}

#[derive(Debug, Clone)]
pub struct ViewportResolveResult {
    pub semantic: SemanticViewport,
    pub committed: CommittedViewport,
    pub drift: ViewportDriftMetrics,
    pub freeze: ViewportFreezeState,
}

#[derive(Resource, Default)]
pub struct ViewportResolver {
    pub pending: VecDeque<ViewportRequest>,
    pub resolved: HashMap<ViewId, ViewportResolveResult>,
    pub revision: u64,
}

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub enum ViewportResolverSet {
    GatherRequests,
    ResolveSemantic,
    CommitResolved,
    PublishContexts,
    Diagnostics,
}

pub fn resolve_viewports_system(
    mut resolver: ResMut<ViewportResolver>,
    windows: Query<&Window>,
) {
    let Ok(window) = windows.single() else {
        return;
    };

    resolver.revision += 1;

    let mut grouped: HashMap<ViewId, Vec<ViewportRequest>> =
        HashMap::default();

    while let Some(req) = resolver.pending.pop_front() {
        grouped.entry(req.view_id).or_default().push(req);
    }

    for (view_id, requests) in grouped {
        let selected = requests
            .iter()
            .max_by_key(|r| r.priority)
            .unwrap();

        let logical = selected.logical_rect;

        let physical_size = UVec2::new(
            logical.width() as u32,
            logical.height() as u32,
        );

        let semantic = SemanticViewport {
            rect: logical,
            revision: resolver.revision,
        };

        let committed = CommittedViewport {
            rect: logical,
            physical_size,
            revision: resolver.revision,
        };

        let drift = ViewportDriftMetrics {
            semantic_to_committed: Vec2::ZERO,
            committed_to_rendered: Vec2::ZERO,
        };

        let freeze = ViewportFreezeState {
            frozen: false,
            freeze_streak: 0,
            hold_reason: None,
        };

        resolver.resolved.insert(
            view_id,
            ViewportResolveResult {
                semantic,
                committed,
                drift,
                freeze,
            },
        );
    }
}
Why This File Matters

This kills the current fragmented authority problem.

Instead of:

measure_sim_map_fill_viewport
commit_authority_from_semantic
publish_simulation_map_viewport
resolve_preview_viewport_requests
resolve_minimap_panel_viewport

you get:

ViewportRequest
    ↓
ViewportResolver
    ↓
ViewContextRegistry
    ↓
render/extract/UI

single authority spine.

Recommended New Schedule
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub enum ViewPipelineSet {
    GatherInput,
    ResolveViewports,
    BuildContexts,
    ApplyCameras,
    ExtractVisibility,
    RenderPrepare,
}
Recommended Final Architecture
INPUT
    ↓
ViewportRequest
    ↓
ViewportResolver
    ↓
ViewContextRegistry
    ↓
Camera Apply
    ↓
Visibility Extraction
    ↓
Projection Graph
    ↓
Render Surfaces
Critical Improvement Over Current Repo

Your current repo still has:

camera state
viewport state
presentation state
render state

partially entangled.

These two files create:

immutable frame view state

which is what your architecture is missing.

Additional Strong Recommendation

Immediately deprecate direct reads of:

MapCameraDesired
SimulationMapViewport
ResolvedViewports

outside:

resolver
camera application
compatibility shims

Everything else should consume:

ViewContextRegistry

ONLY.

That is the clean long-term architecture your Stage 5 render spine is converging toward.