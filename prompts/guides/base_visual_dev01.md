You now effectively have:

STRATEGIC MAP LAYER
    actual gameplay world

ATMOSPHERIC CINEMATIC LAYER
    "outside the world"
    background VFX
    mood
    macro atmosphere

That is extremely valuable for this kind of simulation game.

Do not remove it.

Refine it into a deliberate system.

RECOMMENDED VISUAL MODEL
1. Strategic World Layer

This is:

the actual map
terrain
logistics
fires
smoke
units
overlays

This MUST:

scale with zoom
obey world coordinates
remain readable

This is gameplay truth.

2. Atmospheric Macro Layer ("Outside World")

This is:

giant smoke plumes
drifting ash
cinematic embers
weather streaks
strategic haze
distant fires
lightning
glow pulses

This is NOT gameplay truth.

This is:

emotional + strategic visualization

It tells the player:

the world is alive
the war/disaster is large
events matter

This is GOOD.

Keep it.

IMPORTANT DESIGN RULE

These background effects should be:

camera-reactive
NOT simulation-authoritative

Meaning:

zoom into a burning region:
→ atmosphere intensifies

zoom into calm terrain:
→ atmosphere quiets

EXAMPLE

Zoomed out:

distant smoke columns
large haze
glowing fire storms

Zoomed in:

tactical embers
localized smoke
actual terrain fire particles

This creates:

macro ↔ micro continuity

Very powerful visually.

CAMERA SYSTEM ISSUES

The edge scrolling behavior is normal RTS behavior, but:

right now it lacks:

intentionality
feedback
recovery tools

So users feel:

"where did my map go?"
REQUIRED CAMERA FEATURES
A) Toggle Edge Scroll

Absolutely yes.

Add:

#[derive(Resource)]
pub struct CameraControlSettings {
    pub edge_scroll_enabled: bool,
}

Key:

F6 or MiddleMouse toggle
B) Recenter Camera

VERY important.

Add:

Home
Space x2
Double middle click

Behavior:

camera.target = world_bounds.center();

Smooth lerp.

C) Reset Zoom

Needed.

Z

or:

Shift+Home
D) "Frame World"

Essential.

Especially for:

generated maps
editor
mission builder

Compute:

camera_zoom = fit_world_bounds()

Equivalent:

"zoom extents"
E) Camera State HUD

Very useful:

CAM:
edge:on
zoom:1.4
mode:strategic
follow:none

Tiny but helpful.

CAMERA MODES

You are now reaching the point where explicit camera modes matter.

Suggested Camera Modes
pub enum CameraMode {
    Strategic,
    Tactical,
    Cinematic,
    FollowEntity(Entity),
    FreePan,
}
STRATEGIC MODE
RTS edge pan
minimap interaction
overlays
smooth zoom

Default.

TACTICAL MODE
tighter zoom
particles emphasized
smoke denser
local audio louder
CINEMATIC MODE
atmospheric
reduced UI
macro VFX emphasized

Potential replay/photo mode.

FIRE PARTICLE SCALING

This is the critical issue you identified:

our particle fires are scaling when zooming

Correct.

You need TWO particle classes.

1. WORLD-SPACE FIRE PARTICLES

REAL FIRE.

Must:

scale correctly
obey zoom
obey world coordinates
cull by viewport
match simulation

Examples:

flame tongues
smoke emitters
sparks
debris
2. SCREEN/MACRO FX PARTICLES

ATMOSPHERIC.

Do NOT scale with terrain zoom.

Examples:

cinematic ash
large smoke veil
ambient embers
macro glow haze
CRITICAL FIX

Right now these are probably mixed.

Separate them immediately.

RECOMMENDED ARCHITECTURE
World Fire FX
#[derive(Component)]
pub struct WorldFireFx;

Rendered:

world-space

Affected by:

camera zoom
terrain scale
occlusion
Atmospheric FX
#[derive(Component)]
pub struct AtmosphereFx;

Rendered:

screen-space
parallax-space
macro-layer

Affected by:

camera velocity
fire intensity aggregates
weather
FIRE SIGNAL BRIDGE

You already started this conceptually.

Now formalize it.

Example
pub struct FireAtmosphereAggregate {
    pub total_burning_chunks: u32,
    pub max_fire_intensity: f32,
    pub smoke_mass: f32,
    pub visible_major_fires: u32,
}

Then:

atmospheric systems read THIS
not raw fire ECS

