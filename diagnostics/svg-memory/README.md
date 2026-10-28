# SVG memory reproduction

This standalone Windows application depends only on published `windows-reactor-pre 0.106.0` and `windows-reactor-setup 0.100.0`. Its own Cargo workspace and lock file exclude Uniproc, Guinea, Guicons, agents, Canvas, settings and devtools. It creates 27 original circle SVGs, each with a different fill color and its own EncodedImage.

From this directory:

```powershell
cargo run --locked
```

The window starts empty, displays 27 SVG images after 5 seconds, removes them after another 10 seconds, displays them again after another 10 seconds, then removes them after another 10 seconds. Every image has an explicit width and height of 20. The program prints each transition to stderr. Close the window after observing the final empty state.

Every SVG has explicit intrinsic dimensions 24×24 and a `0 0 24 24` viewBox. The component retains the encoded sources while the controls are hidden, approximating application-lifetime icon assets. There are no environment switches.

Measure **private bytes** for the corresponding process, independently of working set. For example, in another PowerShell window:

```powershell
Get-Process svg-memory-repro | Select-Object Id,PrivateMemorySize64,WorkingSet64
```

The issue draft is in [ISSUE.md](ISSUE.md). The measurements and their limitations are in [RESULTS.md](RESULTS.md).
