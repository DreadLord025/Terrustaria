# 35 — RNG & data-structure utils

> Wave 1 · researched 2026-11-12 · sources: decompiled (Terraria 1.4.5.8) + decompiled-libs (ReLogic) + crates (tr-core/tr-app/tr-render/tr-content) + ROADMAP/ANALYSIS/PORT_MAP

## Scope

The determinism substrate of the port: every random source the C# game owns, the shared
helpers that sit on top of them, and the small data structures the RNG helpers move
(`Point`, `Vector2`, `Point16`, `IntRange`/`FloatRange`).

C# files in this module:

| File | Lines | Role |
| --- | --- | --- |
| `decompiled/Terraria/Utilities/UnifiedRandom.cs` | 159 | `Main.rand`'s type — the legacy **.NET `System.Random`** (Knuth subtractive, 56-int seed array). **Not** xoshiro. |
| `decompiled/Terraria/Utilities/FastRandom.cs` | 89 | The 48-bit LCG (`Terraria.Utilities.FastRandom`) used by the tile scanner, the world renderer and the hive carve. |
| `decompiled/Terraria/Utils.cs` | 2591 | `Lerp`/`Clamp`/`Round`/`SmoothStep`/`Remap` + the `UnifiedRandom` extension helpers (`NextFloat`, `NextFromRectangle`, `NextVector2*`) + `RandomNext`/`RandomNextSeed`/`RandomFloat`/`RandomInt`. |
| `decompiled/Terraria/Main.cs` | ~67 000 | `Main.rand` (`:1291-1292`), `Main._rngs` (`:1294-1295`), `Main.SwapRandom` (`:3204-3215`), `Main._drawRand` (`:67050`), `Main.TileFrameSeed` (`:417`, `:61721`, `:66567`). |
| `decompiled/Terraria/WorldGen.cs` | ~78 000 | `WorldGen.genRand` (`:4445`). |
| `decompiled/Terraria/Item.cs` | ~50 000 | `Item.Prefix` RNG selection (`:506-508`, `:528`). |
| `decompiled/Terraria/DataStructures/Point16.cs` | 1 859 bytes | The 16-bit point the tile APIs use. |

**Responsibility.** Every random draw in Terraria comes from one of four places:
(1) the single global `Main.rand` (`UnifiedRandom`), (2) a *named* stream created by
`Main.SwapRandom(key)`, (3) a `FastRandom` (48-bit LCG) instance, or (4) a per-call
re-seed of one of the above. The order in which a stream is consumed is part of the
game's observable behaviour: the same seed with a different call order is a different
world, a different loot table and a different set of pixels.

## Key entities

- `Terraria.Utilities.UnifiedRandom::SetSeed` (`UnifiedRandom.cs:28-60`) — the .NET
  `System.Random` warm-up: a 56-int `SeedArray`, `MSEED = 161803398`, `MBIG = int.MaxValue`.
- `Terraria.Utilities.UnifiedRandom::InternalSample` (`:67-88`) — the subtractive step
  `SeedArray[i] = SeedArray[i] - SeedArray[i+21] (mod int.MaxValue)`; `inext` cycles 1..55.
- `Terraria.Utilities.UnifiedRandom::Sample` (`:62-65`) — `InternalSample() * 4.656612875245797E-10`
  (i.e. `/ 2^31`).
- `Terraria.Utilities.UnifiedRandom::Next(int max)` (`:134-141`) — `(int)(Sample() * max)`:
  a **truncated multiply**, not a modulo.
- `Terraria.Utilities.UnifiedRandom::Next(int min, int max)` (`:120-132`) — `Sample() * (max-min) + min`.
- `Terraria.Utilities.UnifiedRandom::NextDouble` (`:143-146`) — `Sample()`.
- `Terraria.Utilities.FastRandom::NextSeed` (`FastRandom.cs:85-88`) — `(seed * 25214903917 + 11) & 0xFFFFFFFFFFFF`
  (48-bit LCG).
- `Terraria.Utilities.FastRandom::NextBits` (`:47-51`) — advance, then `Seed >> (48 - bits)`.
- `Terraria.Utilities.FastRandom::Next(int max)` (`:63-78`) — power-of-two fast path +
  rejection loop on `num - num2 + (max - 1) < 0`.
- `Terraria.Utilities.FastRandom::WithModifier` (`:27-35`) — the per-tile / per-position stream.
- `Terraria.Utilities.FastRandom::CreateWithRandomSeed` (`:37-40`) — `Guid.NewGuid().GetHashCode()`
  (OS entropy; the source of the non-deterministic vanilla streams).