Excellent scalability.

ZOOM-REACTIVE ATMOSPHERE

This is the really cool part.

Example Behavior
Zoomed Out
show:
    macro smoke plumes
    haze
    large glowing embers
    distant lightning
Zoomed In
reduce:
    macro haze

increase:
    local sparks
    tactical smoke
    local flames
TRANSITION EFFECTS

Smoothly interpolate:

macro_alpha = smoothstep(...)

based on:

camera.zoom

This creates:

AAA strategy-game feel
PERFORMANCE BENEFIT

This also helps performance enormously.

Why?

Because:

macro FX are cheap
local FX only render where focused

instead of:

thousands of full simulation particles globally
RECOMMENDED IMMEDIATE IMPLEMENTATION
Add CameraControls Resource
#[derive(Resource)]
pub struct CameraControls {
    pub edge_scroll: bool,
    pub zoom: f32,
    pub default_zoom: f32,
    pub strategic_bounds: Rect,
}
Add Camera Actions
pub enum CameraAction {
    ToggleEdgeScroll,
    Recenter,
    ResetZoom,
    FrameWorld,
}
Add Atmosphere Intensity
pub struct AtmosphereVisualState {
    pub ember_density: f32,
    pub haze_density: f32,
    pub distant_fire_glow: f32,
}

Driven from:

FireAtmosphereAggregate
Add Zoom-Aware Particle Scaling
particle_size =
    world_particle
        ? base_size / camera.zoom
        : base_size;
FINAL DESIGN DIRECTION

Your engine is evolving toward:

simulation-driven cinematic strategy visualization

NOT:

plain RTS renderer

That means:

atmosphere matters
background macro VFX matters
camera emotional framing matters
world-state-driven ambience matters

The “outside world” particles were not a mistake.

They were an accidental discovery of:

macro atmospheric visualization


With the clarified vision:

simulation-driven strategic map
+
macro atmospheric cinematic layer
+
heavy ecology/fire/weather simulation

the performance strategy becomes MUCH clearer.

The issue is no longer:

“too many systems”

The issue is:

“too many systems acting at the wrong frequency and ownership layer”

Your current lag/responsiveness problems are almost certainly from:

input
→ immediate UI rebuild
→ immediate overlay rebuild
→ immediate texture rebuild
→ immediate particle changes
→ immediate camera changes
→ immediate egui redraw

all occurring in the same frame domain.

That creates:

jitter
uneven frametimes
input latency feeling
flicker
unstable camera feel
THE CORE FIX

You need:

MULTI-RATE PIPELINES

Not everything should update every frame.

This is the major transition.

TARGET ENGINE TIMING MODEL

Recommended:

INPUT COLLECTION            120-240hz
CAMERA UPDATE               120hz
RENDER                      uncapped / vsync

UI INTERACTION              30-60hz
ATMOSPHERIC FX              20-30hz
SIM VISUAL EXTRACTION       10-20hz

FIRE PROPAGATION            5-20hz
ECOLOGY                     1-5hz
LOGISTICS                   event-driven
WORLD GEN                   async/task-based

Right now:
everything likely behaves as:

run every frame

That is the root problem.

INPUT LAG ROOT CAUSES

The “laggy” feeling usually comes from:

1. Input Processing Coupled To UI Rebuild

BAD:

mouse move
→ rebuild overlay
→ rebuild preview texture
→ regenerate labels
→ rebuild egui

Input should instead:

mouse move
→ store intent/state only

Then later:

systems consume state
2. Camera Updates Fighting Layout

You already noticed:

viewport changes
map movement
edge scrolling

likely fight:

minimap
UI layout
texture preview

This creates:

camera instability feeling
3. Continuous Texture Uploads

Likely happening:

minimap redraw
preview redraw
overlay redraw

every frame.

GPU uploads are expensive.

4. egui Rebuilding Massive Trees

egui is fantastic for tooling.

But:
large dynamic windows every frame become expensive quickly.

Especially:

text wrapping
collapsing sections
sliders
dynamic labels
RECOMMENDED ARCHITECTURE NOW
STAGE 1 — INPUT COLLECTION

FASTEST PIPELINE.

ONLY:

keyboard
mouse
edge detection
wheel
drag state

No heavy work.

