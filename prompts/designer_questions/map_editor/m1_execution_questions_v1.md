# Map editor M1 — questions bubbled from execution

> **Purpose:** When a runbook step hits `ASK:` or two consecutive verify failures, capture the exact decision needed here so the product owner can answer without digging through chat.  
> **M1 run (Rust):** No halt — `cargo check` and tests passed.

---

## Blocking (must answer before next Rust step)

*(none)*

---

## Non-blocking / polish (optional)

1. **Top bar “Load Editor” vs. generator “Open in map editor”** — Do you want one consistent label (e.g. always “Open in map editor”) for every entry into `BaseState::Editor`, or keep “Load Editor” for the empty-editor shortcut from the main bar?

---

## How to use

- Add a row under **Blocking** with: step id (e.g. `M2-S03`), file touch, and the exact ambiguity.  
- After answers land, link the commit or PR that applies the decision and strike through or remove the row.
