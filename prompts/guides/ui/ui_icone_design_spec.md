# Tactical Sim Icon Tilemap Direction

Blend:

- Soviet-era systems diagrams
- cold-war tactical glyphs
- CRT terminal overlays
- modern simulation readability
- magenta cyber-industrial framing
- ecology/logistics/fire/weather layering

NOT:

- flat mobile UI
- generic game icons
- emoji readability
- cartoon RTS style

The style should feel like:

> "Industrial command infrastructure visualizing a living simulation."

---

# Core Visual Identity

## Base Shape Language

Use:

- square frames
- clipped corners
- wire overlays
- concentric rings
- offset technical registration marks
- thin magenta vector lines
- tactical stencil cuts
- tiny alignment ticks
- layered semantic density

Avoid:

- rounded mobile icons
- soft gradients
- isolated symbols without framing

---

# Tilemap Structure

Recommend:

```

```

```
tile_atlas/
  tactical/
  logistics/
  ecology/
  construction/
  overlays/
  alerts/
  infrastructure/
  weather/
  authority/
  simulation/
```

---

# Recommended Tile Sizes

```

```

```
16x16  -> tiny overlays
24x24  -> dense tactical UI
32x32  -> primary HUD
48x48  -> feature / world icons
64x64  -> panel hero icons
```

Primary production target:

-   
32x32  

-   
vector authored  

-   
raster exported  


---

# Rendering Style

## Layer Stack

```

```

```
BASE SILHOUETTE
→ semantic fill
→ wire frame
→ signal overlay
→ glow accents
→ warning marks
→ micro-grid
```

---

# Palette Application

## Backgrounds

Use:

- `bg_elevated`  

- `bg_interactive`  


For:

-   
inset icon tiles  

-   
tactical modules  

-   
command widgets  


---

## Wire Layer

Primary:

- `wire_magenta`  


Hover:

- `accent_hot`  


Danger:

- `wire_red`  


---

# Icon Concepts

---

# Logistics

## Freight Hub

```

```

```
╔══╤══╗
║ ■│■ ║
╟──┼──╢
║ ▓▓▓ ║
╚═╧═══╝
```

Concept:

-   
stacked container lanes  

-   
rail routing  

-   
throughput visualization  


Color:

-   
cyan + magenta frame  

-   
orange congestion pulse  


---

## Supply Flow

```

```

```
◉════▶
```

Variants:

-   
low throughput  

-   
overloaded  

-   
broken chain  

-   
rerouted  


Animated:

-   
moving packet lights  


---

## Industrial Processing

```

```

```
┌─◉─┐
│▓▓▓│
└─┬─┘
```

Feels:

-   
refinery  

-   
reactor  

-   
chemical plant  


---

# Ecology

## Forest Density

```

```

```
▲▲▲
▲▓▲
▲▲▲
```

Overlay:

-   
ecological stress rings  

-   
fire spread indicators  


Color:

-   
muted teal + terminal green  


---

## Ecological Collapse

```

```

```
╲▓▓╱
 ╳
╱▓▓╲
```

Use:

-   
pollution  

-   
blight  

-   
toxic spread  

-   
dead biome  


Danger overlay:

-   
wire_red fractures  


---

# Fire / Disaster

## Fire Zone

```

```

```
╱╲
▓▓
╲╱
```

Animated:

-   
scanline shimmer  

-   
heat pulse  

-   
smoke drift  


Colors:

-   
accent_action  

-   
wire_red  

-   
gold edge  


---

## Firestorm Front

```

```

```
████╲
▓▓▓▓▶
████╱
```

Large-scale directional disaster icon.

---

# Weather

## Storm Cell

```

```

```
╔═◌═╗
║╱╲║
╚═╤═╝
```

Animated:

-   
rotating pressure ring  

-   
interference lines  


---

## Atmospheric Distortion

```

```

```
▒▒▒
╱╲╱
▒▒▒
```

Use:

-   
heat  

-   
toxic gas  

-   
electromagnetic disturbance  


---

# Tactical / Military

## Radar Sweep

```

```

```
◜◝
 ◉
◟◞
```

Animation:

-   
rotating sweep arm  


---

## Sensor Jammed

```

```

```
◉
╳╳
```

Colors:

-   
magenta + red flicker  


---

## Artillery Zone

```

```

```
╲│╱
─◉─
╱│╲
```

Overlay:

-   
impact radius rings  


---

# Construction

## Ghost Placement

Use:

-   
dashed magenta outlines  

-   
partial transparency  

-   
registration corners  


```

```

```
┌ ─ ┐
│▒▒▒│
└ ─ ┘
```

Hover:

-   
hot pink stabilization glow  


Invalid placement:

-   
red diagonal interruption  


---

# Authority / ECS / Simulation

This is where your engine can become unique.

---

## View Authority

```

```

```
◉══◉
 ║║
◉══◉
```

Represents:

-   
synchronized view graph  


Broken authority:

-   
disconnected line segments  


---

## Resource Conflict

```

```

```
◉⇄◉
```

Meaning:

-   
multi-writer ECS conflict  


---

## Parallel Simulation

```

```

```
╲╱╲╱
╱╲╱╲
```

Represents:

-   
chunked parallel execution  


---

## Extraction Pipeline

```

```

```
■→□→◉
```

Meaning:

-   
sim  

-   
extraction  

-   
projection  


---

# HUD Chrome Concepts

## Side Rail Frames

Use:

-   
magenta outer rails  

-   
asymmetrical corner cuts  

-   
tiny index numbers  

-   
thin scan marks  


Like:

-   
aerospace consoles  

-   
tactical plotting hardware  

-   
industrial control systems  


---

# Minimap Style

Avoid:

-   
smooth fantasy minimap  


Use:

-   
rasterized tactical density  

-   
edge glow  

-   
topographic interference  

-   
pressure overlays  

-   
logistics pulse lines  


---

# Animation Language

Animations should feel:

-   
infrastructural  

-   
machine-driven  

-   
electrical  

-   
signal-based  


NOT:

-   
cute  

-   
bouncy  

-   
casual  


Use:

-   
scanlines  

-   
pulse propagation  

-   
vector sweeps  

-   
CRT flicker  

-   
interference noise  

-   
thermal bloom  


---

# Recommended Texture Workflow

## Authoring

Use:

-   
SVG  

-   
Figma  

-   
Inkscape  

-   
procedural generation  


Then:

-   
export atlas  

-   
bake SDF variants  


---

# Recommended Atlas Categories

```

```

```
atlas_simulation_core.png
atlas_logistics.png
atlas_ecology.png
atlas_weather.png
atlas_construction.png
atlas_authority_debug.png
atlas_alerts.png
atlas_infrastructure.png
```

---

# Procedural Generation Opportunity

A huge opportunity:

Generate icons from ECS metadata.

Example:

```

```

```
#[derive(Component)]
struct FireSpreadNode;

#[derive(Component)]
struct LogisticsHub;
```

Auto-generate:

-   
debug glyphs  

-   
overlays  

-   
minimap symbols  

-   
authority warnings  


This creates:

-   
visual consistency  

-   
scalable debugging  

-   
simulation readability  


---

# Distinctive Aesthetic Goal

You should aim for:

```

```

```
Soviet systems cartography
+
CRT tactical overlays
+
industrial cybernetics
+
ecological simulation maps
+
modern readability
+
magenta/cyan command infrastructure
```

Result should feel like:

> "A planetary operations console for a living systemic world."

