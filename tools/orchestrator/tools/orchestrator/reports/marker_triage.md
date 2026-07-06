# Marker triage (A-06)

| Kind | File | Line | Notes |
|------|------|-----:|-------|
| VIEWPORT_AUTHORITY | `src/architectural.rs` | 76 | subsystem: "GUI / VIEWPORT_AUTHORITY".into(), |
| VIEWPORT_AUTHORITY | `src/architectural.rs` | 114 | if markers.iter().any(/m/ m.kind == "VIEWPORT_AUTHORITY") { |
| TODO | `src/architectural.rs` | 115 | systems.push("viewport_authority (marked TODO/MIGRATION in source)".into()); |
| MIGRATION | `src/architectural.rs` | 115 | systems.push("viewport_authority (marked TODO/MIGRATION in source)".into()); |
| VIEWPORT_AUTHORITY | `src/knowledge.rs` | 99 | "GUI / VIEWPORT_AUTHORITY / SEMANTIC_SOLVER".into(), |
| REMOVE_AFTER | `src/main_thread_shift.rs` | 275 | "REMOVE_AFTER" => ( |
| REMOVE_AFTER | `src/main_thread_shift.rs` | 278 | "REMOVE_AFTER requires migration successor", |
| MIGRATION | `src/main_thread_shift.rs` | 281 | "MIGRATION" => ("B_transitional", "preserve", "active migration"), |
| HACK | `src/main_thread_shift.rs` | 282 | "TEMP" / "HACK" / "WORKAROUND" => ("D_incomplete", "preserve", "staging debt"), |
| WORKAROUND | `src/main_thread_shift.rs` | 282 | "TEMP" / "HACK" / "WORKAROUND" => ("D_incomplete", "preserve", "staging debt"), |
| REMOVE_AFTER | `src/main_thread_shift.rs` | 401 | kind: "REMOVE_AFTER".into(), |
| REMOVE_AFTER | `src/main_thread_shift.rs` | 402 | text: "REMOVE_AFTER migration".into(), |
| TODO | `src/reports.rs` | 249 | - TODO-06–11 — frame fence, fire alignment, GPU spine (`STAGE5_TODOS`) |
| MIGRATION | `src/reports.rs` | 305 | s.push_str("- `VIEWPORT_AUTHORITY` / `MIGRATION` → do not auto-clean\n"); |
| VIEWPORT_AUTHORITY | `src/reports.rs` | 305 | s.push_str("- `VIEWPORT_AUTHORITY` / `MIGRATION` → do not auto-clean\n"); |
| TODO | `src/reports.rs` | 306 | s.push_str("- `TODO` / `FIXME` → link to STAGE5 or continuation queue\n"); |
| FIXME | `src/reports.rs` | 306 | s.push_str("- `TODO` / `FIXME` → link to STAGE5 or continuation queue\n"); |
| HACK | `src/reports.rs` | 307 | s.push_str("- `HACK` / `TEMP` → requires owner in knowledge JSON\n"); |
| DEPRECATED | `src/reports.rs` | 377 | s.push_str("\n## Static scan (`#[deprecated]`)\n\n"); |
| TODO | `src/scanner.rs` | 9 | ("TODO", r"\bTODO\b"), |
| FIXME | `src/scanner.rs` | 10 | ("FIXME", r"\bFIXME\b"), |
| HACK | `src/scanner.rs` | 11 | ("HACK", r"\bHACK\b"), |
| TEMP | `src/scanner.rs` | 12 | ("TEMP", r"(//\s*TEMP\b/@TEMP\b/\bTEMP:)"), |
| MIGRATION | `src/scanner.rs` | 13 | ("MIGRATION", r"\bMIGRATION\b"), |
| REMOVE_AFTER | `src/scanner.rs` | 15 | ("REMOVE_AFTER", r"\bREMOVE_AFTER\b"), |
| WORKAROUND | `src/scanner.rs` | 16 | ("WORKAROUND", r"\bWORKAROUND\b"), |
| VIEWPORT_AUTHORITY | `src/scanner.rs` | 17 | ("VIEWPORT_AUTHORITY", r"VIEWPORT_AUTHORITY"), |
| DEPRECATED | `src/scanner.rs` | 149 | if !line.contains("#[deprecated") { |
| VIEWPORT_AUTHORITY | `src/subsystem.rs` | 17 | system: "VIEWPORT_AUTHORITY", |
| VIEWPORT_AUTHORITY | `src/subsystem.rs` | 24 | system: "VIEWPORT_AUTHORITY", |
| VIEWPORT_AUTHORITY | `src/subsystem.rs` | 168 | ("GUI", "VIEWPORT_AUTHORITY", _) / ("GUI", "HUD", "VIEWPORT_SYNC_DEBUG") => vec! |

## Triage policy

- `VIEWPORT_AUTHORITY` / `MIGRATION` → do not auto-clean
- `TODO` / `FIXME` → link to STAGE5 or continuation queue
- `HACK` / `TEMP` → requires owner in knowledge JSON
