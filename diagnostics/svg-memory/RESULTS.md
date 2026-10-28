# SVG memory measurements

## Current timerless queued update, 28 October 2026

The component starts with `shown = false`. Its constructor queues one message with `cx.sender().send(())`, and the update handler only sets `shown = true`. All 27 differently colored SVGs are then displayed and retained. No timer, delay, rescheduling, hide/show cycle or environment switch is used.

| Sampling time | Private MiB |
| --- | ---: |
| 3 s | 439.11 |
| 6 s | 439.11 |
| 10 s | 439.11 |
| 12 s | 439.11 |

The process remained responsive, stderr was empty and the window closed normally after measurement. Build, formatting and Clippy passed. A timer is not required for this high-memory case: one queued state update is sufficient. The exact native initialization condition remains unmeasured.

## Historical single-step timer, 28 October 2026

Commit `351d3ff` used one timer firing after five seconds. The update handler only changed `shown` from false to true without scheduling another message. All 27 differently colored SVGs remained displayed afterward. These are historical timer measurements, not measurements of the current timerless executable.

| Sampling time | State | Private MiB |
| --- | --- | ---: |
| 3 s | Initial empty window | 83.73 |
| 10 s | Images displayed after the only update | 438.74 |
| 16 s | Still displayed, no further updates | 438.74 |
| 21 s | Still displayed, no further updates | 438.81 |

The process remained responsive, stderr was empty, and the window closed normally after sampling. Build, formatting and Clippy passed. A single deferred display is sufficient to reproduce the large increase; repeated hide/show transitions are not required.

## Historical restored two-cycle timer, 28 October 2026

Commit `1c7b874` started empty, showed all 27 differently colored SVGs after five seconds, then hid, showed and hid the controls at ten-second intervals. These are historical measurements of the two-cycle version, not the current single-step timer. The encoded sources remained owned throughout. Intrinsic SVG dimensions were 24×24, Image dimensions 20×20.

Fresh verification after restoring the timer:

| Sampling time | State | Private MiB |
| --- | --- | ---: |
| 3 s | Initial empty window | 83.58 |
| 10 s | First show | 493.98 |
| 20 s | First hide | 493.98 |
| 30 s | Second show | 493.99 |
| 42 s | Final hide | 493.97 |

All four transitions completed, the window remained responsive, stderr contained only transition messages, and the diagnostic process closed normally. Build, formatting and Clippy passed. The large memory increase returned with deferred loading. Absolute values differ from earlier runs; they are not an exact per-source allocation estimate.

## Historical immediate-display control, 28 October 2026

Commit `4469197` displayed all 27 differently colored SVGs immediately. Probe owned only its encoded sources; there was no timer, step counter, update handler, placeholder or environment switch. These are historical control measurements, not measurements of the restored timer-driven version.

| Fresh run | Stable private memory | Duration |
| --- | ---: | ---: |
| First | 85.23 MiB | Approximately 12 s |
| Second | 87.06 MiB | Approximately 12 s |

Both windows stayed responsive, emitted no stderr errors and were closed normally after measurement. Build, formatting and Clippy passed. The earlier ~438 MiB high-memory result was not reproduced by this immediate-display version. This difference indicates a dependence on when the sources are loaded; it does not establish the exact native initialization mechanism.

## Historical colored show/hide reproduction, 28 October 2026

Commit `130ec29` generated 27 SVG circles with different fill colors, each in its own EncodedImage. Intrinsic dimensions were 24×24, the viewBox was `0 0 24 24`, and every Image was 20×20. The component held all encoded sources throughout two timer-driven show/hide cycles. This timer behavior has now been restored; the measurements below remain the earlier run and are kept separately from the fresh verification.

Fresh run of this version, private memory:

| Sampling time | State | Private MiB |
| --- | --- | ---: |
| 3 s | Initial empty window | 83.23 |
| 10 s | First show | 437.90 |
| 20 s | First hide | 437.98 |
| 30 s | Second show | 437.98 |
| 42 s | Final hide | 437.98 |

All four timer transitions completed and the window closed normally after measurement. Build, formatting and Clippy checks passed. This confirms that the memory increase also reproduces with different SVG content and explicit numeric intrinsic dimensions. It does not establish unbounded growth.

