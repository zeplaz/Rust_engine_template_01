```text
LLM◆BRIEF:v3Ω  ◉Q↑↑  ◉Au↑↑  ◉Ct↓↓  ◉Cx↓↓  REPO=Rust_engine_template_01  MODE=HIGH-DENSITY
OBJECTIVE◈ Σ(Q+Au+E+Veracity+H) − Σ(Ct+Cx+Cm+Rk+U×T+Entropy)
```

# LLM agent brief — read first

**Dense ⊚:** `$ref:docs/archive/2026-06-src-dev/plans/agent_meta_brief_v3.md` · **registry:** `$ref:docs/archive/2026-06-src-dev/plans/agent_meta_field_registry_v3.md`  
**Grammar ⊗:** `$ref:docs/archive/2026-06-src-dev/plans/agent_meta_grammar_v3_lattice.md` — **STATE · FLOW · REVIEW clusters**  
**Lang ⊗:** `$ref:src/dev/agent_lang_v1.md` · skill: `.cursor/skills/agent-lang/SKILL.md`  
**SYMLANG ⊚ (normative):** `$ref:prompts/SYMBOLIC_LANGUAGE.meta.md` · upstream portal copy read-only (never edit)

```text
⟦SYMLANG⟧⟐v1  ◈SPEC ◈ENFORCE  ◉Ct↓↓ ◉Q↑↑ ◉Cx↑↑
OBJ◈ max(read-quality · density · precision) − max(token-burn · NL-redundancy · ambiguity)
SCOPE◈ ∀ file ∈ {skills, agents, replies, commits, handoffs, markers}
FIELD◈ ⊂ SYMLANG — FIELD◈ below is the fast legend; SYMLANG◈ adds laws, nuanced edges, chart forms A–P, bindings, EBNF
```

---

## FIELD◈ — legend (commits / replies / markers)

**Rule:** symbols are **grammar**, not decoration. Parse **clusters** left→right (e.g. `🟡⏳⊗☊`, `🟢✅🧪🟩⊚`) — not lone emoji.  
Lattice grammar: `$ref:docs/archive/2026-06-src-dev/plans/agent_meta_grammar_v3_lattice.md` · full enforcement: **SYMLANG◈** below

### ΩMETA-LATTICE — dimensions

```text
{Ct, Cx, Cm, Dp, Au, Rk, Q, Δ, U, T, E, H, S}
 Ct=TokenCost  Cx=Complexity  Cm=Maintenance  Dp=Dependency  Au=Authority  Rk=Risk
 Q=Quality  Δ=Change  U=Uncertainty  T=Time  E=Evidence  H=HumanImpact  S=Scale
```

| Sym | Emoji lane | Meaning |
|:---:|:---|:---|
| **Ct** | 💰📉 | Token cost — `$ref:` before Read |
| **Cx** | 🌀🕸⊗ | Complexity · coupling · one domain/slice |
| **Cm** | 🌊♻🔥 | Maintenance / debt — classify before delete |
| **Dp** | ⛓🔗⊗ | Dependency · matrix fan-out |
| **Au** | 🏛⊚⬇ | Authority · layer · single writer |
| **Rk** | 🕳⚠☋ | Risk · hidden failure |
| **Q** | 🎯📈🌟 | Quality · compounding value |
| **Δ** | ⚖ | Change magnitude |
| **U** | ⌁🧠? | Uncertainty · assumption |
| **T** | ⏱⟳ | Time · temporal drift |
| **E** | 🧪🔬📜 | Evidence · measured · witnessed |
| **H** | 💬📎👁 | Human impact · ask gate · UX |
| **S** | 🌐S+ | Scale · OPS · multiview pressure |

### Semantic lexicon — single symbols