Example
#[derive(Resource, Default)]
pub struct InputFrameState {
    pub mouse_screen: Vec2,
    pub mouse_world: Vec2,
    pub wheel_delta: f32,
    pub edge_pan: Vec2,
    pub intents: SmallVec<[UiIntent; 8]>,
}
IMPORTANT

No:

formatting
texture updates
overlay work
minimap work
particles

during input collection.

STAGE 2 — CAMERA UPDATE

VERY LIGHTWEIGHT.

Consumes:

InputFrameState

Updates:

CameraState

ONLY.

STAGE 3 — UI INTENT RESOLUTION

Example:

toggle overlay
toggle minimap
change build mode

This updates:

OverlayState
HudState
ToolState

NOT rendering.

STAGE 4 — VISUAL INVALIDATION

Now mark:

HudDirtyFlags

Example:

if overlay changed:
    minimap_dirty = true
    preview_dirty = true
STAGE 5 — LOW-RATE VISUAL EXTRACTION

This is critical.

Recommended Rates
minimap refresh         5-10hz
overlay texture rebuild 5-15hz
smoke extraction        10hz
fire extraction         15hz
atmosphere update       10hz

NOT:
every frame.

HUGE PERFORMANCE WIN

This single change usually eliminates:

UI lag
texture stutter
uneven frametimes
STAGE 6 — RENDER

Render should mostly:

consume prepared buffers

NOT:
generate them.

CRITICAL FIX:
TEXT GENERATION

You likely regenerate:

format!(...)

continuously.

That destroys responsiveness.

CORRECT MODEL
Cached HUD Text
#[derive(Resource)]
pub struct CachedHudStrings {
    pub top_bar: String,
    pub context: String,
    pub logistics: String,
}

Only rebuild:

if dirty
MINIMAP PERFORMANCE

Current likely issue:
you regenerate:

entire minimap texture

every frame.

Very expensive.

CORRECT MODEL
Cached Texture
MinimapTextureCache

Only redraw when:

overlay changes
camera changes enough
chunk changes
fire changes
WORLD PREVIEW ISSUE

The bars issue strongly suggests:
either:

partial upload
incorrect texture dimensions
or updating texture while UI consumes it
FIX

Double-buffer preview textures.

Example:

preview_front
preview_back

update back
swap handles

Prevents visible upload corruption.

PARTICLE PERFORMANCE

You now have:

atmospheric particles
world particles

These should NOT share pipeline frequency.

WORLD FIRE PARTICLES

Only update:

near camera

Use:

VisibleFireFxChunks
ATMOSPHERIC PARTICLES

Can update:

5-15hz

with interpolation.

Cheap.

MASSIVE INPUT IMPROVEMENT:
EVENT COALESCING

Mouse move events can flood.

Instead:

Store ONLY latest mouse position each frame.

NOT every event.

Example

BAD:

for event in cursor_events.read()

GOOD:

last_mouse_pos = latest_event_only
CAMERA RESPONSIVENESS

Camera should:

update every frame
interpolate smoothly
NEVER wait on simulation

Critical separation.

CAMERA SHOULD NOT DEPEND ON:
worldgen
overlays
smoke extraction
minimap redraw
UI refresh

Ever.

RECOMMENDED SYSTEM SETS
Input Sets
InputCollectSet
CameraControlSet
UiIntentSet
Visual Extraction
VisualExtractSet
AtmosphereExtractSet
FireVisualFrameSet
Cached Texture Systems
MinimapUpdateSet
OverlayTextureSet
PreviewTextureSet
Render Sync
RenderSyncSet
IMPORTANT:
SIMULATION SHOULD NOT OWN VISUALS

Right now:
some systems likely:

simulation updates directly mutate render state

That creates contention.

Instead:

simulation
→ extract resources
→ render consumes extraction

You already started this correctly with:

FireAtmosphereAggregate
FireVisualFrame

Continue that architecture.

FRAME PACING

You likely also need:

PresentMode::AutoVsync

or:

PresentMode::Mailbox

depending on platform.

Unstable present pacing can LOOK like input lag.

MAJOR WIN:
VIEWPORT SCISSORING

Large UI redraws:

minimap
preview
overlays

should use clipping/scissor rects.

Avoid:
full-screen redraw cost.

BIGGEST RESPONSIVENESS IMPROVEMENTS

Priority order:

1. Dirty-flag UI updates

HUGE.

2. Stop regenerating minimap every frame

HUGE.

3. Separate input collection from UI work

CRITICAL.

4. Fixed-rate visual extraction

