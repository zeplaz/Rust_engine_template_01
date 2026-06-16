# Concrete industrial simulation runbook `v1`

> **STATUS:** Draft **v1** — design scaffold. Implementation **Pending**.

Version: `v1.0.0`  
Parent: [`simulation_expansion_orchestrator_v1.md`](simulation_expansion_orchestrator_v1.md)

---

## 1. Purpose

Evolve concrete from **taxonomy / enums** to a **physical-industrial simulation**: mix design, transport, pour, cure, structural outcome — coupled to **weather**, **logistics**, and **infrastructure durability**.

---

## 2. Target industrial chain

```
Aggregate + binder + water + additives
  → Mix design
  → Transport
  → Pouring
  → Curing
  → Structural properties (and failure modes)
```

---

## 3. Concrete types (gameplay differentiation)

| Type | Role |
|:---|:---|
| Limecrete | Cheap, weak |
| Portland | Standard |
| Geopolymer | Heat resistant |
| Gypsum | Rapid interior |
| Rapid-set | Emergency builds |
| High-strength | Military / industrial |
| Lightweight | Bridges / high-rises |

---

## 4. Batch state (sketch)

```rust
pub struct ConcreteBatch {
    pub hydration: f32,
    pub curing_progress: f32,
    pub structural_strength: f32,
    pub moisture_content: f32,
    pub temperature_factor: f32,
}
```

Owning system interprets curing timers, quality, and environmental modifiers.

---

## 5. Weather interaction

Curing and strength should respond to **freezing**, **humidity**, **rain during cure**, and **heat stress**.

---

## 6. Infrastructure impact

Road / structure **durability** may depend on:

- Concrete type and curing quality  
- Traffic load  
- Climate / weather stress  

Link to mobility / damage layers when those matrices exist.

---

## 7. Cross-links

- [`infrastructure_environment_integration_v1.md`](infrastructure_environment_integration_v1.md)  
- [`weather_simulation_runbook_v1.md`](weather_simulation_runbook_v1.md)  
- [`petroleum_industry_simulation_runbook_v1.md`](petroleum_industry_simulation_runbook_v1.md) — asphalt / heavy fractions  
- Existing code pointers: `src/entities/production/concrete/` (evolve under this runbook)

---

## 8. Step packs

[`../matrix/simulation_expansion/runbook/README.md`](../matrix/simulation_expansion/runbook/README.md)