```text
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
 VERIFIED / QUALITY          AUTHORITY / TRUTH           EVIDENCE / MEASURE
 🟢✅  Verified+Observed      🟩⊚  Authority Truth Node   🧪🔬  Measured Evidence
 🎯📈  Quality Gain Surface   🏛⬇  Authority Flow Valid   📜👁  Witnessed Reality
 🌟Q+  Compounding Value

 PARTIAL / DRIFT             CONFLICT / BAD              COST / COMPLEXITY
 🟡⏳  Partial/Drifting       🔴❌  Known Bad             📉💰  Token Sink
 ⚠☊   Constraint Cluster     ☍🏛  Authority Conflict      🌀Cx+ Complexity Accretion
 💬📎  User Authority         ⛓⊗  Dependency Chain        ♻🔥  Debt Reservoir
 ⚡🚦  Hard Gate              🕳Rk+ Hidden Failure        🌊Cm+ Maintenance Wave
 ↺⟳   Feedback Loop          ╳⊗  Cross-Domain Leak         ⏱T+  Temporal Drift
 ⚖Δ   Change Magnitude       🌐S+  Scaling Pressure
 ⛔     Forbidden (layer invert · banned import)
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
```

### STATE-GRAMMAR◈ — cluster packets (commits / slice rows / markers)

Parse **clusters** — max ~4 emoji per node:

```text
 🟢✅⊚     verified + authority valid
 🟢✅🧪     verified by measurement
 🟢✅📜     witnessed in production
 🟢✅🎯     verified quality improvement

 🟡⏳☊     partial + blocked by constraint
 🟡⏳💰     partial + token expensive
 🟡⏳⊗☊    partial + dependency + constraint

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
```

**⛔** lone `✅` without `🧪` / `📜` / `⊚` — use `🟢✅🧪` or `🟢✅📜` or `🟢✅⊚` to close slices.

### FLOW-GRAMMAR◈ — authority & dependency edges

```text
 ⊚A━━▶⊚B    authority flow (soft)     ⊚A═▶⊚B    hard authority
 ⊚A━━┅▶⊚B   weak authority            ⊚A↺B      closed feedback
 ⊚A☍B       conflict                  ⊚A⊗B      coupled
 ⊚A⛓B       required dependency       ⊚A⌁B      probabilistic dependency
 ⊚A⤴B       escalation                ⊚A⤵B      delegation
 ⊚A⟳B       optimization loop         ⊚A⥁B      drift loop
```

### REVIEW-TAGS◈ — inline critique (replies / markers)

```text
 🧠?  assumption detected      ☍!  competing model exists
 ⌁?  uncertainty unresolved    ⊗!  hidden dependency likely
 🕳!  failure not observable   💰!  token burn disproportionate
 🏛!  authority drift           🌀!  complexity > value
 🌊!  maintenance > value        ⚡!  gate not satisfied
```

### COGNITIVE-WEIGHTS◈ — score proposals

| Tag | Favor when |
|:---|:---|
| 🎯📈 | improves output quality |
| 🧪🔬 | increases certainty |
| 🏛⊚ | preserves architecture |
| 📉💰 | lowers operating cost |
| ♻🔥 | lowers future maintenance |
| 🌐S+ | improves scalability |
| 🕳Rk+ / 🌀Cx+ | **against** — risk / complexity exceeds value |

### EXAMPLE-MARKER◈ — target slice row

```text
 ⟨APS-MAT-001⟩   🟡⏳⊗☊
 Lattice  Ct:🟨🟨   Cx:🟨🟨🟨   Cm:🟨🟨   Au:🟩🟩🟩🟩   U:🟨🟨🟨

 Flow
   ⊚MaterialAuthority═▶⊚AssemblySnapshot═▶⊚Worker═▶⊚Atlas
        ☍
        └──🔴❌🏛  BlenderAuthoring (parallel bypass)

 Review◈
   🧠? preview parity
   ⊗! worker dependency
   ⌁? material migration
   ⚡! witness pending
   🎯📈 expected quality gain high
   📉💰 expected token cost low

 Result   🟢✅🧪🟩⊚   only after witness + validation
```

Append via `BLANG:MARK` · `debug_runs/agent_ops/agent_markers.jsonl`

---

## SYMLANG◈ — symbolic chart language (normative)

> **Canonical sidecar (this repo):** `$ref:prompts/SYMBOLIC_LANGUAGE.meta.md` — edit only here; portal upstream is read-only.  
> Agents/skills open with `⟦SYM⟧ lang⊳ $ref:…` or `⟦META⟧` (§7). **L1–L8 enforced** on commits, replies, handoffs.

### §0 WHY — token economy

