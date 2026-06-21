```text
⟦SYMLANG⟧⟐v1  ◈SPEC ◈ENFORCE  ◉Ct↓↓ ◉Q↑↑ ◉Cx↑↑
OBJ◈ max(read-quality · density · precision) − max(token-burn · NL-redundancy · ambiguity)
SCOPE◈ ∀ file ∈ {skills, agents, replies, commits, handoffs, markers}
```

# SYMLANG — the symbolic chart language (sidecar enforcement spec)

> **Sidecar to the whole generic skill/agent set.** Every artifact in this folder is
> authored in SYMLANG and carries a `⟦META⟧` header (§7) that links back here. SYMLANG
> exists to move agent communication **away from imprecise, token-wasteful natural
> language toward a dense, scannable symbolic chart system** — without losing precision.
>
> Derived from a multi-agent engine's field grammar (the `agent-lang` lineage, FIELD◈),
> then **extended** here — new operator classes §2.7–§2.13, typed bindings §6, a formal
> grammar §8.

---

## §0 WHY — the token economy

Natural language spends tokens on grammar, hedging, and redundancy that carry no
decision content. A symbolic cluster packs *status + evidence + routing* into ~3 glyphs.
Charts are scanned in O(1); prose is read in O(n).

```text
COST◈ (illustrative, English ≈ 1 tok/word)
  NL  「The viewport resolve system passed validation and was witnessed in       ⟶ ~22 tok
       production, but I'm only moderately confident.」
  SYM  ⟨VIEWPORT-RESOLVE⟩ 🟢✅📜 ◑                                               ⟶ ~6 tok
  Δ   −73% tokens · +scan-speed · −ambiguity (evidence + confidence made explicit)
```

**Thesis:** every token spent on NL connective tissue is read-input burn. Replace
sentences with **packets** (§3.1). Reserve prose for ≤1-line annotations a chart cannot
carry.

---

## §0b OUTCOME MAP — every glyph earns its keep

A glyph exists **only** if it drives one of three outcomes. No outcome ⟶ cut it (L2).

```text
⏩ PRODUCTIVITY   instant parse · no prose to decode · unambiguous routing
💰↓ TOKEN-ECONOMY one symbol replaces a clause · graph replaces a paragraph (≈ −70%)
🎯 QUALITY        forces evidence/confidence/authority to be explicit · no hand-wave
🔍 FIDELITY-GATE  must DECODE COLD (fresh reader · no legend · ≥95%) — the axis the others can't fake ($REPORT §9)
```

| Glyph class | Drives | Why it earns the slot |
|:--|:--|:--|
| dense status ●◐○✗ (emoji sparse) · confidence ~.5/.75/1.0 | 🎯 + ⏩ | state at a glance; ½ the tokens of emoji ($REPORT §8) |
| evidence closers 🧪🔬📜⊚ | 🎯 | a bare ✅ is banned — proof is mandatory |
| dimensions (§2.1, **1 glyph each**) | 💰↓ + 🎯 | one symbol = a sentence of caveats |
| edges + annotations (§2.4) | ⏩ + 🎯 | topology **+** condition **+** state in one scan |
| chart forms (§3) | 💰↓ + ⏩ | a graph replaces a paragraph |
| scope/stream (§2.12, §5) | ⏩ | handoff/continuation without restating context |

```text
REFINE-RULE
 • decorative glyph (serves no outcome) = ⛔ — delete it (this is L2, enforced)
 • 1 glyph / concept — never a cluster where one suffices
 • COST: an emoji is 1–3 input tokens; a geometric operator (⊚ ▷ ◆ ⬡ ═▶) is usually cheaper
   → reserve EMOJI for SPARSE headline status only; use the cheap GEOMETRIC status set ●◐○✗
     for DENSE vectors (§2.2) and geometric operators for STRUCTURE. ($REPORT §8,§15)
 • BLEND ≻ pure: pure-symbolic is the WORST encoding ($REPORT §5) — symbolise status/rules/relations/
   recurring concepts only; leave narrative as prose. Single-glyph token wins are tokenizer-fragile
   ($REPORT §30); STRUCTURE + AMORTIZATION wins are invariant — trust those.
```

---

## §1 LAW — authoring laws (enforced)

```text
L1  CHART-FIRST   every artifact = chart/packet/table; prose only as ≤1-line annotation
L1b GRAPH-FIRST   order/choice/parallel/actors ⟶ a flowing graph (§3.9 A–H), never a sentence
L2  CLUSTER       parse glyphs in clusters L→R (≤4 per node); never lone, never decorative
L3  EVIDENCE-CLOSE no ✅ without an evidence closer 🧪|🔬|📜|⊚  (lone ✅ = ⛔)
L4  CITE          claims carry $ref/$sym (§4); unset numbers ⟶ ASK:, never invented
L5  COMPRESS      prefer ⊚digest over 📄full; emit at ▾3–▾4 unless asked (§3.7)
L6  ROUTE         end with NEXT: ⚑ / ΔWF→@role — never a dangling state
L7  PRECISION     a glyph means exactly one thing (§2); ambiguity ⟶ pick a sharper glyph
L8  NO-NL-WALLS   ⛔ paragraphs of status; ⛔ restating a table in prose
L9  COLD-DECODE   ship no notation w/o (a) cold-decode ≥95% (fresh reader, no legend) ∧ (b) token before/after vs prose ($REPORT §9). A token win that won't decode = ⛔ not a win.
L10 BLEND-DEFAULT default = the blend ($REPORT §5); pure-symbolic loses. Symbolise dense-status/rules/relations/recurring-concepts; narrative stays prose.
```

---

## §2 LEXICON

### §2.1 Dimensions — the meta-lattice `{13}`

```text
{ Ct  Cx  Cm  Dp  Au  Rk  Q  Δ  U  T  E  H  S }
  Ct=TokenCost  Cx=Complexity  Cm=Maintenance  Dp=Dependency  Au=Authority
  Rk=Risk  Q=Quality  Δ=Change  U=Uncertainty  T=Time  E=Evidence  H=HumanImpact  S=Scale
```
**Canonical glyph — exactly one per dimension** (clusters pruned for token economy, §0b):

