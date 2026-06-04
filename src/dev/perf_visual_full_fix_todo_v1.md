# PERF / visual full-fix todo (v1)

Owner: active execution lane  
Scope: streaming spine, world representation stall, viewport stability, render throughput

## 0) Completed in this pass

- [x] Re-enable chunk-interest cap before scheduler sync (`schedule_chunk_streaming_from_interest`).
- [x] Add pending-cap budget field (`StreamingSpineBudget.max_pending_chunks`, env `MAX_STREAMING_PENDING_CHUNKS`).
- [x] Add startup streaming config log (hydrate/reconstruct/pending cap + sync/async mode).
- [x] Add warning when pending set reaches cap (deferred far-field chunks).
- [x] Fix spine work classification to report async hydration when IO is in flight.
- [x] Add async hydrate submit diagnostics.

## 1) Streaming spine hardening

- [ ] Verify runtime now shows capped pending size (`pending_len <= max_pending_chunks`).
- [ ] Verify runtime no longer reports full-frame sync hydration under normal run.
- [ ] If sync fallback still appears, emit explicit reason and callsite metadata.
- [ ] Add guardrail test: scheduler never exceeds cap after interest merge.

## 2) World representation stall (`post_world_repr`)

- [ ] Instrument substage timings inside world representation compute path.
- [ ] Add change-detection cache; skip full rebuild when frame inputs unchanged.
- [ ] Split heavy rebuild into budgeted incremental work across frames.
- [ ] Re-run perf witness and track p95/p99 against target.

## 3) Viewport authority / validity churn

- [ ] Add invariant: resolved-valid simulation viewport cannot be invalidated by camera apply in same frame.
- [ ] Add debounce/hysteresis for validity flip churn.
- [ ] Add single-writer authority assertion and per-frame ownership trace.
- [ ] Validate no repeated `CAMERA_SCISSOR_CHANGED` / `SIM_VIEWPORT_VALIDITY_CHANGED` bursts.

## 4) Raster + UI spike control

- [ ] Add dirty-region raster path for large static intervals.
- [ ] Audit causes of tile raster invalidation and suppress non-essential invalidations.
- [ ] Add per-panel egui cost metrics and throttle non-critical panels in simulation.
- [ ] Add frame spike digest with top 3 culprit spans.

## 5) Fire/overlay pipeline stability

- [ ] Validate overlay cells -> fire rows -> dispatch chain invariants every frame.
- [ ] Add explicit zero-output reason tagging (cull/budget/phase/input-empty).
- [ ] Stabilize zoom-alpha transitions to avoid visible popping across zoom bands.

## 6) Acceptance gate (must pass together)

- [ ] `STREAM spine frame summary` shows no runaway pending set.
- [ ] No sustained `HydrateSync` in normal simulation entry.
- [ ] `post_world_repr` sustained under target budget band.
- [ ] No viewport validity/scissor flip storms after simulation is entered.
- [ ] End-to-end witness captures updated and attached.
