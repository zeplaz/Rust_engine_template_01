# PLAN-SLICE-C-SENSORY-v1 — audio · sprite ASM · ballistics/blast

| Field | Value |
|:---|:---|
| **Id** | PLAN-SLICE-C-SENSORY-v1 |
| **Parent** | [`plan_ref_books_upgrade_use_v1.md`](plan_ref_books_upgrade_use_v1.md) Ranking **C** / Slice C |
| **Status** | **CHARTER + SCAFFOLD** (2026-11-14) |
| **Cites** | gea_v2_e4 p442–522 (audio) · p253/p314 (anim/ASM) · p423/p440 (physics) |

## Authority

| Resource / surface | Sole writer | Layer | Must not |
|:---|:---|:---|:---|
| `SimAttentionFrame` | `build_sim_attention_frame` (`AttentionSet::Build`) | Sim | Dual writers; UI mut |
| `AudioMixFrame` | `build_audio_mix_frame` (`AudioMixSet::Build`) | Presentation | Write sim / MissionBook |
| `AudioOneShotQueue` | UI / combat **emitters** enqueue; mix drains | Ingress → mix | Direct `bevy_audio` from sim systems |
| `VisualActionState` | Presentation systems only | View/Presentation | Sim tick owning phase |
| Ballistic / blast truth | Combat / construction execute (TBD named systems) | Sim | Rapier world for every truck |

## C1 Audio event classes

| Class | Examples | Drive |
|:---|:---|:---|
| UI | click, confirm, deny | `AudioOneShotQueue::push_ui` |
| Ambient | rain, wind | weather aggregate in mix builder |
| Spatial continuum | near-field when zoomed (truck/plant later) | listener zoom × attention |
| Combat one-shot | shot, explosion, missile launch | `push_combat` from resolve dumps |
| Warning | launch warning, weather alert | `push_warning` → HUD + mix |

**Next code:** bind `bevy_audio` playback to `AudioMixFrame.requests`; MCP/SFX asset catalog; wire missile sortie dump → `MissileLaunch` / `LaunchWarning`.

## C2 Sprite / building / truck ASM

- Types: `presentation::visual_asm::{VisualActionPhase, VisualActionState}`
- Maps onto existing `tile_visual_state::VisualState` (variant × facing × frame)
- Phases: Idle / Work / Move / Damaged / Destroyed / Launch / Alert
- **Reject:** skeleton skinning / motion-matching until hero 3D units

**Next code:** stamp `VisualActionState` on logistics vehicles + plants; drive phase from order-sim / MFG RO snaps.

## C3 Ballistics / blast / destruction

| Concern | Pattern | Reject |
|:---|:---|:---|
| Missile / drone flight | Deterministic sim integrate (extend ember-ballistic spirit) → presentation reads | Per-airframe physics bodies |
| Blast / crater | Construction execute / env damage sole funnel | Overlay writing HP |
| Collision queries | Footprint / radius vs graphs | Full rigid middleware by default |

**Next code:** `combat` ballistic step resource + blast apply through construction invariants; visual FX + audio one-shots as consumers.

## Schedule

```text
AdvanceSimTick → AttentionSet::Build → ChunkEnvironmentSet::Lod …
ViewAuthoritySystemSet::SyncViewManager → AudioMixSet::Build
```

## Exit (scaffold)

- [x] `SimAttentionFrame` + plugin wired (+ AO/fire foci)
- [x] `AudioMixFrame` + queue + plugin wired
- [x] `VisualActionState` types + unit test
- [x] This charter + authority map rows
- [x] Playback binder + `SfxCatalog` + Startup load of `assets/sfx/*.wav`
- [x] Formation / plant ASM stamp + posture sync
- [x] `BallisticFlightBook` step + impact audio enqueue
- [x] SFX placeholder WAVs registered under catalog tags
- [x] Blast damage through construction funnel (`BlastIntentQueue` → demolish + crater)
- [x] Logistics truck presentation entities + `VisualActionState`
