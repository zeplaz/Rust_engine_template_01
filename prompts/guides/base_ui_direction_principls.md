> **Canonical UX runbook:** [`ui_operational_direction_runbook_v1.md`](ui_operational_direction_runbook_v1.md) and [`experience_layer_orchestrator_v1.md`](experience_layer_orchestrator_v1.md). This file is the original principles draft (filename is historical).

UI Direction Principles

Your game should NOT look like:

spreadsheet spam
RTS button carpets
floating debug windows
MMO hotbars
Paradox-style dense map clutter

The simulation already has:

overlays
networks
operational blobs
logistics fields
ecology fields
infrastructure systems

So the UI should act like:

a strategic operations table

more than a classic RTS HUD.

MOCKUP 1 — “Operational Command Table”

Best for:

modern conflict
infrastructure warfare
logistics-first gameplay

Inspired by:

military GIS
satellite planning systems
command operations centers
Layout
┌──────────────────────────────────────────────────────┐
│ TOP BAR                                              │
│ Time | Alerts | Intel | Economy | Weather | Power    │
├───────────────┬──────────────────────────────────────┤
│ LEFT TOOLBAR  │                                      │
│                MAP / WORLD                           │
│ [Build]       operational overlays                   │
│ [Logistics]   recon coverage                         │
│ [Intel]       power grid                             │
│ [Military]    fires                                  │
│ [Ecology]     rail congestion                        │
│ [Utilities]   artillery threat                       │
│ [Regions]                                            │
│                                                     │
├───────────────┴──────────────────────────────────────┤
│ BOTTOM CONTEXT PANEL                                │
│ Selected region / city / corridor / front           │
│ throughput | shortages | construction | risk        │
└──────────────────────────────────────────────────────┘
Key UX Rules
No giant permanent windows

Use:

slide-up inspectors
collapsible context trays
radial actions
overlays
Map is primary

Everything should reinforce:

reading the world
understanding systems
seeing flows

NOT menu navigation.

MOCKUP 2 — “Infrastructure Planner”

Best for:

city design
transport construction
utility planning

Inspired by:

CAD/GIS
transport planners
utility management systems
Layout
┌────────────────────────────────────────────────────┐
│ TRANSPORT | POWER | WATER | INDUSTRY | DEFENSE    │
├────────────────────────────────────────────────────┤
│                                                    │
│                 WORLD MAP                          │
│                                                    │
│  overlays:                                         │
│  - terrain slope                                   │
│  - logistics cost                                  │
│  - congestion                                      │
│  - flood risk                                      │
│  - power demand                                    │
│                                                    │
├────────────────────────────────────────────────────┤
│ TOOL CONTEXT BAR                                   │
│ road class | est throughput | est maintenance      │
│ terrain penalty | required materials               │
└────────────────────────────────────────────────────┘
Placement UX

Instead of:

click-click roads

Use:

corridor sketching
spline dragging
route painting
Example
player drags:
city A → valley → river crossing → city B

engine previews:
- bridges required
- estimated throughput
- flood vulnerability
- construction cost
- maintenance burden
MOCKUP 3 — “War Operations Overlay”

Best for:

operational warfare
drones
artillery
logistics interdiction

Inspired by:

NATO operational maps
ISR systems
battlefield overlays
Layout
┌───────────────────────────────────────────────────┐
│ Threat | Recon | EW | Logistics | Air | Fires    │
├───────────────────────────────────────────────────┤
│                                                   │
│              OPERATIONAL MAP                      │
│                                                   │
│ heatmaps:                                         │
│ artillery threat                                  │
│ drone visibility                                  │
│ EW coverage                                       │
│ supply throughput                                 │
│ fortification depth                               │
│                                                   │
├───────────────────────────────────────────────────┤
│ FRONT ANALYSIS                                    │
│ Attrition ↑                                       │
│ Rail disruption severe                            │
│ Ammo reserves 38%                                 │
│ EW contested                                      │
└───────────────────────────────────────────────────┘
Important Difference

Front lines are NOT:

hard borders

They are:

pressure gradients
threat envelopes
recon confidence zones
MOCKUP 4 — “Minimal Strategic UI”

Best for:

immersion
cleaner aesthetics
reduced clutter
Layout
┌───────────────────────────────────────┐
│ time | weather | alerts | pause       │
└───────────────────────────────────────┘


         FULLSCREEN WORLD


┌───────────────────────────────────────┐
│ context-sensitive tray                │
│ selected object / overlay info        │
└───────────────────────────────────────┘
Philosophy

Almost everything becomes:

hotkeys
radial menus
contextual inspectors
overlay toggles

Very clean.

Very scalable.

MOCKUP 5 — “Geo-Strategic Systems UI”

Probably closest to your engine direction.

Combines:

logistics
ecology
weather
infrastructure
warfare

into layered world analysis.

Layout
┌──────────────────────────────────────────────────────┐
│ OVERLAYS                                             │
│ [Logistics] [Power] [Hydrology] [Fire] [Intel]       │
├───────────────┬──────────────────────────────────────┤
│ REGION PANEL  │                                      │
│                dynamic world map                     │
│ city stats     animated field overlays               │
│ shortages       flow arrows                          │
│ climate         pressure contours                    │
│ unrest          rail loads                           │
│                smoke/fire/weather                    │
│                                                      │
├───────────────┴──────────────────────────────────────┤
│ EVENT / ANALYSIS FEED                                │
│ Bridge collapse near Voronezh                        │
│ Blackout cascade spreading west                      │
│ Drone losses increasing in EW corridor               │
└──────────────────────────────────────────────────────┘
Recommended Direction

You likely want a hybrid of:

System	UI Style
gameplay HUD	minimal native Bevy UI
overlays	fullscreen map-centric
dev/debug tools	egui
construction	contextual planners
logistics	inspector trays
warfare	operational overlays
Strong Recommendation

Move toward:

overlay-first UX

instead of:

window-first UX

because your simulation is:

spatial
systemic
networked
Recommended Interaction Model
Interaction	Best UX
inspect	click/select
build corridor	drag spline
artillery strike	paint target zone
fortification	line sketch
logistics	route overlay
power	network visualization
ecology	field overlays
Suggested Visual Language
Element	Representation
logistics	flow arrows
power	glowing network
artillery	danger heat
recon	soft visibility fields
EW	static/noise contours
fire	animated spread
congestion	pulsating bottlenecks
Bevy UI Structure Recommendation
Native Bevy UI

Use for:

game HUD
overlays
notifications
contextual inspectors
radial menus
egui

Use ONLY for:

dev panels
debug editors
simulation tuning
visualization tools