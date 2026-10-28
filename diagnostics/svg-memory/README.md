# SVG memory reproduction

This standalone Windows application depends only on published `windows-reactor-pre 0.106.0` and `windows-reactor-setup 0.100.0`. Its own Cargo workspace and lock file exclude Uniproc, Guinea, Guicons, agents, Canvas, settings and devtools. It creates 27 original circle SVGs, each with a different fill color and its own EncodedImage.

From this directory:

```powershell
cargo run --locked
```

The component starts with `shown = false`. Its constructor immediately queues one message with `cx.sender().send(())`; the update handler sets `shown = true` and displays all 27 SVG images. They remain on screen until the window closes. Every Image is 20×20. No timer or hide/show cycle is involved.

Every SVG has explicit intrinsic dimensions 24×24 and a `0 0 24 24` viewBox. The component retains the encoded sources while the window is open. There are no environment switches.

The earlier unconditional immediate-display control (commit `4469197`, with no state update) measured approximately 85 and 87 MiB. Those results are historical and do not describe this queued-update version.

The current timerless queued-update run stabilized at approximately 439 MiB through 12 seconds. Current and historical measurements, including earlier timer controls, are separated in RESULTS.md.

Measure **private bytes** for the corresponding process, independently of working set. For example, in another PowerShell window:

```powershell
Get-Process svg-memory-repro | Select-Object Id,PrivateMemorySize64,WorkingSet64
```

The investigation status is in [ISSUE.md](ISSUE.md). Current and historical measurements are separated in [RESULTS.md](RESULTS.md).