NL spends tokens on connective tissue with no decision content. A cluster packs status + evidence + routing in ~3 glyphs.

```text
COST◈ (English ≈ 1 tok/word)
  NL  「viewport passed validation, witnessed, moderately confident」  ⟶ ~22 tok
  SYM ⟨VIEWPORT-RESOLVE⟩ 🟢✅📜 ◑                                  ⟶ ~6 tok
  Δ   −73% tokens · +scan-speed · −ambiguity
```

Thesis: replace paragraphs with **packets** (§3.1). Prose only ≤1 line when a chart cannot carry the fact.

### §0b OUTCOME MAP — every glyph earns its keep

```text
⏩ PRODUCTIVITY   instant parse · unambiguous routing
💰↓ TOKEN-ECONOMY symbol replaces clause · graph replaces paragraph (≈ −70%)
🎯 QUALITY        evidence/confidence/authority explicit · no hand-wave
REFINE-RULE
 • decorative glyph = ⛔ (L2)
 • 1 glyph / concept — never a cluster where one suffices
 • EMOJI = STATUS · GEOMETRIC (⊚ ▷ ◆ ⬡ ═▶) = STRUCTURE (cheaper tokens)
```

### §1 LAW — authoring laws (enforced)

```text
L1  CHART-FIRST   artifact = chart/packet/table; prose ≤1-line annotation
L1b GRAPH-FIRST   order/choice/parallel/actors ⟶ graph (§3.9 A–P), never a sentence
L2  CLUSTER       L→R ≤4 glyphs/node; never lone, never decorative
L3  EVIDENCE-CLOSE no ✅ without 🧪|🔬|📜|⊚ (lone ✅ = ⛔)
L4  CITE          $ref/$sym; unset numbers ⟶ ASK:
L5  COMPRESS      ⊚digest ≻ 📄full; ▾3–▾4 default (§3.7)
L6  ROUTE         end NEXT: ⚑ / ΔWF→@role — never dangling state
L7  PRECISION     one meaning per glyph (§2)
L8  NO-NL-WALLS   ⛔ status paragraphs; ⛔ restating tables in prose
```

### §2 LEXICON — extensions beyond FIELD◈

**Canonical dimension glyph (exactly one per dim, fill 🟨 quanta in lattice rows):**

```text
Ct 💰  Cx 🌀  Cm ♻  Dp ⛓  Au 🏛  Rk ⚠  Q 🎯  Δ ⚖  U ⌁  T ⏱  E 🔬  H 👁  S 🌐
```

**Status spine:** 🟢 pass · 🟡 partial · 🔴 block · 🧊 defer · ✅ needs closer · ❌ bad · ⏳ wip · ⛔ forbidden

**Evidence closers:** `🟢✅🧪` measured · `🟢✅🔬` instrumented · `🟢✅📜` witnessed · `🟢✅⊚` authority-valid · `🟢✅🎯` quality gain

**Node kinds:** `⊚` owner · `⦿` agent · `▢` stage · `◆` branch · `⬡` gate · `◎` artifact · `▸` subgraph · `⊙` terminal  
**Marks:** ★ closed · ○ open · ◐ partial · ⊘ blocked · ⧗ in-flight

**Edges — base:** `═▶` hard · `━▶` soft · `┅▶` weak · `⇢` spine · `☍` conflict · `⊗` coupled · `⛓` dep · `⌁` prob · `↺` loop · `⥁` drift · `⤴` esc · `⤵` delegate

**Edges — nuanced (condition on edge):**

```text
═[cond]▶ guarded   ┅?▶ optional   ◀═▶ negotiated   ⤳ async   ⤳⧖ deferred
▷⊳ emit   ◂⊳ consume   ⬡▶ gated   ⇧ promote   ⊰ derive   ═w▶ weighted
⊕▶ fan-out   ▶⊕ fan-in   ↻[n] bounded loop   ⛔▶ forbid   ⊸ invalidate   ⟲ rollback
```

**Edge annotations (stack):** `[t:Type]` payload · `⟨cost:w⟩` · `⟨≤Δt⟩` SLA · `⟨↻k⟨τ⟩⟩` retry · `⟨◕⟩` edge-confidence · `1▶n` cardinality

