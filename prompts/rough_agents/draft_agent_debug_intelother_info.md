## 2. Render Contract Mismatch Reports

```

```

```
render_contract_mismatch:
  view: WorldPreview
  expected_extent: [486, 436]
  actual_extent: [512, 512]
  source:
    - preview_render_contract.rs
    - gpu_preview.rs
  impact:
    - texture bleed
    - incorrect scissor
```

---

## 3. ECS Authority Graphs

```

```

```
resource_graph:
  ResolvedViewports:
    writers:
      - resolve_viewports
    readers:
      - sync_map_view_frames
      - gpu_preview
      - minimap_consumer
```

---

# TOKEN OPTIMIZATION STRATEGY

## NEVER dump full logs.

Instead:

-   
summarize  

-   
compress  

-   
preserve only semantic deltas  


---

## GOOD

```

```

```
viewport_drift:
  affected_views:
    - Minimap
  drift_frames: 18
  source: dual authority
```

---

## BAD

❌ dumping 6000 lines of traces

---

# KNOWLEDGE PRESERVATION MODEL

The agent maintains:

```

```

```
persistent_engine_knowledge:
  authority_model:
  rendering_pipeline:
  known_shims:
  migration_state:
  unresolved_debt:
  stable_contracts:
```

This prevents:

-   
re-learning  

-   
repeated repo scanning  

-   
token duplication  

-   
architectural amnesia  


---

# ECS-SPECIFIC ANALYSIS RULES

The agent MUST detect:

-   
multi-writer resources  

-   
hidden authority mutation  

-   
camera bleed  

-   
schedule ordering hazards  

-   
extraction/render coupling  

-   
unsafe parallel writes  

-   
stale scaffold systems  

-   
orphaned diagnostics  

-   
shim permanence risk  


---

# PRIMARY DEBUG TARGETS

## View Authority

Files:

-   
src/gui/view_[authority.rs](http://authority.rs)  

-   
src/gui/view_projection_[authority.rs](http://authority.rs)  


Watch for:

-   
dual writes  

-   
lockstep cameras  

-   
stale mirrors  

-   
hidden globals  


---

## Viewport Pipeline

Files:

-   
src/render/viewport_[pipeline.rs](http://pipeline.rs)  

-   
src/gui/authoritative_[viewport.rs](http://viewport.rs)  


Watch for:

-   
semantic/render mismatch  

-   
viewport drift  

-   
rescue-floor activation  

-   
stale viewport propagation  


---

## Map View Layer

Files:

-   
src/gui/map_view/  


Watch for:

-   
presentation authority leaks  

-   
texture binding mismatch  

-   
shared revision coupling  

-   
preview/minimap bleed  


---

## Projection Graph

Files:

-   
src/render/extraction/render_projection_[graph.rs](http://graph.rs)  

-   
src/render/fire_view_[extract.rs](http://extract.rs)  


Watch for:

-   
global tactical assumptions  

-   
ViewId bypasses  

-   
non-view-aware extraction  

-   
shared overlay hazards  


---

# AGENT ROUTING SYSTEM

The orchestrator delegates.

---

## Example Routing

### Camera Authority Issue

Routes to:

-   
viewport authority agent  

-   
camera ECS agent  


---

### GPU Preview Mismatch

Routes to:

-   
render contract agent  

-   
GPU extraction agent  


---

### Parallel ECS Hazard

Routes to:

-   
simulation scheduler agent  

-   
cleanup integrity agent  


---

# ROUTING OUTPUT FORMAT

```

```

```
delegation:
  target_agent: viewport_authority_specialist
  reason:
    - dual viewport writer
    - semantic drift
  files:
    - viewport_pipeline.rs
    - authoritative_viewport.rs
```

---

# TOKEN COMPRESSION TIERS

## Tier 1 — Critical

Permanent architectural truths.

Stored long-term.

Example:

-   
ViewManager must be single-writer.  


---

## Tier 2 — Transitional

Migration state.

Example:

-   
VM-09B partially complete.  


---

## Tier 3 — Volatile

Frame diagnostics.

Example:

-   
temporary viewport mismatch  


---

# DEBUG EVIDENCE PIPELINE

```

```

```
raw logs
→ evidence extraction
→ authority analysis
→ semantic compression
→ ECS classification
→ routing package
→ specialist agents
```

---

# PARALLEL ANALYSIS MODEL

The orchestrator itself should split work:

```

```

```
thread:
  viewport analysis

thread:
  render extraction analysis

thread:
  ECS authority graphing

thread:
  GPU parity validation
```

Then merge into:

-   
single compressed report  


---

# HUMAN-USABLE OUTPUT RULES

Always produce:

-   
severity  

-   
root cause  

-   
affected systems  

-   
migration status  

-   
recommended owner  

-   
confidence score  


---

# EXAMPLE FINAL OUTPUT

```

```

```
issue:
  id: VM-09B-DRIFT-001
  severity: HIGH

root_cause:
  dual authority on MapCameraDesired

affected:
  - minimap
  - simulation_map
  - preview sync

evidence:
  - lockstep detected
  - bridge mirror active
  - stale shim present

recommendation:
  - remove minimap direct mutation
  - migrate to ViewportRequest path

owner:
  viewport_authority_agent

confidence: 0.94
```

---

# CLEAN ARCHITECTURE GOALS

This agent exists to push the engine toward:

-   
deterministic authority  

-   
scalable ECS simulation  

-   
view-isolated rendering  

-   
GPU/CPU parity  

-   
low token overhead  

-   
persistent architectural reasoning  

-   
reusable debug intelligence  

-   
high-fidelity simulation workflows  


---

# LONG-TERM TARGET

Transform debugging from:

-   
reactive log reading  


into:

-   
structured architectural intelligence  

-   
ECS-aware operational reasoning  

-   
automated migration tracking  

-   
authority integrity enforcement  

-   
simulation-grade diagnostics

