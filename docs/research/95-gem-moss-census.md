# 95 — Gem-moss biome gen census + fidelity port (W4-c29)

Code-only census of the gem-moss worldgen residuals against
`decompiled/Terraria/WorldGen.cs`, scoped to the task brief: **gem-moss
placement** (`Spread.Gem`, the exposed-gem `PlaceTile` arm, `neonMossBiome`,
the moss pass's arms) vs the landed M5-STRUCT-5 port
(`crates/tr-core/src/worldgen/spider_gem_moss.rs`). The spider-cave residuals
(web placement / egg sacs / wall coverage) are the **g41 co-lane's claim** —
read first, taken by them, NOT re-ported here.

## Co-lane claim check (read FIRST, per the brief)

- **`origin/task/W4-g41-biome-spider-cave`** (tip `ee937411`): the claim
  commit `65cdf3fe` is empty (claim-only); the lane then landed the full
  spider-cave residual set (`e5e2d783` + `c14f4be1` + `8ffd9b7b` +
  `ee937411`): the egg-sac validator chain (`CheckStalactite` /
  `UpdateStalagtiteStyle` / `GetStalagtiteStyle` /
  `GetDesiredStalagtiteStyle` / `InvalidTileForPilesOrSpeleothems`,
  `WorldGen.cs:38643-39175`), the missing stalactite arms, the
  `PlacePot`/`PlaceSmallPile` fixes, the webs-pass DST/ILied arms, the spider
  shroom veto + ILied dispatch + remix y-bands, `SpiderWallRemover`. Its
  census (`docs/research/93-spider-cave-census.md`) explicitly records
  **`Spread.Gem`'s duplicate-visit redraw as "left for the gem-fidelity
  row"** — that handoff is this task's lane. g41 left `spread_gem` and
  `pass_moss` byte-identical to the M5-STRUCT-5 base.
- **`origin/task/W4-d105-worldgen-passes-5`** (tip `5bb4116e`): the lane-5
  range claim (`docs/research/12e-worldgen-passes-5.md`) owns the classic
  unexecuted passes #37-45, including **#43 `Random Gems`
  (`ExposedGemsUnderground`, `:20729`) and #44 `Moss Grass` (`LongMoss`,
  `:20769`)** — whole PASSES, not the fidelity fixes to the already-landed
  `Gem Caves`/`Moss` passes. Those two pass ports stay d105's; this task
  takes only the fidelity residuals inside the landed passes.
- The other worldgen lanes (d101-d104, d106, d107, f24, f25, g36-g39, c30,
  c31, c32) claim disjoint ranges (granite split, wood tree walls, cave
  walls, traps, settle liquids, special seeds, skyblock, surface ore/stone,
  fallen log, dither snake, hive rewrite, dungeon styles) — none touches
  `spread_gem`/`neon_moss_biome`/`pass_moss`.

## Census verdicts (gem-moss placement, vs the landed M5-STRUCT-5 port)

