# Scroll extent collapses when zero-height items are realized at the end of a virtual source

Investigation draft: the behavior reproduces through Reactor, but a direct native WinUI ItemsRepeater comparison has not been run. Reactor ownership is not established.

## Reproduction

Use the attached standalone `scroll` binary with published `windows-reactor-pre 0.106.0` (preview source snapshot `2c6ce7a` according to package metadata).

Source revision: `2c6ce7a1edaefb6e585aa3fb112617d1d8211392`; a direct upstream dependency was not separately built.

```rust
ItemsRepeater::new().virtual_source(VirtualSource::new(
    0,
    1024,
    |index| index,
    |index| {
        if index < 1001 {
            Border::new().height(32.0).content(TextBlock::new().text(index.to_string())).into()
        } else {
            Border::new().into()
        }
    },
))
```

Place it inside a ScrollViewer with a finite viewport and scroll toward the end. The example supports this with `SCROLL_MODE=root`, `SCROLL_COUNT=1024`, `SCROLL_PADDING=1`. A child-component variant uses `SCROLL_MODE=component`.

Control: advertise exactly 1001 items and omit the zero-height tail. The real rows and their order remain the same.

## Observed behavior

In the instrumented component run, the first forward request toward 32320 produced a backward offset transition from 31234.91 to 24822 while the estimated extent changed sharply. The source remained unchanged. Repeated attempts to reach the end continued to change the extent and position.

The exact-count control had no backward offset events and settled at 31584 with a 448 DIP viewport. The complete paired results are in [SCROLL.md](SCROLL.md).

## Question

Is the zero-height-tail case supported by the native virtualization path, and should the estimated extent converge without large backward jumps? Please compare it with a direct native WinUI implementation before attributing the defect to Reactor.

The original consumer should not have padded its source this way; that padding was removed separately. This report preserves the smaller input case for diagnosing the underlying virtualization behavior.
