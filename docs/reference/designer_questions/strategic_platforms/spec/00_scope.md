# Strategic platforms — scope `00`

**Design:** [`../platforms_ew_munitions_v1.md`](../platforms_ew_munitions_v1.md)

## In scope

- **Platforms:** buildings (fixed), land vehicles, **supply drones/UAS**, **ships**, **cargo haulers**, armed units — unified where possible under one component taxonomy 📎.
- **Weapons & munitions:** line-of-sight, ballistic, powered — magazines tie to production resources.
- **Sensors & EW:** radar, **jamming**, RWR-lite 📎; C2 latency buckets (not marketing).
- **LOD:** promotion/demotion tied to chunk/streaming (`terrain_world/chunks_streaming_v1.md`).

## Out of scope / defer

- Classified real-world ordnance tables; use stylized data files.

## Authority

- **Fire orders**, track custody, and damage application: **server**-validated for MP (`implementation_questions_v1.md`).