| C# | Port (base) | Verdict |
| --- | --- | --- |
| `Spread.Gem` `:3594-3680` — the BFS, `hashSet.Add(item)` return IGNORED (`:3617`) | `spread_gem` | **WRONG (RNG)**: the port's `if !seen.insert(item) { continue; }` dedup dropped the duplicate-visit re-draws. A cell two open neighbours enqueue in one wave is processed twice; the second visit reads the freshly painted gem wall (`has_real_wall`) and takes the solid branch, re-drawing the `randGemTile` conversions. **Ported: the insert ignores its result.** |
| `PlaceTile`'s 178 arm `:59691-59701` — `CheckAndAdjustMultiDirectionalTile` gate + `frameX = style*18`, `frameY = Next(3)*18` | `place_framed` (the gem call site) | **MISSING**: the port's single-tile idiom carried neither the anchor gate nor the frame-row draw — it filled the deep pocket interior (no solid neighbour) and wrote `frame_y = 0`. **Ported: `place_exposed_gem` + `check_and_adjust_multi_directional_tile` + `valid_anchor_for_multi_directional_tile`.** |
| `CheckAndAdjustMultiDirectionalTile` `:60314-60366` + `ValidAnchorForMultiDirectionalTile` `:60297-60312` | — | **MISSING**: the four-face anchor capture (above/below/left/right), the boulder/door veto (`Sets.Boulders`, kind 10), the priority pick below(0) → left(2) → right(3) → above(1), each requiring solid-not-solidTop. **Ported.** |
| `SolidTileAllowTopSlope` `:70320-70338`, `SolidTileAllowLeftSlope` `:70340-70358`, `SolidTileAllowRightSlope` `:70360-70378` | only `solid_tile_allow_bottom_slope` landed (`edit/kill.rs`) | **MISSING**: the three sibling slope-allowing predicates. **Ported beside the bottom sibling** (the top variant accepts the planter box 380 + platform-halfbrick; the left/right variants veto platforms + halfBrick and test the opposite face's slope). |
| `neonMossBiome` `:9783-9876` — the rock-line clamp on `num6` (TOP edge, `:9843`), the clamps SET ±5.0 (`:9848`/`:9856`), the scaled-double step budget (`:9800-9807`), the exclusive paint loops (`:9860-9862`) | `neon_moss_biome` | **WRONG ×4**: the draft clamped `ymax` (the BOTTOM edge) instead of `num6`, nudged the drift ±2.0 instead of setting ±5.0, truncated the step budget to i32 before the loop, and painted the inclusive rim (`..=`). **Ported: all four classic-path fixes.** The remix clamp (`Main.remixWorld`, `:9832-9842`) stays unported (the seed-fidelity rows own the alternative draw schedules). |
| Moss arm A `:17554-17566` — the `num2`/`num3` reject ledger | `pass_moss` arm A | **WRONG**: the draft advanced the placed count on EVERY attempt; the C# bumps the consecutive-reject streak `num2` and only advances `num3` once the streak passes `maxTilesX`, resetting `num2` on an accepted anchor. **Ported.** |
| Moss arm A exclusion `:17545` — `Main.tileDungeon` in the scan set | `pass_moss` arm A | **MISSING**: the draft's set omitted the dungeon bricks (41/43/44/677/678/679, `Main.cs:7992-7997`). **Ported: the `TILE_DUNGEON` const.** |
| Moss arm F `:17659-17698` — the LIVE column-major walk over `Main.tileMoss` (627 included, `Main.cs:7031`) | `pass_moss` arm F | **WRONG ×2**: the draft snapshotted a ROW-major list (a different visit order — the sweep's conversions feed later cells) and omitted the rainbow moss 627. **Ported: the live column-major walk over the generated `TILE_MOSS` set.** |
| `Main.tileFrameImportant[178]`/`[184]` (`Main.cs:7413-7414`) | `boot_statics` | **MISSING**: the framer must keep the stored gem frames (`frameX = style*18`, `frameY = Next(3)*18`) through the load-time re-frame. **Ported.** |

## Landed-and-verified (NOT changed)

- `Spread.Moss` `:3526-3592`: the duplicates are draw-free idempotent writes
  (the wall paint + the stone→moss retype), so the port's dedup is faithful
  (the g41 census's own note). The `BASELINE_WALLS` wall-0 adaptation stands.
- `gemCave` `:9719-9735`, `randGem` `:9737-9745`, `randGemTile`
  `:9747-9762`, `Spread.Gemmable` `:3791-3798`: draw order + the rejection
  loops + the 19/20 stone gate faithful.
- `randMoss` `:9764-9781`, `setMoss` `:9878-9884`: the neon pick + the three
  distinct region indices + the x-banded pair faithful.
- Moss arms B/C/D/E (`:17574-17658`): the pocket search, the scatters, the
  exposed-surface moss (the capped arm-D loop is the documented M5-STRUCT-5
  termination guard), the lava-moss budget loop faithful.
- The `countTiles`/`nextCount` pocket measure (`:9546-9635`): the
  iterative-stack rewrite is draw-free, the baseline-wall adaptation stands.

## RNG / hash contract

- Every new draw rides the EXISTING world-pass slots: the 178 arm's
  `Next(3)*18` on `pass_rng("Gem Caves")` (via `Spread.Gem`), the moss arm
  A/F changes on `pass_rng("Moss")`. **No new stream slot.**
- The duplicate-visit re-draws, the anchor-gated 178 placements (fewer, but
  each with a frame-row draw), the neon-moss clamp/step/loop fixes and the
  arm A ledger change the gem-cave and moss-pass tiles and draw counts — so
  `DEFAULT_WORLD` moves BY DESIGN (arm A places `4200/2100 = 2` neon biomes,
  the gem/moss cave passes find pockets). `SEED42_512`'s bands collapse on
  the compact 512 terrain (arm A's `width/2100` rounds to 0), so whether it
  moves depends on the 512 pockets. Both re-pins are DEFERRED to the merge
  gate (no compiler on this branch — the S1-K-2 wave B / g41 precedent); the
  `a == b` determinism halves stay green.
- 5 witnesses written NOT run: the `Spread.Gem` a==b + rim-only placement,
  the 178 anchor gate + frame row + priority pick + boulder veto, the
  neon-moss rock-line clamp, the arm A ledger + dungeon exclusion + a==b,
  the arm F rainbow-moss live sweep.

## Status

- [x] Co-lane claims read (g41 spider residuals taken; d105 owns the
      Random Gems / Moss Grass passes; the gem-fidelity handoff unclaimed)
- [x] Census verdicts recorded (this doc)
- [x] `Spread.Gem` duplicate-visit redraw ported
- [x] The 178 `PlaceTile` arm + the multi-directional anchor chain ported
- [x] The three slope-allowing solid predicates ported
- [x] `neonMossBiome` classic clamp/step/loop fixes ported
- [x] Moss arm A reject ledger + `tileDungeon` exclusion ported
- [x] Moss arm F live column-major `Main.tileMoss` sweep ported
- [x] The 178/184 frame-important boot statics ported
- [x] 5 witnesses written (NOT run — code-only)
- [x] ANALYSIS / PORT_MAP / ROADMAP / anchors updated
- [ ] Hash re-pins at the merge gate (deferred — no compiler on this branch)
