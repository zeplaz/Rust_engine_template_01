# Map editor M1 — questions bubbled from execution

> **Purpose:** When a runbook step hits `ASK:` or two consecutive verify failures, capture the exact decision needed here so the product owner can answer without digging through chat.  
> **M1 run (Rust):** No halt — `cargo check` and tests passed.

---

## Blocking (must answer before next Rust step)

*(none)*

---

## Resolved

1. **Menu labels vs. single editor state (answered 2026-04-30)**  
   **Decision:** Use **context-aware copy** on every control that enters **`BaseState::Editor`** — each label says *what the user is doing from that screen* (new blank session, opened save stub, handoff after generation).  
   **Invariant:** The **canonical state** remains one: `BaseState::Editor` (+ `WorldGenFlowState::Idle`). Labels must not imply different engine states; they only describe **entry context** for humans.

   **Concrete copy (engine):**

   | Where | Button / affordance | Meaning |
   |:---|:---|:---|
   | Main menu bar | `New map in editor` | Blank session, no save path |
   | Load screen (stub) | `Open saved map in editor (stub)` | Intend to hydrate from path (M5) |
   | World gen `FullReady` | `Edit generated world in map editor` | Handoff from procedural result |

---

## Non-blocking / polish (optional)

*(none — moved to Resolved.)*

---

## How to use

- Add a row under **Blocking** with: step id (e.g. `M2-S03`), file touch, and the exact ambiguity.  
- After answers land, record under **Resolved** and link the commit or PR.
