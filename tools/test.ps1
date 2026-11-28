# Test runner: generate the local `assets/test` fixtures first (they are
# `include_bytes!` compile-time inputs), then run the suite.
#
#   pwsh tools/test.ps1                          # nextest, whole workspace
#   pwsh tools/test.ps1 test -p tr-core          # any cargo subcommand
#   pwsh tools/test.ps1 nextest run -p tr-app --bin terraria-rust
#
# The default skips `persist::tests::migrates_version_2_kinds_to_item_ids`
# (the one known-broken test; see AGENTS.md). A plain `cargo test` is fine
# once the fixtures exist; use this wrapper on a fresh checkout or after
# deleting `assets/test/`.
#
# Everything passed after the script name is forwarded to `cargo` verbatim.

$ErrorActionPreference = 'Stop'

$repo = Split-Path -Parent $PSScriptRoot
& (Join-Path $PSScriptRoot 'ensure_fixtures.ps1')
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }

$cargoArgs = @($args)
if ($cargoArgs.Count -eq 0) {
    $cargoArgs = @(
        'nextest', 'run',
        '--workspace',
        '--no-fail-fast',
        '--test-threads', '4',
        '-E', 'not test(migrates_version_2_kinds_to_item_ids)'
    )
}

Push-Location $repo
try {
    & cargo @cargoArgs
    exit $LASTEXITCODE
}
finally {
    Pop-Location
}