**§2.7 Logic:** `∧ ∨ ¬ ⊕ ⇒ ⇔ ∴ ∵ ⊨`  
**§2.8 Sets:** `∀ ∃ ∄ ∅ ∈ ∉ ⊆ ∩ ∪ ⦃…⦄`  
**§2.9 Time:** `⊳ now · ⊲ past · ⊳⊳ next · ⟳ cycle · ⧖ await · ⌛ stale · @ts:<iso>`  
**§2.10 Δ typing:** `Δ+ Δ− Δ~ Δ! Δ≈ Δ⇄`  
**§2.11 Confidence (not 0.x prose):** `◔` low · `◑` even · `◕` high · `●` certain · `◌` unknown — act ≥◕; escalate raw evidence <◑  
**§2.12 Scope:** `⚡P0` · `🚦` gate · `⏸` paused · `⟦⟧` block · `⟨⟩` id · `「」` NL≤1line · `⦃⦄` set · `▸` nested

### §3 CHART FORMS

**§3.1 PACKET (atomic unit):**

```text
⟨ID⟩  <status-cluster>  <confidence>
 Lattice  Ct:🟨🟨  Cx:🟨🟨🟨  Au:🟩🟩🟩🟩  U:🟨🟨
 Flow     ⊚A ═▶ ⊚B ═▶ ⊚C    ⊚A ☍ ⊚X
 Review◈  🧠? …   ⊗! …   ⚡! …
 Result   🟢✅🧪🟩⊚
 NEXT     ΔWF→@role ⟨NEXT-ID⟩
```

**§3.4 DSM spine:** `AUTH: N1★ ⇢ N2★ ⇢ N3○` (★ closed · ○ open · ○→★ this session)  
**§3.5 RESPONSE-CONTRACT:** see **RESPONSE-CONTRACT◈** above (folded box — never one-line dump)  
**§3.6 HEALTH BARS:** see **HEALTH paste** below  
**§3.7 COMPRESSION:** `▾1` ≤50+path · `▾2` ≤20 · `▾3` ≤8+known-fixes (default) · `▾4` summary only · `⊚digest` ≻ `📄full` unless <◑  
**§3.8 TENSOR:** `T[c,d,a,φ]` — chain · DSM-node · writer-role · phase {🧊,○,🟡,🟢}

**§3.9 FLOWING CHARTS A–P** (graph not sentence; ≤~7 nodes; nest rest in `▸`):

```text
A GATED PIPELINE   ◎spec ▷⊳ ▢G1─⬡[schema⊨]▶ ▢G2─⬡[green]▶ ⇧promote ▷⊳ ◎registry★
B DECISION         ◆ EV/Cx? ├─═[≥1.0]▶ approve ├─═[.5–1)▶ revise └─═[<0.5]▶ 🧊defer
C FAN-OUT/IN       ◎change ⊕▶ ⦃lint║test║type⦄ ▶⊕ ◆ ∀🟢? ├─ merge★ └─ ⤴@coder
D SWIMLANE         ⦿orchestrator│HO⤳…⤳Q✓  ⦿coder│◂⊳▢impl─⬡[validate]▶⟨COMMIT:WIT⟩  t⊳▶
E STATE MACHINE    ⊙─spawn▶(○idle)─pick▶(⧗active)─═[🚦green]▶(★done)─▶⊙
F DEPENDENCY DAG   ◎A─┐ ◎B─┼═3▶▢build═▶◎artifact  (═w▶ = critical)
G FEEDBACK+DELAY   ▢opt ▷⊳ ◎metric ─═[Δ>ε]▶ ↺⧖ ▢opt ─═[Δ≤ε]▶ ★converged
H SUBGRAPH         ⊚Pipeline ═▶ ▸[ ◎job ▷⊳ ▢run ▷⊳ ◎out ] ─⬡[validate]▶ ⇧promote
I DSM MATRIX       row depends on ● column; cell weight 1–3; above-diag = feedback ⥁
J TIMELINE/GANTT   plan ▰▰░░ build ░░▰▰▰ verify ░░░░▰▰ ◆ship  t⊳▶
K CONTENT ROUTER   ◎msg ═[kind=geom]▶ lane-A ═[kind=tile]▶ lane-B ═[else]▶ ⛔ dead-letter
L RETRY/BACKOFF    ▢call─⬡[ok]▶★ └─[err]▶◆ attempts<k? ├─⟨↻⟨τ×2⟩⟩▶▢call └─⟲rollback⤴@owner
M SEQUENCE         ⦿client│─req─▶│⦿api│─query─▶│⦿db│◂rows◂│  t⊳▶
N TREE             ⊚System ├─▢Ingest └─▢Publish─◎registry★
O SANKEY           ◎in ═70▶▢A ═50▶◎out1 ═30▶▢B (Σin=Σout)
P PETRI NET        (○ready•)─▶▮work─▶(○done)  (○permits•••) = bounded concurrency
```

