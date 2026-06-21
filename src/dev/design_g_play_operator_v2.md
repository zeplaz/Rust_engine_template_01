# G-PLAY operator chrome `v2` — session playback HUD

| Field | Value |
|:---|:---|
| **ID** | **DES-G-PLAY-OPERATOR-V2-001** |
| **Program** | PLAN-DESIGNER-WORK-202606-001 · Track D |
| **Date** | 2026-06-20 |
| **Owner** | `@designer` |
| **Depends** | [`session_playback_issues_todos.md`](session_playback_issues_todos.md) · PLAY-01 sim HUD |
| **Verdict** | **PASS** |

```text
DES-G-PLAY-OPERATOR-V2-001 Q✓
Transport · speed · chapter — collapsed sim chrome
```

---

## 1. Transport bar (simulation + replay)

```text
◀  ▶  ⏸   1×   2×   4×   ·  Ch.3  ·  12:34
```

| Control | Behavior |
|:---|:---|
| Play/pause | icon + tooltip |
| Speed | 1× default · 2×/4× badge |
| Chapter | `Ch.{n}` when script active |
| Clock | sim time `MM:SS` mono |

---

## 2. PLAY-01 alignment

| Rule | Spec |
|:---|:---|
| Default | collapsed command tray |
| Editor panels | hidden in `BaseState::Simulation` |
| Diagnostics | verbose sections collapsed |

---

## Sign-off

| Role | Verdict | Date |
|:---|:---|:---|
| `@designer` | **PASS** | 2026-06-20 |
