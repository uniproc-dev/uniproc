param(
    [string]$LogDirectory = "$PSScriptRoot/../../target/table-probe-results/ranked",
    [string]$OutputDirectory = "$PSScriptRoot/results"
)
$ErrorActionPreference = 'Stop'
$records = @(Get-ChildItem -LiteralPath $LogDirectory -Filter '*.log' | ForEach-Object {
    $run = $_.BaseName
    Get-Content -LiteralPath $_.FullName | Where-Object { $_ -like 'RESULT *' } | ForEach-Object {
        $record = $_.Substring(7) | ConvertFrom-Json
        $record | Add-Member -NotePropertyName run -NotePropertyValue $run
        $record
    }
})
New-Item -ItemType Directory -Path $OutputDirectory -Force | Out-Null
$records | ConvertTo-Json -Depth 8 | Set-Content -LiteralPath (Join-Path $OutputDirectory 'measurements.json') -Encoding utf8
$summary = $records | Where-Object { $_.phase -eq 3 -and -not $_.native_identity_check } | Group-Object mode,rows,period_ms,mixing,identity | ForEach-Object {
    $group = $_.Group
    $sample = $group[0]
    [pscustomobject]@{
        Mode = $sample.mode
        Identity = $sample.identity
        Rows = $sample.rows
        PeriodMs = $sample.period_ms
        Mix = $sample.mixing
        Runs = $group.Count
        CpuMs = [math]::Round(($group | Measure-Object cpu_ms -Average).Average, 2)
        CpuMsPerTick = [math]::Round(($group | ForEach-Object { $_.cpu_ms / $_.ticks } | Measure-Object -Average).Average, 2)
        PrivateMiB = [math]::Round(($group | Measure-Object private_mib -Average).Average, 2)
        HeartbeatP99Ms = [math]::Round(($group | Measure-Object heartbeat_p99_ms -Maximum).Maximum, 2)
        Creates = [math]::Round(($group | Measure-Object creates -Average).Average)
        MeanRankDistance = [math]::Round(($group | ForEach-Object { $_.rank_distance / ($_.rows * $_.ticks) } | Measure-Object -Average).Average, 2)
    }
}
$summary | Export-Csv -LiteralPath (Join-Path $OutputDirectory 'summary.csv') -NoTypeInformation -Encoding utf8
$summary | Format-Table -AutoSize