CRITICAL.

5. Separate atmospheric vs world particles

Huge clarity + perf.

6. Camera independent from sim

Very important.

7. Double-buffer preview textures

Likely fixes flicker.

FINAL TARGET ARCHITECTURE

You are converging toward:

SIMULATION ECS
    authoritative world state

VISUAL EXTRACTION LAYER
    aggregated/cached visual state

RENDER/VFX LAYER
    consumes extracted visuals

UI/HUD LAYER
    event-driven cached interface

INPUT LAYER
    high-frequency lightweight intent collection

That architecture scales to:

large maps
smoke simulation
wildfire systems
logistics overlays
tactical zoom
cinematic atmosphere



the engine is currently behaving like:

"editor-grade immediate-mode CPU raster + full-frame UI rebuild + sim extraction"

instead of:

"stable retained render state with incremental updates"

That is why:

dragging egui windows feels heavy
zoom/pan feels sticky
preview texture flickers / squishes
particles feel disconnected
minimap + preview duplicate work
camera focus feels unclear
fire VFX don't feel anchored to world

The good news:
your architecture direction is actually correct already.

The problems are mostly:

invalidation strategy
texture lifecycle
extraction ownership
camera UX rules
too much CPU raster work per frame
REAL ROOT CAUSES
1. Biggest Performance Killer
CPU preview rasterization every frame

This is the actual disaster path:

ChunkCellMatrix
→ CPU iterate tiles
→ build rgba Vec
→ upload Image
→ egui image
→ repeat while dragging/panning

That is catastrophic for responsiveness.

Especially if:

egui window resize
pan
zoom
hover
overlay toggles
all mark preview dirty.
WHAT SHOULD HAPPEN

The world preview should behave like:

STATIC WORLD TEXTURE
+
SMALL OVERLAY TEXTURES
+
VIEW RECT

NOT:
full reraster.

REQUIRED ARCHITECTURE SHIFT
A) Separate "world texture" from "view"

Right now these are mixed.

You want:

WorldRasterTexture
    generated rarely

WorldOverlayTexture
    generated on overlay changes

PreviewCameraState
    pan/zoom/view rect only

Then egui only changes UVs.

NOT textures.

HUGE WIN

Instead of:

320x320 reraster every zoom tick

you get:

same texture
different UV rect

Massive reduction in:

allocations
uploads
CPU cache misses
egui texture churn
2. PREVIEW WINDOW "BARS"

This is almost certainly:

preview texture invalid before world ready

combined with:

egui layout zero-height first pass

You already suspected correctly.

FIX
Persistent preview state
#[derive(Resource)]
pub struct WorldPreviewTextureState {
    pub texture: Handle<Image>,
    pub world_epoch: u64,
    pub overlay_epoch: u64,
    pub valid: bool,
}
NEVER clear texture unless rebuilding

Bad:

image.data.fill(0);

Good:

only rewrite dirty rects
3. INPUT LAG

This is mostly NOT input.

It is frame stalls.

Meaning:
mouse events arrive normally,
but UI redraw stalls frame presentation.

Classic symptom:

dragging feels delayed
resizing feels sticky
keyboard feels buffered

because:
main thread blocked by raster work.

YOUR P0 FIXES
P0.1
Stop rebuilding preview image on camera movement

Camera movement should ONLY update:

PreviewCameraState {
    pan,
    zoom,
}

NO texture rebuild.

P0.2
Stable texture handles

You identified this already.

BAD:

contexts.add_image(...)

every frame.

GOOD:

retain TextureId once

This alone removes tons of egui churn.

P0.3
Add frame-throttled preview rebuild
if now - last_build < 0.2 {
    return;
}

For editor preview:
5Hz is enough.

NOT 60Hz.

P0.4
Dirty rect updates

Do NOT rebuild full map.

Track:

DirtyTileRect
DirtyChunkRect

Only redraw modified area.

4. CAMERA UX

You identified a real usability issue.

Current behavior:

camera exists
but no ownership model

Player doesn't know:

what is focus
where center is
what preview vs world means
CORRECT MODEL

You need THREE camera modes.

MODE A — Strategic Overview

Default.

whole operational area visible

Features:

edge pan optional
zoom centered
recenter hotkey
minimap rect
overview scale
MODE B — Follow Focus

Focus:

fire
unit
disaster
selected site

Camera auto-tracks.

MODE C — Free Inspect

