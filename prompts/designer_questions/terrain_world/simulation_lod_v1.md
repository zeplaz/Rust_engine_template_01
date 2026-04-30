# Designer Q — simulation & render LOD framework `v1`

Audience: design + systems architects · **📎** many constants need user sign-off before implementation.

---

## References (mental model only)

Games cited for **loading bubble** / **distance degradation** ideas — **not** to copy flaws:

- Falcon-style **active bubble**
- Stalker-style **switching**
- X4-style **out-of-sector abstraction** (known issue: **combat feels different** off-screen — **avoid**)

---

## Locked intent

1. **Tiered simulation**, not binary on/off for whole world.
2. **No harsh pop-in** — entities entering “live” tier should **blend** (temporal/spatial hysteresis, fade, or pre-warm).
3. **Consistency** — far combat / missiles / logistics must **not** use a totally different ruleset than near (X4 pitfall). **Same governing equations**, different **time step / aggregation / simplification**.
4. **Dynamic LOD “pockets”** — regions can **temporarily promote** fidelity (e.g. missile track, impending impact, player attention, strategic strike) then **relax** when resolved.
5. Couples to **chunk streaming** — see `chunks_streaming_v1.md`.

---

## Proposed framework (conceptual)

### Axis A — **LOD level** (discrete steps — **v1 target: 3 sim layers**)

| Lvl | Working name | Render | Physics | Sim detail | Typical cadence |
|:---:|:---|:---|:---|:---|:---|
| 0 | **Focus** | Full | Full (or game-defined) | Per-entity | Every sim tick |
| 1 | **Near** | Simplified | Reduced | Batched / coarse | Every N ticks |
| 2 | **Regional / distant** | Abstract / off | None or statistical | Flow aggregates, graph solvers, ledgers | Event-driven / rare |

📎 **Note:** Earlier drafts used 4 named rows (0…3). **Implementation starts at 3 layers** — if a 4th tier is added later, update **this table and** `chunks_streaming_v1.md` together.

📎 Exact **N**, tier count, and names — **ASK user** after prototype metrics.

### Axis B — **Promotion triggers** (elevate toward 0)

- Player camera / selection proximity
- **Faction interest** (war, trade route under attack)
- **Projectile / strike corridor** — “impact zone” pre-promotes before hit
- **Scheduled event** (arrival time known)
- **User pin** (“watch this grid”)

### Axis C — **Hysteresis**

- **Enter** tier on threshold T_in; **leave** on T_out > T_in to prevent oscillation.
- **Min dwell time** at high tier before demotion (stops flicker).

### Axis D — **Cross-tier handoff**

- **State summary** passed upward (mass, momentum, damage, cargo, intent) — no hard reset.
- **Replay / verify** optional on promotion (server validates summary vs recomputed micro — 📎 cost tradeoff).

---

## Sub-questions (⏳) — overlap with **Answers** section

1. **Authority**: server assigns LOD per chunk/entity; clients only render what they’re told? (align `_legacy_designer_questions_v1.md` §A3.)
2. **Missiles / long-range**: single “corridor entity” with LOD schedule, or tile ribbon promotion?
3. **Max concurrent pockets** at L0 — cap to protect frame budget?
4. **MP fairness**: if two players “watch” same distant battle, does pocket merge?
5. **Save/load**: persist current LOD tier or recompute on load?
6. **Pathfinding**: must use **same graph** at all tiers with **relaxed refinement** — see `prompts/designer_questions/navigation/pathfinding_hierarchical_v1.md`.

---

## Implementation note for agents

Do **not** implement tiers until **chunk bounds + sim tick** are chosen. Start with **three** tiers (focus vs a middle level, vs everything-else) in prototype, then grow.

---

## Answers & implementation cues (append-only)

**Round:** 2026-04-27 · **For:** LLM + engineers

| Topic | Decision |
|:---|:---|
| Initial sim LOD count | **3 layers** — must stay **consistent** with chunk ring policy in `chunks_streaming_v1.md` (same doc family; update both if tier count changes). |
| Multiple foci | **Moving orbs of interest** (player blobs, bookmarks, strikes, script POIs) each carry **priority** → **different effective ring structures** (not one-size radius for all). |
| Typical footprint | Align with chunks: **9 chunks** (center + **8** Moore neighbors) is the **usual** high-attention footprint; **promoted pockets** may temporarily **expand** (missile corridor, flood front). |
| Server vs render | **Server** assigns sim LOD / interest; **client** renders per camera (decoupled), but **does not** decide authoritative sim tier. |

### 📎 Consistency check

- v1 **Axis A** = **3 rows**. Any future **4th tier** must be documented **here and** in `chunks_streaming_v1.md` on the same commit.

### Cross-links

- Pathfinding LOD handoff: `prompts/designer_questions/navigation/pathfinding_hierarchical_v1.md`
- Bookmark limits / radii: `chunks_streaming_v1.md`
