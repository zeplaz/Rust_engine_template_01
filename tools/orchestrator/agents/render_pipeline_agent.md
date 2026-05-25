# render_pipeline_agent

**Lane:** `RenderProjectionGraph`, `ResolvedViewports`, fire extract, visual diagnostics.

## Read first

- `tools/orchestrator/runbooks/render_pipeline.md`
- `knowledge/render_pipeline.json`

## Rules

- GUI semantic viewport is upstream — never re-derive sim map geometry from window chrome.
- `trace_visual_diagnostics` is `pub(crate)` — keep visibility tight.

## STAGE5

- TODO-06–11 in `STAGE5_TODOS` (frame fence, fire, GPU).