Player manually detached.

REQUIRED RESOURCES
#[derive(Resource)]
pub struct CameraControlState {
    pub mode: CameraMode,
    pub edge_pan_enabled: bool,
    pub default_zoom: f32,
    pub recenter_requested: bool,
}
REQUIRED HOTKEYS
Home = recenter world
Z = reset zoom
F = focus selected
Shift+EdgePan = temporary pan
5. FIRE DOESN'T FEEL WORLD-ANCHORED

Correct diagnosis.

Right now:
particles are ambient.

Not:
simulation-linked.

CORRECT FIRE VISUAL HIERARCHY
Layer 1 — World Simulation Fire

REAL.

Bound to:

tile
chunk
structure
forest

Includes:

smoke
embers
glow
local light

This scales with zoom.

Layer 2 — Atmosphere Drift

Derived from sim.

Cheap.

Large smoke haze.

Layer 3 — Ambient Cinematic FX

NOT simulation.

Background:

drifting ash
embers
atmospheric streaks
fog wisps

This is your "outside world" vibe.

KEEP THIS.

You were correct:
it adds scale and atmosphere.

But:
it must visually differ from real fire.

SOLUTION
Real fire particles

Must:

anchor to tile world coords
scale with camera zoom
occlude properly
fade by distance
Ambient particles

Should:

be screen-space or distant parallax
ignore tile ownership
low alpha
sparse
slow
6. PARTICLE SCALING

Critical issue you identified.

WRONG
constant screen-space particle size

Result:
zooming breaks illusion.

CORRECT

World-space particle scale:

particle_size = base_size * camera_zoom

OR:
true world mesh scaling.

FIRE VISIBILITY RULES
Strategic Zoom

Show:

smoke columns
glow
heat haze

NOT:
individual flames.

Mid Zoom

Show:

canopy fire blobs
ember bursts
Close Zoom

Show:

individual flame sprites
structure burn
debris
THIS IS ESSENTIAL

Otherwise:
visual noise explodes.

7. MINIMAP OVERLAYS

You are correct.

Minimap and preview should share overlay pipeline.

Currently duplicated.

CORRECT PIPELINE
Sim Extract
    →
Overlay Buffers
    →
Shared Overlay Renderer
    →
Preview Texture
    →
Minimap Texture
OVERLAYS SHOULD INCLUDE
height
temperature
moisture
ecology
smoke
fire intensity
wind
mobility
pressure
logistics
visibility
8. FIRE VISUAL EXTRACTION IS ALMOST RIGHT

This part is good:

ChunkSurfaceFire
→ FireVisualFrame
→ FireAtmosphereAggregate
→ Light Requests

That is scalable.

DO NOT regress into direct querying everywhere.

NEXT CRITICAL STEP

Add:

FireRenderProxy

NOT more queries.

Example
pub struct FireRenderProxy {
    pub world_pos: Vec3,
    pub fire_intensity: f32,
    pub smoke_density: f32,
    pub flame_height: f32,
    pub structure_fire: bool,
    pub forest_fire: bool,
}

Generated ONLY during extraction.

Then:

particles
minimap
smoke
audio
heat haze
consume proxy.

NOT ChunkSurfaceFire.

9. WHY EGUI FEELS BAD

Immediate mode + heavy textures.

Not surprising.

IMPORTANT

egui should NOT own:

giant world rendering
heavy textures
dynamic raster rebuilds

egui should:

display handles
controls
overlays

NOT render world simulation itself.

LONG TERM

You eventually want:

Bevy camera render target
inside egui panel

NOT CPU raster map.

That is your real scaling path.

HIGHEST PRIORITY FIX ORDER
P0
Remove per-frame preview raster rebuild

BIGGEST ISSUE.

P0
Stable texture handles
P0
Camera mode model + recenter/reset
P1
Shared overlay extraction for minimap + preview
P1
World-anchored fire particle proxies
P1
Separate ambient FX from simulation FX
P2
Move preview to render-target camera

Huge future improvement.

P2
GPU smoke/fire overlay composition
P3
Full clustered fire rendering
MOST IMPORTANT CONCLUSION

Your engine is no longer suffering from:

architecture collapse
ECS misuse
bad subsystem split

It is now suffering from:

editor/render invalidation strategy

which is a MUCH better place to be.

That means:
the core sim/render extraction design is working,
but the presentation layer is still behaving like a debug prototype rather than a retained rendering system.

