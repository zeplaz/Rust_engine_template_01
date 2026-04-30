# Faction editor — validation `04`

**Principle:** no **gameplay-enforced** max faction count; validate for **correctness** and **UX at scale**.

## Names & ids

- **Unique** `name` / `short_code` within scenario scope (warn or block duplicates 📎).
- **Reserved** names for system factions 📎 if any.

## Colors

- **Contrast:** optional check vs map background / minimap (warn only).
- **Collision:** warn when two factions share near-identical HSL in same scenario.

## Diplomacy graph

- **Symmetric vs asymmetric** edges: enforce consistency rules (e.g. war must be mutual 📎 or allow asymmetric embargo).
- **Cycle / constraint** checks: invalid treaty combos flagged in editor before apply.

## Scale (soft)

- **Roster > N:** switch roster panel to virtualized list or paging 📎.
- **Save size:** optional diagnostic when graph or tag bags grow (no hard reject).

## Live edit safety

- Reject or queue changes that violate MP authority or locked scenario 📎.
