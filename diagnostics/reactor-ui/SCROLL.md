# Downward scroll investigation and Uniproc fix

The native table in `crates/ui/src/widgets/table/rows.rs` advertised a row count rounded up to a multiple of 16. Indices beyond the real rows returned an empty Border. This introduced zero-height items that were not part of the domain data.

The independent `scroll` example reproduced large backward movements when the end of this source entered the realization range. It used a native ScrollViewer, Reactor ItemsRepeater and fixed 32 DIP real rows. Data views were refreshed once per second. Automatic animated ChangeView calls requested increasing offsets until reaching the estimated end.

Final paired run on 28 October 2026:

| Input | Backward offset events over 0.25 DIP | Final offset |
| --- | ---: | ---: |
| Exactly 1001 real rows | 0 | 31584 |
| 1024 advertised items, first 1001 real and 23 zero-height placeholders | 36 | 7325.18 |

In the padded case, the first backward event occurred during a forward request from 30720 toward 32320: the observed offset fell from 31234.91 to 24822. Earlier paired runs reproduced the same collapse. Subsequent requests were clamped against the already reduced extent, so they are not all independent evidence of backward motion under increasing targets.

The exact-count control settled at `(1001 × 32) − 448 = 31584`, retained the expected extent and produced no settled row-position errors. Direct fixed-height row views, child component views and a non-virtual StackPanel also had monotonic offsets in the earlier ordinary-scroll controls.

The Uniproc fix removes the padding/bucket mechanism. VirtualSource now receives the actual count and no longer advertises phantom rows. Workspace verification: 522 tests passed, one ignored, with `cargo test --workspace --locked --offline`.

This establishes a concrete consumer defect and its associated end-of-list jitter. It does not prove that every possible scroll animation symptom has the same cause. The behavior of genuine zero-height items may still deserve upstream investigation; a direct native WinUI ItemsRepeater control has not been run, so Reactor versus WinUI responsibility remains open.
