# Virtual metric rows

[NATIVE.md](NATIVE.md) contains the follow-up for the fully native production path: stable rank presenters and keyed heat/text children. It supersedes using the initial process-keyed baseline to assess Uniproc's native table. run.ps1 now defaults to native cells and slot identity; use -Identity process to repeat the original comparison.

This standalone probe compares numeric metric views inside the same native ItemsRepeater and ScrollViewer. It does not implement table scrolling, row selection, hit testing or a custom table renderer. The subsequent production change only keys the native heat/value layers in metric_cell, as documented in NATIVE.md.

The primary workload changes all four metrics of every process each tick and then changes process order, with stable process IDs as VirtualSource keys. The fixed-order updating phase is a control, not the primary conclusion. Synthetic process names and IDs remain stable. There are no service calls, icon extraction, grouping or process-start events in this workload.

## Implementations

| MODE | Metrics in one row |
| --- | --- |
| native | Four native TextBlocks in a four-column Grid |
| positioned | Four native TextBlocks at fixed positions in a native Canvas |
| dwrite | One native Image backed by CanvasImageSource, drawing four values through DirectWrite; all rows share one GPU device |
| joined | One native TextBlock containing all four values in a monospaced string |

Every row also has the same native process-name TextBlock and background. Row height is 32 DIP, the metric area is 480 DIP wide, and the viewport is approximately 16 rows. The native virtualizer also realizes a cache around the viewport. Metrics use Consolas 14 in all variants. The drawing surface uses scale 1; other DPI settings are outside this experiment.

The DirectWrite variant uses the existing windows-canvas SurfaceImageSource integration. It does not allocate a swap chain or GPU device per cell. Its draw routine formats and draws four strings; it does not contain custom glyph shaping, layout, scrolling or selection code.

Joined text is a lower-bound control, not a drop-in replacement for independently aligned, resized or styled columns. The DirectWrite Image does not yet supply native text accessibility, per-cell text selection, device-loss recovery or DPI changes. Their implementation costs are not included in the timings.

## Workload and measurements

Each fresh process runs these phases:

1. Three-second warmup, including initial sort and GPU/font initialization.
2. Six seconds without data changes.
3. Ten seconds updating every metric with order frozen.
4. Ten seconds updating every metric and sorting by the first metric on every tick.

MIX=mild keeps a stable process-specific baseline with bounded jitter. MIX=shuffle replaces the score with a deterministic hash of process ID and tick. Both update all other metric values each tick. Sorting uses a stable process-ID tie break. The probe logs moved positions and total rank displacement, not merely a claim that rows changed.

The earlier development runs with scrolling are retained only as preliminary controls; the ranked runs answer the main question. In the ranked probe the scroll offset stays fixed.

CPU is the sum of process kernel and user CPU time, including render workers. It is not wall time or the duration of one UI-thread stall. Private memory is sampled at the phase boundary, not the peak. The 20 ms heartbeat reports event-loop interval percentiles; it is not frame-present latency or FPS, and normal Windows timer granularity makes its floor about 32 ms here.

Counters include row/component creation, destruction, input updates, view-factory calls, DirectWrite draw calls and surface attachment results. A transient row can disappear before its queued surface attachment completes; these cancellations are counted rather than treated as successful rendering. Successful attachments and draws must also be present. The counters are reset at each phase boundary.

## Build and run

The manifest uses read-only local renderer and canvas dependencies from the sibling guinea checkout to keep a single compatible renderer instance. No Guinea framework code is used or modified.

```powershell
$env:WINUI_WINMD = 'D:/projects/windows-rs/crates/tools/reactor/winmd'
$env:CARGO_TARGET_DIR = 'D:/projects/uniproc/target/table-probe-build'
cargo build --manifest-path diagnostics/table-probe/Cargo.toml --bin table-probe-layout --offline
./diagnostics/table-probe/run.ps1 -Rows 2000 -Period 500 -Mix mild -Repeats 2
```

The development profile optimizes both probe and dependencies at level 2. Native bindings are generated into Cargo output from bindings.txt. Existing generated bindings and vendor files are not edited.

The runtime DLLs must be present beside the executable. In the measured environment the setup helper could not download them from the restricted build, so the existing working runtime was copied by its explicit runtime.txt allowlist, excluding empty entries. All measured implementations use the same executable and runtime. The first failed launch without runtime and development runs interrupted by invalid focus events are excluded from the final matrix.

Raw RESULT records and aggregate measurements are recorded with RESULTS.md. Shared-machine background load is uncontrolled, so modest differences need repeats and should not be interpreted as precise universal speedups.
