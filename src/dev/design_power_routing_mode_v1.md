# Power routing mode — Curved vs 90° `v1`

| Field | Value |
|:---|:---|
| **ID** | **DES-POWER-ROUTING-MODE-001** |
| **Program** | PLAN-POWER-GRID-CONSTRUCTION-UX-001 · Track A |
| **Date** | 2026-06-18 |
| **Owner** | `@designer` |
| **Charter** | [`design_power_line_construction_ux_v1.md`](design_power_line_construction_ux_v1.md) §2.2 |
| **Handoff** | COD-POWER-ORTHOGONAL-ROUTER-001 · COD-POWER-SPLINE-ROUTER-001 |
| **Verdict** | **PASS** |

```text
DES-POWER-ROUTING-MODE-001 Q✓
Preview and commit share one RoutingMode — no curved preview / orthogonal commit
```

---

## 1. Modes

| Mode | UI label | Icon chip | Router | Best for |
|:---|:---|:---:|:---|:---|
| **Curved** | `Curved` | ~ arc | Catmull-Rom spline (reuse road spline spine) | Long HV transmission, terrain |
| **Orthogonal90** | `90°` | ⊞ | Manhattan axis-aligned only | Yards, substations, city blocks |

**Enum (coder):** `RoutingMode::Curved` | `RoutingMode::Orthogonal90`

---

## 2. Toggle & keybinds

| Action | Binding |
|:---|:---|
| Tool sheet chips | Click `Curved` or `90°` |
| Cycle | **`O`** cycles Curved → 90° → Curved |
| Direct | **`[`** Curved · **`]`** 90° |

**Default on tool open:**

| Context | Default |
|:---|:---|
| First use session | **Curved** |
| Remember last | per `PowerLineToolPrefs.routing_mode` |
| Near substation yard (≤8 tiles) | suggest **90°** hint caption once |

---

## 3. Curved rules

| Rule | Spec |
|:---|:---|
| Min points | 2 |
| Min commit | 2 snapped endpoints + ≥1 interior point for HV spans >32 tiles |
| Segment sample | Spline subdivide like `roads/spline.rs` — preview matches commit polyline |
| Diagonals | Allowed along spline — not axis-locked |
| Self-intersect | Warn `◐ risky crossing` — commit allowed if anchors valid |
| Grid snap | **Off** (toggle disabled in sheet) |

**Visual:** preview stroke dashed @ 60% α · class color from voltage picker.

---

## 4. Orthogonal (90°) rules

| Rule | Spec |
|:---|:---|
| Segments | **Axis-aligned only** — no diagonal tiles |
| Corners | Auto-insert corner node on direction change |
| Angle | 90° only — **no** 45° chamfers in v1 |
| Grid snap | **On** by default — snap to tile center / grid line |
| Min segment | 1 tile length |
| U-turn | Collapse to 2-tile minimum unless blocked |

### 4.1 Corner algorithm (design intent)

```text
Point A ──horizontal──► corner ──vertical──► Point B
```

- Direction change at **tile edge** or **grid intersection**
- Each corner is a **graph node** candidate for junction tee
- **Invalid:** diagonal shortcut between non-adjacent axis points → preview red hatch `blocked: diagonal not allowed in 90° mode`

### 4.2 Yard feed pattern (reference)

```text
Substation pad ──H──┐
                    └──V── factory bus
```

---

## 5. Mode switch mid-draw

| Situation | Behaviour |
|:---|:---|
| Switch with ≥2 points | **Recompute** preview from same control points |
| Curved → 90° | Project points to grid · insert corners |
| 90° → Curved | Keep corner points as spline knots |
| Invalid after switch | Strip `POWER · blocked: mode switch — {reason}` |

**No** silent drop of points.

---

## 6. Road parity matrix

| Aspect | Road today | Power target |
|:---|:---|:---|
| Curved toggle | `use_curved_preview` checkbox | **Mode chip** (clearer) |
| Orthogonal | — | **First-class mode** |
| Commit match | preview = commit | **same rule** |
| Shift+LMB | commit segment | **same** |

---

## 7. Acceptance

| # | Test |
|:---:|:---|
| R1 | 90° rectangle — no diagonal segments in committed polyline |
| R2 | Curved HV — smooth preview = committed stroke |
| R3 | `O` cycles without closing sheet |
| R4 | Grid snap only active in 90° mode |

---

## 8. Sign-off

| Role | Verdict | Date |
|:---|:---|:---|
| `@designer` | **PASS** | 2026-06-18 |