- `Terraria.Utils::NextFloat` (`Utils.cs:1198-1201`) — `(float)r.NextDouble()`.
- `Terraria.Utils::NextFloatDirection` (`:1203-1206`) — `NextDouble() * 2f - 1f`.
- `Terraria.Utils::NextFromRectangle` (`:1246-1249`), `NextVector2*` (`:1251-1299`).
- `Terraria.Utils::RandomNext`/`RandomNextSeed`/`RandomFloat`/`RandomInt` (`:2294-2320+`) —
  the `ref ulong seed` flavour of the same 48-bit LCG as `FastRandom`.
- `Terraria.Utils::Lerp` (`:107-110`), `Round(Vector2)` (`:112-115`), `Clamp<T>` (`:223-234`),
  `Clamp(Rectangle)` (`:236-239`), `SmoothStep` (`:126-134`), `GetLerpValue` (`:315`, `:730`),
  `Remap` (`:345-350`).
- `Terraria.Main::rand` (`Main.cs:1291-1292`) — `[ThreadStatic] public static UnifiedRandom rand;`
- `Terraria.Main::_rngs` (`Main.cs:1294-1295`) — `[ThreadStatic] Dictionary<string, UnifiedRandom>`.
- `Terraria.Main::SwapRandom(string key)` (`Main.cs:3204-3215`) — the named-stream factory:
  `Main._rngs[key] = new UnifiedRandom(Main.ActiveWorldFileData.Seed)` on first use, then
  `Main.rand` is swapped for the duration of the `using` block (`Main.cs:118` restores it).
- `Terraria.WorldGen::genRand` (`WorldGen.cs:4445`) — **`public static UnifiedRandom genRand => Main.rand;`**
  (an alias, not a separate stream).
- `Terraria.Item::Prefix` (`Item.cs:528`) — picks `WorldGen.genRand` while
  `WorldGen.isGeneratingOrLoadingWorld`, else `Main.rand`; same single stream either way.
- `Terraria.Main::TileFrameSeed` (`Main.cs:417`) — `ulong`, reseeded every 5 draws via
  `Utils.RandomNextSeed` (`Main.cs:61721`), initially `(ulong)Guid.NewGuid().GetHashCode()` (`:66567`).
- `Terraria.Main::_drawRand` (`Main.cs:67050`) — a second `UnifiedRandom`, re-seeded per draw
  with `Main._drawRand.SetSeed(seed)` (`Main.cs:35795`, from `proj.localAI[0]`).

Rust side:

- `tr_core::rng::Xorshift64` (`crates/tr-core/src/rng.rs:25-97`) — the shared xorshift64
  core: `from_seed` = splitmix64 finaliser (`:33-42`), `next_u64` = the 13/7/17 triple
  (`:46-53`), `next_u32` = **high half** of the stepped word (`:62-64`), `range` (`:73-79`),
  `chance` (`:82-84`), `next_f64` (`:94-96`).
- `tr_core::tile_light::palette::FastRandom` (`crates/tr-core/src/tile_light/palette.rs:19-111`)
  — the exact 48-bit LCG port.
- `tr_core::worldgen::pipeline::Streams` (`crates/tr-core/src/worldgen/pipeline.rs:79-123`) —
  the three worldgen contract streams (`rng`/`drng`/`srng`) plus `pass_rng(name)` =
  `Rng(seed ^ fnv1a64(name))`.
- `tr_core::worldgen::pipeline::fnv1a64` (`crates/tr-core/src/worldgen/pipeline.rs:63-77`).
- `tr_core::worldgen::world_hash::world_hash` (`crates/tr-core/src/worldgen/world_hash.rs:44-63`).
- `tr_app::spawner::FastRandom` (`crates/tr-app/src/spawner.rs:28-51`) — the `Main.rand`
  stand-in for the spawn pipeline.
- `tr_app::prefix::roll_prefix` (`crates/tr-app/src/prefix.rs:477+`) — `Item.Prefix` draws through a
  one-method `PrefixRandom` seam: runtime rolls take `App::prefix_rng` (`UnifiedRandom`, S1-A-5); the worldgen chest seam adapts the splitmix `chest_rng` (`roll_prefix_splitmix`, `:568`).
- `tr_app::chest_gen` (`crates/tr-app/src/chest_gen.rs:126-143`) — another splitmix stream.
- `App::{npc_rng, craft_rng, item_rng, ambience_rng, clock_rng, light_rng_seed, growth_rng}`
  (`crates/tr-app/src/app/mod.rs:180, 301-303, 547-548, 568, 607-609, 611-612, 634`) — the
  port's per-consumer streams, all seeded from the world seed.

## Dependencies

- Calls into: `UnifiedRandom` is called from **7 558 `Main.rand.` call sites** in the
  decompiled tree (measured), plus `WorldGen.genRand` and `Main.SwapRandom`.
  `FastRandom` is called from `TileLightScanner`, `TileDrawing`, `DesertHive`, `GraniteBiome`,
  `Star.SpawnStars` and `Main`'s wind counters.
  `Utils`' helpers are called from everywhere (`NextFloat`, `NextVector2*`, `Clamp`, `Lerp`).
