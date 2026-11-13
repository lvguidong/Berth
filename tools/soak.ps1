param(
    [int]$Minutes = 1440,
    [int]$IntervalSeconds = 60,
    [string]$OutputPath = (Join-Path $PSScriptRoot '..\artifacts\soak.csv'),
    [int]$RootProcessId = 0
)
$ErrorActionPreference = 'Stop'
if ($Minutes -lt 1 -or $IntervalSeconds -lt 1) { throw 'Duration and interval must be positive.' }
$resolvedOutput = [IO.Path]::GetFullPath($OutputPath)
New-Item -ItemType Directory -Path ([IO.Path]::GetDirectoryName($resolvedOutput)) -Force | Out-Null
$deadline = (Get-Date).AddMinutes($Minutes)
$samples = 0
do {
    $processes = @(Get-CimInstance Win32_Process)
    $ids = [Collections.Generic.HashSet[int]]::new()
    if($RootProcessId -gt 0){[void]$ids.Add($RootProcessId)}else{foreach ($p in $processes | Where-Object Name -eq 'berth.exe') { [void]$ids.Add([int]$p.ProcessId) }}
    do {
        $before = $ids.Count
        foreach ($p in $processes) { if ($ids.Contains([int]$p.ParentProcessId)) { [void]$ids.Add([int]$p.ProcessId) } }
    } while ($ids.Count -gt $before)
    $live = @($ids | ForEach-Object { Get-Process -Id $_ -ErrorAction SilentlyContinue })
    $working = ($live | Measure-Object WorkingSet64 -Sum).Sum
    $private = ($live | Measure-Object PrivateMemorySize64 -Sum).Sum
    $cpu = ($live | Measure-Object CPU -Sum).Sum
    [pscustomobject]@{Time=(Get-Date).ToString('o');Processes=$live.Count;WorkingSetMB=[math]::Round($working/1MB,2);PrivateMB=[math]::Round($private/1MB,2);CpuSeconds=[math]::Round($cpu,2)} | Export-Csv -LiteralPath $resolvedOutput -Append -NoTypeInformation
    $samples++
    if ((Get-Date) -lt $deadline) { Start-Sleep -Seconds $IntervalSeconds }
} while ((Get-Date) -lt $deadline)
Write-Output "Recorded $samples samples to $resolvedOutput. Includes Berth supervisors and descendant WebView processes."
