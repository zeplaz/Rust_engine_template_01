# Subagent continuity playbook (mission-critical work)

> **Problem:** Cursor **Task** background workers sometimes fail with *“Increase limits for faster responses / Switch to Auto”*. That is a **billing/quota gate on the Task API**, not a hard stop on the project. Work must continue via **alternate channels** documented here.

**Related:** [`AGENTS.md`](../../AGENTS.md) agent routing · [`llm_agent_brief.md`](../llm_agent_brief.md) · lane playbooks `tools/orchestrator/agents/`

---

## 1. Three channels (not interchangeable)

| Channel | How you invoke it | Quota profile | Best for |
|---------|-------------------|---------------|----------|
| **A — Main chat (Auto)** | Default composer in this thread | Parent session | Implementation, tests, fixes after Task failure |
| **B — Chat agents (`@coder`)** | Agent picker / `@planner` `@coder` `@designer` `@sim-steward` | Uses **`model: auto`** from [`.cursor/agents/`](../../.cursor/agents/) | Same roles as Task, **no separate Task spawn**; **`@sim-steward`** = sequential shifts when Task blocked |
| **C — Task tool** | Multitask mode, or explicit `Task` in agent | **Separate subagent quota**; premium models exhaust first | Long background slices **only when quota is green** |

**Rule:** Treat Task **C** as optional acceleration. **A** and **B** are the continuity backbone.

---

## 2. Why Auto works but Task keeps failing

These are **not the same product path**:

| What you see | API path | Typical quota |
|--------------|----------|----------------|
| **This chat (Auto)** | Main Composer agent session | Your normal Composer / Auto allowance — **works for you now** |
| **Task tool** (`Task(subagent_type=coder, …)`) | Background **subagent** runner (Multitask mode) | **Separate** “agent” / subagent pool — can be **0** while Auto still works |
| **`@coder` in a chat** | Custom agent with `model: auto` in [`.cursor/agents/`](../../.cursor/agents/) | Usually billed like **parent chat**, not like Task |

So *“Switch to Auto”* literally means: **stop using the subagent Task API; use the main agent (Auto) instead.** Retrying `Task(composer-2.5-fast)` does **not** switch pools — it still hits the **same Task meter**, which is why it keeps failing even with a “fast” model name.

**Your agents are not broken** — the **Task delegation channel** is rate-limited. Role agents (`orchestrator`, `coder`, …) work when invoked as **`@coder` in chat**, not when the parent only spawns `Task(coder)`.

---

## 3. When Task fails — do not retry Task

```
Task error (usage limit)  ← Task pool exhausted (all models)
    ├─► 1. Parent (Auto) or @coder in THIS chat implements now   ← correct fix
    ├─► 2. New chat: @coder + playbook + file list (NOT Task)
    ├─► 3. HANDOFF.md for next session
    └─► 4. SDK / admin quota increase (only ways to “turn Task back on”)

    ✗ Do NOT: Task(composer-2.5-fast) again — same pool, same error
    ✗ Do NOT: Task(coder) / Task(orchestrator) on premium models
```

**Only** retry Task after Cursor usage shows **subagent/agent** budget available again (or admin raises limits).

### If Task quota is available later (rare tuning)

| Parameter | Note |
|-----------|------|
| `model: "composer-2.5-fast"` | Still Task pool — helps only when pool is non-zero and premium models were the issue |
| `run_in_background: false` | Foreground Task — same pool, sometimes different queue behavior |
| `subagent_type: "explore"` + `readonly: true` | Cheaper recon, still Task |
| `resume: "self"` | Continuation fork — still Task |

---

## 4. Chat-agent path (recommended substitute for Task)

1. Open a **normal** agent chat (not Multitask-only delegation).
2. Invoke **`@coder`** or **`@planner`** — definitions live in [`.cursor/agents/`](../../.cursor/agents/) with `model: auto`.
3. Paste a **compact handoff** (see §6) + playbook path, e.g. `tools/orchestrator/agents/viewport_cleanup_agent.md`.
4. Parent or `@coder` runs `cargo test` via Shell — **do not** Task-compile.

This preserves role separation (planner vs coder) without the Task quota bucket.

---

## 5. Multitask mode vs mission-critical sessions

**Multitask mode** instructs the parent to spawn background Task workers. If Task quota is zero, the parent may exit without doing work.

