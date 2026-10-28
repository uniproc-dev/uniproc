# Small SVG images retain hundreds of MiB with distinct encoded sources

## Problem

With `windows-reactor-pre 0.106.0`, 27 independent byte allocations containing the same SVG circle, displayed in `Image` controls explicitly sized 20×20, increase process private memory by roughly 418 MiB. Removing the controls does not release that increase while the component still owns the tiny encoded byte arrays. The second mount/unmount does not cause another comparable increase.

This is an excessive memory usage / image-cache retention report, not a claim of a demonstrated unbounded leak. The reproduction does not establish the exact native allocation size, raster dimensions, or which lower layer is responsible.

## Reproduction

Run the attached standalone Cargo project with `cargo run --locked` on Windows. It requires only `windows-reactor-pre = 0.106.0` and setup 0.100.0. This is a preview repackaging, not an upstream version number: its package metadata identifies the microsoft/windows-rs source snapshot as `2c6ce7a`. A direct upstream checkout was not separately built for this report.

Full source revision, verified against the locally retained upstream snapshot: `2c6ce7a1edaefb6e585aa3fb112617d1d8211392`.

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
| Same standalone, one source and one image | ~84 | 101 | 101 |
| Same standalone, one shared source and 27 images | ~84 | 102 | 102 |

The SVG uses `width="1em" height="1em" viewBox="0 0 24 24"`. A separate control using numeric intrinsic dimensions `24` still reproduced the large increase. The attached project exposes this through `SVG_PIXELS=1`.

## Expected behavior / investigation request

Small fixed-size SVG icons should not add hundreds of MiB of private memory. Please investigate the native SVG decode/rasterization path and the lifetime/budget of cached native image sources. Is an explicit rasterization size required in addition to setting the `Image` dimensions, and should it participate in source caching?

## Environment

- Windows 11 Home Single Language, build 26300, x64.
- `rustc 1.99.1 (58cf8f9c3 2026-10-13)`.
- Reactor 0.106.0; standalone lock uses windows-window-pre 0.104.0.
- Loaded Microsoft.ui.xaml.dll file version 3.2.3.2608.
- Loaded NVIDIA user-mode driver nvwgf2umx.dll version 32.0.15.8157.
- Measurements taken with `Process.PrivateMemorySize64`; working set is not treated as proof of a leak.