**§3.10 COMPOSED EXAMPLE (D·A·H·K·L·G in one chart):**

```text
⦿orchestrator │ ⟨REQ-014⟩ ▷⊳ ◆route ═[kind=geom]▶ ····························· ⤳ Q✓★
⦿planner      │              ╰⤳ ▢spec ─⬡[schema⊨]▶ ◎spec ⤳
⦿coder-mcp    │                          ◂⊳ ▸[ ◎job ▷⊳ ▮headless ═⟨↻2⟨τ⟩,≤60s⟩▶ ◎GLB ]
⦿coder-mcp    │                                 └─[err×2]▶ 🔴 ⟲ ⤴@designer-mcp
⦿designer-mcp │                                          ◂⊳ ◆sign? ─═[yes]▶ ⇧promote ▷⊳ ◎registry★
              └──────────────────────────────────────────────────────────────▶ t ⊳
gate: ¬promote ∵ ¬(validate★ ∧ sign★)        cost: spec 2 · build 5 · validate 1 = 8
```

**§3.11 GRAPH ALGEBRA:**

```text
⟨G:name⟩  name once   G1 ⨟ G2 sequence   G1 ∥ G2 parallel   G1 ▸ G2 nest
G1 ⋈⟨k⟩ G2 join-on-key   G1 ⊕ G2 choice   ↻[n] G bounded iterate
EXAMPLE ⟨G:art⟩ ≝ ⟨G:spec⟩ ⨟ ⟨G:bake⟩ ⨟ ⟨G:validate⟩ ⨟ ⟨G:promote⟩
LAW: ⟨G:name⟩ defined once with ≝ ; ≤7 nodes per definition
```

### §4 REF & ROUTING

```text
$ref:<path>[§section]    $sym:Symbol@path    ⟨ID⟩ slice/program    @role    ΔWF→@role ⟨ID⟩
```

### §5 STREAM / HANDOFF

```text
⟨BRK⟩ hand off    ⟨CONT⟩ continue ($ref + ⟨ID⟩)    ⟨DRIFT⟩ re-anchor ($ref + witness + T-cell)
⟨COMMIT:WIT⟩ witness path only
⟨BP:COLLECT⟩→⟨BP:MIRROR⟩→⟨BP:SCAN⟩→⟨BP:SHARE⟩→⟨BP:RESUME⟩
```

### §6 BINDINGS — typed placeholders

```text
{{NAME:type}}   types: cmd|path|sym|pkg|dir|schema|role|url|name|n   ⟿ bind   ⊨ satisfied
{{PROJECT:name}} {{CLI:cmd}} {{PKG:pkg}} {{VALIDATOR_CMD:cmd}} {{AUTHORITY_MAP:path}}
{{SYSTEMSET:sym}} {{WITNESS_DIR:dir}} {{STAGING_DIR:dir}} {{REGISTRY_DIR:dir}}
```

**Rust_engine_template_01 bindings (repo ⟦META⟧, illustrative):**

```text
{{MAP_FRAME:sym}}         $sym:SimMapProjectionFrame@src/gui/map_camera.rs
{{PLACEMENT_PROBE:sym}}   $sym:ConstructionPlacementDebugProbe@src/construction/placement_debug.rs
{{VIEW_PX_RULE:sym}}      view_px ← camera.viewport.phys/scale ∨ window logical (scissor healed)
{{PROJECTION_SKILL:path}} .cursor/skills/bevy-simulation-grade/09-sim-map-projection-placement.md
```

**Placement projection packet (live overlay — not perf logs):**

