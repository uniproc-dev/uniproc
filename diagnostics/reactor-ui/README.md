# Reactor scroll and SVG reproductions

[RASTERIZER.md](RASTERIZER.md) isolates harsh edges during strong native bitmap reduction, with a raster-size sweep and a numerical bilinear-sampling control. This is separate from source-sharing behavior.

Independent Cargo workspace using published `windows-reactor-pre 0.106.0`, a preview repackaging whose package metadata identifies microsoft/windows-rs snapshot `2c6ce7a`. It does not link Uniproc or Guinea. The SVG drawing is original.

The full source revision is `2c6ce7a1edaefb6e585aa3fb112617d1d8211392`, verified against the locally retained upstream snapshot. The experiments use the published preview package, not a separately built direct-upstream dependency.

## Prepared issue attachments

| Report | Standalone source archive |
| --- | --- |
| [Shared source becomes blurry across sizes](SVG-ISSUE.md) | [svg-shared-size-repro.zip](attachments/svg-shared-size-repro.zip) |
| [Explicit raster dimensions improve edge quality](ANTIALIAS-ISSUE.md) | [svg-antialias-repro.zip](attachments/svg-antialias-repro.zip) |

Each archive contains its own Cargo package, Cargo.lock, README, issue draft and evidence. Extract into an empty directory. The shared-source package needs only the published dependencies; the native comparison additionally generates bindings from the WinUI metadata directory described in its README. Both packages were independently built, linted and launched; all three native source loads succeeded. No application services or Guinea checkout are required. The drafts have not been published.

## SVG source shared across sizes

```powershell
cargo run --locked --bin svg_sizes
```

The first two rows appear immediately: Shared uses one SVG source at 24, 48, 96 and 192 DIP, and Independent uses separate byte allocations. A third row, After update, creates and mounts independent sources when the first queued component message is handled. There is no prewarm phase or environment switch. Earlier measurements found blur in both initial rows and a sharper independent source in a five-second delayed-mount control; that timing is different from this new queued-update control.

[SVG-ISSUE.md](SVG-ISSUE.md) is the upstream issue draft. [evidence/svg-comparison.png](evidence/svg-comparison.png) shows the measured difference.

[ANTIALIAS.md](ANTIALIAS.md) records a separate native control for harsh edge quality with automatic raster dimensions. Its measurement binary is `svg_antialias_control`; the visual `svg_sizes` example remains unchanged.

For a persistent visual comparison of Reactor auto, native auto, native raster 192 and native raster 384, configure `WINUI_WINMD` as below and run `cargo run --locked --bin svg_antialias_preview`. This binary has no capture/focus instrumentation and does not close its window automatically. [ANTIALIAS-ISSUE.md](ANTIALIAS-ISSUE.md) is the separate API request draft, with native behavior and limits of attribution stated explicitly.

## Virtual list

```powershell
cargo run --locked --bin scroll
```

The default is an ItemsRepeater with 1024 fixed-height rows rendered through child components. `SCROLL_MODE=root` returns direct row views; `SCROLL_MODE=stack` uses a non-virtualized StackPanel. `SCROLL_COUNT=1001` changes the virtual source count. `SCROLL_PADDING=1` makes indices 1001 and above empty, zero-height borders, reproducing Uniproc's former padded source. Run the padded case with count 1024 and the control with count 1001 and no padding.

[SCROLL.md](SCROLL.md) records the actual cause found in Uniproc and the fix. [SCROLL-ISSUE.md](SCROLL-ISSUE.md) is a separate draft for investigating the underlying zero-height-item behavior; its native WinUI ownership is not established.

## Optional automatic measurements

The examples run without native instrumentation. To compile automatic scrolling and pixel export, provide the WinUI metadata directory from a windows-rs checkout:

```powershell
$env:WINUI_WINMD = '<windows-rs>/crates/tools/reactor/winmd'
cargo build --locked
```

Older checkouts use `crates/tools/reactor-metadata/winmd`. Binding generation writes only to Cargo's build output. The native code uses public WinUI interfaces inside the diagnostic application's own visual tree. It does not inspect other applications or modify vendor source.

The scroll probe focuses its own header button, finds its own ScrollViewer, records ViewChanged offsets, and requests an animated downward ChangeView every second. `SCROLL_STRIDE` selects the increment in DIP; the measured end-of-list runs used 2048. Stop after approximately 24 seconds and compare `OFFSET` records. `ROW_ERROR` checks run only at settled offsets; intermediate compositor transforms and UI-thread offsets need not be synchronized.

The SVG preview is a visual comparison. The evidence PNGs were exported by an earlier instrumented version through RenderTargetBitmap, with premultiplied BGRA converted to straight RGBA. The current SVG binary has no native capture calls, focus instrumentation or polling timer.

## Native SVG memory control

[MEMORY.md](MEMORY.md) records the causal bounds experiment and the real Uniproc comparison. With `WINUI_WINMD` configured, run `cargo run --locked --bin svg_native_memory`.

| Environment | Control |
| --- | --- |
| `MEMORY_RASTER=20` | Set physical raster width and height before loading. Unset means native defaults. |
| `MEMORY_NO_ATTACH=1` | Load native sources without mounting Image controls. |
| `MEMORY_NO_LOAD=1` | Only activate native sources. |
| `MEMORY_RELEASE=1` | Drop source owners on every hide and recreate them on the next show. |
| `MEMORY_CYCLES=6` | Number of show/hide cycles; default two. |

Unset controls from previous runs before comparing. The process prints load outcomes, phase changes and ordinary heap summary counters, then closes its own window when the cycles finish. Sample private memory externally; the heap counters do not cover every allocator in the process.

Build and lint verification:

```powershell
cargo fmt -- --check
cargo clippy --locked --all-targets -- -D warnings
```

The current visual reproduction also includes a fourth row, After 1 second: independent SVG sources are created and mounted by a one-shot one-second timer. The existing queued-update row is retained. Earlier pixel measurements predate this fourth row.

A fifth row, Reused after 1s, mounts after the same timer using the existing self.fresh sources from the second row, without creating new source bytes. The five-row grid is now inside a native ScrollViewer so the last row remains accessible; earlier measurements used the previous layout.