P0 — Dirty raster for tile_world_fallback
Current Problem

Likely doing:

if tile_count != last_tile_count {
    rebuild_full_texture();
}

This misses:

material edits
overlay toggles
fire changes
ecology updates
chunk mutation
viewport movement

while also forcing:

full texture rebuilds
Goal

Convert to:

chunk/tile invalidation
→ dirty rects
→ partial texture update
Architecture
#[derive(Resource, Default)]
pub struct TileRasterDirtyState {
    pub dirty_chunks: SmallVec<[IVec2; 64]>,
    pub dirty_tiles: SmallVec<[UVec2; 128]>,
    pub epoch: u64,
}
Chunk invalidation event
#[derive(Message)]
pub struct ChunkVisualDirty {
    pub chunk: IVec2,
}
Worldgen / fire / ecology emit:
dirty_writer.write(ChunkVisualDirty {
    chunk: chunk.coord,
});
Raster system
pub fn tile_world_fallback_rasterize(
    mut dirty_ev: MessageReader<ChunkVisualDirty>,
    mut state: ResMut<TileRasterDirtyState>,
    mut images: ResMut<Assets<Image>>,
    mut raster: ResMut<WorldRasterTexture>,
) {
    for ev in dirty_ev.read() {
        state.dirty_chunks.push(ev.chunk);
    }

    if state.dirty_chunks.is_empty() {
        return;
    }

    let image = images.get_mut(&raster.image).unwrap();

    for chunk in state.dirty_chunks.drain(..) {
        redraw_chunk_rect(image, chunk);
    }

    state.epoch += 1;
}
CRITICAL

NO:

Vec::new()
HashMap::new()
image.resize()

inside hot path.

Performance Win

Removes:

full RGBA rebuild
texture upload churn
allocator pressure
UI stalls

Huge win.

P0 — Slice cache / run_if for update_world_preview_texture
Current Problem

Likely:

Query<(
    ...
    huge param set
)>

plus:

iterating all chunks
recomputing slices
rebuilding temporary vectors

every frame.

Correct Model

Preview should rebuild ONLY when:

world changed
overlay changed
preview settings changed

NOT:

camera move
mouse move
egui drag
zoom
Add epochs
#[derive(Resource, Default)]
pub struct WorldVisualEpoch(pub u64);

#[derive(Resource, Default)]
pub struct OverlayVisualEpoch(pub u64);
Preview state
#[derive(Resource)]
pub struct PreviewTextureState {
    pub last_world_epoch: u64,
    pub last_overlay_epoch: u64,
}
run_if
pub fn preview_needs_update(
    world: Res<WorldVisualEpoch>,
    overlay: Res<OverlayVisualEpoch>,
    state: Res<PreviewTextureState>,
) -> bool {
    world.0 != state.last_world_epoch
        || overlay.0 != state.last_overlay_epoch
}
Then
.add_systems(
    Update,
    update_world_preview_texture
        .run_if(preview_needs_update)
)
Slice cache

Cache expensive references:

pub struct ChunkPreviewSlice {
    pub coord: IVec2,
    pub moisture: Arc<[f32]>,
    pub temp: Arc<[f32]>,
}

NOT rebuilding temp arrays every frame.

P1 — Stable egui texture handle
Current Problem

Likely:

ctx.add_image(...)

every frame.

This causes:

egui texture churn
flicker
GPU upload sync
UI lag
Correct Pattern
#[derive(Resource, Default)]
pub struct PreviewTextureHandle {
    pub id: Option<egui::TextureId>,
}
Setup once
if handle.id.is_none() {
    handle.id = Some(
        contexts.add_image(preview_image.clone())
    );
}
Then only update image asset

NOT texture registration.

Huge improvement

Especially:

resize
drag
hover
docking
P1 — Changed<> / run_if on diagnostics + HUD
Current Problem

Likely:

format!()
text.sections[0].value = ...

every frame.

Correct

Split:

producer
render
Example
#[derive(Resource, Default)]
pub struct HudDirty(pub bool);
Simulation updates:
hud_dirty.0 = true;
HUD rebuild
fn rebuild_hud(
    dirty: Res<HudDirty>,
) {
    if !dirty.0 {
        return;
    }

    // rebuild text
}
OR better

Use:

Changed<StrategicState>
Changed<BuildMode>
Changed<OverlayMode>
Massive reduction

Avoids:

