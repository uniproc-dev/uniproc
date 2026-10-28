# SVG memory investigation: immediate-display control

## Problem

The current program immediately displays 27 SVG circles with different fill colors in 20×20 Image controls. Two fresh short runs stabilized at approximately 85 and 87 MiB of private memory. This immediate-display version did not reproduce the earlier hundreds-of-MiB result.

The earlier version first displayed an empty window and loaded these images after a timer; it measured approximately 438 MiB. Those historical measurements are kept separately. The current program is a control for loading order, not a successful reproduction of that high-memory case. The exact initialization condition behind the difference remains unmeasured.

## Reproduction

Run the attached standalone Cargo project with `cargo run --locked` on Windows. It requires only `windows-reactor-pre = 0.106.0` and setup 0.100.0. This is a preview repackaging, not an upstream version number: its package metadata identifies the microsoft/windows-rs source snapshot as `2c6ce7a`. A direct upstream checkout was not separately built for this report.

Full source revision, verified against the locally retained upstream snapshot: `2c6ce7a1edaefb6e585aa3fb112617d1d8211392`.

The component holds 27 distinct `EncodedImage::new` byte allocations. Each contains a circle with a different fill color, explicit intrinsic dimensions 24×24 and viewBox `0 0 24 24`. It creates images using:

```rust
Image::new()
    .width(20.0)
    .height(20.0)
    .source_svg(source.clone())
```

All 27 images appear immediately and remain displayed. The encoded sources remain owned by the component throughout the run. There is no timer or show/hide logic.

## Observations

| Current run | Stable private memory | Observation |
| --- | ---: | --- |
| First launch | 85.23 MiB | Responsive, no stderr errors, closed normally |
| Second launch | 87.06 MiB | Responsive, no stderr errors, closed normally |

The runs lasted approximately 12 seconds. Current measurements and historical controls are recorded separately in [RESULTS.md](RESULTS.md). Earlier show/hide and shared-source measurements do not describe the current program.

## Interpretation

Showing all images during initial construction changes the observed memory behavior compared with loading them after the window has already been displayed. Count, distinct SVG content and fixed control dimensions alone are therefore insufficient to reproduce the earlier result. The earlier high-memory measurements cannot be attributed to this simplified executable.

## Environment

- Windows 11 Home Single Language, build 26300, x64.
- `rustc 1.99.1 (58cf8f9c3 2026-10-13)`.
- Reactor 0.106.0; standalone lock uses windows-window-pre 0.104.0.
- Loaded Microsoft.ui.xaml.dll file version 3.2.3.2608.
- Loaded NVIDIA user-mode driver nvwgf2umx.dll version 32.0.15.8157.
- Measurements taken with `Process.PrivateMemorySize64`; working set is not treated as proof of a leak.
