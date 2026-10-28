# SVG edge quality: native automatic raster size

Measured on 28 October 2026 with the published Reactor preview used by this workspace. All four images use identical original SVG bytes, 192×192 DIP Image controls and rasterization scale 1.

| Image | Source creation | Partial-alpha pixels | Opaque pixels |
| --- | --- | ---: | ---: |
| 0 | Reactor source_svg, automatic raster size | 1,410 | 8,400 |
| 1 | Direct native SvgImageSource, automatic raster size | 1,410 | 8,400 |
| 2 | Direct native SvgImageSource, raster size 192×192 before load | 2,107 | 7,987 |
| 3 | Direct native SvgImageSource, raster size 384×384 before load | 2,185 | 7,971 |

Images 0 and 1 are pixel-identical after compositing onto white. Both reproduce the harsh staircase edges. Explicit raster dimensions produce visibly smoother boundaries without changing SVG geometry, layout size or the Image control. Supersampling at 384 gives another control, but is not necessary to demonstrate the difference: 192 already improves the edges.

The user's 50×38 screenshot matches the earlier independent 192×192 image at crop (96, 28), with mean absolute RGB difference 0.00263 on a 0–255 scale. The artifact therefore already appears in the image exported through RenderTargetBitmap and does not require sharing one source across different sizes.

This isolates the observed quality difference to native automatic raster sizing in the tested loading sequence. Antialiasing is not completely absent: partially covered edge pixels exist in the automatic case. The experiment does not identify the internal automatic decode dimensions or resampling filter. Counts describe the exported output, not a direct readback of the decoder's bitmap.

Reactor creates SvgImageSource and loads the bytes with SetSourceAsync without setting raster dimensions. Upstream WinUI's [SvgImageDecoder](https://github.com/microsoft/microsoft-ui-xaml/blob/8463f45162149de0ec3ad7df752596893fe3e13e/dxaml/xcp/components/imaging/SvgImageDecoder.cpp) draws through Direct2D into a bitmap whose dimensions come from metadata or explicit decode dimensions. This source explains where the size enters the pipeline; it is not runtime tracing of the installed decoder.

## Run the isolated measurement

```powershell
$env:WINUI_WINMD = 'D:/projects/windows-rs/crates/tools/reactor/winmd'
cargo run --locked --bin svg_antialias_control
```

The diagnostic loads one Reactor source and three native sources, captures its own Image controls, writes four PNGs and exits. Native source loading must report success. Capture logs must report actual size 192×192 and scale 1 before comparing against the recorded measurements. The native sources bypass Reactor's source creation and byte cache through ElementRef. No Guinea dependency is involved.

The separate visual `svg_sizes` reproduction has not been changed or given capture instrumentation.

For a visual window that stays open, use `cargo run --locked --bin svg_antialias_preview` with the same metadata environment. It has four labeled columns and no native capture/focus instrumentation. The independently built [standalone package](attachments/svg-antialias-repro.zip) contains this visual comparison and the [issue draft](ANTIALIAS-ISSUE.md).

[Full images](evidence/antialias-comparison.png). [Edges at 4× nearest-neighbor magnification](evidence/antialias-edges.png); magnification only makes individual output pixels visible.

Verification: the native control builds and runs successfully, all three native loads succeed, and all four captures complete. Explicit native bounds are a diagnostic control, not a production fix. A renderer change still needs size/DPI-aware source handling, including multiple consumers of the same SVG.
