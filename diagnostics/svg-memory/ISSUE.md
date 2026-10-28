# SVG memory increases when sources load after initial display

## Problem

The current program initially displays an empty window, then shows 27 SVG circles with different fill colors in 20×20 Image controls after five seconds. The timer fires only once; the images remain displayed and the encoded sources remain owned until the window closes.

The earlier immediate-display control (commit `4469197`) stabilized at approximately 85 and 87 MiB, whereas the two-cycle timer version (`1c7b874`) measured approximately 494 MiB. Fresh verification of the single-step timer is recorded separately in RESULTS.md. The exact native initialization condition behind this loading-order difference remains unmeasured.

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

After five seconds all 27 images appear and remain displayed. The update handler only sets `shown = true`; it does not schedule further messages. There are no environment switches.

## Observations

The fresh single-step run measured 83.73 MiB before display, 438.74 MiB after the only update, and 438.81 MiB at 21 seconds. The process remained responsive and emitted no stderr errors. No additional timer messages or hide/show transitions were involved.

Current single-step measurements and historical controls are recorded separately in [RESULTS.md](RESULTS.md). Immediate-display and repeated hide/show controls are historical and do not describe the current executable.

## Interpretation

Showing all images during initial construction changes the observed memory behavior compared with loading them after the window has already been displayed. Count, distinct SVG content and fixed control dimensions alone are therefore insufficient to describe the triggering condition; source-loading timing also matters.

## Environment

- Windows 11 Home Single Language, build 26300, x64.
- `rustc 1.99.1 (58cf8f9c3 2026-10-13)`.
- Reactor 0.106.0; standalone lock uses windows-window-pre 0.104.0.
- Loaded Microsoft.ui.xaml.dll file version 3.2.3.2608.
- Loaded NVIDIA user-mode driver nvwgf2umx.dll version 32.0.15.8157.
- Measurements taken with `Process.PrivateMemorySize64`; working set is not treated as proof of a leak.
