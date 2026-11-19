# WorldGenParityHarness — the S1-A-7 worldgen parity proof

Runs the **real decompiled genRand-driven worldgen leaf** (`WorldGen.TileRunner`,
`WorldGen.cs:76770-77142`) and writes the resulting tile array through the **real
decompiled save path** (the S1-J-3 `WldFixtureHarness` bodies, reused verbatim via
the `.csproj` `Compile` links). This is the S1-J-1 / S1-J-3 oracle pattern: the
decompile is the source, only the global lookups become shim state.

## Why a differential, not a full-world byte compare

The port's worldgen draws **xorshift64** (`tr_core::rng::Xorshift64`) where vanilla
draws the **Knuth subtractive `UnifiedRandom`** (`WorldGen.genRand => Main.rand`,
`WorldGen.cs:4445`). This is a deliberate, documented divergence (research/35
deviation #5; ANALYSIS "Determinism, proven (reproducible), frozen (algorithm)"):
the port's per-pass streams are `Rng(seed ^ fnv(pass_name))`, not vanilla's
`new UnifiedRandom(worldSeed)` per pass. So a full generated world is **not**
byte-comparable with a C#-generated one by design — comparing them would only
re-prove the known stream divergence.

What **is** provable is the **algorithm**: feed the verbatim C# `TileRunner` the
**port's** stream and the two tile arrays must agree field-by-field. That isolates
the walker's logic (the protection chain, the Manhattan wobble, the mud/liquid
arms, the velocity ladder) from the deliberately-different stream.

## Two modes

* **DIFF** (the parity proof) — `genRand` replays the port's xorshift64 stream
  (state passed on the command line). The verbatim C# `TileRunner` then draws the
  same sequence as the port's `tile_runner`, so the two tile arrays can be
  compared byte-for-byte. Writes `parity_diff.wld` + `parity_tiles.bin` (the raw
  post-runner tile array, x-major, 11 bytes/tile: kind u16, wall u16, liquid u8,
  active u8, slope u8, halfbrick u8, lava u8, honey u8, shimmer u8).
* **REAL** — `genRand` is the real Knuth `UnifiedRandom(seed)`. This is the actual
  vanilla stream; the `.wld` it writes (`parity_real.wld`) is a genuine
  C#-generated world. It is **not** byte-comparable with the port (the stream
  divergence above); it exists as the C# artifact the ROADMAP row asks for.

## How it works

* `extract.py` slices the worldgen leaf bodies **verbatim** out of `decompiled/`
  into `Generated/*.cs`: `WorldGen.TileRunner` + `PlaceWall` +
  `CanBeClearedDuringGeneration` (`WorldGen.cs:76770`/`:75125`/`:80168`), the whole
  `UnifiedRandom` class (renamed `UnifiedRandomCore` so the shim can wrap it), and
  ReLogic's `Vector2D` (the walker's position/velocity type). **Zero** substitutions
  on the method bodies.
* `Shims/WorldGenGlobals.cs` declares the global statics the bodies read
  (`Main.rand`/`tileCut`/`tileStone`, `WorldGen.genRand` + the secret-seed flags +
  `beachDistance`, `GenVars.mudWall`/`waterLine`/`lavaLine`/`UndergroundDesertLocation`,
  `SecretSeed.dualDungeons`) and the `UnifiedRandom` wrapper (real Knuth core OR the
  port-stream replay). `SquareWallFrame` is a **no-op** — a visual wall-framing pass
  the port defers (`tile_runner.rs` `place_wall` doc); wall frames are runtime-only
  and never serialized, so the comparison stays on saved fields.
* `Shims/Ids.cs` is a superset of the S1-J-3 `Ids.cs` (which is excluded from this
  project): the four save-path sets plus the three `TileID.Sets` the runner reads
  (`SaveSlopes`, `CanBeClearedDuringGeneration`, `Ore`), at the decompiled
  static-constructor lines.
* `Program.cs` parses the five `Main.*` tile tables from the **real** decompiled
  `Main.cs` at runtime (direct assignments + for-loop ranges + `AddEchoFurnitureTile`),
  builds a 200×400 all-stone world with a protection-zoo / liquid scene, runs six
  `TileRunner` calls covering the ore placement, the protection chain, the mud
  (kind 59) arms, the liquid carve/fill (type -1/-2) and the mud-wall lining, then
  dumps the tile array and writes the `.wld` through the real save path.

## Regenerate

```powershell
python tools/WorldGenParityHarness/extract.py
# DIFF: <state> = Xorshift64::from_seed(42).0 = 13679457532755275413
dotnet run --project tools/WorldGenParityHarness -- assets/test diff 13679457532755275413
# REAL: the genuine C#-generated world (Knuth stream, seed 42)
dotnet run --project tools/WorldGenParityHarness -- assets/test real 42
```

The Rust test `s1_a_7_the_tile_runner_matches_the_csharp_differential`
(`crates/tr-core/src/worldgen/tile_runner.rs`) replays the same scenario on the port
and compares every tile field + the final stream state against `parity_tiles.bin`.

## Scope / known divergences (documented, not faked)

* The scenario uses only branches the port has ported. The port's **unported** arms
  are excluded: the 367/368 shell wall placements (`WorldGen.cs:76914-76921`), the
  `case 53` underground-desert containment (`GenVars.UndergroundDesertLocation`,
  `:76934`), and the remix `oceanDepths` lava split (`:76899-76905`) — see
  `tile_runner.rs` module doc. The shim's empty `UndergroundDesertLocation` and
  `oceanDepths => false` match the port's deferral exactly.
* `SquareWallFrame` is a no-op on both sides (the port defers wall framing; wall
  frames are never serialized).
* The **stream** divergence (xorshift64 vs Knuth) is the whole point of the DIFF
  mode: it is factored out so the algorithm can be proven. The REAL mode's world is
  the genuine C# output and is not claimed byte-equal to the port.
