# Native-only follow-up

The production Processes page already builds native View columns. It does not meet the all-painted-columns condition in widgets/table/view.rs and does not enable that branch's palette. The native rows_source already keys presenters by index and uses the real row count. Therefore the earlier process-keyed probe is not a measurement of the existing production table.

## Presenter identity

The renderer's keyed source path attempts to preserve the intersection of represented process keys, but changing the key order emits SetVirtualSource and a native observable-vector Reset. It does not guarantee preserving every process component across a reset or leaving the realized range. This mechanism was inspected read-only and confirmed with the Guinea owner; no vendor changes were made.

The slot experiment keys visual presenters by rank position while keeping the actual process ID in their Input. Both metric values and process names update on each tick. The key revision stays constant while slot count and key sequence are unchanged; new view closures supply current data. This is a pool of native row presenters, not a custom renderer or a claim that process-local state may safely live in a slot.

Process-owned state, selection, asynchronous work and actions must remain attached to logical process identity outside the slot. Any row-local effect must handle a changed process/generation. A click must carry the ID of its drawn snapshot, not resolve an index against a later order. Inspection of the current page confirmed selection is stored as a PID and press mapping uses the captured display snapshot; production selection logic was not changed by this experiment.

CPU times below cover ten seconds of concurrent metric changes and sorting. All runs completed normally and delivered every expected tick. Only native Grid/TextBlock cells are used.

| Workload | Process-keyed presenters | Slot-keyed presenters | Ranked component creations |
| --- | ---: | ---: | --- |
| 2000 rows, 500 ms, mild jitter; mean of two runs | 0.906 s | 0.508 s | 700 → 0 |
| 2000 rows, 500 ms, full shuffle; one run | 0.797 s | 0.563 s | 700 → 0 |
| 10000 rows, 100 ms, mild jitter; one run | 3.891 s | 2.063 s | 3500 → 0 |

The main paired slot runs used 0.422 and 0.594 seconds, versus 0.906 and 0.906 for process keys. This is about 44% lower mean CPU in this comparison. Slot memory stayed around 89–91 MiB. These are fresh shared-machine runs, not a universal speedup claim or a production before/after patch: production already uses slots.

The fully native slot configuration is sufficient to remove the component-recreation cost observed in the initial baseline. It should be the starting point for further work instead of adding custom metric rendering.

## Dynamic metric backgrounds

Inspection of the actual metric_cell found a smaller native lifecycle problem. Its positional children changed between [TextBlock] and [heat Border, TextBlock]. When the heat layer appears or disappears, the text changes positional slot and can be replaced.

Two synthetic heat variants reproduce this shape inside the native slot table. All four metric values and background presence change on every data tick. A validation run holds references to the original native TextBlock objects and compares their canonical COM identities after updates and sorting. This prevents freed addresses being mistaken for preserved objects.

| Cell children | Native text objects before/after | Original objects retained |
| --- | --- | ---: |
| Positional, optional heat | 136 / 136 | 0 |
| Keyed heat and text | 136 / 136 | 136 |

A further keyed validation reads native TextBlock.Text values and checks them against the current sorted snapshot: all 136 retained controls display the correct current values. It delivered all 20 ranked ticks without recreating row components. The comparison is not retaining stale text merely to improve object counts.

In an uninstrumented single pair, positional heat used 0.656 seconds CPU and keyed heat 0.547 seconds in the ranked phase. The identity-instrumented pair showed opposite CPU ordering, so there is not yet a robust CPU speedup claim for this small cell patch. The established result is removal of unnecessary text replacement with unchanged output semantics.

Production metric_cell now uses keyed children named heat and value. No renderer, scrolling, domain state or selection behavior was added. The complete workspace test run passed: 522 tests, one existing ignored test. The initial restricted run failed only on a temporary settings-file write; the full run outside that restriction passed.

## Reproduce

```powershell
./diagnostics/table-probe/run.ps1 -Modes native -Identity process -Rows 2000 -Period 500 -Mix mild
./diagnostics/table-probe/run.ps1 -Modes native -Identity slot -Rows 2000 -Period 500 -Mix mild
./diagnostics/table-probe/run.ps1 -Modes heat,heat_keyed -Identity slot -Verify -Repeats 1
```

Build as described in README.md. VERIFY adds native identity/value checks; do not mix those runs into CPU comparisons. identity.json contains all 32 phase records of the eight presenter runs. heat.json contains the two identity checks, two uninstrumented runs and the final native-value validation. Earlier raw logs remain under target/table-probe-results/identity and target/table-probe-results/heat.