```text
Ct 💰  Cx 🌀  Cm ♻  Dp ⛓  Au 🏛  Rk ⚠  Q 🎯  Δ ⚖  U ⌁  T ⏱  E 🔬  H 👁  S 🌐
```
Fill level uses 🟨 quanta (§3.2), e.g. `Au:🏛🟨🟨🟨🟨` = authority dimension, 4 quanta filled.

### §2.2 Status — the spine

**Dense status — the cheap geometric set** (½ the tokens of emoji, equally glance-legible — $REPORT §8,§16):

| Glyph | Means | Glyph | Means |
|:--|:--|:--|:--|
| ● | pass / done | ✗ | fail |
| ◐ | running / partial | ○ | skip / open |
| ⊘ | closed / blocked | ✎ | draft |

**Sparse headline status** — reserve emoji for a *single* attention mark, never a dense vector (emoji cost 2–3 tok each, $REPORT §15): 🟢 green · 🟡 qualified · 🔴 blocked · 🧊 defer · ✅ + closer (§2.3, never lone) · ❌ known-bad · ⛔ forbidden · ⏳ wip

### §2.3 Evidence closers (attach to ✅)

```text
🟢✅🧪 verified by measurement   🟢✅🔬 measured/instrumented
🟢✅📜 witnessed in production    🟢✅⊚ authority-valid (single-writer truth)
🟢✅🎯 verified quality gain
```

### §2.4 Flow / edge grammar (authority · dependency · data)

**Node kinds** (a graph is typed nodes + typed edges, not arrows-as-decoration):

```text
⊚ authority/owner   ⦿ agent/role   ▢ process/stage   ◆ decision/branch
⬡ gate              ◎ data/artifact ▸ subgraph (collapsed)   ⊙ start/end terminal
node marks:  ★ closed/green · ○ open · ◐ partial · ⊘ blocked · ⧗ in-flight
```

**Edges — base set:**

```text
⊚A ═▶ ⊚B   hard authority flow     ⊚A ━▶ ⊚B   soft authority
⊚A ┅▶ ⊚B   weak / advisory         ⊚A ⇢ ⊚B    spine flow (DSM)
⊚A ☍ ⊚B    conflict                ⊚A ⊗ ⊚B    coupled
⊚A ⛓ ⊚B    required dependency      ⊚A ⌁ ⊚B    probabilistic dependency
⊚A ↺ B     closed feedback loop     ⊚A ⥁ B     drift loop
⊚A ⤴ B     escalate                 ⊚A ⤵ B     delegate
```

**Edges — nuanced set (SYMLANG ext — carry the *condition/semantics* on the edge):**

```text
A ═[cond]▶ B   guarded (fires iff cond)     A ┅?▶ B      optional / conditional
A ◀═▶ B        bidirectional / negotiated    A ⤳ B        async handoff
A ⤳⧖ B         deferred handoff (awaits)      A ⛔▶ B       forbidden edge (must-not)
A ▷⊳ B         produces / emits → B           A ◂⊳ B       consumes ← A
A ⬡▶ B         passes a gate to reach B        A ⇧ B        promotes / elevates
A ⊰ B          derives-from B (B is source)    A ═w▶ B      weighted/priority edge (w∈1..n)
A ⊕▶ ⦃B,C,D⦄    fan-out (split)               ⦃A,B,C⦄ ▶⊕ D  fan-in (join / barrier)
A ↻[n] B       bounded loop (≤n iterations)    A ⊸ B        breaks / invalidates B
```

**Reading rule:** an edge's glyph = *kind of dependency*; a `[guard]` on it = *when it
fires*; the node mark (★/○/◐/⊘) = *current state*. A graph thus states topology + condition
+ status in one scan. `★ closed · ○ open · ○→★ closed-this-session`.

**Edge annotations — connections as contracts (stack any of these on an edge):**

```text
A ▷⊳[t:Type] B       payload — A emits an artifact of type t to B
A ─1▶n─ B            cardinality — one A fans to many B   (also  n▶1 join · n▶m mesh)
A ═⟨cost:w⟩▶ B        cost/weight on the edge (tokens · time · risk units)
A ═⟨≤Δt⟩▶ B           latency / SLA bound; breach ⟶ ⌛
A ═⟨↻k⟨τ⟩⟩▶ B         retry ≤k with backoff τ on failure of the hop
A ⤳⧖⟨≤Δt⟩ B           deferred handoff that must resolve within Δt
A ⟲ B                compensation / rollback edge (undo B's effect)
A ═⟨◕⟩▶ B             confidence ON the link itself (this connection is ◕-trusted)
A ═[content]▶ ⦃…⦄     content-based routing — the guard selects the arm (see §3.9-K)
stack freely →  A ▷⊳[t:GLB] ═⟨cost:3, ≤30s, ↻2⟩▶ B
```

### §2.5 Review tags (inline critique)

```text
🧠? assumption detected     ☍! competing model exists     ⊗! hidden dependency likely
⌁? uncertainty unresolved   🕳! failure not observable     💰! token burn disproportionate
🏛! authority drift          🌀! complexity > value          ⚡! gate not satisfied
🌊! maintenance > value
```

### §2.6 Cognitive weights (score a proposal)

```text
favor → 🎯📈 quality · 🧪🔬 certainty · 🏛⊚ preserves-arch · 📉💰 lowers-cost
        ♻🔥 lowers-maintenance · 🌐 scales
against → 🕳Rk+ risk>value · 🌀Cx+ complexity>value
gate → EV/Cx ≥ 1.0 ✅APPROVE · 0.5–1.0 ⚠REVISE · <0.5 🧊DEFER
```

> §2.7–§2.13 are SYMLANG **extensions** beyond the source FIELD◈ — they add formal
> precision (logic, sets, time, change-typing, graded confidence) so a packet can state
> *exactly* what it means without prose.

### §2.7 Logic connectives

```text
∧ and   ∨ or   ¬ not   ⊕ xor   ⇒ implies   ⇔ iff   ∴ therefore   ∵ because   ⊨ entails/satisfies
```

