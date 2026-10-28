# SVG memory reproduction

This standalone Windows application depends only on published `windows-reactor-pre 0.106.0` and `windows-reactor-setup 0.100.0`. Its own Cargo workspace and lock file exclude Uniproc, Guinea, Guicons, agents, Canvas, settings and devtools. It creates 27 original circle SVGs, each with a different fill color and its own EncodedImage.

From this directory:

```powershell
cargo run --locked
```

The window starts empty. After 5 seconds it displays all 27 SVG images, then hides, shows and hides them at ten-second intervals. Every Image is 20×20. The encoded sources remain owned throughout both cycles. Close the window after the final hide.

Every SVG has explicit intrinsic dimensions 24×24 and a `0 0 24 24` viewBox. The component retains the encoded sources while the window is open. There are no environment switches.

The earlier immediate-display control (commit `4469197`) measured approximately 85 and 87 MiB. Those results are historical and do not describe the restored timer-driven version.

A fresh run after restoring the timer measured approximately 84 MiB before the first show and 494 MiB after loading, remaining near 494 MiB after both hides. See RESULTS.md for the separate current and historical measurements.

Measure **private bytes** for the corresponding process, independently of working set. For example, in another PowerShell window:

```powershell
Get-Process svg-memory-repro | Select-Object Id,PrivateMemorySize64,WorkingSet64
```

The investigation status is in [ISSUE.md](ISSUE.md). Current and historical measurements are separated in [RESULTS.md](RESULTS.md).
