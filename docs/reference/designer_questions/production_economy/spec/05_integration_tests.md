# Production — integration tests `05`

1. **Round-trip:** spawn facility + power graph, run N ticks, save, reload, compare snapshot subset.
2. **Damage → repair:** apply damage component, enqueue repair, assert state after fixed ticks.
3. **Grid island:** disconnect edge; detect islands; UI or debug assert expected count.