### §2.8 Quantifiers & sets

```text
∀ for-all   ∃ exists   ∄ none-exist   ∅ empty   ∈ member   ∉ not-member
⊆ subset   ∩ intersect   ∪ union   ⦃ … ⦄ set literal
```

### §2.9 Temporal operators

```text
⊳ now   ⊲ past   ⊳⊳ future/next   ⟳ recurring/per-cycle   ⧖ pending/awaiting
⌛ expired/stale   ⏱T+ temporal drift   @ts:<iso> timestamp anchor
```

### §2.10 Change typing (Δ)

```text
Δ+ add   Δ− remove   Δ~ modify   Δ! breaking   Δ≈ refactor (behavior-equiv)   Δ⇄ move/rename
⚖Δ change-magnitude (pair with bars, §3.6)
```

### §2.11 Graded confidence (replaces "confidence: 0.x")

```text
◔ ≈.25 low    ◑ ≈.50 even    ◕ ≈.75 high    ● =1.0 certain    ◌ unknown/unmeasured
gate: escalate raw evidence when < ◑ ; act on known-fix when ≥ ◕
COST: the ◔◑◕● set ≈ 7 tok ($REPORT) — for INLINE confidence prefer ~.5 / ~.75 / 1.0 (cheaper, clearer);
reserve ◔◑◕● for a dense confidence vector where the glance-bar earns it.
```

### §2.12 Priority / gate / scope delimiters

```text
⚡P0 do-first   🚦 gate   ⏸ paused (non-blocking)   ⛔ forbidden   ⚠ caution
⟦ … ⟧ block/packet   ⟨ … ⟩ id/slice   「 … 」literal/quoted-NL   ⦃ … ⦄ set   ▸ child/nested
```

### §2.13 VEG/LAND landscape glyphs (topology grammar extension)

Registered for landscape program plans, solutions, and witness charts. **Complete tables:**
$ref:prompts/guides/landscape_grammar_lexicon_v1.md (§1.0–§1.19 planning · §2 extract · §3 mapping · §1.11 semantic · §1.17 composite).

**Law:** `VegetationTopology ≠ VegetationShape` — glyphs denote **intersecting topologies + field overlays**, not biome labels.

**⚠ GATE — domain glyphs lose by default ($REPORT §11 Rec5):** a terse English term usually beats a bespoke glyph (a cold reader resolves a domain glyph only via expensive search). Register/keep a §2.13 glyph ONLY when it (a) recurs ≥3× in the artifact, (b) cold-decodes ≥95% OR carries a 1-line in-context legend, AND (c) beats the English term on payload+fidelity (L9). Otherwise write the term. This set is opt-in, scoped under a `⟨VEG⟩` block header.

**Planning / chart set — canopy & structure:**

```text
█ mature_canopy   ▓ secondary/mid   ▒ shrub   ░ grass   ● old_growth   ○ gap   ◇ regrowth   □ cleared
Core/Mid/Edge density zones — never uniform █ blocks
Patch internals: Core · Edge · Gap · Regrowth · Deadfall (metadata tag)
```

**Planning / chart set — topology & flow:**

```text
◊ node (◊A…◊I weighted — lexicon §1.6)   ═ heavy_transport   ─ observation/weak_boundary
≈ hydrology   ╬ convergence   ▲ elevation_source   ▼ flow_sink   │ vertical_flow   ║ network_arm
⌂ human_management   ⚶ wind   ~ wind_wave_front (planning ONLY)   ☍ barrier
```

**Planning / chart set — pressure & species:**

```text
⊕ expansion   ⊖ suppression   ⊗ disturbance   ⊙ attractor   ⇡ advance   ⇣ retreat
Intensity: repeat glyph (▲▲▲ ⚶⚶⚶ ⊕⊕⊕ ⊖⊖⊖ ⊗⊗⊗)
Directional suffix: Wind → · enemy ▼ · upstream ▲ · fire vector ▲
```

**Planning / chart framing & graphs:**

```text
╔╗╚╝ frame   ╦╩ T-junction   ╱╲ diagonal_edge   ◉ chart_header_prefix
Chart suffix families: Ω network · Σ complex · Δ drainage/ag · Λ delta · Ψ succession · Ξ hierarchy
```

**Succession ladder (NESTED-SUCCESSION-Ψ18 only):** ○ gap → ▒ regrowth → ▓ shrub → ▲ sapling → █ canopy
(▲ = sapling **only** inside this ladder — elsewhere ▲ = elevation source)

**Overlapping fields (8–20 layers):** Canopy · Age · Hydrology · Wind · Disturbance · Species · Visibility · Fire · HumanPressure

**Extract / encoder set (MCP + tile fields ONLY — never in SYMLANG flow graphs):**

```text
@ old_growth · # mature · % mid_story · * regrowth · . shrub · , grass
~ water/wet · ^ ridge · v gully · = transport · + node · x disturbance · : sparse
[ built · ] field/clearance · <> convergence/divergence · | edge · () ring · {} nested
```

**Disambiguation:** planning `~` = wind domain · extract `~` = water — never mix layers (lexicon §1.0).

**Semantic operators (full depth — lexicon §1.11):**

```text
⚶ wind domain/front     ☍ ecological barrier (⟨VEG⟩ scope — not SYMLANG routing conflict)
⊙ habitat attractor     ⊕ expansion pressure (field) — NOT op:⊕ seed · NOT §2.7 xor
⊖ suppression field     ⊗ disturbance event       ⌂ human management anchor
◇ regeneration nucleus
```

**Composition & interactions:** lexicon §1.12 recipes · §1.13 field matrix · §1.16 master index.

**Composite grammar (build complex symbols):** lexicon §1.17 connection edges · §1.17.3 stack notation · §1.17.4 macros · §1.18 patterns · §1.19 extension protocol.

**Operator vs field layers:** lexicon §9 — tag `field:` vs `op:` when ⊕ ⊖ ⌂ ◊ □ ≈ share glyphs.

**Chart IDs (15 — full art in $ref:prompts/guides/olant_grammer.md):**