- Called by: `Main.rand` is the root of the whole gameplay RNG tree; `WorldGen.genRand`
  aliases it, so the worldgen draws and the gameplay draws interleave on **one** stream.
- Shared data structures: `Point`, `Vector2`, `Vector2D`, `Rectangle`, `Point16`
  (`decompiled/Terraria/DataStructures/Point16.cs`), `IntRange`, `FloatRange`.
  `Terraria.Utils::GetPointHash` — **not found** anywhere in the 1.4.5.8 decompilation
  (grep over `decompiled/**/*.cs` returned zero hits); the same idea is served by
  `Utils.RandomNext`/`RandomNextSeed` (the 48-bit LCG) and by `FastRandom.WithModifier(x, y)`.

## Port status

### Centrepiece — RNG source matrix

| RNG source | C# algorithm & seed | Rust implementation | Exact stream? | Impact | Subsystems affected |
| --- | --- | --- | --- | --- | --- |
| `Main.rand` (global) | .NET `System.Random`, Knuth subtractive 56-int array; `SetSeed` (`UnifiedRandom.cs:28-60`); `Next(max)` = `(int)(Sample()*max)` | **PORTED (exact)** as `tr_core::unified_random::UnifiedRandom` (`unified_random.rs`, S1-A-1 — C#-golden vectors); the *consumers* are mostly still splitmix/xorshift stand-ins (`spawner.rs:30-51`, `chest_gen.rs:127-138`), but the runtime prefix rolls already take a real `UnifiedRandom` (`App::prefix_rng`, S1-A-5; the `PrefixRandom` seam at `prefix.rs:438`, the field at `app/mod.rs:638`) | **YES** (type) / **PARTIAL** (prefix rolls wired) | determinism + behaviour | everything (7 558 call sites) |
| `Main.SwapRandom(key)` named streams | `Main._rngs[key] = new UnifiedRandom(worldSeed)` (`Main.cs:3210-3212`), swapped per update stage (`:17749-17809`) | **PORTED (registry, S1-A-2)** as `tr_app::app::rng::{NamedRngs, RngKey}` (`app/rng.rs`): all 17 keys, lazily `UnifiedRandom::new(worldSeed)`, cleared per world (`Main.cs:6465`). `UpdateTime` is the first migrated consumer (the clock's boundary rolls); the other stages still draw from world-seeded splitmix bridges (`npc_rng`, `item_rng`, `craft_rng`, `ambience_rng`, `growth_rng`, `prefix_rng`) until S1-A-11 | **YES** (registry) / **PARTIAL** (one stage wired) | determinism | world update (players, NPCs, items, projectiles, gores, dust, time) |
| `WorldGen.genRand` | alias of `Main.rand` (`WorldGen.cs:4445`) — same stream, interleaved with gameplay draws | no equivalent (worldgen uses its own `Streams`) | **NO** | determinism | worldgen + `Item.Prefix` during generation |
| vanilla per-pass stream | `Main.rand = new UnifiedRandom(this._seed)` before every pass (`WorldGenerator.cs:497`, quoted at `pipeline.rs:7, :110`) | `Streams::pass_rng(name)` = `Rng(seed ^ fnv1a64(name))` (`pipeline.rs:120-122`) | **NO** (deliberate) | determinism | worldgen chain |
| `FastRandom` (48-bit LCG) | `(seed*25214903917 + 11) & 0xFFFFFFFFFFFF` (`FastRandom.cs:85-88`) | `tr_core::tile_light::palette::FastRandom` (`palette.rs:19-111`), worldgen hive (`desert.rs`) | **YES** | — | tile lighting, tile drawing, hive, granite |
| `Utils.RandomNext/RandomNextSeed/RandomFloat/RandomInt` | same 48-bit LCG on `ref ulong` (`Utils.cs:2294-2320+`) | `FastRandom::next_seed` (`palette.rs:34-36`) | **YES** (algorithm) / **NO** (seed source) | determinism | tile-frame seeds |
| `Main.TileFrameSeed` | reseeded every 5 draws (`Main.cs:61721`), init `Guid.NewGuid().GetHashCode()` (`:66567`) | pinned to `world_seed | 1` (`app/mod.rs:2826`) / world-derived (`render/tiles.rs:301-303`) | **NO** (deliberate: OS entropy → world seed) | determinism | tile frames, leaf rolls |
| `Main._drawRand` | separate `UnifiedRandom` (`Main.cs:67050`), `SetSeed(proj.localAI[0])` (`:35795`) | reseed folded into the shared stream (`projectile.rs:4213, :4377`) | **NO** | behaviour (draw-only) | projectile draw effects |
| `FastRandom.CreateWithRandomSeed()` | `Guid` hash (`FastRandom.cs:37-40`), used by `Main`'s wind counters (`Main.cs:58700-58704`) and `Star.SpawnStars` | xorshift64 core (`tr-core/src/wind.rs:12-16`); stars pinned to the world seed (`render/sky.rs:470`, `ANALYSIS.md:985-986`) | **NO** | determinism | wind, sky |
| `Utils.NextFloat` / `NextVector2*` | `(float)NextDouble()` etc. (`Utils.cs:1198-1299`) | `Xorshift64::next_f64` (`rng.rs:94-96`) — different numerics, admitted at `rng.rs:89-93` | **NO** | behaviour | everything that rolls a float |
| `Utils.Lerp/Clamp/Round/SmoothStep/GetLerpValue/Remap` | `Utils.cs:107-115, 126-139, 223-239, 315, 345-350` | plain Rust arithmetic / `clamp` | **YES** (semantics) except `Math.Round`'s **banker's** rounding (`Utils.cs:114`) — the Rust side must use `round_ties_even` | pixel | UI, camera, tile frames |
| `Terraria.DataStructures.Point16` | `DataStructures/Point16.cs` | own Rust point types | n/a | — | tile APIs |
| `Utils.GetPointHash` | **not found** in 1.4.5.8 | — | n/a | — | — |
| `Item.seed`, `NPC.seed`, `NPC.netSeed` | **not found** — no `public int seed` in `Item.cs`/`NPC.cs`/`Projectile.cs`/`Player.cs`, and `netSeed` has zero hits across `decompiled/**/*.cs` | — | n/a | — | — |

### Subsystem table

| C# subsystem | Status | Rust location | Evidence / notes |
| --- | --- | --- | --- |
| `UnifiedRandom` (the type) | **PORTED (exact)** | `crates/tr-core/src/unified_random.rs` | Byte-faithful port (S1-A-1, 2026-11-12): `SetSeed`/`InternalSample`/`Sample`/`Peek`/`Next`/`NextDouble`/`NextBytes`/the large-range branch + the `NextFloat`/`NextFloatDirection`/`NextFromList` extensions, pinned by 11 golden tests against a real `dotnet` run of the decompiled class. Only the geometry helpers (`NextVector2*`/`NextFromRectangle`) are deferred — no shared `Vector2`/`Point`/`Rectangle`/`IntRange` types. |
| `Main.rand` global stream | **PARTIAL** | `crates/tr-app/src/spawner.rs:30-51`, `chest_gen.rs:127-138`, `prefix.rs:455-461` (chest-seam adapter only) | Mostly per-consumer splitmix64 stand-ins; **S1-A-5** moved the runtime prefix rolls onto a real `UnifiedRandom` (`App::prefix_rng`), leaving only the worldgen chest seam on splitmix until S1-A-4; `spawner.rs`'s doc calls itself "xorshift64\*" but the code is the splitmix finaliser — the comment is wrong. |
| `Main.SwapRandom` named streams | **PORTED (registry) / PARTIAL (consumers)** | `crates/tr-app/src/app/rng.rs` | **S1-A-2**: `NamedRngs`/`RngKey` model `Main._rngs` exactly — all 17 keys, lazy `UnifiedRandom::new(worldSeed)` (`Main.cs:3204-3215`), cleared per world (`:6465`), with C#-golden and independence tests. `_rngs["UpdateTime"]` drives the clock's boundary rolls (drawn at the boundary, not per frame). The other stage keys exist but their stage bodies still draw from world-seeded splitmix bridges; wiring them is S1-A-11. |
| `WorldGen.genRand` | **MISSING** | — | The alias itself is not modelled; the worldgen-time prefix rolls (chest stocking) still draw the splitmix `chest_rng` seam (`prefix.rs::roll_prefix_splitmix` `:568`) and the runtime rolls a separate `App::prefix_rng` (`UnifiedRandom`, S1-A-5); neither is the C#'s shared `genRand`/`Main.rand`. |
| Vanilla per-pass re-seed | **PARTIAL** | `crates/tr-core/src/worldgen/pipeline.rs:109-122` | `pass_rng(name)` = `Rng(seed ^ fnv1a64(name))` instead of `new UnifiedRandom(worldSeed)`; admitted in the file's own comment ("xorshift stands in for the Knuth subtractive `UnifiedRandom`", `:113-114`). |
| `FastRandom` | **PORTED (exact)** | `crates/tr-core/src/tile_light/palette.rs:19-111` | Multiplier/mask/`NextBits`/`WithModifier`/the rejection loop are byte-faithful (`FastRandom.cs:47-88`); `next_double` keeps the C#'s `float` widening (`palette.rs:77-79`). |
| `Utils.RandomNext*` family | **PORTED (algorithm)** | `crates/tr-core/src/tile_light/palette.rs:34-36` | Same LCG; the *seed* is world-derived instead of `Guid`-derived. |
| `Utils` float/vector RNG extensions | **PARTIAL** | `crates/tr-core/src/rng.rs:94-96` | `next_f64` is `next_u32()/2^32`; the C# is `int/2^31` cast to `float`. Same `[0,1)` semantics, different quantum. |
| `Utils.Lerp/Clamp/SmoothStep/Remap` | **PARTIAL** | ad-hoc | `Clamp` is order-sensitive (`>max` then `<min`) — Rust `clamp` panics on `min > max` where C# returns `max`. Banker's rounding (`Utils.cs:114`) is not exercised by the Rust side. |
| `Terraria.DataStructures.Point16` | **MISSING** | — | The Rust side uses its own point types; no 16-bit tile point. |
| `DropAttemptInfo.rng` (loot) | **PARTIAL** | `crates/tr-core/src/loot.rs:11-14` (`DropRng` trait) | The trait abstracts the source, but no implementation is the C#'s `UnifiedRandom`. |
| `Sandstorm` RNG | **PARTIAL** | `crates/tr-core/src/scene.rs:528` | Still draws from an isolated `FastRandom`; the C#-exact `UnifiedRandom` now exists (`tr_core::UnifiedRandom`, S1-A-1) but is not wired here — consumer migration is S1-A-2+. |

## Tests

Cargo tests that cover this module (static read, not executed):

- `crates/tr-core/src/rng.rs::tests::next_u32_matches_the_pre_refactor_worldgen_stream`
  (`:127`), `range_matches_the_pre_refactor_wind_next_int` (`:141`),
  `from_seed_matches_the_pre_refactor_splitmix_chain` (`:182`),
  `determinism_and_the_empty_range_contract` (`:201`) — all pin the **port against its own
  previous self**, not against the C#.
- `crates/tr-core/src/tile_light/palette.rs::tests::fast_random_matches_the_original_stream`
  and `fast_random_next_n_matches_the_next_zero_min_range` — the only true
  "byte-stream equals the C#" pins in the module.
- `crates/tr-core/src/worldgen/world_hash.rs::tests::the_512_seed_42_world_hashes_to_the_pinned_constant`
  (`SEED42_512 = 1360708804`), `the_default_world_hashes_to_the_pinned_constant`
  (`DEFAULT_WORLD = 289070280`), `the_hash_covers_every_tile_field`.
- `crates/tr-core/src/worldgen/mod.rs::tests::deterministic_for_seed` (`:259-276`).
- `crates/tr-app/src/spawner.rs` tests (spawn-rate multipliers, caps, moon event) — behaviour
  only, no stream pin.
- `crates/tr-core/src/worldgen/pipeline.rs::tests::test_DW_1_1_fnv1a64_known_vectors`
  (`:2889`) — the FNV mixer's own reference vectors.
- **S1-A-2** `crates/tr-app/src/app/rng.rs::tests` (6): the exact `SwapRandom` key
  literals (`the_registry_uses_the_csharp_key_names`), the C# seed-42 prefix on every
  stream (`every_stream_is_seeded_with_the_world_seed`, vector from the same `dotnet`
  run as S1-A-1's `GOLDEN_42`), stream independence
  (`streams_are_independent`, `asking_twice_continues_the_same_stream`), lazy creation
  and the per-world reset (`keys_are_created_lazily`,
  `reset_restarts_every_stream_from_the_new_seed`). `world_sim/clock.rs::tests` (10)
  now drive `advance_time` with a real `UnifiedRandom` (seeded so the first boundary
  roll hits/misses), not a raw `roll01`.

**Gap:** the RNG *type* is proven against the C# (S1-A-1/2), but no test compares a
migrated *stage's draw sequence* against the C# call order — that differential harness
is research task seed 7, and the stage wiring itself is S1-A-11.

The *contract* every RNG-touching task must satisfy is frozen by **S1-A-10**:
`docs/ANALYSIS.md` → "The RNG-order contract (S1-A-10)" lists the four C# sources
(this report's Responsibility table) and the stream/draw-order checklist.

## The determinism proof the repo claims — what it actually pins

`ROADMAP.md:334-335` asks for M5-GEN-1 to "keep `deterministic_for_seed` FNV proof".
The two artefacts behind that phrase are:

1. `crates/tr-core/src/worldgen/mod.rs:259-276` `deterministic_for_seed` — generates the
   same `GenOptions { width: 512, height: 512, seed: 42 }` **twice** and asserts the two
   tile vectors are element-wise equal. It pins **self-consistency** (the same seed
   reproduces the same world on the same binary) and nothing else. It contains **no**
   comparison against a vanilla world and **no** hash.
2. `crates/tr-core/src/worldgen/world_hash.rs` — a `u32` fold over the tile grid
   (`rotl13` + `h*5 + 3864292196`, `world_hash.rs:18-25`) with two pinned constants.
   Two caveats:
   - the per-tile field mix is **the port's own** ("C# hashes via
     `TileSnapshot.TileStruct.GetHashCode()`, whose .NET internals are not worth
     reproducing", `world_hash.rs:10-13`) → the number is **not comparable** with vanilla's
     `WorldGenerator.HashWorld` (`WorldGenerator.cs:517-545`);
   - the constants are **re-pinned on every worldgen change** — the test's own comment
     block enumerates ~20 re-pins ("every world on every seed changes by design"), so the
     pin is a *regression tripwire for the port's own pipeline*, not a fidelity proof.

**Verdict: the proof pins reproducibility of the port's own generator, not fidelity to
the C# generator.** It cannot detect that the port draws from xorshift64 where vanilla
draws from the Knuth subtractive `UnifiedRandom` — the class of divergence this module is
about.

## ROADMAP `DONE` rows claiming determinism — re-verified

| ROADMAP claim | Verdict | Evidence |
| --- | --- | --- |
| "RNG discipline: every world stage runs under `Main.SwapRandom("<label>")` — a *named RNG stream* per stage" (`ROADMAP.md:102-106`) | **PARTIAL (registry landed)** | The C# side is read correctly (`Main.cs:3204-3215`, `:17749-17809`). **S1-A-2** added the exact registry (`app/rng.rs`: all 17 keys, same world seed per key, lazy create, per-world reset) and re-keyed every bridge seed off the world seed, so the "same seed ⇒ same sequence per stage" property now holds for `_rngs["UpdateTime"]` and is available for every stage. The stage bodies that still draw from splitmix bridges are S1-A-11. |
| "keep `deterministic_for_seed` FNV proof" (`ROADMAP.md:334-335`) | **PARTIAL** | `deterministic_for_seed` has no FNV in it; the FNV-ish hash lives in `world_hash.rs` and is the port's own construction, re-pinned on every change. Reproducibility only. |
| M4-SPAWN-1 "spawn engine core (multiplier chain + caps + **named RNG stream**)" (`ROADMAP.md:666`) | **PARTIAL** | `spawner.rs` reproduces the multiplier/cap chain, but the stream is `spawner::FastRandom` — a splitmix stand-in for `Main.rand` with a *different* seed (`app/mod.rs:2519` `npc_rng: world_seed`) and a per-tick advance that the C# does not have (`ANALYSIS.md:4420-4424`). The C#'s named stream is `Main._rngs["SpawnNPC"]`, seeded `new UnifiedRandom(worldSeed)`. |
| "spawn RNG is the named `Main.SwapRandom("SpawnNPC")` stream" (`ROADMAP.md:619-621`) | **PARTIAL** | The named stream is a single `UnifiedRandom` per key; the Rust stream is a single `u64` per consumer. Structurally parallel, byte-wise different. |
| "hardmode is **not** seed-deterministic in vanilla" (`ROADMAP.md:319-320`) | **PASS** | Confirmed on the C# side (`Main.rand` is re-seeded from `DateTime.Now` in `initializeHardMode`); the Rust side stays deterministic, which is a deliberate, documented deviation. |
| "`FastRandom` in `tile_light/palette.rs` reproduces the multiplier/mask, `NextBits`, …" (`ANALYSIS.md:405-407`) | **PASS** | `palette.rs:25-27, 34-36, 64-67` match `FastRandom.cs:7-11, 47-51` exactly; `fast_random_matches_the_original_stream` is the proof. |
| "the port's worldgen contract is already xorshift; the draw ORDER is faithful, the byte stream is not" (`ANALYSIS.md:5123`, `worldgen/spider_gem_moss.rs:46-49`) | **PASS (documented)** | Correctly recorded as a deviation. |

## Deviations & risks

1. **`UnifiedRandom` itself was not ported — RESOLVED for the type (S1-A-1, 2026-11-12)**;
   the *consumers* are still on splitmix64/xorshift64 stand-ins (S1-A-2+), so every
   RNG-driven outcome still differs from vanilla: loot rolls, prefix rolls, spawn picks,
   chest contents, star fields, wind counters. The type now lives in
   `tr-core/src/unified_random.rs` and is byte-pinned by the golden tests; the remaining
   work is wiring (R-35-3/R-35-4).
2. **The `Main.rand` call ORDER is not modelled at all** — severity: **determinism**.
   In the C# a single stream serves the whole game; the order of the draws is the
   contract. The Rust splits it into seven fields, so there is no "order" to preserve
   and no way to align with vanilla without the type. *Not explicitly admitted.*
3. **`Main.SwapRandom` named streams: registry landed, consumers split** — severity:
   **determinism**. **S1-A-2 (2026-11-12)** added the exact registry (`NamedRngs`,
   all 17 keys, lazy `new UnifiedRandom(worldSeed)`) and re-keyed every App stream to
   the world seed, deleting the XOR constants and `craft_rng: 1`; `UpdateTime` is
   wired (the clock rolls at the boundary). The other stage bodies still draw from
   world-seeded splitmix bridges whose *values* are not the C# stream, so their
   outcomes still differ; **S1-A-11** migrates them.
4. **`WorldGen.genRand` alias not modelled** — severity: **behavioural**.
   `WorldGen.cs:4445` returns `Main.rand`, so `Item.Prefix` during world generation draws
   from the *same* stream as the worldgen. The Rust keeps two isolated streams instead:
   the runtime rolls take `App::prefix_rng` (a `UnifiedRandom`, S1-A-5) and the
   worldgen chest seam still draws the splitmix `chest_rng` (`prefix.rs::roll_prefix_splitmix`
   `:568`), so neither is the C#'s shared `genRand`/`Main.rand`. A divergence, but a *safe*
   one — the prefix roll cannot drift the world. *Admitted* in `ANALYSIS.md`
   ("Prefix rolls (S1-A-5)").
5. **The vanilla per-pass re-seed is replaced by name-mixed streams** — severity:
   **determinism**. `pass_rng(name)` = `Rng(seed ^ fnv1a64(name))` vs vanilla's
   `new UnifiedRandom(worldSeed)` per pass (`pipeline.rs:109-122`). *Admitted* in the
   file's own comment and in `ANALYSIS.md`.
6. **`deterministic_for_seed` is a self-consistency test, not a fidelity proof** —
   severity: **determinism** (documentation). `ROADMAP.md:334-335` calls it an "FNV
   proof"; it is neither an FNV hash nor a comparison with vanilla. *Not admitted.*
   — **FIXED 2026-11-13** (S1-K-5): the test is renamed
   `generation_is_reproducible_for_a_seed` with a doc comment stating it is
   self-consistency, not C# fidelity; the wording was corrected in `ANALYSIS.md`
   ("Determinism, proven (reproducible), frozen (algorithm)"), `ARCHITECTURE.md`
   and `ROADMAP.md` (S1-A-7 / M5-GEN-1 rows).
7. **`world_hash.rs`'s pinned constants are the port's own hash, re-pinned on every
   worldgen change** — severity: **documentation / determinism**. The test block itself
   records ~20 re-pins; the pin cannot detect a C#-divergent stream. *Partly admitted*
   (the header says "the per-tile field mix is the port's own").
   — **FIXED 2026-11-13** (S1-K-5): the *algorithm* (`mix` `*5 + 3864292196`, the
   per-tile `*31` field mix, the `rotl13` fold order) is now frozen by
   `the_fnv_constants_are_frozen_by_a_fixed_input` over a fixed 4×3 synthetic grid,
   with the expected value derived from an independent implementation (not from the
   port) and mutation-checked; the generator-output constants are documented as a
   re-pinned tripwire for the port's own pipeline, explicitly **not** C# parity.
8. **`Utils.NextFloat` numerics differ** — severity: **behavioural** (sub-pixel). C#:
   `(float)(int/2^31)`; Rust: `u32/2^32` (`rng.rs:89-96`). Same `[0,1)` range, different
   quantum → different rounded values at ~2^-31 resolution. *Admitted* at `rng.rs:89-93`.
9. **`spawner.rs`'s doc comment says "xorshift64\*" but the code is the splitmix64
   finaliser** — severity: **cosmetic** (`spawner.rs:28-43` vs `prefix.rs:453-461`,
   which calls the same shape "the splitmix step"). Misleading for the next reader.
10. **`Utils.Clamp`'s argument-order semantics** — severity: **behavioural** (rare).
    C# `Clamp` returns `max` when `value > max` even if `min > max`
    (`Utils.cs:223-234`); Rust's `Ord::clamp` panics when `min > max`. *Not audited.*
11. **`Math.Round` is banker's rounding** (`Utils.cs:112-115`) — severity: **pixel**.
    Rust's `f32::round` is half-away-from-zero. *Not audited.*
12. **`Point16` / `GetPointHash` are absent** — severity: **cosmetic**. `GetPointHash` does
    not exist in 1.4.5.8 at all (zero grep hits); `Point16` has no Rust counterpart but
    no ported code needs it yet. *N/A.*
13. **`Item.seed` / `NPC.seed` / `NPC.netSeed` do not exist in 1.4.5.8** — severity: **N/A
    (premise correction)**. The audit brief lists them as per-entity seeds; the
    decompilation has none. The nearest real per-entity RNG state is
    `Main._drawRand.SetSeed(proj.localAI[0])` (`Main.cs:35795`) and the `FastRandom`
    re-seeds (`WithModifier`). Anyone porting "NPC.netSeed" from memory would be
    inventing an API.

## Task seeds

| # | Task | Size | Depends on |
| --- | --- | --- | --- |
| 1 | **Port `Terraria.Utilities.UnifiedRandom`** into `tr-core` (`rng.rs` or a new `unified_random.rs`): the 56-int `SeedArray`, the `SetSeed` warm-up, `InternalSample`, `Sample`, `Next(max)`, `Next(min,max)`, `NextDouble`, `NextBytes`. Add a `NextFloat`/`NextVector2*` extension set mirroring `Utils.cs:1198-1299`. Gate it behind `tr-core::rng` so the worldgen contract streams stay untouched. — **DONE 2026-11-12** (`S1-A-1-unified-random`): `crates/tr-core/src/unified_random.rs`; the scalar extensions are in, the `NextVector2*`/`NextFromRectangle` geometry helpers are deferred (no shared geometry types) | **L** | — |
| 2 | **Golden-vector test for `UnifiedRandom`**: hard-code the first 64 `InternalSample()` values for seeds 0/1/42/-1 from a reference .NET run, so the port's stream is provably the C#'s. Without this, task 1 is unverifiable. — **DONE 2026-11-12**: 11 golden tests; the vectors were produced by `dotnet run` of the decompiled `UnifiedRandom.cs` (not by the Rust port) | **M** | 1 |
| 3 | **`Main.SwapRandom` named-stream registry** in `App` (`HashMap<String, UnifiedRandom>` + a scope guard), mirroring `Main.cs:3204-3215` and the 13 stage names of `Main.cs:17749-17809`. Keep the port's `u64` fields as *aliases* until each stage is migrated, so nothing regresses mid-migration. — **DONE 2026-11-12** (`S1-A-2-named-rng-streams`): `crates/tr-app/src/app/rng.rs` (`NamedRngs`/`RngKey`, all 17 keys, lazy `UnifiedRandom::new(worldSeed)`, reset per world); `clock_rng` is already replaced by `_rngs["UpdateTime"]` and the remaining bridges are S1-A-11 | **L** | 1, 2 |
| 4 | **Migrate the runtime consumers** (spawn, prefix, loot, chest, sandstorm, wind, item drops) off the splitmix stand-ins onto the `UnifiedRandom` streams, one call site group at a time, each with a before/after determinism pair. | **L** | 1, 3 |
| 5 | **Re-key the per-consumer streams to the C#'s seed rule** (`new UnifiedRandom(worldSeed)` per named key) and delete the arbitrary XOR constants (`0x5DEECE66D1A49B5F`, `0x0A5CE17A1C1E5EED`, `0x1DA91750E5C10C0C`, `0x2545F4914F6CDD1D`) and the `craft_rng: 1` constant. — **DONE 2026-11-12** (`S1-A-2-named-rng-streams`): every App stream seed is now the world seed (`npc_rng`, `craft_rng`, `ambience_rng`, `item_rng`, `prefix_rng: UnifiedRandom::new(world_seed)`, `growth_rng: FastRandom::new(world_seed)`); the four XOR constants are gone from the App seeds. The same `0x2545F4914F6CDD1D` literal survives only in the wind/rain/sand/sky/chest/audio stand-ins, which are separate tasks | **M** | 3 |
| 6 | **Fix the determinism proof**: rename `deterministic_for_seed` → `generation_is_reproducible_for_a_seed`, and split the worldgen contract into two tests — "reproducible" (the current one) and "byte-identical to the pinned pipeline hash" (`world_hash.rs`). Correct `ROADMAP.md:334-335` and the `ROADMAP`/ANALYSIS wording so "FNV proof" stops implying C# fidelity. — **DONE 2026-11-13** (S1-K-5): renamed, the header/wording corrected (ROADMAP S1-A-7 + M5-GEN-1 rows, ANALYSIS, ARCHITECTURE), and the algorithm frozen by a fixed-input golden test (`world_hash.rs::the_fnv_constants_are_frozen_by_a_fixed_input`). | **S** | — |
| 7 | **Add a `UnifiedRandom`-vs-port differential harness**: a `tools/` binary that, for a given seed, replays the C#'s draw order (transcribed from the decompiled call sites) and the port's, and reports the first divergence. This is the only way to prove the call ORDER, not just the algorithm. | **L** | 1, 2 |
| 8 | **Audit `Utils.Clamp` / `Math.Round` / `NextFloat` numerics** across the Rust tree: replace `Ord::clamp` with a C#-order `clamp_cs`, use `round_ties_even` where `Utils.Round` is ported, and align the float quantum. | **M** | — |
| 9 | **Correct the premise in the audit docs**: `Item.seed`, `NPC.seed`, `NPC.netSeed` and `Utils.GetPointHash` do not exist in 1.4.5.8 — record the actual per-entity RNG state (`Main._drawRand`, `Main.TileFrameSeed`, `FastRandom.WithModifier`, `Main.SwapRandom` keys) so later modules do not chase phantom fields. | **S** | — |
| 10 | **Fix the `spawner.rs` doc comment** ("xorshift64\*" → splitmix64) and cross-link `spawner.rs`, `prefix.rs`, `chest_gen.rs` to the planned `tr-core` stream module so the three copies of `next_u64` can collapse into one. | **S** | 1 |
