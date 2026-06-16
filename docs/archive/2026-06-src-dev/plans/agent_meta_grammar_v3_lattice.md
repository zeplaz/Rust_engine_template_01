# AGENT-META grammar `v3Ω` — ΩMETA-LATTICE · STATE · FLOW · REVIEW

| Field | Value |
|:---|:---|
| **ID** | **AGENT-META-GRAMMAR-003-LATTICE** |
| **Status** | **ACTIVE** 🟢✅⊚ |
| **Parent** | [`agent_meta_brief_v3.md`](agent_meta_brief_v3.md) |
| **Entry prompt (FIELD◈ mirror)** | [`prompts/llm_agent_brief.md`](../../prompts/llm_agent_brief.md) §FIELD◈ |

**Human agents:** read `llm_agent_brief.md` §FIELD◈ first. This file is the **normative superset** (same grammar, full tables).

**Rule:** symbols are **grammar**, not decoration. Parse **clusters** (e.g. `🟡⏳⊗☊`, `🔴❌🏛`) as compact semantic packets — not prose.

---

## FIELD◈ΩMETA-LATTICE

```text
{Ct, Cx, Cm, Dp, Au, Rk, Q, Δ, U, T, E, H, S}
```

| Sym | Name | Emoji lane | Emit when |
|:---:|:---|:---|:---|
| **Ct** | TokenCost | 💰📉 | Read · rewrite · chat burn |
| **Cx** | Complexity | 🌀🕸⊗ | coupling · schedule · accretion |
| **Cm** | Maintenance | 🌊♻🔥 | debt wave · scaffold · dual-writer |
| **Dp** | Dependency | ⛓🔗⊗ | matrix fan-out · `$ref:` chains |
| **Au** | Authority | 🏛⊚⬇ | layer · SystemSet · single writer |
| **Rk** | Risk | 🕳⚠☋ | hidden failure · VM-* · regression |
| **Q** | Quality | 🎯📈🌟 | correctness · observability · compounding |
| **Δ** | Change | ⚖⊕⊖ | diff surface · witness delta |
| **U** | Uncertainty | ⌁🧠? | assumption · unresolved model |
| **T** | Time | ⏱⟳ | temporal drift · schedule slip |
| **E** | Evidence | 🧪🔬📜 | measured · witnessed · validated |
| **H** | HumanImpact | 💬📎👁 | operator · ask gate · UX readability |
| **S** | Scale | 🌐S+ | multi-view · fleet · OPS breadth |

**Objective (extended):**

```text
Σ(Q + Au + E + Veracity + H) − Σ(Ct + Cx + Cm + Rk + U×T + Entropy)
```

---

## SEMANTIC-LEXICON◈ — single symbols

```text
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
 VERIFIED / QUALITY          AUTHORITY / TRUTH           EVIDENCE / MEASURE
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
 🟢✅  Verified+Observed      🟩⊚  Authority Truth Node   🧪🔬  Measured Evidence
 🎯📈  Quality Gain Surface   🏛⬇  Authority Flow Valid    📜👁  Witnessed Reality
 🌟Q+  Compounding Value

 PARTIAL / DRIFT             CONFLICT / BAD              COST / COMPLEXITY
 🟡⏳  Partial/Drifting       🔴❌  Known Bad             📉💰  Token Sink
 ⚠☊   Constraint Cluster     ☍🏛  Authority Conflict      🌀Cx+ Complexity Accretion
 💬📎  User Authority         ⛓⊗  Dependency Chain        ♻🔥  Debt Reservoir
 ⚡🚦  Hard Gate              🕳Rk+ Hidden Failure        🌊Cm+ Maintenance Wave
 ↺⟲   Feedback Loop          ╳⊗  Cross-Domain Leak         ⏱T+  Temporal Drift
 ⚖Δ   Change Magnitude       🌐S+  Scaling Pressure
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
```

---

## STATE-GRAMMAR◈ — cluster packets (max 4 emoji per node)

**Parse left→right:** status · evidence · coupling · constraint

```text
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
 STATE-GRAMMAR◈
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

 🟢✅⊚     verified + authority valid
 🟢✅🧪     verified by measurement
 🟢✅📜     witnessed in production
 🟢✅🎯     verified quality improvement

 🟡⏳☊     partial + blocked by constraint
 🟡⏳💰     partial + token expensive
 🟡⏳⊗     partial + dependency unresolved

 ⚠☊🏛     authority constraint
 ⚠☊💰     budget constraint
 ⚠☊🌀     complexity constraint

 🔴❌🏛     authority violation
 🔴❌⊗     dependency violation
 🔴❌📉     proven waste

 ♻🔥🕳     hidden debt source
 ♻🔥🌊     maintenance growth source

 ⚡🚦🧪     validation gate
 ⚡🚦📜     witness gate
 ⚡🚦🏛     architecture gate
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
```

| Cluster | Meaning | ⚑ Action |
|:---|:---|:---|
| `🟢✅⊚` | closed · authority clean | ship · BLANG:Q✓ |
| `🟢✅🧪` | validator green | cite compress=4 JSON |
| `🟢✅📜` | witness landed | `$ref:debug_runs/…` only |
| `🟡⏳⊗☊` | blocked on deps + constraint | `⟨BP:COLLECT⟩` · shrink scope |
| `🔴❌🏛` | layer / writer violation | halt · ui_boundary / sim-grade |
| `⚡🚦📜` | witness gate open | BLANG:WIT before ✅ |

---