```text
VEG-NETWORK-Ω7 · OLD-GROWTH-COMPLEX-Σ4 · AGRI-LANDSCAPE-Δ9 · VEGETATION-HIERARCHY-Ξ12
DELTA-FOREST-Λ5 · DEFENSIVE-VEGETATION-Ω13 · RIPARIAN-REGENERATION-Ψ8
ECOLOGICAL-PRESSURE-Ω27 · NESTED-SUCCESSION-Ψ18 · MOUNTAIN-FOREST-DRAINAGE-Δ44
FIRE-CORRIDOR-Ω51 · URBAN-FOREST-FRACTURE-Σ63 · ECOLOGICAL-NETWORK-Ω91
FOREST-WARFARE-Ω113 · MEGA-BIOSPHERE-Ω200
```

**Scale bands:** S micro (1–4) · M meso (5–16) · L macro (17–64) · XL mega (65+).

**Context tags:** `#` biome · `@` moisture · `^` elevation · `[` land_use `]` · `!` disturbance · `=` infrastructure.

**SYMLANG collisions (scope by block header `⟨VEG⟩`):** `●` old-growth vs §2.11 certainty · `⊕` field vs §2.7 xor · `☍` barrier vs §2.4 conflict · `⊗` disturbance vs review tag `⊗!`

**Do not:** mix §2.13 planning glyphs into extract layers; use lexicon §3 mapping when translating.

---

## §3 CHART FORMS

### §3.1 PACKET (the atomic unit — replaces a paragraph)

```text
⟨ID⟩  <status-cluster>  <confidence>
 Lattice  Ct:🟨🟨  Cx:🟨🟨🟨  Au:🟩🟩🟩🟩  U:🟨🟨
 Flow     ⊚A ═▶ ⊚B ═▶ ⊚C    ⊚A ☍ ⊚X
 Review◈  🧠? …   ⊗! …   ⚡! …   🎯📈 …
 Result   🟢✅🧪🟩⊚   (only after evidence + gate)
 NEXT     ΔWF→@role ⟨NEXT-ID⟩
```

### §3.2 LATTICE ROW (dimension fill — 🟨 per filled quantum)

```text
⟨MAT-001⟩ 🟡⏳⊗☊  Ct:🟨🟨  Cx:🟨🟨🟨  Au:🟩🟩🟩🟩  U:🟨🟨🟨
```

### §3.3 FLOW GRAPH (authority spine with a conflict)

```text
⊚MaterialAuthority ═▶ ⊚Snapshot ═▶ ⊚Worker ═▶ ⊚Atlas
        ☍
        └─ 🔴❌🏛 BypassAuthoring (parallel write — forbidden)
```

### §3.4 DSM SPINE (closure state)

```text
AUTH: N1★ ⇢ N2★ ⇢ N3★ ⇢ N4○ ⇢ N5○      (★ closed · ○ open · ○→★ closed-this-session)
```

### §3.5 RESPONSE-CONTRACT (folded reply box — never a one-liner dump)

```text
╔═ PATHS ⊚ ═════════════════════════════╗
║ $ref:… · $sym:Symbol@path · file:Lx-Ly ║
╠═ STATUS ═══════════════════════════════╣
║ 🟢✅ ⏳ ⚠ 🔬 🧪 📎 🏛 💰 ◕             ║
╠═ FINDINGS ═════════════════════════════╣
║ ◉ primary ├─ ⊚ evidence └─ 🔗 cross     ║
╠═ RISKS ════════════════════════════════╣
║ ⚠ … 🕳 … ☋ …                            ║
╠═ NEXT ═════════════════════════════════╣
║ ⚑ … ΔWF→@role ⟨ID⟩                      ║
╚════════════════════════════════════════╝
```

### §3.6 HEALTH BARS (dimension gauges)

```text
Au [█████░░░░░] 🏛   Cx [██████░░░░] 🕸   Cm [████░░░░░░] 🔥
Dp [███████░░░] 🔗   Ct [█████░░░░░] 💰   Verify [███░░░░░░░] 🔬
```

### §3.7 COMPRESSION markers

```text
▾1 ≤50 issues +raw-path   ▾2 ≤20   ▾3 ≤8 +known-fixes (default)   ▾4 summary +known-fixes only
⊚digest (peek)   📄full (escalate only when < ◑ or empty-errors∧failed)
```

### §3.8 TENSOR projection (orchestration overlay)

```text
T[c,d,a,φ]   c=chain(A…J)  d=DSM-node  a=writer-role  φ∈{−1 🧊,0 ○,1 🟡,2 🟢}
```

### §3.9 FLOWING CHARTS & GRAPHS

A process with branches, parallelism, gates, or actors is a **graph**, not a sentence.
Draw it. Each form below packs topology + condition + state into one scannable chart
(node kinds + edges per §2.4). Prefer the richest form the situation actually has — don't
flatten a branching pipeline into a linear arrow.

**▸ A. GATED PIPELINE** — stages, gates, owners, fail-routes on one chart:

```text
◎spec ▷⊳ ▢G1·validate ─⬡[schema⊨]▶ ▢G2·build ─⬡[green]▶ ▢G3·stage ─⬡[sign]▶ ⇧promote ▷⊳ ◎registry★
            │                          │                    │
            └─🔴[schema✗]⤴@author       └─🔴⤴@author          └─💬[sign?]⤴@reviewer
   owners:  ⦿validator⊨G1   ⦿builder⊨G2   ⦿reviewer⊨G3      ¬promote ∵ ¬(G2★∧G3★)
```

**▸ B. DECISION / BRANCH** — `◆` with guarded arms (mutually exclusive):

```text
        ◆ EV/Cx ?
   ┌──═[≥1.0]▶ ✅ ▢approve ▷⊳ @owner
   ├──═[.5–1)▶ ⚠ ▢revise ↻[≤2] ◆          (bounded re-loop)
   └──═[<0.5]▶ 🧊 ▢defer ▷⊳ ◎backlog
```

**▸ C. FAN-OUT / FAN-IN** — parallel split, barrier join, guarded merge:

