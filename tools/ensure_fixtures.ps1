# Generates the five local test fixtures under `assets/test/` (gitignored).
#
# The Rust tests `include_bytes!` these files at COMPILE time, so they must
# exist before `cargo test`/`cargo nextest` builds the test targets. This
# script is idempotent: it does nothing when all five are already present
# (`-Force` regenerates anyway). All three generators are deterministic
# (the C# writers' wall clock is pinned by WldFixtureHarness), so two runs
# produce byte-identical files.
#
#   pwsh tools/ensure_fixtures.ps1            # generate only what is missing
#   pwsh tools/ensure_fixtures.ps1 -Force     # regenerate all five
#
# The generated set:
#   golden.wld, golden.plr                      tools/WldFixtureHarness
#   parity_diff.wld, parity_tiles.bin           WorldGenParityHarness DIFF
#   parity_real.wld                             WorldGenParityHarness REAL
#
# DIFF replays the port's seed-42 xorshift stream. The state is the pinned
# `S1_A_7_PORT_SEED42_STATE` from crates/tr-core/src/worldgen/tile_runner.rs
# (0xbdd732262feb6e95); `ulong.Parse` takes it in decimal.

[CmdletBinding()]
param(
    [switch]$Force,
    [string]$OutDir
)

$ErrorActionPreference = 'Stop'

$repo = Split-Path -Parent $PSScriptRoot
if (-not $OutDir) { $OutDir = Join-Path $repo 'assets/test' }
$out = [System.IO.Path]::GetFullPath($OutDir)
New-Item -ItemType Directory -Force -Path $out | Out-Null

$fixtures = @(
    'golden.wld',
    'golden.plr',
    'parity_diff.wld',
    'parity_tiles.bin',
    'parity_real.wld'
)
$missing = @($fixtures | Where-Object { -not (Test-Path -LiteralPath (Join-Path $out $_)) })
if (-not $Force -and $missing.Count -eq 0) {
    Write-Host "ensure_fixtures: all 5 fixtures present in $out (use -Force to regenerate)"
    exit 0
}

$dotnet = Get-Command dotnet -ErrorAction SilentlyContinue
if (-not $dotnet) {
    throw 'ensure_fixtures: the .NET SDK is required (dotnet not found on PATH)'
}
if (-not (Test-Path -LiteralPath (Join-Path $repo 'decompiled/Terraria/Main.cs'))) {
    throw "ensure_fixtures: decompiled/Terraria/Main.cs is missing under $repo (the C# oracle sources are required)"
}

# The harnesses parse the decompiled sources relative to the repo root.
Push-Location $repo
try {
    if ($Force -or ($missing -contains 'golden.wld') -or ($missing -contains 'golden.plr')) {
        Write-Host 'ensure_fixtures: WldFixtureHarness -> golden.wld, golden.plr'
        & dotnet run --project (Join-Path $repo 'tools/WldFixtureHarness') -- $out
        if ($LASTEXITCODE -ne 0) { throw "WldFixtureHarness failed (exit $LASTEXITCODE)" }
    }

    if ($Force -or ($missing -contains 'parity_diff.wld') -or ($missing -contains 'parity_tiles.bin')) {
        Write-Host 'ensure_fixtures: WorldGenParityHarness DIFF -> parity_diff.wld, parity_tiles.bin'
        & dotnet run --project (Join-Path $repo 'tools/WorldGenParityHarness') -- $out diff 13679457532755275413
        if ($LASTEXITCODE -ne 0) { throw "WorldGenParityHarness (diff) failed (exit $LASTEXITCODE)" }
    }

    if ($Force -or ($missing -contains 'parity_real.wld')) {
        Write-Host 'ensure_fixtures: WorldGenParityHarness REAL -> parity_real.wld'
        & dotnet run --project (Join-Path $repo 'tools/WorldGenParityHarness') -- $out real 42
        if ($LASTEXITCODE -ne 0) { throw "WorldGenParityHarness (real) failed (exit $LASTEXITCODE)" }
    }
}
finally {
    Pop-Location
}

$still = @($fixtures | Where-Object { -not (Test-Path -LiteralPath (Join-Path $out $_)) })
if ($still.Count -gt 0) {
    throw "ensure_fixtures: still missing after generation: $($still -join ', ')"
}
Write-Host "ensure_fixtures: generated 5 fixtures in $out"
