# Results: metric updates with process reordering

Follow-up: [NATIVE.md](NATIVE.md) compares process identity with rank-slot presentation and identifies the actual native configuration already used by Uniproc. The process-keyed baseline below is an experimental configuration, not a measurement of the current production table. Its DirectWrite advantage should not be generalized to the existing native slot-based path.

Measured 29 October 2026. The primary result is phase 3: all metric values change and the table is sorted each tick. Every mode uses native ItemsRepeater and ScrollViewer, the same stable process keys, 32-DIP rows and the same viewport. The ranked phase does not scroll.

## Primary workload

2000 processes, 500 ms metric interval, bounded score jitter, two fresh-process runs per mode. Each measured phase lasts approximately 10 seconds and delivers all 20 ticks. About 99.6% of row positions change each tick; average rank displacement is 81.38 positions. This is actual sorting by the changing first metric, not simply changing text in fixed slots.

| Metric view | CPU time in 10 s, mean | Individual runs | Private memory, mean |
| --- | ---: | --- | ---: |
| Four native TextBlocks in Grid | 0.836 s | 0.922 / 0.750 s | 90.86 MiB |
| Four native TextBlocks positioned in Canvas | 0.969 s | 1.063 / 0.875 s | 91.89 MiB |
| One Image with DirectWrite-drawn metrics | 0.695 s | 0.750 / 0.641 s | 111.44 MiB |
| One native monospaced TextBlock | 0.719 s | 0.781 / 0.656 s | 89.84 MiB |

DirectWrite reduced total process CPU by 16.8% on the mean, with 14.6–18.6% improvements in the individual pairs. The absolute mean saving is 0.141 CPU seconds per 10 seconds, or about 1.4 percentage points of one CPU core. Private memory increased by 20.6 MiB. This is a modest benefit, not a demonstrated need for a custom table renderer.

The single native text control comes close to the same result without custom drawing. It is a lower-bound experiment: it cannot directly replace separately resized, aligned, colored or accessible metric cells. It indicates that reducing native control/layout count matters, rather than proving DirectWrite itself is uniquely responsible for the gain. Merely replacing the inner Grid with native Canvas was not consistently beneficial.

## Additional ranked workloads

Each mode below has one fresh-process run, so the percentages are exploratory.

| Metric view | Full shuffle: 2000 rows, 500 ms | Bounded jitter: 10000 rows, 100 ms |
| --- | ---: | ---: |
| Four native TextBlocks in Grid | 1.031 s CPU | 3.484 s CPU |
| Four native TextBlocks in Canvas | 0.797 s CPU | 3.688 s CPU |
| DirectWrite Image | 0.547 s CPU | 2.641 s CPU |
| One native TextBlock | 0.563 s CPU | 2.359 s CPU |

All runs delivered the expected 20 or 100 ranked ticks. Full shuffle changed 99.95% of positions, with average rank displacement 668.66. The 10000-row run changed 99.60% of positions, with average displacement 83.48.

In the 10000-row stress run, native Grid used 94.32 MiB, DirectWrite 114.95 MiB, and joined native text 90.55 MiB. Event-loop heartbeat p99 was 38.91, 34.81 and 34.06 ms respectively. This heartbeat is not presentation latency or an FPS measurement.

## Lifecycle and cost attribution

The fixed-order control updates approximately 34 realized rows per tick without creating new row components. All process values are nevertheless recalculated; the entire dataset is not kept stale offscreen.

With sorting, this tested VirtualSource integration creates and drops 35 row components per tick, despite stable process IDs as keys: 700 over the 20-tick runs, 3500 over the 100-tick runs. These are observations of this probe and integration, not a claim that native ItemsRepeater inherently requires full visible-row recreation or a proven renderer defect.

DirectWrite draw calls themselves took 48–55 ms across the entire 10-second primary ranked phase. Data generation and sorting took about 3–4 ms. Most total process CPU therefore lies outside these two timed sections, including row lifecycle, native integration, rendering and composition work. This suggests examining visible-row reuse and update topology before investing in a production custom text view; it is not a complete profiler attribution.

Surface attachment results are logged. Each DirectWrite ranked tick had 34 successful attachments and one unavailable attachment associated with the transient 35th component. A separate liveness validation run confirmed zero attachment errors for live components: all 20 unavailable responses in the ranked phase arrived after their component was destroyed, while 680 attachments succeeded. Its records are stored separately and are not included in the comparative means.

## Practical scope

The hybrid is feasible using an existing CanvasImageSource implementation: only the metric area is drawn, and row containers and scrolling remain native. The current primary measurements do not justify taking on a full custom renderer. If the production design needs independently accessible and styled cells, the single-text control is only a performance reference, and the DirectWrite view still needs accessibility, DPI and device-loss work.

This synthetic workload excludes icons, hierarchy/grouping, selection, process starts/exits and rich localization. All metric text uses the same numeric ASCII format and Consolas font. Shared-machine load is uncontrolled, and the two primary runs show meaningful variation. No production sources or vendor implementation were changed.

## Artifacts and verification

- results/measurements.json contains all 64 phase records from 16 successful ranked runs, with run IDs.
- results/summary.csv is generated by summarize.ps1 from phase 3 records.
- Original logs remain in target/table-probe-results/ranked.
- The matrix executable SHA-256 was 498EFCEB0DA2D0BF2678DBFFF25A4CCA20A11A2520218DD929491087AAAB2EAD. The final source adds attachment-liveness instrumentation; that validation run is kept separate from comparative timing records.
- Every run used the same Microsoft.ui.xaml.dll, file version 3.2.3.2608. No runtime upgrade was involved in comparisons.
- The probe and dependencies were built with optimization level 2. Formatting and Clippy for the probe pass; the canvas dependency emits its existing unexpected-cfg warnings.

The initial development runs with missing runtime, an invalid focus event or interruption are excluded. The comparative matrix completed normally, with exit code zero for all 16 runs.
