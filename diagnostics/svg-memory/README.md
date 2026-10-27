# SVG memory reproduction

This standalone Windows application depends only on published `windows-reactor-pre 0.106.0` and `windows-reactor-setup 0.100.0`. Its own Cargo workspace and lock file exclude Uniproc, Guinea, Guicons, agents, Canvas, settings and devtools. The SVG is an original white circle.

From this directory:

```powershell
cargo run --locked
```

The window starts empty, displays 27 SVG images after 5 seconds, removes them after another 10 seconds, displays them again after another 10 seconds, then removes them after another 10 seconds. Every image has an explicit width and height of 20. The program prints each transition to stderr. Close the window after observing the final empty state.

Controls, set before starting a fresh process:

| Environment | Meaning |
| --- | --- |
| `SVG_COUNT=1` | One image/source instead of 27. |
| `SVG_SHARED=1` | All image controls share one `EncodedImage` allocation. |
| `SVG_PIXELS=1` | Root SVG dimensions are `24` instead of `1em`; the viewBox remains `0 0 24 24`. |
| `SVG_RELEASE_ON_HIDE=1` | Drop source byte owners while images are hidden and recreate them on the next show. |

Unset the other controls before comparing runs. By default the component retains the tiny encoded byte arrays while the controls are hidden. This approximates application-lifetime icon assets; it is intentional and is different from retaining native image controls.

Measure **private bytes** for the corresponding process, independently of working set. For example, in another PowerShell window:

```powershell
Get-Process svg-memory-repro | Select-Object Id,PrivateMemorySize64,WorkingSet64
```

The issue draft is in [ISSUE.md](ISSUE.md). The measurements and their limitations are in [RESULTS.md](RESULTS.md).
