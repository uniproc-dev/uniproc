# Initial mount changes SVG quality

The measured initial-mount version of svg_sizes mounted both rows immediately. The current visual reproduction retains those rows and adds a third row, After 2 updates, created and mounted by the second queued component message. It has no timer, prewarm stage, environment switch or capture instrumentation. The recorded captures below predate that third row; they do not measure its quality.

A separate svg_initial_measurement binary uses the same initial image tree and captures all eight Image controls after four seconds. Unlike the visual reproduction, this isolated measurement has a focus button, capture instrumentation and an automatic exit. It requires WINUI_WINMD, like the other native measurements.

All controls reported their requested actual size and rasterization scale 1. At 192×192, both shared and independent outputs have 29,984 partially covered pixels and no fully opaque pixels. At 24×24 both have 354 partially covered pixels; at 48×48, 1,886; at 96×96, 7,497. Independent bytes do not avoid blur when mounted initially.

In the earlier delayed comparison the independent 192×192 image had 8,400 opaque pixels and 1,410 partially covered pixels. Those earlier captures remain valid for that loading sequence, but do not establish source sharing as the sole cause. The current issue draft and archive describe initial mounting explicitly. Exact decoder dimensions and the internal cause remain unmeasured.

[Immediate comparison](evidence/svg-initial-comparison.png). Earlier evidence is labeled separately in SVG-ISSUE.md.
