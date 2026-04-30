# Platforms, munitions, sensors & EW `v1`

**For:** engine + simulation design · **Not** distribution, pricing, or “what ships to customers.”

---

## Locked intent (game/engine)

- **Strategic depth** includes **munitions** (missiles, rockets, guided / unguided), **drones / UAS**, **manned aircraft**, **naval fires**, **land launchers**, **radar / ESM**, and **jamming / EA** — all constrained by **logistics, power, damage, and LOD** (see `terrain_world/simulation_lod_v1.md`).
- **Buildings** remain **modifiable infrastructure** (production, sensors, launchers, comms) — pairs with `production_economy/` and structure blueprints.
- **Terrain** affects **line of sight**, clutter, hydrology (ports, flooding), and **degraded EW** in weather/low altitude 📎.

---

## Layer model (technical)

| Layer | Simulation focus | Tools / debug |
|:---|:---|:---|
| **C2 / automation** | Orders, queues, delayed updates; respects server sim tier | Order timeline inspector |
| **Platform kinematics** | Road / rail / sea / air LOD handoff (`navigation/`) | Per-entity state panel |
| **Weapons / magazines** | Reload, compatibility, arming delay | Magazine + safety interlocks view |
| **Flight / kinetic segment** | Missile phases (boost, mid, terminal); corridor promotion | Track debugger + LOD pocket viz |
| **Sensors** | Abstract SNR / Pd; waveform as config | Emitter list, track fusion stub 📎 |
| **EW** | Noise, deception, burn-through vs radar equation abstraction | Jammer ERP, frequency notch UI 📎 |
| **Damage integration** | Same `DamageState` family as industry (`production_economy/`) | Component outage → track break |

---

## Coupling notes

- **Jamming vs missiles:** jammer affects **seeker / uplink** quality — same server tick as track update; client sees **degraded cues**, not alternate physics.
- **Ships firing missiles:** **magazine + launcher** components; **vessel motion** feeds fire solution input (coarse at low LOD).
- **Drones:** battery / link budget as **resources**; LOS to relay towers or sat stub 📎.
- **Air + ground radar:** **rotation / scan** as simplified scheduler — expensive modes only in **focus** LOD.

---

## Open technical questions (📎)

1. **Track **representation:** entity-per-track vs aggregated blob for distant tier?
2. **Seeker laws:** canned proportional vs table-driven — first milestone?
3. **EW fidelity:** additive noise model only v0 vs separate **EP / ES** chains?
4. **Cyber layer:** defer until power + comms graph stable — yes/no?

---

## Cross-links

- Matrix: `prompts/matrix/strategic_platforms/strategic_platforms_matrix_v1.md`
- Phasing: `phased_engine_delivery_v1.md`
- Path + LOD: `terrain_world/simulation_lod_v1.md`, `navigation/pathfinding_hierarchical_v1.md`