```text
◎change ⊕▶ ⦃ ▢lint ║ ▢test ║ ▢typecheck ⦄ ▶⊕ ◆ ∀🟢 ?
                                              ├─═[∀🟢]▶ ⇧merge★
                                              └─═[∃🔴]▶ ⤴@coder  (◂⊳ failing report)
```

**▸ D. SWIMLANE** — multi-actor async sequence across time `⊳`:

```text
⦿orchestrator │ HO ⤳····················⤳ Q✓
⦿planner      │     ╰⤳ ▢plan ▷⊳ ◎spec ⤳
⦿coder        │                  ◂⊳ ▢impl ─⬡[validate]▶ ⟨COMMIT:WIT⟩ ⤳
              └──────────────────────────────────────────────▶ t⊳
```

**▸ E. STATE MACHINE** — states, triggers, guards, terminal:

```text
⊙ ─spawn▶ (○idle) ─pick⟨ID⟩▶ (⧗active) ─═[🚦green]▶ (★done) ─▶ ⊙
                       │  ▲
            block[dep∅] │  │ unblock[dep★]
                       ▼  │
                   (⊘blocked)
```

**▸ F. DEPENDENCY DAG** — fan-in deps with a critical edge (`═w▶` heavier = critical):

```text
◎A ──┐
◎B ──┼═3▶ ▢build ═▶ ◎artifact         (B→build weighted 3 = critical path)
◎C ──┘        ⛓ ◎toolchain
```

**▸ G. FEEDBACK LOOP w/ delay** — `↺⧖` = next cycle, not same frame (avoids ⥁ drift):

```text
▢optimize ▷⊳ ◎metric ─═[Δ>ε]▶ ↺⧖ ▢optimize        ─═[Δ≤ε]▶ ★converged
```

**▸ H. NESTED SUBGRAPH** — `▸[ … ]` collapses detail at the parent altitude:

```text
⊚Pipeline ═▶ ▸[ ◎job ▷⊳ ▢headless-run ▷⊳ ◎output ] ─⬡[validate]▶ ⇧promote
            expand ▸ only when reasoning inside the stage
```

**▸ I. DSM / DEPENDENCY MATRIX** — every dependency at once; row depends on `●` column:

```text
        │ MAT APS SNAP WRK
   MAT  │  ★   ·    ·   ·
   APS  │  3   ★    ·   ·     cell = dep weight 1–3 (3 = critical) · blank = none
   SNAP │  ·   2    ★   ·     below-diag = forward dep · above-diag = feedback ⟶ re-sequence (⥁)
   WRK  │  ·   ·    3   ★     column sum = fan-in load · row sum = blast radius
```

**▸ J. TIMELINE / GANTT** — stage spans on a time axis `⊳` (`▰` active · `░` slack · `◆` milestone):

```text
   plan    ▰▰░░░░░░
   build   ░░▰▰▰▰░░   ⛓ after plan
   verify  ░░░░░░▰▰   ◆ship
   t ⊳ ───────────────▶
```

**▸ K. CONTENT-BASED ROUTER** — one input, guarded arms select destination by content:

```text
◎msg ═[kind=geom]▶ ▢geometry-lane
     ═[kind=tile]▶ ▢tile-lane
     ═[kind=mat ]▶ ▢material-lane
     ═[else]▶ ⛔ ▢dead-letter ⤴@triage
```

**▸ L. ERROR / RETRY-BACKOFF** — try → bounded retry w/ backoff → escalate + compensate:

```text
▢call ─⬡[ok]▶ ★done
      └─[err]▶ ◆ attempts<k ?
                 ├─═[yes]⟨↻⟨τ×2⟩⟩▶ ▢call         (backoff grows)
                 └─═[no]▶ 🔴 ⟲rollback ⤴@owner
```

**▸ M. SEQUENCE (messages between actors)** — request `▷` / reply `◂` / async `⤳`, time `⊳`:

```text
⦿client │ ─req(x)─▶ │           │
⦿api    │           │ ─query─▶  │
⦿db     │           │ ◂──rows── │
⦿client │ ◂────reply(y)─────────│
   t ⊳ ─────────────────────────▶
```

**▸ N. HIERARCHY / TREE** — containment / decomposition (`├─ └─` branches):

```text
⊚System
├─ ▢Ingest  ─ ◎raw ▷⊳ ▢parse
├─ ▢Process ─ ▸[ derive · enrich ]
└─ ▢Publish ─ ◎registry★
```

**▸ O. WEIGHTED FLOW (Sankey-style)** — quantity splits/merges; weight ≈ volume; conserve in=out:

```text
◎input ═70▶ ▢A ═50▶ ◎out1
       ═30▶ ▢B ═20▶ ◎out2
              ▢A ═20▶ ◎out2     (A's remainder merges into out2 — Σin=Σout)
```

**▸ P. CONCURRENCY (Petri-style)** — places `○` hold tokens `•`; transition `▮` fires when all inputs hold:

```text
(○ready•) ─▶ ▮acquire ─▶ (○lock•) ─▶ ▮work ─▶ (○done)
                ▲
   (○permits•••) ┘        3 tokens = 3 parallel permits (bounded concurrency)
```

**Authoring:** any artifact describing *order, choice, parallelism, actors, time, or
quantity* uses one of A–P rather than prose. **Compose** them — a swimlane whose stage is a
gated pipeline whose node is a subgraph with a feedback loop (worked in §3.10). Keep ≤~7
nodes per chart; collapse the rest into `▸` subgraphs.

### §3.10 WORKED COMPOSED EXAMPLE (compose the forms)

A request → shipped artifact as ONE chart composing **D** swimlane · **A** gated pipeline ·
**H** subgraph · **K** router · **L** retry · **G** feedback. Read top→bottom, time `⊳`:

```text
⦿orchestrator │ ⟨REQ-014⟩ ▷⊳ ◆route ═[kind=geom]▶ ·················································· ⤳ Q✓★
⦿planner      │              ╰⤳ ▢spec ─⬡[schema⊨]▶ ◎spec ⤳
⦿coder-mcp    │                          ◂⊳ ▸[ ◎job ▷⊳[t:JSON] ▮headless ═⟨↻2⟨τ⟩,≤60s⟩▶ ◎GLB ]
⦿coder-mcp    │                                 │ └─[err×2]▶ 🔴 ⟲ ⤴@designer-mcp
⦿coder-mcp    │                                 ◎GLB ─⬡[validate ◕]▶ ▢stage
⦿designer-mcp │                                          ◂⊳ ◆sign? ─═[💬yes]▶ ⇧promote ▷⊳ ◎registry★
⦿designer-mcp │                                                   └─═[no]▶ ↺⧖ ▢spec   (feedback: re-spec)
              └────────────────────────────────────────────────────────────────────────▶ t ⊳
gate: ¬promote ∵ ¬(validate★ ∧ sign★)        cost: spec 2 · build 5 · validate 1 = 8
```

The NL equivalent runs ~90–120 tok and still leaves order/retry/rollback/feedback fuzzy;
this chart is ~35 tok and states actors, order, payloads, gates, retry+backoff, rollback,
and the feedback edge **unambiguously**. That gap is the whole point of SYMLANG.

### §3.11 GRAPH ALGEBRA (compose & nest named graphs)

Build big systems from small **named** charts, combined with operators — so you reuse a
graph instead of re-drawing it (⏩ + 💰↓):

```text
⟨G:name⟩          name a graph once, then reference it
G1 ⨟ G2           sequence — G2 begins when G1 closes (★)
G1 ∥ G2           parallel lanes (independent; rejoin with ▶⊕)
G1 ▸ G2           nest — G2 is the expansion of a node in G1
G1 ⋈⟨k⟩ G2        join on key k (graphs share nodes/artifacts keyed by k)
G1 ⊕ G2           choice — one of G1 | G2, selected by a ◆ guard
↻[n] G            iterate G ≤ n times (bounded)
```

```text
EXAMPLE  ⟨G:art⟩ ≝ ⟨G:spec⟩ ⨟ ⟨G:bake⟩ ⨟ ⟨G:validate⟩ ⨟ ⟨G:promote⟩
HYBRID   swimlane-with-gates = D ⋈⟨stage⟩ A   ; each lane stage IS a gated pipeline (§3.10)
LAW      a referenced ⟨G:name⟩ must be defined once with ≝ ; ≤7 nodes per definition (nest the rest)
```

### §3.12 I/O FORMS (from $REPORT §6,§12,§13 — the measured-cheapest shapes)

**STATUS-VECTOR** — dense state as default+exception (`$REPORT W2,W5`; −65%→−87% at scale, −73% vs JSON):

```text
states: ●=pass ◐=running ○=skip ✗=fail   (gate ≜ build∧test∧appr≥2)
Δmods(12): all ● except ✗ test:appstarter · ◐ build:nuke   release ○ (blocked on gate)
```

**SIGNATURE-BOOK** — tool/CLI schemas as one line each (`$REPORT W1`; −92% @ 100% callable):

```text
# codebook once: NS=<const-prefix>  PK=project_key:str
# sig: name(req, [opt]):type =default -> result
create_issue(PK, summary, type, [assignee, desc:md]) -> issue
⚡ SACRED: keep the EXACT tool name — stripping the disambiguating segment ⟶ 12% callable ($REPORT §13)
```

**Δ-HANDOFF** — lossless record set for a consumer that parses it back (`$REPORT W7`; −61% vs JSON, 100% round-trip):

```text
cols: id title state appr pipe        [legend: state ●done ◐wip ○open ✗fail]
DEV-1 "fix race" ◐ 1 ✗ ; DEV-2 "atlas" ● 2 ●
keep the column header + legend; drop NO field (the §3.12 vector is lossy by design — use this full table when records must reconstruct)
```

**REASONING-LATTICE** — deep diagnosis as HYP/EV/INFER with computed ρ (`$REPORT §12`; −16/−22% AND auditable):

```text
LEX  H<n>=hypothesis · π prior · ρ posterior · ▣ observed ⊡ measured · ⊕→ supports ⊖→ refutes (╱ weak ╱╱ strong) · ⤳ causes ⊸ prevents
HYP  H1 layer-race · H2 import-cycle
EV   E1 repro⟺concurrent ⊕╱╱→H1 ⊖→H3 · E2 path off-by-1 ⊕╱╱→H2
INFER ρ(h) ∝ π(h)·∏ₑ LR(e,h)  ⟶  H2 0.84 ◕ (root) · H1 0.16 (trigger: H1⤳H2)
FIX  break import cycle  NEXT ΔWF→@owner
```
(round-trips to JSON lossless ⟹ doubles as machine output. `do(·)`/LTL/decision-matrix add *correctness* at +token cost — use when auditability matters, not for thrift.)

---

## §4 REF & ROUTING (drift-resistant schematization)

```text
$ref:<repo-rel-path>[§<section>]      file/section pointer (resolve via digest, not full Read)
$sym:<Symbol>@<path>                   code symbol anchor
⟨ID⟩                                   queue/slice/program id
@role                                  agent role
ΔWF→@role ⟨ID⟩                         route work-next
```

## §5 STREAM / HANDOFF delimiters

```text
⟨BRK⟩ stop-stream → hand off        ⟨CONT⟩ continue same slice ($ref + last ⟨ID⟩)
⟨DRIFT⟩ re-anchor ($ref + witness + T-cell)   ⟨COMMIT:WIT⟩ witness landed (path only)
⟨BP:COLLECT⟩→⟨BP:MIRROR⟩→⟨BP:SCAN⟩→⟨BP:SHARE⟩→⟨BP:RESUME⟩   forced-continuation when idle/blocked
```

---

## §6 BINDING grammar (templating — how generic artifacts get specialized)

Generic artifacts contain **typed placeholders**. The per-file `⟦META⟧` (§7) lists each
slot; a project binds them in its environment.

```text
{{NAME:type}}        a slot to bind        ⟿  "bind to"        ⊨ "slot satisfied"
types: cmd | path | sym | pkg | dir | schema | role | url | name | n
example:  {{VALIDATOR_CMD:cmd}} ⟿ 「npm run lint --json」 ⊨
```

**Canonical slot legend** (shared across the set — see README):