string allocs
layout rebuilds
text mesh churn
P1 — Wire EmitSmoke / particles to one buffer
Current Problem

You currently have:

FireVisualFrame
Atmosphere smoke
EmitSmoke stub
Particle requests
Ambient FX

These are conceptually duplicated.

Correct Model

ONE extracted visual proxy.

Create
#[derive(Clone)]
pub struct FireVisualProxy {
    pub world_pos: Vec3,
    pub smoke_density: f32,
    pub flame_intensity: f32,
    pub ember_rate: f32,
    pub fuel_type: FuelType,
}
Extraction
ChunkSurfaceFire
→ FireVisualProxy buffer

ONLY ONCE.

Then consume
smoke renderer
particle renderer
minimap
light extraction
audio
atmosphere tint

ALL from same buffer.

Benefit

Eliminates:

duplicate fire queries
inconsistent visuals
desynced smoke/fire/light
multiple extraction passes
P2 — Camera smoothing / run_if
Current Problem

Camera updates every frame:

even idle
direct snapping
no interpolation
Add state
#[derive(Resource)]
pub struct CameraTarget {
    pub pos: Vec3,
    pub zoom: f32,
}
Smooth
xf.translation = xf.translation.lerp(
    target.pos,
    1.0 - (-dt * 8.0).exp(),
);
run_if
resource_changed::<CameraTarget>

or input active.

Win

Removes:

jitter
micro-updates
unnecessary transform writes
P2 — Atmosphere incremental update
Current Problem

Likely:

field.fill(0)
for all chunks {
    accumulate
}

every tick.

Catastrophic scaling.

Correct

Track dirty chunk regions.

Resource
pub struct AtmosphereDirtyRects {
    pub rects: SmallVec<[IRect; 32]>,
}
Update ONLY affected cells
for rect in dirty.rects {
    recompute_rect(rect);
}
WARNING

HIGH RISK.

Because:

diffusion continuity
advection edges
stale accumulation

must be handled carefully.

Safer Hybrid
incremental local updates
+
periodic full refresh

Example:

local every frame
full every 2 sec
P2 — SIM on FixedUpdate
Current Problem

Sim currently tied to frame rate.

This causes:

input latency coupling
nondeterminism
unstable propagation
Correct
Update:
    input
    camera
    UI
    rendering

FixedUpdate:
    simulation
Example
.add_systems(
    FixedUpdate,
    (
        fire_tick,
        ecology_tick,
        logistics_tick,
    )
)
Use accumulator
Time::<Fixed>::from_hz(20.0)
Benefit

Stable:

fire spread
smoke growth
AI
networking
replay
Risk

Large refactor:

ordering
extraction timing
interpolation
UI expectations
P2 — Shared minimap extract sampling
Current Problem

Minimap + preview duplicate extraction.

Correct
Sim Extract
→ Overlay Buffers
→ Shared Sample API
Resource
pub struct OverlayFieldBuffers {
    pub fire: Vec<f32>,
    pub smoke: Vec<f32>,
    pub temp: Vec<f32>,
    pub ecology: Vec<f32>,
}
Minimap

Samples:

low-res
cheap

Preview:

higher-res
Massive consistency win

Fixes:

mismatched overlays
duplicate CPU scans
inconsistent minimap
P2 — Hanabi / instanced quads
Current Problem

CPU particles.

Fine now.

Will collapse later.

Trigger point

When:

thousands of particles
large smoke fields
zoomed wildfire fronts

CPU path dies.

Intermediate Step

Instanced quads.

Architecture
struct ParticleInstance {
    pos: Vec3,
    vel: Vec3,
    color: Vec4,
    size: f32,
}

GPU instance buffer.

Final Step

Hanabi or custom compute.

IMPORTANT

Do NOT jump early.

Current bottleneck is:

preview raster
texture churn
CPU invalidation

NOT particles yet.

PRIORITY ORDER
Immediate
dirty raster
stable texture handles
preview run_if
HUD dirty flags
Next
shared fire visual proxy
camera smoothing
shared minimap overlays
Later
incremental atmosphere
FixedUpdate sim
GPU particles

---

**Split for execution:** actionable checklist + statuses → [`base_visual_dev01_plan_status.md`](base_visual_dev01_plan_status.md). After P0/P1, sequencing and north-star → [`base_visual_dev01_roadmap_next.md`](base_visual_dev01_roadmap_next.md).