# SVG memory: causal control, 28 October 2026

## Result

Explicit physical raster bounds set **before** `SvgImageSource.SetSourceAsync` remove the dominant memory increase. This was confirmed both through direct WinUI sources and in a temporary real Uniproc build. Merely specifying the `Image` layout dimensions does not have the same effect.

In the real application, bounding only the 27 main Tooling icons reduced final private memory from approximately 697 MiB to 298 MiB. Ten navigation cycles did not produce further sustained growth in either variant. This identifies the dominant retained cost in the supplied scenario; it is not a complete accounting of every byte in the earlier real-agent run.

## Direct WinUI control

`svg_native_memory` creates 27 independent native SvgImageSource objects, each loaded from an in-memory stream containing the original SVG circle. The controls are 20×20 DIP, at scale 1. It uses public `ElementRef<Image>::request_set_native_source` to bypass Reactor's native source creation and byte cache. Every loaded source returned success.

| Configuration | Before creation | After load/show | After hide, sources retained |
| --- | ---: | ---: | ---: |
| Default raster properties | 83.25 MiB | 438.26 MiB | 438.22 MiB |
| RasterizePixelWidth/Height = 20 before load | 81.89 MiB | 84.03 MiB | 83.99 MiB |
| Load sources without attaching any Image | 84.17 MiB | 296.11 MiB | 296.07 MiB |
| Activate sources, never load them | 83.44 MiB | 83.44 MiB | 83.40 MiB |

This separates image decoding from control count and from Reactor's cache. The smaller-memory variant still loads and displays the images successfully; it does not suppress the icons.

## Repeated destruction and recreation

The release control drops source owners on every hide, creates 27 new sources on every show, and closes its own window after six cycles. Private memory samples:

| Sampling time | Default raster | Explicit 20×20 raster |
| --- | ---: | ---: |
| 10 s, first show | 436.50 MiB | 84.03 MiB |
| 20 s, first hide | 227.15 MiB | 83.92 MiB |
| 30 s, second show | 508.96 MiB | 84.24 MiB |
| 40 s, second hide | 299.32 MiB | 83.84 MiB |
| 70 s, fourth show | 509.17 MiB | 84.02 MiB |
| 80 s, fourth hide | 299.85 MiB | 83.94 MiB |
| 110 s, sixth show | 501.56 MiB | 84.99 MiB |
| 120 s, sixth hide | 276.49 MiB | 85.14 MiB |

The default path warms additional retained resources during the first two cycles, then reaches a plateau in this run. It does not demonstrate unbounded per-cycle growth. After dropping the sources, private bytes alone cannot distinguish live native resources from memory retained by allocators or graphics caches.

HeapSummary succeeded for every heap returned by GetProcessHeaps. Ordinary heap allocated bytes remained approximately 19–21 MiB, with approximately 22–25 MiB committed. These counters do not account for all allocations in the process, so they must not be presented as total native live memory or as proof that all remaining private pages are free.

## Real Uniproc control

Two instances of the same temporary binary used an in-memory settings store, no devtools, and the synthetic 500-process Windows backend. Both ran Processes → System overview → Tooling → Processes ten times at five-second intervals. WSL remained enabled. The bounded variant changed only the 27 main tool icons: the same painted bytes were passed to native SvgImageSource with `ceil(logical_size × observed_scale)` raster bounds before loading. Other icons and feature behavior remained the same.

| Completed cycles | Original source path | Bounded tool SVG path |
| --- | ---: | ---: |
| 2 | 697.57 MiB | 296.96 MiB |
| 5 | 698.50 MiB | 297.46 MiB |
| 10, final idle sample | 696.59 MiB | 298.36 MiB |

The bounded run completed all 270 SVG loads without load or attachment errors. The scope-owned native sources were recreated when Tooling remounted. This temporary consumer adapter is an experimental control, not a permanent replacement for the renderer's image pipeline; it was removed from normal application sources after measurement. Its patch is retained locally under `target/uniproc-bounded-control.patch`.

## Mechanism and permanent fix boundary

WinUI's SVG metadata parser initializes the encoded dimensions large enough to cover the screen while preserving the document's aspect ratio; intrinsic SVG width/height do not directly constrain that initial metadata size. See [EncodedImageData::ParseSvg](https://github.com/microsoft/microsoft-ui-xaml/blob/8463f45162149de0ec3ad7df752596893fe3e13e/dxaml/xcp/components/imaging/EncodedImageData.cpp).

The SVG decoder uses explicit decode bounds when present, otherwise the metadata dimensions, and creates the software bitmap at that size. See [SvgImageDecoder::DecodeFrame](https://github.com/microsoft/microsoft-ui-xaml/blob/main/dxaml/xcp/components/imaging/SvgImageDecoder.cpp). The inspected upstream source explains the measured behavior; it is not a binary-level allocation trace of the installed WinUI runtime.

Reactor 0.106 creates and starts loading the native SVG source without a raster hint. Its native image cache shares sources by encoded-data identity and keeps static source entries alive for the window. This combines an expensive initial decode with long retention. The observed per-size blur is also relevant: native sources shared across different raster sizes cannot simply receive whichever consumer size was assigned last.

A permanent renderer fix needs a way to supply physical raster bounds before decoding and to include the effective raster size in native SVG source sharing. Automatic sizing must also handle the time before layout is known, changes of DPI, and the distinction between logical layout size and physical decode size. Those requirements belong at the renderer's source/decode boundary. No vendor source or neighboring repository was modified by these experiments, and no issue was published.
