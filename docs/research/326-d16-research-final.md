# 326 — W4-d16 RESEARCH FINAL (the aiStyles 151-160 lane, SEALED)

> **RENUMBER 246 → 326** per the 150 registry §9.2 FINAL assignment table
> (UNIFIED FINAL, d54 tip `b243c7086`): **246 = e23's published
> reservation WINS** (§7.4, publish `72226e500` 14:47:50 — the 18b-fork
> `246-element-census-e23.md`); d16's `246-d16-research-final.md` renumbers
> to **326**. The companion bible renumbers 247 → 327 (247 = d10's
> reservation, EXECUTED `72431e61f`). Chain: 246 → 326.

Branch `task/W4-d16-proj-ai-151-160`, base `a725acb04` (main, wave-3
complete). Code-only lane: no cargo/tests/rustfmt/graphify invoked; every
witness WRITTEN NOT RUN. The original number was self-picked from the
free block a fresh 382-tip `git ls-tree docs/research/` scan
(2026-11-21, this lane) showed open (max bare number taken **245**,
`245-39-item-ownership-seed.md`) — the scan missed the published
reservations; the 150 registry §9.2 adjudication now stands: this doc is
**326**, the bible is **327**.
The merger-facing family record is `327-kite-family-bible.md`.

## 1. The commit arc (17 commits, base..41080e159)

