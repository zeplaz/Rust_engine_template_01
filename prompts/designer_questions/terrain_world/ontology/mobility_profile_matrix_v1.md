# Mobility profile matrix (v1)

**Status:** gameplay contract — how **agents / vehicles / modes** interpret **terrain facts**.  
**Inputs:** per-cell `TagSet` (facts) + optional **derived metrics** (see [`derived_metric_pipeline_v1.md`](derived_metric_pipeline_v1.md)) + graph edges (roads/rail) when applicable.

---

## 1. Separation of concerns

| Layer | Owns |
|:---|:---|
| Terrain | Facts only ([`fact_vocabulary_rulebook_v1.md`](fact_vocabulary_rulebook_v1.md)) |
| Mobility profile | **Requirements** and **reaction rules** to fact combinations |
| Transport graph | Infrastructure precedence, lane occupancy, regimes — not tile tags |

Same tile, different rows in the matrix below — **without** changing terrain.

---

## 2. Mobility profile (data shape)

**Implementation (repo):** `assets/config/terrain/mobility_profiles.example.ron` (Bevy `AssetLoader`), [`MobilityProfileRegistry`](../../../src/terrain/mobility/mod.rs), [`evaluate_tile`](../../../src/terrain/mobility/mod.rs) → [`MovementHint`](../../../src/terrain/mobility/mod.rs).

Conceptual Rust (aligned with code):

```rust
pub struct MobilityProfileId(pub &'static str); // e.g. "wheeled_logistics"

pub struct MobilityProfile {
    pub id: MobilityProfileId,
    pub traction_requirement: f32,   // min “traction index” cell must provide
    pub max_grade: f32,              // degrees or normalized slope from derived metric
    pub amphibious: bool,
    // width, clearance, etc. as needed
}

pub struct MobilityRule {
    pub when_all: Vec<TagName>,      // fact tags that must all be present
    pub when_any: Vec<TagName>,      // optional alternate group
    pub cost_multiplier: f32,
    pub stuck_risk: f32,             // 0..1 event risk per edge or per hour — project choice
    pub blocked: bool,                // hard veto for this profile
}
```

**Evaluation order (v1 — locked in code)**

1. Gather **facts** (`TagSet`) + **derived** scalars (`slope_grade`, …); traction index is a stub until sim provides it.
2. If **on-network** (edge traversal), apply graph-specific costs / permissions first (future).
3. **Hard veto:** `slope_grade > max_grade` (and optional traction / amphibious rules) **or** any matching rule with `blocked: true` → movement **impossible**.
4. **Cost:** `final_cost_mul = Π (cost_multiplier)` over all matching rules (multiplicative stacking).
5. **Risk:** `stuck_risk = max(rule.stuck_risk)` over all matching rules.
6. Emit **`MovementHint`** for pathfinder / preview: `{ cost_mul, blocked, stuck_risk }`.

**Asset format:** **RON** for hand-edited mobility profiles (Rust-native, comments, deterministic loads). JSON/YAML examples in docs are illustrative only.

---

## 3. Example profiles (illustrative)

| Profile | Intent |
|:---|:---|
| `foot_infantry` | Baseline off-road; penalize `steep`, `muddy`, `dense_vegetation` |
| `wheeled_logistics` | Strict on `max_grade`, `muddy`, `loose_surface` |
| `tracked` | Better on `soft_surface`, still blocked by `cliff` unless engineered |
| `rail` | Almost never from tile tags alone — **graph-only** with bridge/tunnel assets |
| `rotor` | Weak coupling to surface facts; strong coupling to **derived** exposure / wind (future) |
| `engineering_surface_prep` | Temporary reduction of `stuck_risk` for followers after work (sim state, not tag rewrite) |

---

## 4. Example rule table fragment (YAML-like)

Illustrative YAML shape — runtime: **RON** (see `mobility_profiles.example.ron`):

```yaml
profile: WheeledLogistics
rules:
  - when_all: [steep]
    blocked: true
  - when_all: [muddy]
    cost_multiplier: 4.0
    stuck_risk: 0.3
  - when_all: [hard_surface]
    cost_multiplier: 1.0
  - when_all: [loose_surface]
    cost_multiplier: 2.0
    stuck_risk: 0.15
```

**Note:** `cliff` may be `blocked: true` for wheeled but not for rope/air — different profiles, same facts.

---

## 5. AI and UX

- **AI** consumes: cost, delay, attrition risk, exposure, engineering requirement — not a boolean passable.
- **Player overlays**: per-profile **heatmap**, not global green/red (see preview evolution in matrix U5 / composite preview docs).

---

## 6. Contract

Mobility assets are the **gameplay contract** for:

- pathfinding costs
- logistics planning
- scenario tooling (“what-if” profiles)

They MUST NOT be duplicated as terrain tags.
