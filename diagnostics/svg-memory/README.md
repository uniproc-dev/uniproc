# SVG memory reproduction

This standalone Windows application depends only on published `windows-reactor-pre 0.106.0` and `windows-reactor-setup 0.100.0`. Its own Cargo workspace and lock file exclude Uniproc, Guinea, Guicons, agents, Canvas, settings and devtools. It creates 27 original circle SVGs, each with a different fill color and its own EncodedImage.

From this directory:

```powershell
cargo run --locked
```

The window immediately displays all 27 SVG images. Every image has an explicit width and height of 20. There is no timer or view switching. Close the window after measuring the process.

Every SVG has explicit intrinsic dimensions 24×24 and a `0 0 24 24` viewBox. The component retains the encoded sources while the window is open. There are no environment switches.

Two fresh short launches of this immediate-display version stabilized at approximately 85 and 87 MiB private memory. The earlier timer-driven version's ~438 MiB result does not reproduce in these runs; that historical behavior is recorded separately.

Measure **private bytes** for the corresponding process, independently of working set. For example, in another PowerShell window:

```powershell
Get-Process svg-memory-repro | Select-Object Id,PrivateMemorySize64,WorkingSet64
```

The investigation status is in [ISSUE.md](ISSUE.md). Current and historical measurements are separated in [RESULTS.md](RESULTS.md).