**Mitigations:**

- Disable Multitask for critical implementation sessions; parent implements directly.
- Or: parent policy **“Task error → immediate foreground implementation”** (no wait, no user ping-pong).

---

## 6. Handoff artifact (continuity without any subagent)

Copy [`tools/orchestrator/queues/HANDOFF.template.md`](../../tools/orchestrator/queues/HANDOFF.template.md) to `HANDOFF.md` (gitignored or short-lived branch) and fill:

- **Goal** (one sentence)
- **Lane** (Stage 5 / construction / LOG / VM)
- **Files touched / next files**
- **Commands run** + pass/fail
- **Witness JSON** paths + key flags
- **Blockers**

Next session: `@coder` + “Continue `HANDOFF.md`” — works even if all Task IDs failed.

**Transcripts:** Cursor agent transcripts under the project’s agent-transcripts folder retain full context; search by task name / file path before re-explaining.

---

## 7. SDK path (outside IDE Task quota)

For automation or when the IDE Task pool is exhausted, use the **Cursor TypeScript SDK** (`@cursor/sdk`) with an API key:

```typescript
import { Agent } from "@cursor/sdk";

const result = await Agent.prompt(handoffPrompt, {
  apiKey: process.env.CURSOR_API_KEY!,
  model: { id: "composer-2" },
  local: { cwd: "C:/dev/github/Rust_engine_template_01" },
});
```

- **Local runtime** edits the real repo on disk.
- Billing is **API-key / cloud account**, not the same meter as IDE Task subagents.
- See the user skill `sdk` for `Agent.create` + `resume` multi-turn flows.

Optional: wire a repo script `tools/orchestrator/invoke_sdk_handoff.ps1` in CI or a scheduled job (document only until implemented).

---

## 8. Shell + orchestrator (no LLM subagent)

Heavy but deterministic work needs no subagent:

```powershell
cargo test -p proc_A_dine01 <filter> --lib
cargo orchestrate --skip-clippy --skip-test
./tools/orchestrator/ci/run.ps1   # CI-shaped pipeline
```

Use **background Shell** in Cursor for long compiles; parent consumes output when notified.

---

## 9. Account / org (real quota increase)

If Task limits are organization policy:

- Admin raises subagent limits in Cursor team settings.
- Split **thinking** vs **fast** usage across team members.
- Use **Cloud Agents** / API for batch work so IDE Task quota is reserved for interactive Multitask.

---

## 10. Parent-agent checklist (embed in orchestrator behavior)

When coordinating mission-critical work:

1. Prefer **one coherent slice** per turn in main chat over fan-out Task workers.
2. On Task `status: error` + usage message → **implement in foreground** same turn; do not summarize failure and stop.
3. Attach **playbook path + 3 file paths + test command** to every delegation.
4. Update `HANDOFF.md` or `construction_active_progress.md` when switching lanes (LOG → VM-C).
5. After `src/` edits: `cargo test` (filtered) → witness JSON → `cargo orchestrate`.

### Fail-cycle escalation (main-thread orchestrator)

Use [`.cursor/agents/main-thread-orchestrator.md`](../../.cursor/agents/main-thread-orchestrator.md) when Task + Multitask are unreliable.

| Cycle | Channel | On failure |
|-------|---------|------------|
| 0 | Task(`debug-intelligence` \| `cleanup-intelligence`) | Advance same turn — **do not retry Task** |
| 1 | `@debug-intelligence` / `@cleanup-intelligence` in chat | Advance to cycle 2 |
| 2 | Main-thread Shift A→B (witness + YAML + cleanup gate) | Advance to cycle 3 |
| 3 | Shift C implement or `@coder` handoff + tests | `HANDOFF.md` with one next command |

**Failure includes:** usage errors, empty returns, partial work, timeout. Log each attempt in a `fail_cycle_ledger` (see agent file). **Serialize** failed slices on the main thread when they share files or authority.

---

## 11. Quick decision tree

```
Need work done now?
  ├─ Task quota OK? → Task(composer-2.5-fast) OR @coder in chat
  ├─ Task failed?   → Main chat implements + tests (same session)
  ├─ Session ending? → HANDOFF.md + witness paths
  └─ Batch/offline?  → SDK Agent.prompt local + composer-2
```

**Exit:** Lane witness green or HANDOFF explicitly names the next owner and command.
