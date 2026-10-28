# SVG rasterization versus bitmap reduction

## Measured result

The staircase artifact can be reproduced with direct native SvgImageSource, independently of Reactor's source cache. Increasing the raster size is not monotonically better: a 3072×3072 raster displayed in a 192×192 Image has harsher boundaries than a raster made directly at 192×192.

All small Image controls reported actual size 192×192 and rasterization scale 1. Each native source had its raster dimensions set before loading; every load succeeded. The same original SVG geometry was used throughout.

| Raster dimensions | Partial-alpha pixels in 192 output | Opaque pixels |
| --- | ---: | ---: |
| Automatic, Reactor | 1,410 | 8,400 |
| Automatic, direct native | 1,410 | 8,400 |
| 24×24 | 29,984 | 0 |
| 96×96 | 10,142 | 3,859 |
| 192×192 | 2,107 | 7,987 |
| 384×384 | 2,185 | 7,971 |
| 768×768 | 1,605 | 8,290 |
| 1536×1536 | 1,402 | 8,408 |
| 3072×3072 | 424 | 8,470 |

These counts describe coverage, not a general image-quality score. The visual boundaries and controlled resampling comparison establish the particular artifact.

## Isolating the reduction

An additional native Image displays an independently loaded 3072×3072 source at 3072×3072. It is arranged inside Canvas to avoid a layout clip from the small comparison cells. RenderTargetBitmap exported the complete image; its nontransparent bounds are (64, 64)–(3008, 3008), consistent with the SVG geometry. A preceding attempt arranged inside a narrow cell was clipped and was excluded from analysis.

The full-size image was composited onto white and reduced analytically from 3072 to 192. Fixed-footprint bilinear sampling uses source coordinates `16*x + 7.5`, `16*y + 7.5`: four neighboring source pixels contribute equally to each output pixel. This reconstruction matches the exported native 3072→192 output with mean absolute RGB error 0.0022403 and maximum channel error 0.95589 on a 0–255 scale. No pixel differs by more than one channel level.

By contrast, reducing that same full-size image with area averaging produces smoother boundaries. Rendering directly at 192 also gives smoother boundaries. This isolates a loss of edge coverage during strong bitmap reduction in the measured path; it does not require antialiasing to be disabled in the SVG rasterizer.

The measurement covers exported Image output. It is not a GPU instruction trace or a readback of WinUI's internal decoder bitmap. In particular, it does not establish the exact dimensions of the automatic raster or assign the source-sharing defect to a specific native component. The observed fixed-footprint bilinear behavior is established numerically for the explicit 3072→192 control.

## Source context

WinUI's [SvgImageDecoder](https://github.com/microsoft/microsoft-ui-xaml/blob/8463f45162149de0ec3ad7df752596893fe3e13e/dxaml/xcp/components/imaging/SvgImageDecoder.cpp) renders the SVG through Direct2D into a WIC bitmap, choosing explicit decode dimensions when present and otherwise using metadata dimensions. [EncodedImageData::ParseSvg](https://github.com/microsoft/microsoft-ui-xaml/blob/8463f45162149de0ec3ad7df752596893fe3e13e/dxaml/xcp/components/imaging/EncodedImageData.cpp) can derive metadata dimensions from the root bounds rather than the eventual Image dimensions. This is source-level context, not tracing of the installed runtime.

The practical capability demonstrated here is specifying raster dimensions near the physical display dimensions before decoding. Arbitrarily increasing them can worsen reduction artifacts. A renderer API must also account for DPI and multiple consumers; this experiment does not prescribe a final API or solve the shared-source problem.

## Reproduce

For a persistent visual window, run `cargo run --locked --bin svg_raster_preview` with WINUI_WINMD configured. It compares Reactor automatic, native automatic and native raster sizes 24, 96, 192, 384, 768, 1536 and 3072; all Image controls display at 192×192. This visual program has no capture or focus instrumentation and does not close automatically.

Configure WINUI_WINMD as described in README.md, then run `cargo run --locked --bin svg_raster_measurement`. It loads the comparison after one second, captures after another four seconds and exits after exporting ten PNGs. The final image is 3072×3072; the others are 192×192. This measurement is separate from the interactive svg_sizes reproduction, which remains unchanged.

[Raster size sweep](evidence/raster-sweep.png). [Reduction comparison](evidence/raster-reduction.png). Edge crops are enlarged 4× with nearest-neighbor sampling to expose individual output pixels.
