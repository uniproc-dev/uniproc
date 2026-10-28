# SVG memory increases when sources load after initial display

## Problem

The current program initially displays an empty window, then shows 27 SVG circles with different fill colors in 20×20 Image controls after five seconds. Private memory rises from approximately 84 to 494 MiB. The timer then hides, shows and hides them at ten-second intervals, retaining the encoded sources throughout.

The earlier immediate-display control (commit `4469197`) stabilized at approximately 85 and 87 MiB, whereas the earlier timer-driven version measured approximately 438 MiB. Fresh verification of the restored timer is recorded separately in RESULTS.md. The exact native initialization condition behind this loading-order difference remains unmeasured.

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

After five seconds all 27 images appear; subsequent transitions every ten seconds hide, show and hide the controls. The encoded sources remain owned by the component throughout the run. There are no environment switches.

## Observations

| Current restored-timer run | Private memory |
| --- | ---: |
| Initial empty window | 83.58 MiB |
| First show | 493.98 MiB |
| First hide | 493.98 MiB |
| Second show | 493.99 MiB |
| Final hide | 493.97 MiB |

Current measurements and historical controls are recorded separately in [RESULTS.md](RESULTS.md). The immediate-display control is historical and does not describe the restored timer-driven executable.

## Interpretation

Showing all images during initial construction changes the observed memory behavior compared with loading them after the window has already been displayed. Count, distinct SVG content and fixed control dimensions alone are therefore insufficient to describe the triggering condition; source-loading timing also matters.

## Environment

- Windows 11 Home Single Language, build 26300, x64.
- `rustc 1.99.1 (58cf8f9c3 2026-10-13)`.
- Reactor 0.106.0; standalone lock uses windows-window-pre 0.104.0.
- Loaded Microsoft.ui.xaml.dll file version 3.2.3.2608.
- Loaded NVIDIA user-mode driver nvwgf2umx.dll version 32.0.15.8157.
- Measurements taken with `Process.PrivateMemorySize64`; working set is not treated as proof of a leak.