| # | commit | scope |
| --- | --- | --- |
| 1 | `a03fbcbd4` | aiStyle 151 AI_151_SuperStar verbatim (`Projectile.cs:22138-22168`) + the 728 row |
| 2 | `939816d43` | aiStyle 152 AI_152_SuperStarSlash verbatim (`:22230-22351`) + the 729/977/976/1045 rows |
| 3 | `0581aab17` | aiStyle 153 AI_153_ToiletPop verbatim (`:52198-52236`) + the 733 row |
| 4 | `37cbc001f` | aiStyle 154 AI_154_VoidLens verbatim (`:52154-52196`) + the 734 row + the pool-side dedupe |
| 5 | `43650c225` | aiStyle 155 AI_155_MysticSnakeCoil verbatim (`:51939-52152`) + the 754 row + the rope seams |
| 6 | `a6fe65614` | aiStyle 159 AI_159_PaperAirplanes verbatim (`:48695-48755`) + the 761/762 rows + the Kill recovery drop (4343/4344) |
| 7 | `e122a6b24` | aiStyle 160 AI_160_Kites + KiteLogic verbatim (`:48421-48693`) + the 24 DefaultToKite rows + kite_ctx/visual_scene seams + the Item192 cue seam |
| 8 | `fa8405474` | residual 1: the kite items' use-path (IsAKite 24, LMB/RMB/channel/holdStyle/refusals/anchor route) |
| 9 | `9a573cd89` | residual 2: the kite string DRAW (Main.DrawKite `Main.cs:37134-37569`, kite_string.rs + draw_kite + Extra sheets) |
| 10 | `1b7e87a96` | residual 3: the 155 rope PlaceTile/KillTile seams verified LIVE (world.revision bumps) |
| 11 | `7f3e3da89` | residual 4: the 4343/4344 recovery item-rows census (`Item.cs:37117-37129`) + the Kill-gate witnesses |
| 12 | `d414afbca` | residual 5: the aiStyle-161 shortsword helper census (g4 G4 input; rows UNROWED, d17's claim) |
| 13 | `4d870ea1c` | residual 6: the per-kind TrailCacheLength ring FIX (60/65/80; trail_len + TRAIL_RING_CAP 80 + reset on every spawn seam) |
| 14 | `de6105f22` | residual 7: the kite WIND interplay census + the InAPlaceWithWind out-of-world SKIP fix |
| 15 | `6d324abb7` | residual 8: the afk auto-kite LANDED (the hotbar.rs afkCounter skip closed) |
| 16 | `1634e6989` | residual 9: the 723 kite-anchor place/kill chain LANDED (TileEntity place hook + kill teardown) |
| 17 | `41080e159` | residual 8/7 witness fixes (the selectedKite PRE-override re-pin; the wind-place-gate leash fixture) |

The SEAL round (this doc's commit) adds: the shared `tr_core::wind`
copy's out-of-world fix + witness, the stale-comment corrections (the
afk seam order, the trailing_mode ring notes, the empress 872 ring note),
the 728/729 d77 compose contract, this FINAL + the bible + the INDEX/
PORT_MAP/anchors rows.

## 2. The witness ledger (69, ALL WRITTEN NOT RUN)

| file | count | families |
| --- | --- | --- |
| `projectile_ai_misc.rs` | 37 | 151×4, 152×7, 153×5, 154×3, 155×9, 159×9 |
| `world_sim/editing/kites.rs` | 13 | the use-path 6, afk 4 (incl. the reeled-kill zero), anchor 3 |
| `projectile_ai_kites.rs` | 10 | the row/kill-gates/reel-cue/substep/frame/buff 6 + the wind census/lift/place-gate/dispatch 4 |
| `kite_string.rs` | 6 | the parameter table/string walk/tail chains/history clamp + the TrailCacheLength census + the deep-ring walk |
| `projectile/data.rs` | 1 | the 161 shortsword helper census (d17's rowing input) |
| `projectile.rs` | 1 | the kite rings resize to the TrailCacheLength |
| `tr-core/src/wind.rs` | 1 | the InAPlaceWithWind out-of-world SKIP arm (SEAL round) |

## 3. The SEAL checks (the Lead's five items)

1. **TrailCacheLength-kites-60/65/80-fix SURVIVES the f20 F8 ripple.**
   f20's retire (`0883a80bc`, `research/26f` F8) publishes the
   FIRST-COMPILE-FIX-LIST for the ONE merge friction: d16 `4d870ea1c`
   edits `projectile_ai_moonlord.rs`, the same file f20's F7 widened to
   `step(p, world, npcs, immune)`. Re-read at this tip: d16's additions
   are exactly the two trail_len lines — the settled-scan
   `p.old_pos[1..p.trail_len]` (`:184`, the C# `1..oldPos.Length`
   `:55441`) and the witness fill `p.old_pos[..p.trail_len].fill(...)`
   (`:446`); the 11 two-arg `step(&mut …, &w)` witness calls + the
   two-arg dispatch call (`projectile.rs:17154`) are UNTOUCHED by d16
   (f20's tree already converted its own copies). The merge fix-list is
   therefore f20's, executed by the BUILD lane: take f20's four-arg
   signature + charm tails, keep BOTH d16 trail_len lines inside the
   widened body, keep d16's trail_len-aware fill. d16's `Projectile`
   field additions (`trail_len`, the 80-cap rings,
   `reset_trailing_rings`, `trail_cache_length`, `TRAIL_RING_CAP`) live
   in `projectile.rs` — f20's tree has no such fields, so they survive
   any keep-both resolution; f20's F7 `charm_tail` calls read none of
   them. NO d16-side code change owed (changing the arity here would
   break THIS branch's compile against its own two-arg definition).
2. **wind-InWorld-guard-fix: out-of-world SKIP, not refuse.** BOTH
   copies now `continue` on the out-of-world cell (the C# `InWorld`
   guard `WorldGen.cs:86868`): the kite pool's own
   `projectile_ai_kites.rs:79` (residual 7) AND the shared
   `tr_core::wind::Wind::in_a_place_with_wind` (the SEAL round — its
   `drawdata_cases_b`/`player.rs`/`render/tiles.rs` consumers had the
   same refuse-on-`None` bug). The in-world liquid/wall refusals stand
   (`WallID.Sets.AllowsWind`, `WallID.cs:95`); the height gate runs
   first. Witness: `w4_d16_in_a_place_with_wind_skips_out_of_world_cells`.
3. **afk-auto-kite + the 723-anchor-chain LANDED** — re-verified at
   this tip: the counters/scan/override (`kites.rs:141-251`, the seam
   `world_sim/player.rs:506-583` post-selection pre-movement), the
   place hook (`kites.rs:283`, the `editing.rs:7617` seam, the
   pre-consume capture), the kill teardown (`tr-core/src/edit/kill.rs:317`
   `anchor_teardown`, the 723/724 type match + the stored-item drop).
   The SEAL round corrected the `afk_counter_tick` doc (it claimed
   "AFTER the movement tick" — the seam is BEFORE it, the previous
   tick's velocity, the C# `:25497` order).
4. **d77-cues-728/729-armed COMPOSE** — d77 `0573b70f2` arms
   `728 | 729 => (2, 10, pos, 1.0)` on its `kill_tail_cue` map (the
   re-probe wave, its witness drives hand-built slots); d16's half is
   the ROWS + the 151/152 dispatch arms (landed, commits 1-2). d16 does
   NOT self-arm the pair (a second arm would double-play post-merge);
   the 761/762 paper-airplane cue stays d16-self-armed per d77's own
   keep-one list ("the self-armed lanes … 761/762 d16 … stay keep-one").
   Merger instruction: keep d77's map row, drop any duplicate 728/729
   arm, keep d16's 761/762 death-hook block verbatim.
5. **research FINAL + kite-family bible** — this doc (`326`) + `327-kite-family-bible.md`.

## 4. Standing deviations (unchanged, ANALYSIS "W4-d16 kite residuals")

- The 640's C#-20 ring keeps the port's 10 (its own recorded deviation;
  the kites' 60/65/80 CLOSED). The unported 120/41/51/71 kinds raise
  `TRAIL_RING_CAP` when they land (the empress 872 note now cites the
  seam).
- The selectedKite scan re-pins PRE-override (the one-tick-early
  deviation, observation-identical for a stable inventory — `41080e159`).
- The ring shift runs once per update call, not per sub-step (the
  recorded trail-density divergence, `shift_trailing_history`).
- Skips: the leashed-kite visual (M7-TE-2), the CheckAnchor reorient
  (the framing pass), the netMode-1 SendData arms, the mouse-item
  slot-58 override arm, petting/spectating (no port fields), the
  holdStyle item-instance mutation (pool-derived per tick).

## 5. Verification record (SEAL round)

Direct-file re-reads only (no junction recursion): the C# windows
(`WorldGen.cs:86841-86882`, `Player.cs:17410-17432`/`:25490-25514`,
`Projectile.cs:75125-75158`/`:7659-7687`), the f20 retire doc + its
widened `projectile_ai_moonlord.rs` (read from the git object, not the
junction), the d77 tip's cue map + witness (git object), the 150
registry (git object) + the fresh 382-tip ls-tree scan. No
cargo/tests/rustfmt/graphify. The two code fixes (the shared wind copy,
the comment corrections) carry one new witness; all others re-verified
by read.