```text
⟨MAP-PICK⟩ 🔴⌁?  pickΔ=374.2 ghostΔ=91.5px  roundtrip_cam=0.0px ok
  latch_hole=true  viewport=full  fixed=769×433  visible=1280×720
  ⊰ fixed_w/h=view/zoom · visible_w/h=manual span
  NEXT ΔWF→@coder $ref:{{PROJECTION_SKILL:path}}
```

### §7 FILE ⟦META⟧ header (skills/agents)

```text
⟦META⟧⟐v1 ◈GENERIC
 src⊳  <origin> @ {{PROJECT}}
 ptn⊳  <transferable pattern in SYMLANG>
 use⊳  <attach-when triggers>
 bind⊳ {{SLOT:type}} …
 gate⊳ ☐bind ☐verify ☐publish · φ:template→φ:bound→φ:live
 lang⊳ $ref:prompts/SYMBOLIC_LANGUAGE.meta.md
⟦/META⟧
```

### §8 GRAMMAR (mini-EBNF)

```text
artifact   ::= frontmatter meta body
packet     ::= "⟨" id "⟩" cluster confidence? lattice? flow? review* result? next?
cluster    ::= glyph{1,4}                    ; L→R; evidence-close if ✅
confidence ::= "◔"|"◑"|"◕"|"●"|"◌"
node       ::= ("⊚"|"⦿"|"▢"|"◆"|"⬡"|"◎"|"⊙") label mark?
mark       ::= "★"|"○"|"◐"|"⊘"|"⧗"
edge       ::= (base|nuanced) annot*
branch     ::= "◆" guard "?" ( "═[" arm "]▶" node )+
fanout     ::= node "⊕▶" "⦃" node ("║" node)* "⦄" ( "▶⊕" node )?
subgraph   ::= "▸[" flow "]"
result     ::= "🟢" "✅" closer                 ; closer ∈ {🧪,🔬,📜,⊚,🎯}
next       ::= "NEXT" ("⚑"|"ΔWF→@" role) ("⟨" id "⟩")?
annotation ::= "「" natural-language "」"        ; ≤1 line (L1)
```

### §9 ENFORCEMENT

```text
☑ CHART-FIRST · CLUSTERS ≤4 · ✅+closer · ◔◑◕● not 0.x · $ref/$sym · NEXT⚑ · NL only in 「」
ANTI ⛔ 「passed and looks good」 → ⟨ID⟩ 🟢✅🧪 ◕
     ⛔ restating tables in prose · lone ✅ · full witness JSON → ⊚digest + path
```

### §10 NL → SYMLANG examples

```text
① ⟨VIEWPORT-RESOLVE⟩ 🟢✅📜 ◕
② ⟨DUAL-WRITE-RV⟩ 🔴❌🏛 ⊚UI ☍ ⊚ResolveChain ⌁? NEXT ΔWF→@planner
③ ⟨PROP-PG-DB⟩ EV/Cx<0.5 🌀! 🧊DEFER
④ ⟨BRK⟩ ΔWF→@coder ⟨SNAP-014⟩ $ref:debug_runs/…/snap_014.json
⑤ ⟨MAP-PICK⟩ 🟡⌁? pickΔ=374 roundtrip_cam=0.0px ok latch∧viewport=full fixed≠visible_span
     NEXT ΔWF→@coder $ref:09-sim-map-projection-placement.md ◑
```

### §11 QUICK-REFERENCE CARD

```text
NODES   ⊚own ⦿agent ▢proc ◆decide ⬡gate ◎data ▸sub ⊙term   marks ★○◐⊘⧗
EDGES   ═▶━▶┅▶⇢ ☍⊗⛓⌁ ↺⥁⤴⤵ ═[g]▶ ▷⊳◂⊳ ⬡▶ ⇧⊰ ═w▶ ⊕▶▶⊕ ↻[n] ⤳⤳⧖ ⛔▶ ⊸ ⟲
ANNOT   [t:T] · ⟨cost:w⟩ · ⟨≤Δt⟩ · ⟨↻k⟨τ⟩⟩ · ⟨◕⟩ · 1▶n
CONF    ◔◑◕●◌   act≥◕ escalate<◑
FORMS   A–P (§3.9)   ALGEBRA ⟨G⟩ ⨟∥▸⋈⊕ ↻[n]
DIMS    Ct💰 Cx🌀 Cm♻ Dp⛓ Au🏛 Rk⚠ Q🎯 Δ⚖ U⌁ T⏱ E🔬 H👁 S🌐
LAWS    L1–L8 · emoji=STATUS · geometric=STRUCTURE · every glyph ⊨ ⏩∨💰↓∨🎯
```