## Historical measurements, 27 October 2026

All measurements below were collected with the earlier reproduction, before simplification. Its distinct source allocations contained identical white-circle SVG bytes; separate shared-source, intrinsic-dimension and owner-release controls were also run. These are historical controls, not measurements of the current 27 differently colored SVGs. The earlier source is available in commit `92705c5`.

### Uniproc reproduction

The supplied scenario was automated in a real WinUI window: Processes → System overview → Tooling → Processes, twice, at ten-second intervals. Temporary instrumentation used an in-memory settings store, disabled single-instance exclusion and devtools, and generated 500 Windows process rows. WSL was not removed; its connection differed between runs, so the result is an ablation rather than a perfectly isolated allocation trace.

| Configuration | Before Tooling | First Tooling | After second circle |
| --- | ---: | ---: | ---: |
| Original 27 main tool icons | ~233 MiB private | ~675 MiB | ~700 MiB |
| Only those icons replaced by empty 20×20 grids | ~233 MiB private | ~266 MiB | ~301 MiB |

The second run retained the remaining SVGs, card controls and callbacks. This attributed the dominant difference to the main tool icon path; the standalone reproduction then removed the application and framework confounders.

The profiled original-icons run allocated 161,098,584 bytes cumulatively, with **4,321,535 live bytes at the global Rust heap peak**, and 505,735 at profiler termination. These cumulative and live quantities must not be interchanged. In the earlier real-agent profile the corresponding values were 1,029,079,425 total, 15,634,631 at the global peak, and 716,429 at termination. The user observed approximately 800 MB in that same earlier profiled process.

### Standalone confirmation

The earlier standalone project built against registry Reactor 0.106.0 with an independent Cargo.lock. It displayed identical white-circle SVGs and text. At stable samples:

| Time from start | Default distinct sources, private MiB | Release source owners on hide, private MiB |
| --- | ---: | ---: |
| 5 s, before first show completes | 85.6 | 84.0 |
| 10 s, first show | 503.3 | 501.7 |
| 20 s, first hide | 503.4 | 292.4 |
| 30 s, second show | 503.4 | 502.5 |
| 42 s, final hide | 503.4 | 292.9 |

Times are approximate because the sampler and UI timer run independently. The release control demonstrates partial recovery, not complete native memory accounting. These samples do not distinguish live native allocations from pages retained by Windows allocators/graphics infrastructure.

The independent registry executable was also tested with one source/image (~101 MiB private) and one shared source across 27 controls (~102 MiB private). Both remained at those levels across the two mount/unmount cycles.

Earlier consumer-example builds measured ~102 MiB with either one source/image or one shared source across 27 controls. Distinct sources measured ~446 MiB with `1em` dimensions and ~487 MiB with numeric dimensions. Numeric intrinsic SVG dimensions did not fix the problem. Absolute values differ between runs; these observations do not establish an exact per-source allocation size.

The tested dependency is the published preview package `windows-reactor-pre 0.106.0`, not an upstream 0.106 release. Its package metadata identifies the source snapshot as microsoft/windows-rs `2c6ce7a`. The direct upstream commit was not separately built.

### Historical conclusions and open questions

The dominant navigation-related increase reproduces without Uniproc domain code, Guinea, Guicons or agents. It scales with distinct SVG sources and remains cached while encoded source owners remain alive. Application-lifetime static icon bytes make this retention relevant even after a page is unmounted.

The exact native allocation mechanism, actual rasterization dimensions, and the split between Reactor caching, WinUI decoding and graphics-driver allocation remain unmeasured. The reproduction supports an upstream memory-usage issue; it does not establish an indefinitely growing leak or account for every byte in the earlier ~800 MB process.

An initial broad WPR recording lost 448,552 events and was not used for quantitative attribution. A later heap-snapshot recording had no reported loss, but did not cover the navigation reproduction. Neither trace is sufficient to assert a native call-stack diagnosis. Raw traces, profiles and machine-specific paths are excluded from this directory.

Temporary production instrumentation and icon ablation were removed after the experiments. No vendor source was modified.
