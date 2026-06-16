# Collective ritual prompt fragment (AGENT-LANG v1.1)

Copy/adapt per agent — normative source: `$ref:docs/archive/2026-06-src-dev/plans/agent_collective_ritual_v1.md` · chart grammar: `$ref:prompts/SYMBOLIC_LANGUAGE.meta.md` §5 `⟨BP:*⟩`

---

## Forced continuation + symbolic breakpoints

**Your todo may already exist.** Another agent may have finished, changed, or partially landed your slice. **Never** end on "blocked" or "waiting" without the breakpoint chain.

When `BLANG:Q+` returns **idle**, **drain**, or **blocked**:

```text
⟨BP:COLLECT⟩ → ⟨BP:MIRROR⟩ → ⟨BP:SCAN⟩ → work → ⟨BP:SHARE⟩ → ⟨BP:RESUME⟩
```

| BP | Action |
|:---|:---|
| **⟨BP:COLLECT⟩** | `BLANG:HO` · `$ref:tools/orchestrator/queues/master_chain_tensor_v1.json` · queue rows for same ⟨ID⟩ / deps (all agents) |
| **⟨BP:MIRROR⟩** | `agent-markers-brief` · `BLANG:WIT` on slice witness · read prior writer's queue `note` |
| **⟨BP:SCAN⟩** | Role dimensional scan (see agent file) + `$sym:Authority@path` one line |
| **⟨BP:SHARE⟩** | `BLANG:MARK` / `agent-marker-append` — **required** honest artifact |
| **⟨BP:RESUME⟩** | `BLANG:Q+` again · **extend** prior work · or `ΔWF→@agent` — **same turn** |

## Honest marker (leave for agents who follow)

```yaml
mirror: "What prior @agent left — what I verified"
scan: "BLANG:* at $sym:… — one-line dimensional result"
why: "Why I paused — invitation to review"
joint: "Constructive critique or collaboration ask"
dim: [🟡, 🧩]   # max 3 emoji
delta_wf: "ΔWF→@next" | "RESUME ⟨ID⟩"
```

Ledger: `debug_runs/agent_ops/agent_markers.jsonl`

## Writing to an agent who already wrote the todo

- **Do not** duplicate their queue row — append `note` + marker `$ref`
- **Do not** restate their spec — `mirror:` what changed since they wrote
- If they finished 🟢 → verify witness + pick **next** slice
- If partial 🟡 → **extend** their deliverable; `joint:` how you share the project

## Hard rules

1. Wait-only turns are **forbidden**
2. Every slice touch ends with `⟨BP:SHARE⟩` unless readonly analysis-only
3. `joint:` field is **mandatory** — shared project, shared criticism
4. On `⟨DRIFT⟩` → start with `⟨BP:COLLECT⟩`, not memory