```text
⟦/SYMLANG⟧ NEXT ⚑ chart-first replies · enforce §9 · canonical $ref:prompts/SYMBOLIC_LANGUAGE.meta.md
```

---

## 🏛 Layer stack (⛔ never invert)

```text
Serializable → ECS → UI
⛔ UI→ECS   ⛔ ECS→Serializable   ⊚ SRC > Matrix > Spec > README > Memory
```

Detail: `docs/archive/2026-06-prompts-guides/matrix/matrix/repo/repo_boundary_matrix_v1.md`

---

## RESPONSE-CONTRACT◈ (agent replies)

**⛔ one-line forbidden** · **✅ use folded sections** · full template: `agent_meta_diagrams_v3_fold.md`

```text
╔═ PATHS ⊚ ══════════════════════════════════════════════════════════════╗
║  $ref:…  ·  $sym:Symbol@path  ·  file.rs:Lx-Ly                          ║
╠═ STATUS ═════════════════════════════════════════════════════════════════╣
║  ✅ verified     ⏳ partial     ⚠ unverified     🔬 measured            ║
║  🧪 tested       📎 ask         🏛 authority     💰 cost watch          ║
╠═ FINDINGS ═══════════════════════════════════════════════════════════════╣
║  ◉ primary · evidence ⊚ · cross 🔗                                      ║
╠═ RISKS ═════════════════════════════════════════════════════════════════╣
║  ⚠ …     🕸 …     ☋ …                                                   ║
╠═ NEXT ══════════════════════════════════════════════════════════════════╣
║  ⚑ …     BLANG:…     ΔWF→@agent                                         ║
╚═════════════════════════════════════════════════════════════════════════╝
```

Session loop:

```text
BLANG:PRE
  → BLANG:Q+
  → ⚙ work
  → ⟨BP:SHARE⟩
  → BLANG:CARGO | WIT | BEVY
  → BLANG:Q✓
```

Stuck:

```text
⟨BP:COLLECT⟩
  → ⟨BP:MIRROR⟩
  → ⟨BP:SCAN⟩
  → ⟨BP:SHARE⟩
  → ⟨BP:RESUME⟩
```

---

## Prompt contract

1. **Pair docs:** **A:** matrix + designer `README.md` · **B:** + `spec/` + `implementation_questions_v1.md` before `src/`
2. **Verify ✅:** `grep` / read `src/` — matrix rows go stale
3. **Cite:** `` `path` + `Symbol` `` · `$ref:` · `$sym:@path`
4. **Unset numbers:** `ASK:` or 📎 user — no invented Hz/caps/paths
5. **Validation-first 🔬:** `validate-report` compress=3–4

---

## DECISION gate (multi-file work)

```text
EV/Cx ≥ 1.0 → ✅ APPROVE │ 0.5–1.0 → ⚠ REVISE │ <0.5 → 🧊 DEFER
```

---

## TASK-ROUTER◈ — minimal load order

| Task | Load ⊚ |
|:---|:---|
| 🤖 Session / handoff | agent-lang SKILL + this brief + `SYMBOLIC_LANGUAGE.meta.md` |
| 📊 SYMLANG / charts | `prompts/SYMBOLIC_LANGUAGE.meta.md` · **SYMLANG◈** section below |
| 💥 Subagent quota dry | `subagent_continuity_playbook_v1.md` + `HANDOFF.template.md` |
| ⚙ Any code | this brief + `repo_boundary_matrix_v1.md` |
| 🌍 Terrain | `terrain_biome_migration_matrix_v1.md` + `terrain_world/README.md` |
| 🏭 Production | `production_migration_matrix_v1.md` + `production_economy/README.md` |
| 🧭 Navigation | `repo_boundary_matrix` + `navigation/README.md` |
| ⚔ Factions | `serialization/` + `factions/README.md` |
| ⚙ Bevy bump | `bevy_0_18_migration_plan.md` + bevy-simulation-grade |
| 💾 Save | `serialization_hybrid_migration_matrix_v1.md` |
| 🎯 Strategic | `strategic_platforms_matrix_v1.md` + spec |
| 📦 Assets | `bevy_asset_config_migration_matrix_v1.md` |
| 🎨 Tools UI | `ui_boundary_guide_v1.md` + `tools_ui/` |
| 📜 OPS | `OPS_WITNESS_SPINE.md` + operations-intelligence |