## FLOW-GRAMMAR◈ — authority & dependency edges

```text
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
 FLOW-GRAMMAR◈
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

 ⊚A━━▶⊚B           authority flow (soft)
 ⊚A━━┅▶⊚B          weak authority
 ⊚A═▶⊚B            hard authority (normative)
 ⊚A↺B              closed feedback
 ⊚A☍B              conflict
 ⊚A⊗B              coupled
 ⊚A⋮B              observational relationship
 ⊚A⤴B              escalation
 ⊚A⤵B              delegation
 ⊚A⟳B              optimization loop
 ⊚A⥁B              drift loop
 ⊚A⛓B              required dependency
 ⊚A⌁B              probabilistic dependency
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
```

**Repo instantiation (MCP art spine):**

```text
⊚MaterialAuthority═▶⊚AssemblySnapshot═▶⊚Worker═▶⊚Atlas═▶⊚Runtime
       ☍
       └──🔴❌🏛  (example: parallel BlenderAuthoring bypass)
```

**ECS spine:**

```text
⊚Serializable⛓⊚ECSRuntime═▶⊚ToolsUI
       ⛔ UI→ECS invert = 🔴❌🏛
```

---

## COGNITIVE-WEIGHTS◈ — score proposals

| Tag | Weight | Favor when |
|:---:|:---|:---|
| 🎯📈 | +Q | improves output quality |
| 🧪🔬 | +E | increases certainty |
| 🏛⊚ | +Au | preserves architecture |
| 📉💰 | −Ct | lowers operating cost |
| ♻🔥 | −Cm | lowers future maintenance |
| 🌐S+ | +S | improves scalability |
| 🕳Rk+ | −Rk | increases unknown risk |
| 🌀Cx+ | −Cx | increases systemic complexity |

**Utility (grammar-aware):**

```text
Utility = (🎯📈 + 🧪🔬 + 🏛⊚ + 📉💰⁻¹ + ♻🔥⁻¹ + 🌐S+)
          ─────────────────────────────────────────
          (🌀Cx+ + 🕳Rk+ + ⌁U + ⏱T+ + ⛓⊗)
```

---

## REVIEW-TAGS◈ — inline critique markers

```text
🧠?   assumption detected
☍!    competing model exists
⌁?    uncertainty unresolved
⊗!    hidden dependency likely
🕳!   failure not observable
💰!   token burn disproportionate
🏛!   authority drift
🌀!   complexity growth exceeds value
🌊!   maintenance growth exceeds value
⚡!   gate not satisfied
```

**Review block template:**

```text
Review◈
  🧠? …
  ⊗! …
  ⌁? …
  ⚡! …
  🎯📈 …
  📉💰 …
```

---

## EXAMPLE-MARKER◈ — slice row (target style)

```text
╔═ SLICE ⟨APS-MAT-001⟩ ═══════════════════════════════════════════════════╗
║  STATE   🟡⏳⊗☊                                                         ║
║  LATTICE Ct:🟨🟨  Cx:🟨🟨🟨  Cm:🟨🟨  Au:🟩🟩🟩🟩  U:🟨🟨🟨              ║
╠═════════════════════════════════════════════════════════════════════════╣
║  FLOW                                                                 ║
║    ⊚MaterialAuthority═▶⊚AssemblySnapshot═▶⊚Worker═▶⊚Atlas             ║
║           ☍                                                           ║
║           └──🔴❌🏛 BlenderAuthoring (parallel bypass)                  ║
╠═════════════════════════════════════════════════════════════════════════╣
║  Review◈                                                              ║
║    🧠? preview parity                                                 ║
║    ⊗! worker dependency                                               ║
║    ⌁? material migration                                              ║
║    ⚡! witness pending                                                ║
║    🎯📈 expected quality gain high                                    ║
║    📉💰 expected token cost low                                       ║
╠═════════════════════════════════════════════════════════════════════════╣
║  Result◈  🟢✅🧪🟩⊚  only after witness + validation                   ║
╚═════════════════════════════════════════════════════════════════════════╝
```

**Ledger:** append via `BLANG:MARK` · `agent_marker_append` · `debug_runs/agent_ops/agent_markers.jsonl`

---

## BLANG / v1 mapping

| Grammar | AGENT-LANG v1 |
|:---|:---|
| `🟢✅📜` | 🟢 + `BLANG:WIT` |
| `⚡🚦🧪` | `BLANG:CARGO` · `BLANG:BEVY` |
| `⚡🚦📜` | `BLANG:WIT` before close |
| `🟡⏳⊗☊` | 🟡 + 🧩 + `⟨BP:COLLECT⟩` |
| `⊚A═▶⊚B` | AUTH spine `★` / `⇢` |
| Review tags | honest marker `mirror` · `scan` · `why` |
| LATTICE bars | HEALTH◈ vertical fold |

---

## Anti-patterns

| Don't | Do |
|:---|:---|
| Random emoji suffix | Valid STATE-GRAMMAR cluster |
| One-line slice status | EXAMPLE-MARKER fold block |
| `✅` without `🧪`/`📜`/`⊚` | `🟢✅🧪` or `🟢✅📜` |
| Prose-only review | Review◈ with `🧠?` `⊗!` `⚡!` tags |
| Invent flow arrows | Use FLOW-GRAMMAR symbols only |

---

## Changelog

| Version | Date | Note |
|:---|:---|:---|
| v3.3.0 | 2026-06-07 | ΩMETA-LATTICE · STATE · FLOW · REVIEW grammar |
