param([switch]$SkipBuild)
$ErrorActionPreference='Stop'
Push-Location (Join-Path $PSScriptRoot '..')
$oldConfig=$env:BERTH_CONFIG_DIR
$oldTest=$env:BERTH_LIFECYCLE_TEST
try {
    if(-not $SkipBuild){npm exec tauri build -- --debug --no-bundle --config tools/smoke.tauri.conf.json;if($LASTEXITCODE -ne 0){throw 'Lifecycle build failed'}}
    $profile=Join-Path (Get-Location).Path 'artifacts\lifecycle-profile'
    New-Item -ItemType Directory -Force -Path $profile | Out-Null
    @{version=1;panel=@{mode='sidebar';edge='left';pinned=$true;hideFullscreen=$false};hotkeys=@{toggle='';capture='';direct=$false};pins=@(@{id='lifecycle-pin';path=(Get-Location).Path;alias='Lifecycle fixture';color='#75a99c';groupId=$null;order=0})} | ConvertTo-Json -Depth 6 | Set-Content -LiteralPath (Join-Path $profile 'config.json')
    $env:BERTH_CONFIG_DIR=$profile;$env:BERTH_LIFECYCLE_TEST='1'
    $started=Get-Date
    $process=Start-Process -FilePath (Join-Path (Get-Location).Path 'src-tauri\target\debug\berth.exe') -ArgumentList '--supervised' -WindowStyle Hidden -PassThru
    & "$PSScriptRoot\soak.ps1" -Minutes 1 -IntervalSeconds 2 -RootProcessId $process.Id -OutputPath 'artifacts\lifecycle-soak.csv'
    if(-not $process.WaitForExit(1000)){throw 'Lifecycle test did not exit'}
    $result=Get-Item -LiteralPath (Join-Path $profile 'lifecycle-result.json')
    if($result.LastWriteTime -lt $started){throw 'Lifecycle result was not refreshed'}
    $content=Get-Content -Raw -LiteralPath $result.FullName | ConvertFrom-Json
    if($content.error){throw $content.error}
    if($process.ExitCode -ne 0){throw "Lifecycle test exit code $($process.ExitCode)"}
    $content | Format-List
} finally {
    $env:BERTH_CONFIG_DIR=$oldConfig;$env:BERTH_LIFECYCLE_TEST=$oldTest
    Pop-Location
}
