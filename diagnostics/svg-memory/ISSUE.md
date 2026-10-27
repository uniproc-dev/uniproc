# Small SVG images retain hundreds of MiB with distinct encoded sources

## Problem

With Reactor 0.106.0, 27 SVG sources containing a single circle, displayed in `Image` controls explicitly sized 20×20, increase process private memory by roughly 418 MiB in a standalone reproduction. Removing the controls does not release that increase while the component still owns the tiny encoded byte arrays. The second mount/unmount does not cause another comparable increase.

This is an excessive memory usage / image-cache retention report, not a claim of a demonstrated unbounded leak. The reproduction does not establish the exact native allocation size, raster dimensions, or which lower layer is responsible.

## Reproduction

Run the attached standalone Cargo project with `cargo run --locked` on Windows. It requires only `windows-reactor-pre = 0.106.0` and setup 0.100.0; there is no Guinea, Guicons, Canvas, telemetry, process monitoring or application settings.

The component holds 27 distinct `EncodedImage::new` byte allocations. Each contains the same original SVG circle. It creates images using:

```rust
Image::new()
    .width(20.0)
    .height(20.0)
    .source_svg(source.clone())
```

After 5 seconds the images appear; subsequent transitions every 10 seconds remove, recreate and remove the image controls. The tiny encoded sources remain owned by the component in the default run.

## Observations

Measured private memory, rounded MiB:

| Run | Empty window | Images mounted | After unmount |
| --- | ---: | ---: | ---: |
| Standalone registry 0.106.0, 27 distinct sources | 86 | 503 | 503 |
| Same standalone, release byte owners on hide | 84 | 502 | 292–293 |

Additional controls with the same component compiled as an example against the consumer's unchanged Reactor 0.106.0 checkout:

| Control | Private memory after mount |
| --- | ---: |
| 27 distinct sources, SVG dimensions `1em` | 446 MiB |
| 27 distinct sources, SVG dimensions `24` | 487 MiB |
| One source and one image | 102 MiB |
| One shared source and 27 images | 102 MiB |

The difference between builds/runs is why these are not presented as a precise per-source allocation size. The large increase reproduces independently against the published registry crate.

## Expected behavior / investigation request

Small fixed-size SVG icons should not add hundreds of MiB of private memory. Please investigate the native SVG decode/rasterization path and the lifetime/budget of cached native image sources. Is an explicit rasterization size required in addition to setting the `Image` dimensions, and should it participate in source caching?

Reactor's native SVG creation path in the consumer checkout creates `SvgImageSource` and calls `SetSourceAsync`; no explicit rasterization dimensions are supplied there. This is a candidate mechanism, not proof of the internal surface dimensions or a proposed patch.

## Environment

- Windows 11 Home Single Language, build 26300, x64.
- `rustc 1.99.1 (58cf8f9c3 2026-10-13)`.
- Reactor 0.106.0; standalone lock uses windows-window-pre 0.104.0.
- Loaded Microsoft.ui.xaml.dll file version 3.2.3.2608.
- Loaded NVIDIA user-mode driver nvwgf2umx.dll version 32.0.15.8157.
- Measurements taken with `Process.PrivateMemorySize64`; working set is not treated as proof of a leak.

No raw application heap profile or system trace is needed to reproduce this issue.
