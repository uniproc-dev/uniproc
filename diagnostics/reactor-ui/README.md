# Reactor scroll and SVG reproductions

Independent Cargo workspace using published `windows-reactor-pre 0.106.0`, a preview repackaging whose package metadata identifies microsoft/windows-rs snapshot `2c6ce7a`. It does not link Uniproc or Guinea. The SVG drawing is original.

The full source revision is `2c6ce7a1edaefb6e585aa3fb112617d1d8211392`, verified against the locally retained upstream snapshot. The experiments use the published preview package, not a separately built direct-upstream dependency.

## SVG source shared across sizes

```powershell
cargo run --locked --bin svg_sizes
```

After five seconds the top row displays one SVG source shared between 24, 48, 96 and 192 DIP images. The bottom row displays independent byte allocations of the same SVG, one per image. Compare the two 192 DIP images. Set `SVG_LARGE_FIRST=1` in a fresh process to prewarm the shared source at 192 instead of 24.

[SVG-ISSUE.md](SVG-ISSUE.md) is the upstream issue draft. [evidence/svg-comparison.png](evidence/svg-comparison.png) shows the measured difference.

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
