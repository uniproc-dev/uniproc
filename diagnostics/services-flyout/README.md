# Services flyout scratchpad

Native, isolated UI experiment on windows-reactor. Does not load Uniproc or touch SCM. Contains 120 simulated services in a native ScrollViewer; the bottom flyout is outside the scroll content.

```powershell
cargo run --manifest-path diagnostics/services-flyout/Cargo.toml
```

Select a service and press Start, Stop or Restart. A flyout panel opens automatically along the bottom edge inside the same window, without a separate window frame. Collapse keeps the operation running; the bottom button reopens it. Selecting another service does not retarget the operation. Normal, slow and access-denied scenarios are selectable before an operation. Restart has separate stop and start phases.

The floating surface uses native WinUI controls in the window's overlay grid, not a modal dialog or a WinUI Flyout attachment. This lets the experiment open automatically and stay open while selecting rows. It currently allows one operation at a time.

Checkpoint and wait-hint values are simulated and available under Details. They are not a completion percentage; the progress bar is indeterminate. Slow mode deliberately stops advancing checkpoints for eight seconds. This is a UX prototype, not a verification of SCM reporting or action delivery.