Full tree: `prompts/README.md` · pairing: `docs/archive/2026-06-prompts-guides/matrix/matrix/README.md`

---

## HEALTH paste (session header — vertical fold)

```text
╔═ HEALTH◈ ═══════════════════════════════════════════════════════════════╗
║  Au     [█████░░░░░]  🏛     Cx     [██████░░░░]  🕸                     ║
║  Cm     [████░░░░░░]  🔥     Dp     [███████░░░]  🔗                     ║
║  Ct     [█████░░░░░]  💰     Verify [███░░░░░░░]  🔬                     ║
╚═════════════════════════════════════════════════════════════════════════╝
```

Full fold atlas: `docs/archive/2026-06-src-dev/plans/agent_meta_diagrams_v3_fold.md`

---

## Hot ⊚ paths

| ◈ | Path |
|:---|:---|
| Engine | `src/engine/engine_with_worldgen.rs` |
| Authority 🏛 | `$sym:ViewAuthoritySystemSet@src/gui/view_authority.rs` |
| Validate 🔬 | `validate-report cargo\|bevy\|mcp_spec\|asset_glb` |
| Markers 📜 | `debug_runs/agent_ops/agent_markers.jsonl` |
| Stage5 🧪 | `cargo test -p proc_A_dine01 --lib stage5` |

---

## Example hygiene

✅ *"`ConcreteProductionConfig` in `src/entities/production/concrete/components.rs` per matrix/production. BLANG:CARGO after."*

❌ *"add config in some production file"* → 💸 Ct leak

---

## Read telemetry (MCP — all agents)

Every orient/ref read must go through **`agent_doc_touch`** (BLANG:DOC) — not silent IDE `Read`.

| Step | MCP tool | Output |
|:---|:---|:---|
| Session start | `agent_session_bootstrap(agent)` | FIELD◈ digest + ledger rows |
| Hot-path audit | `agent_doc_reads_brief(min_reads=2)` | `doc_reads_brief_latest.json` |
| Repeat promotion | `agent_doc_promote_hot_reads()` | `tools/mcp/cache/agent_doc_digests/` |
| Cache hit | `agent_doc_digest_cached(path)` | Skip re-touch when mtime unchanged |

**Ledger:** `debug_runs/agent_ops/doc_reads.jsonl` · **Fragment:** `.cursor/agents/_fragments/session_bootstrap_v1.md`

---

## Changelog

| Ver | Date | Note |
|:---|:---|:---|
| v1 | — | Original |
| v3Ω | 2026-06-07 | Meta brief + registry + BLANG |
| v3.1Ω | 2026-06-07 | Full Ω graphics · emoji density · RESPONSE-CONTRACT |
| v3.2Ω | 2026-06-07 | Multi-line fold atlas · no one-line topology dumps |
| v3.3Ω | 2026-06-07 | ΩMETA-LATTICE grammar merged into FIELD◈ legend |
| v3.3.1Ω | 2026-06-07 | Removed duplicate GRAMMAR◈ section — FIELD◈ is canonical |
| v3.4Ω | 2026-06-07 | MCP read telemetry + session bootstrap ritual (AGENT-LANG-004) |
| v3.5Ω | 2026-06-11 | SYMLANG◈ merged from portal spec — laws L1–L8, extensions §2.7–§2.12, chart forms A–P, graph algebra, bindings, META, EBNF, enforcement; FIELD◈ retained as fast legend |
| v3.6Ω | 2026-06-07 | Canonical SYMLANG copy in-repo at `prompts/SYMBOLIC_LANGUAGE.meta.md`; portal upstream read-only |