```text
{{PROJECT:name}}      project / repo name              {{DRIVER:cmd}}     harness invocation
{{CLI:cmd}}           underlying tool CLI               {{BRIEF:path}}     token-contract doc
{{PKG:pkg}}           build/test package/target         {{VALIDATOR_CMD:cmd}} structured-report validator
{{REPORT_SCHEMA:schema}} report fields to reason on      {{AUTHORITY_MAP:path}} ownership/authority doc
{{SYSTEMSET:sym}}     scheduling unit / ordering anchor  {{STAGING_DIR:dir}} pre-publish artifact dir
{{REGISTRY_DIR:dir}}  shipped-artifact registry          {{JOB_SCHEMA:schema}} job/spec schema
{{WITNESS_DIR:dir}}   run/witness telemetry dir          {{QUEUE_CMD:cmd}}   queue/handoff brief cmd
```

**Project illustration (Rust_engine_template_01 — bind in repo ⟦META⟧, not universal):**

```text
{{MAP_FRAME:sym}}           SimMapProjectionFrame @ src/gui/map_camera.rs
{{PLACEMENT_PROBE:sym}}     ConstructionPlacementDebugProbe @ src/construction/placement_debug.rs
{{VIEW_PX_RULE:sym}}        view_px ← camera.viewport.phys/scale ∨ window logical (scissor healed)
{{PROJECTION_SKILL:path}}   .cursor/skills/bevy-simulation-grade/09-sim-map-projection-placement.md
```

**Placement projection packet (domain extension — compress live debug, not perf logs):**

```text
⟨MAP-PICK⟩ 🔴⌁?  pickΔ=374.2 ghostΔ=91.5px  roundtrip_cam=0.0px ok
  latch_hole=true  viewport=full  fixed=769×433  visible=1280×720
  ⊰ fixed_w/h=view/zoom · visible_w/h=manual span · confound: used fixed in in_frame
  NEXT ΔWF→@coder $ref:{{PROJECTION_SKILL:path}} §fixed-vs-visible
```

---

## §7 FILE META-HEADER (the per-file meta layer — gate for fixing)

Every skill/agent in this folder opens (immediately after YAML frontmatter) with this
**visible, dev-layer** block. It declares provenance, the transferable pattern, the slots
to bind, and a gate checklist. Strip on publish if desired (README §publish).

```text
⟦META⟧⟐v1 ◈GENERIC
 src⊳  <origin-artifact> @ {{PROJECT}}
 ptn⊳  <transferable pattern, in SYMLANG — the lesson that survives any project>
 use⊳  <attach-when: trigger glyphs / domains>
 bind⊳ {{SLOT:type}} … (every placeholder this file uses)
 gate⊳ ☐bind ☐verify ☐publish · φ:template   (☐→☑ as the new env satisfies each)
 lang⊳ $ref:prompts/SYMBOLIC_LANGUAGE.meta.md
⟦/META⟧
```

`gate⊳` is the control surface: `☐bind` (all slots ⊨), `☐verify` (commands run green in
the new env), `☐publish` (cleared to copy into the portal). `φ:template → φ:bound → φ:live`.

---

## §8 GRAMMAR (mini-EBNF — SYMLANG is a real notation, not decoration)

```text
artifact   ::= frontmatter meta body
meta       ::= "⟦META⟧" field+ "⟦/META⟧"
body       ::= ( packet | chart | table | annotation )+
packet     ::= "⟨" id "⟩" cluster confidence? lattice? flow? review* result? next?
cluster    ::= glyph{1,4}                         ; L→R, ≤4, evidence-closed if ✅
confidence ::= "◔" | "◑" | "◕" | "●" | "◌"
graph      ::= ( flow | branch | fanout | swimlane | subgraph )+
flow       ::= node ( edge node )+
node       ::= ("⊚"|"⦿"|"▢"|"◆"|"⬡"|"◎"|"⊙") label mark?
mark       ::= "★"|"○"|"◐"|"⊘"|"⧗"
edge       ::= ( base | nuanced ) annot*
base       ::= "═▶"|"━▶"|"┅▶"|"⇢"|"☍"|"⊗"|"⛓"|"⌁"|"↺"|"⥁"|"⤴"|"⤵"
nuanced    ::= "═[" guard "]▶" | "▷⊳" | "◂⊳" | "⬡▶" | "⇧" | "⊰" | "═w▶"
             | "⤳" | "⤳⧖" | "⛔▶" | "⊸" | "⟲" | "↻[" n "]"
annot      ::= "[" type "]" | "⟨cost:" n "⟩" | "⟨≤" dur "⟩"
             | "⟨↻" n ("⟨" backoff "⟩")? "⟩" | "⟨" confidence "⟩" | card
card       ::= n "▶" n
branch     ::= "◆" guard "?" ( "═[" arm "]▶" node )+        ; arms mutually exclusive
fanout     ::= node "⊕▶" "⦃" node ("║" node)* "⦄" ( "▶⊕" node )?
subgraph   ::= "▸[" flow "]"
review     ::= ("🧠?"|"⌁?"|"⊗!"|"🕳!"|"💰!"|"🏛!"|"🌀!"|"⚡!") annotation
result     ::= "🟢" "✅" closer                    ; closer ∈ {🧪,🔬,📜,⊚,🎯}
next       ::= "NEXT" ("⚑"|"ΔWF→@" role) ("⟨" id "⟩")?
ref        ::= "$ref:" path ("§" section)? | "$sym:" symbol "@" path
slot       ::= "{{" NAME ":" type "}}"
annotation ::= "「" natural-language "」"           ; ≤1 line, last resort (L1)
```

---

## §9 ENFORCEMENT — checklist & anti-patterns

