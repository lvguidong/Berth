param([switch]$Native)
$ErrorActionPreference='Stop'
function Run-Check([scriptblock]$Command) { & $Command; if ($LASTEXITCODE -ne 0) { throw "Check failed ($LASTEXITCODE): $Command" } }
Push-Location (Join-Path $PSScriptRoot '..')
try {
    Run-Check { npm run build }
    Run-Check { npm test }
    Run-Check { npm run test:e2e }
    Run-Check { cargo fmt --manifest-path src-tauri/Cargo.toml --check }
    Run-Check { cargo clippy --manifest-path src-tauri/Cargo.toml -- -D warnings }
    Run-Check { cargo test --manifest-path src-tauri/Cargo.toml }
    if ($Native) { Run-Check { cargo test --manifest-path src-tauri/Cargo.toml explorer_lifecycle -- --ignored --nocapture --test-threads=1 } }
} finally { Pop-Location }
