param(
    [string]$Executable = "$PSScriptRoot/../../target/table-probe-build/debug/table-probe-layout.exe",
    [string]$OutputDirectory = "$PSScriptRoot/../../target/table-probe-results/manual",
    [ValidateSet('native','positioned','dwrite','joined')][string[]]$Modes = @('native','positioned','dwrite','joined'),
    [int]$Rows = 2000,
    [int]$Period = 500,
    [ValidateSet('mild','shuffle')][string]$Mix = 'mild',
    [int]$Repeats = 2
)
$ErrorActionPreference = 'Stop'
$probeExecutable = (Resolve-Path -LiteralPath $Executable).Path
$probeOutput = [System.IO.Path]::GetFullPath($OutputDirectory)
New-Item -ItemType Directory -Path $probeOutput -Force | Out-Null
foreach ($repeat in 1..$Repeats) {
    foreach ($mode in $Modes) {
        $env:MODE = $mode
        $env:ROWS = [string]$Rows
        $env:PERIOD = [string]$Period
        $env:MIX = $Mix
        $log = Join-Path $probeOutput "$mode-$Rows-$Period-$Mix-$repeat.log"
        $probeProcess = Start-Process -FilePath $probeExecutable -WorkingDirectory $PSScriptRoot -RedirectStandardError $log -PassThru
        if (-not $probeProcess.WaitForExit(40000)) {
            $probeProcess.CloseMainWindow() | Out-Null
            throw "Probe timed out: $log"
        }
        if ($probeProcess.ExitCode -ne 0) {
            throw "Probe exited with $($probeProcess.ExitCode): $log"
        }
        Get-Content -LiteralPath $log | Where-Object { $_ -like 'RESULT *' } | ForEach-Object {
            $_.Substring(7) | ConvertFrom-Json
        } | Select-Object mode,rows,period_ms,mixing,phase,ticks,cpu_ms,private_mib,creates,inputs,draw_us,heartbeat_p99_ms
    }
}