```text
☑ COLD-DECODE ≥95% (fresh reader, no legend) + token before/after measured — the SHIP gate (L9)
☑ BLEND: narrative=prose · dense status=●◐○✗ · emoji sparse only · rules=∀⇒≥ (L10)
☑ CHART-FIRST: artifact opens with a packet/table, not a paragraph
☑ CLUSTERS ≤4, parsed L→R, every ✅ has a closer (🧪|🔬|📜|⊚)
☑ confidence inline ~.5/.75/1.0 (◔◑◕● dense vectors only) ; numbers cited or ASK:
☑ claims carry $ref/$sym ; reads via ⊚digest at ▾3–▾4
☑ ends with NEXT (⚑ / ΔWF→@role) ; never a dangling state
☑ NL only inside 「…」 and ≤1 line

ANTI ⛔
  ✗ 「The system passed and looks good」      → ⟨ID⟩ 🟢✅🧪 ~.75
  ✗ restating a table's contents in prose      → delete the prose
  ✗ lone ✅ / decorative emoji                 → close it or drop it
  ✗ emoji for a DENSE status vector            → ●◐○✗ (½ the tokens, $REPORT §8)
  ✗ token win never cold-decoded               → measure first; un-decodable = not a win (L9)
  ✗ pasting full logs / full witness JSON       → ⊚digest + path, escalate only < ◑
  ✗ "confidence: 0.9"                           → ~.75
```

---

## §10 EXAMPLES (NL → SYMLANG)

```text
① STATUS
  NL  「Viewport resolve passed validation, witnessed in prod, fairly confident.」 (~16 tok)
  SYM ⟨VIEWPORT-RESOLVE⟩ 🟢✅📜 ◕                                                  (~6 tok)

② FINDING + ROUTE
  NL  「There's a second writer to ResolvedViewports which risks drift; the planner
       should redesign the authority.」                                            (~22 tok)
  SYM ⟨DUAL-WRITE-RV⟩ 🔴❌🏛  ⊚UI ☍ ⊚ResolveChain  ⌁? drift  NEXT ΔWF→@planner      (~13 tok)

③ PROPOSAL GATE
  NL  「This adds a lot of complexity for limited value, so let's defer.」          (~13 tok)
  SYM ⟨PROP-PG-DB⟩ EV/Cx<0.5 🌀! 🧊DEFER                                            (~7 tok)

④ HANDOFF
  NL  「I'm blocked, handing off; continue from the snapshot at this path.」        (~13 tok)
  SYM ⟨BRK⟩ ΔWF→@coder ⟨SNAP-014⟩ $ref:{{WITNESS_DIR}}/snap_014.json               (~9 tok)

⑤ PLACEMENT PROJECTION (live overlay metrics — not cargo perf)
  NL  「Camera pick roundtrips fine but manual egui inverse is 374 world units off;
       latch hole is on but viewport is full window; ortho fixed is 769×433.」     (~28 tok)
  SYM ⟨MAP-PICK⟩ 🟡⌁? pickΔ=374 roundtrip_cam=0.0px ok  latch∧viewport=full
      fixed≠visible_span  NEXT ΔWF→@coder $ref:09-sim-map-projection-placement.md ◑  (~14 tok)
```

---

## §11 QUICK-REFERENCE CARD (author + review at a glance)

```text
NODES   ⊚own ⦿agent ▢proc ◆decide ⬡gate ◎data ▸sub ⊙term      marks ★closed ○open ◐partial ⊘blocked ⧗wip
EDGES   ═▶hard ━▶soft ┅▶weak ⇢spine ☍conflict ⊗coupled ⛓dep ⌁prob ↺loop ⥁drift ⤴esc ⤵del
        ═[g]▶guard ▷⊳emit ◂⊳consume ⬡▶gated ⇧promote ⊰derive ═w▶weight
        ⊕▶fan-out ▶⊕fan-in ↻[n]loop≤n ⤳async ⤳⧖defer ⛔▶forbid ⊸invalidate ⟲rollback
ANNOT   [t:Type]payload · ⟨cost:w⟩ · ⟨≤Δt⟩SLA · ⟨↻k⟨τ⟩⟩retry · ⟨◕⟩edge-confidence · 1▶n card
STATUS  dense ●pass ◐run ○skip ✗fail ⊘blocked  ·  sparse(emoji) 🟢🟡🔴🧊 ⏳ ⛔ ⚠ 🚦 · ✅+closer{🧪🔬📜⊚} never lone
CONF    inline ~.5/.75/1.0 (cheap) · dense-vec ◔◑◕● ◌?  ·  act≥◕ · escalate<◑
LOGIC   ∧∨¬⊕ ⇒⇔ ∴∵ ⊨   SETS ∀∃∄∅∈∉⊆∩∪   Δ +−~!≈⇄   TIME ⊳now ⊲past ⊳⊳next ⟳cycle ⧖await ⌛stale
FORMS   A pipeline · B branch · C fan · D swimlane · E state · F DAG · G feedback · H subgraph
        I matrix · J timeline · K router · L retry · M sequence · N tree · O sankey · P concurrency
I/O     §3.12 status-vector ●◐○ · signature-book name(req,[opt])->r (SACRED name) · Δ-handoff table · reasoning-lattice HYP/EV/INFER+ρ
DIMS    Ct💰 Cx🌀 Cm♻ Dp⛓ Au🏛 Rk⚠ Q🎯 Δ⚖ U⌁ T⏱ E🔬 H👁 S🌐   (1 glyph/dim · fill 🟨)
ALGEBRA ⟨G:name⟩ · G1⨟G2 seq · G1∥G2 lanes · G1▸G2 nest · G1⋈⟨k⟩G2 join · G1⊕G2 choice · ↻[n]G
SCOPE   ⟦block⟧ ⟨id⟩ 「NL≤1line」 ⦃set⦄ ▸nested      COMPRESS ▾1..▾4 · ⊚digest ≻ 📄full
LAWS    cold-decode≥95%+token-Δ (L9) · blend≻pure (L10) · chart/graph-first · cluster≤4 · ✅+closer · cite $ref/$sym · end NEXT⚑
OUTCOME every glyph ⊨ ⏩ ∨ 💰↓ ∨ 🎯 ∨ 🔍fidelity ; else cut (L2)  ·  emoji=SPARSE-status · geometric=STRUCTURE+dense-status
```

---

```text
⟦/SYMLANG⟧  bind⊳ this spec is project-agnostic · ref it from every ⟦META⟧
NEXT ⚑ author artifacts chart-first · enforce §9 in review
```
