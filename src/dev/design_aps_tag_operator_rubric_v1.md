# APS tag + Variants operator rubric `v1` — tier-1 sign-off walk

| Field | Value |
|:---|:---|
| **ID** | **DES-APS-TAG-RUBRIC-001** |
| **Program** | APS-UX-AUDIT-001 tail · tier-1 tag vocabulary |
| **Date** | 2026-06-02 |
| **Owner** | `@operator` (execute) · `@designer` (charter) |
| **Launch** | `python -m art_pipeline_suite.run` · MIN window 960×600 |
| **Verdict** | **READY** (operator pending) |

---

## Preconditions

- [ ] `pytest tools/mcp/python/tests/test_aps_tag_vocabulary.py -q` green
- [ ] Example warehouse assembly loaded
- [ ] Variant set with ≥1 row

---

## Rubric rows

| # | Step | Pass if |
|:---:|:---|:---|
| R1 | Variants → toggle **Window glow** | Context line names emissive read; preview updates **before** Apply |
| R2 | Change Lighting to `night_on` | Context line mentions night; **Draft — not saved** strip visible |
| R3 | Click **Apply layers to selected** | Draft strip clears; list row updates |
| R4 | Reaction filter → **Heritage site destruction** | Context line lists suggested anchors (Burn origin, Heritage marker, …) |
| R5 | Mandate tag checkboxes | **Human labels** only — no raw `cultural_survival` snake_case |
| R6 | Assembly → placement tags | Hover shows semantic hint in next-step or tooltip |
| R7 | Assembly variant tags row | Shows Clean / Night read / Fire damage — not engineer ids |
| R8 | Generation trace strip | Shows archetype · district · seed; Approve toggles |
| R9 | Tooltips on Apply layers + reaction filter | Plain language; no ARCH-MAT jargon |
| R10 | MIN 960×600 Variants tab | Layer controls reachable without h-scroll |

---

## Fail actions

| Fail | Route |
|:---|:---|
| R1–R4 | @coder-mcp · live preview regression |
| R5–R7 | @coder-mcp · `aps_tag_vocabulary` |
| R8 | @coder-mcp · `generation_trace_strip.py` |
| R10 | @designer + @coder-mcp · layout F3 |

---

## Witness

Attach to HANDOFF:

- `debug_runs/aps_session_presence_live.json` (if green)
- Screenshot optional
- Rubric checklist with pass/fail per row

---

## Sign-off

| Role | Verdict | Date |
|:---|:---|:---|
| `@designer` | **READY** | 2026-06-02 |
| `@operator` | **PENDING** | — |

Unblocks: **DES-APS-TAG-TIER2-001** wire
