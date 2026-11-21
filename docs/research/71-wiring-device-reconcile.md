# 71 — Wiring device-arm duplicate reconcile (W4-e20 × W4-f11) + e21 census

> Merger assist · 2026-11-21 · code-only (no cargo/tests run) ·
> sources: `decompiled/Terraria/Minecart.cs`, `Wiring.cs`, `Utils.cs`, `Item.cs`,
> `Chest.cs` + the three branch tips below.

**Scope.** W4-e20's D8-residual arms (`fd34e2c8` + fix round `474bc8b5`,
`dcbb29f8`, `dfabdb5b`, `2322af90`, branch
`task/W4-e20-wiring-device-arms-2`) DUPLICATE W4-f11's batches 1+3
(`8773057a` minecart/314, `e0d1f245` hoppers/extractinators, tip `cf0737da`,
branch `task/W4-f11-wiring-arms-4`): tile 314 `FlipSwitchTrack`, 21/467
`Hopper`, 219/642 `Extractinator`. Both lanes branched from `a725acb0`.
This report compares every duplicated function against the C# and gives a
per-fn KEEP recommendation, the witness union, and the e20-unique fixes that
MUST survive the dedupe. The e21 census (§4) verifies no overlap.

**Files compared** (line numbers are at the named tips):

| Lane | File | Content |
| --- | --- | --- |
| e20 | `crates/tr-app/src/minecart.rs` (existing M2-PHYS-7 module, extended) | `TRACK_FRAMES` :158-231 (pre-existing), `track_switch_options` :1313, `nearby_track_mask` :1346, `frame_track_pound` :1374, `flip_switch_track` :1497 |
| e20 | `crates/tr-app/src/world_sim/wiring.rs` | `coins_count` :564, `coins_split` :580, `try_move_coins_in_chest` :597, `try_adding_to_stack` :668, `try_put_item_in_chest` :693, `hopper_pull` :737, `find_chest_for_extractinator` :814, `extractinator_roll` :835, `spear_cap_allows` :511 |
| e20 | `crates/tr-core/src/wiring.rs` | `WireState::any` :126, request vecs :308-321, `TrapRequest::Spear` `cap_x/cap_y` :384+, cases 314/21/467/219/642 :2385-2397, wall-trap `vy: 0.0` :984 |
| f11 | `crates/tr-core/src/minecart.rs` (NEW module) | `LEFT/RIGHT_SIDE_CONNECTION`, `TRACK_TYPE`, `BOOST_LEFT` :22-52, `nearby_tiles_set_lookup_index` :63, `track_switch_options` (OnceLock) :92, `frame_track` (full) :146, `flip_switch_track` :357 |
| f11 | `crates/tr-app/src/world_sim/wiring_devices.rs` (NEW module) | `run_hopper` :43, `try_to_put_item_in_chest` :110, `try_adding_to_stack` :133, `try_adding_to_empty_slot` :164, `try_move_coins_in_chest` :198, `coins_split` :268, `run_extractinator` :287, `try_find_chest_for_extractinator` :339 |
| f11 | `crates/tr-core/src/wiring.rs` | case 314 (direct core flip) :2668, hopper anchor+CheckMech :2767, extractinator anchor+CheckMech :2786; wall traps still `vy: 2.0` :980; cap still spawn-point-vs-`pos` (app `spear_cap_allows` :506-547) |

## 1. Track tables — BOTH faithful; e20 is the superset

Hand-verified all 36 rows of both transcriptions against the C# `Initialize`
sequential `num++` writes (`Minecart.cs:125-427`):

- f11 `LEFT_SIDE_CONNECTION` / `RIGHT_SIDE_CONNECTION` / `TRACK_TYPE` /
  `BOOST_LEFT`: **every row matches** (left −1/1/−1/1/2/1/1/0/0/2/2/−1/0/−1/−1/
  1/2/−1/0/−1/−1/1/−1/1/−1/1/2/−1/0/−1/1/1/0/2/0/2; right −1/1/1/−1/1/2/0/1/2/
  0/−1/2/−1/0/1/−1/−1/2/−1/0/−1/1/1/−1/1/−1/−1/2/−1/0/1/1/2/0/2/0; type-1 at
  20-23 `:299/:304/:310/:316`, type-2 at 30-35 `:379-425`; boostLeft
  F/T/F/T/T/F `:380-426`).
- e20 `TRACK_FRAMES` (landed with M2-PHYS-7, already consumed by
  `track_collision`): the same left/right/type/boost columns **plus**
  `tile_height[8]` (post-epilogue `(8−v)·2`, `:437-447`) and
  `TEXTURE_POSITION[40]` (`:129-436`). Also row-for-row correct.

**KEEP: e20 `TRACK_FRAMES`** — single source of truth, already wired into the
ride physics; f11's four flat arrays re-transcribe a strict subset into a second
module (`tr-core::minecart` vs the existing `tr-app::minecart`). If the merger
keeps f11's full `FrameTrack` (see §2), its table reads must be re-pointed at
the e20 table (or the table moves to tr-core with an app re-export) — never
ship both copies. `FIRST_PRESSURE_FRAME = 20` exists in both (e20 :284,
f11 :56); keep one.

## 2. Per-fn comparison

| Fn (C# anchor) | e20 | f11 | More faithful | KEEP |
| --- | --- | --- | --- | --- |
| `GetNearbyTilesSetLookupIndex` (`Minecart.cs:1170-1198`) | `nearby_track_mask` :1346 — bits 1/2/4/8/16/32, `kind == 314` with NO active gate (verbatim) | `nearby_tiles_set_lookup_index` :63 — identical | tie (identical) | either; e20's if keeping its module |
| `_trackSwitchOptions` build (`:449-518`) | `track_switch_options(mask)` :1313 — recomputed per call, `count_ones`, exact-match <2 / non-zero-subset ≥2 rule verbatim | OnceLock cache of all 64 masks :92 — the C# `Initialize` builds ONCE; same fit rule | **f11** (build-once structure; e20 allocates a Vec per wire hit — output identical) | f11's caching shape over the e20 table |
| `FrameTrack` pound branch (`:1040-1108` + writes `:1143-1162`) | `frame_track_pound` :1374 — verbatim index walks; front-absent → `assert!` panic (the module's C#-`IndexOutOfRangeException` convention); `flag4`/`KillTile(fail)` omitted (unreachable under `mute: true`) | `frame_track` :146 pound branch — same walks, but front-absent → `num4.max(0)` **silent clamp** (C# throws, `:1085`); ports `flag4` + `KillTile(fail: true)` `:1163-1166` | **e20** on the degenerate state (panic ≈ C# throw; f11's clamp silently changes behaviour). f11 additionally ports the non-pound branch `:976-1038` + the `!pound` no-options reset `:966-975` + the fail-tile kill — all UNREACHABLE from the wiring caller (`pound: true, mute: true`, `:1315`) | e20's pound slice for the wiring arm. f11's non-pound branch + `flag4`/`KillTile` are the future hammer-placement half — salvage as a follow-up note, not as wiring code |
| `FlipSwitchTrack` (`:1296-1319`) | :1497 — front −1 guard, out-of-range guard, type-0 front/back swap (`BackTrack != −1`), type-2 → pound re-frame, type-1 no-op | :357 — identical arms and guards | tie (behaviourally identical; both add the out-of-range guard the C# lacks, both drop `SendTileSquare` — SP) | e20's (reads the kept table; engine seam already witnessed) |
| Engine 314 arm (`Wiring.cs:1744-1748`) | core `CheckMech(5)` → `switch_track_flips` request; app flips after the step (:466-468) | core `CheckMech(5)` → **direct** `crate::minecart::flip_switch_track` inside `hit_wire_single` :2668 | f11 is closer to the C# synchronous call; e20 follows the lane's D1 request seam (the table lives app-side). Both gate identically | e20's request seam (its witnesses pin it); if the table ever moves to tr-core, f11's direct call becomes the simpler shape |
| `Hopper` anchor + gates (`:2638-2659`) | app `hopper_pull` :737 — `%36` anchor, `check_mech(60)`, `is_locked`, `find_chest`, `using_chest` in C# order | core resolves `%36` anchor + `CheckMech(60)` :2767, app `run_hopper` :43 runs locked/find/using | tie on outcome (a second same-trip hit is refused either way — the mech queue is shared). f11 gates during the trip (C# order); e20 at drain | either; keep the seam of whichever hopper body survives (recommend e20's, below) |
| `Hopper` grab walk (`:2660-2673`) | floored top-left `(cx−96) as i32` (`Utils.CenteredRectangle`), strict-`<` `rects_intersect` (474bc8b5), banned set, remainder written back to the drop, `TurnToAir` = in-place `count=0/kind=0` | same rect math (float gx, but `rects_intersect` casts to i32 — equivalent), same banned set; consumed drops **`swap_remove`d after the walk** (reorders the pool; C# deactivates in place) | **e20** (in-place deactivation, explicit floor; f11's sweep is a pool-order deviation) | e20 `hopper_pull` |
| `TryToPutItemInChest` (`:2680-2701`) | :693 — returns `(placed, remaining)`; coin arm, stack pass, empty pass in C# order | :110 — returns bool, mutates the drop in place; same order | tie (both correct incl. partial merge) | e20's tuple shape (the hopper writes the leftover explicitly) |
| `TryAddingToStack` (`:2772-2798`) | :668 — `num > 0` on ANY merge (dcbb29f8), type+prefix `CanStack`, maxStack cap; unreachable `type==0` clone arm documented out | :133 — same semantics (`num > 0` :159 was correct in f11 from the start) | tie | e20's |
| `TryAddingToEmptySlot` (`:2759-2770`) | folded into `try_put_item_in_chest` :713-726 — **NO `PlaySound(7)`** | :164 — clone + `newAndShiny=false` fold + **`play_pickup_sound(false, wiring rng)`** :175 (the case-7 pitch draw, `SoundEngine.PlaySound(7)` `:2765`) | **f11** — e20 drops the sound AND its RNG draw (a determinism-visible omission on the wiring stream) | e20's body + **port f11's sound call into it** (must-survive, §3) |
| `TryMoveCoinsInChest` (`:2703-2757`) | :597 — `value/5 × stack` (registry sell value), `coins_count` over slots, reserve high→low, clear-all-coins, write capped at maxStack, drop air | :198 — `coin_value × stack` ([1,100,10⁴,10⁶] — the `CoinsCount` switch constants directly); same reserve/clear/write | tie (arithmetically identical: sell values 5/500/5·10⁴/5·10⁶ ÷5 = the copper counts) | e20's |
| `Utils.CoinsCount`/`CoinsSplit` (`Utils.cs:1019-1060`) | :564/:580 — i64, the 71/72/73/74 switch verbatim, `(count−acc)/denom` high→low | inline count :203-210 + `coins_split` :268 (u64→i64) | tie (identical maths; f11's witness `coins_split(1_234_567) == [67,45,23,1]` is a good pin) | e20's fns + f11's split witness |
| `TryFindChestForExtractinator` (`:2586-2608`) | :814 — `ax−2..=ax+5` × `ay−2..=ay+5`, x-outer row-major, `!is_locked` then `find_chest` | :339 — `(x−2)..=(x+3+2)` (same bounds), same order | tie (identical) | e20's |
| `Extractinator` scan (`:2554-2584`) | :835 — `%54/18` anchor, `check_mech(60)`, find+using gates, reverse walk `maxItems−1..0`, `IsAir` skip (`kind==0 || count==0`), mode roll, `--stack<=0` air | :287 — same anchor (engine-side), same gates, same reverse walk and air test, same consume | tie on the scan | e20's |
| Extractinator **drop spawn** (`Item.NewItem(num*16, num2*16, 32, 32, …)` `:2579` → center `(num*16+16, num2*16+16)`, `Item.cs:49649-49651`) | `spawn_item_drop_px` at `Vec2(ax*16+16, ay*16+16)` :883-889 — the anchor CENTRE, the same convention as the player-mode extractinator keep-one seam (`extract.rs:121`, cursor center) | `spawn_item_drop(items, rng, …, num, num2)` :322-329 — the tile ORIGIN `(num*16, num2*16)`; the drop lands ~8 px up-left of the C# center | **e20** | e20's |
| Extractinator roll RNG | roll + spawn both on `app.wiring.rand_mut()` (S1-L-6c) | same | tie | e20's |
| `SortInventory` on success (`:2674-2677`) | `item_sorting::sort(slots, &[], false, mult)` :801-808 | same :94-104 | tie | e20's |

**Net recommendation:** keep the **e20 implementation** as the surviving
device-arm code (it is the more faithful hopper/extractinator body, reuses the
existing M2-PHYS-7 track table, and its witnesses are written against its
seam), and **drop f11's duplicate halves** (`tr-core/src/minecart.rs`,
`world_sim/wiring_devices.rs`, the core 314/21/467/219/642 arms) — EXCEPT the
three f11-unique fidelity wins that must be merged INTO the kept code (§3b).
f11's batch-2 doors/trapdoors/tall-gates and batch-1 torch/campfire/slab arms
are NOT duplicated by e20 and land untouched.

## 3. Must-survive lists

### 3a. e20-unique fixes (f11 still carries the old WRONGs — verified at cf0737da)

1. **Wall traps −10/−9/−8 `speedY = 0`** (`Wiring.cs:1898/:1914/:1930` assign
   only `speedX`; the `+2` is `vector3.Y += 2f`, `:1903/:1919/:1935`).
   e20 core :984 `vy: 0.0`; **f11 core :980 still `vy: 2.0`**. The e20 witness
   (`trap_tests`, "the +2 is a position offset, not speedY") must survive.
2. **Spiky-ball cap anchor + Center (F3)** (`Wiring.cs:1819/:1950`): the cap
   measures `(i*16+8, j*18+8)` — the **18** verbatim — against
   `Main.projectile[..].Center`. e20 adds `cap_x/cap_y` to `TrapRequest::Spear`
   and `spear_cap_allows` reads `p.center()`; **f11 still measures the spawn
   point against `p.pos` (top-left)** with the 16-scaled y. The re-derived
   fixtures (`dfabdb5b`: blocking at centers ~40 px, discriminating at
   pos 161,392+k) and `dw_4_2_the_spiky_ball_cap_measures_from_the_verbatim_anchor_to_center`
   must survive. NOTE: adding the two fields changes `TrapRequest::Spear`'s
   shape — f11's constructors/patterns must be updated to the e20 shape at
   merge, not the reverse.
3. **Hopper partial-merge remainder** (dcbb29f8) — f11 happened to be correct
   here too, but the e20 witness `dw_e20_the_hopper_partial_merge_keeps_the_remainder`
   (9990+20 dirt → 9999 chest / 11 ground) is the only pin; keep it.
4. **Hopper grab-box strict-`<` + floored top-left** (474bc8b5) — both lanes
   use the same `rects_intersect` helper, but keep the e20 comment/fixture.
5. **Extractinator drop at the anchor centre** (§2 last rows) — f11's
   tile-origin spawn is wrong; the e20 center spawn must survive.
6. **`WireState::any`** (`Tile.anyWire`, e20 core :126) — new helper, no f11
   counterpart (currently unconsumed; the `IsHopperInRangeOf` `:2628` follow-up
   reads it). Keep.
7. **e20 witness set with no f11 counterpart** (all in `s1_l6_tests` /
   `trap_tests` / app `tests`): the b=0 neighbour-rule pipe-loop pin (D1,
   `Wiring.cs:900-902`), the 429 WireBulb four-colour bits (1→18/2→72/3→36/
   4→144), the light-family multi-cell flips + SkipWire sets (42/93/95/100/
   126/173/564/34/92/149/244/565) incl. the corrected lantern witness
   (`2322af90`: both cells share frameX 0/18), the cap fixtures, the six
   `dw_e20_*` device witnesses, and the two core request-emission witnesses
   (:6294/:6306).

### 3b. f11-unique wins to merge INTO the kept e20 code

1. **`PlaySound(7)` in the hopper's empty-slot fill** (`Wiring.cs:2765`,
   f11 `wiring_devices.rs:175`): port `app.audio.play_pickup_sound(false,
   app.wiring.rand_mut())` into e20's empty-slot arm (`try_put_item_in_chest`
   :713-726 needs the rng threaded in, or the sound played at the `hopper_pull`
   call site on the empty-slot success path). Without it the wiring RNG stream
   diverges from C# on every empty-slot hopper fill.
2. **Build-once switch-options cache** (f11's OnceLock, `Minecart.cs:449-518`
   builds in `Initialize`): wrap e20's `track_switch_options` in the same
   64-mask OnceLock (output identical; removes the per-hit Vec allocation).
3. **f11 witnesses worth keeping** (re-pointed at the e20 seam):
   `w4_f11_coins_split_matches_the_csharp_denominations` (1234567 → [67,45,23,1]),
   `w4_f11_the_hopper_grab_rect_is_192_centred`,
   `w4_f11_a_locked_chest_refuses_the_hopper` (the locked-gate app witness e20
   lacks), `w4_f11_the_hopper_resolves_its_2x2_anchor` /
   `w4_f11_the_extractinator_resolves_its_3x3_anchor_and_kind` (non-anchor-cell
   hits — e20's app witnesses only hit the anchor cell; re-point these at
   `hopper_pull`/`extractinator_roll` with a non-anchor frame),
   the cooldown-expiry re-hit witnesses (`w4_f11_the_switch_track_flips_and_the_cooldown_gates_the_rehit`,
   `w4_f11_the_hopper_cooldown_gates_the_rehit`,
   `w4_f11_the_extractinator_cooldown_gates_the_rehit` — e20 pins the refusal
   but not the post-expiry re-fire).
4. **Follow-up note (not wiring code):** f11's non-pound `FrameTrack` branch
   (`:976-1038`), the no-options reset (`:966-975`) and `flag4` +
   `KillTile(fail: true)` (`:1110-1166`) are the hammer-placement half
   (`edit/hammer.rs`'s `Consumed` seam). Record as the future task; when it
   lands, use the e20 assert-on-front-absent convention, NOT f11's `.max(0)`
   clamp (C# throws `IndexOutOfRangeException`, `:1085`).

### 3c. Witness union (dedupe overlap)

Both lanes wrote a plain-rail front/back swap witness (e20
`dw_e20_the_switch_track_flips_front_and_back` + `flip_switch_track_swaps_a_normal_tracks_front_and_back`;
f11 `w4_f11_a_plain_rail_swaps_front_and_back` + the core cooldown witness) and
a hopper-pull + extractinator-roll app witness each — keep ONE of each pair
(the e20 versions, they ride the kept seam) plus the f11-unique pins in §3b.
e20-only: pressure no-op + booster 30→31→30 re-frame witnesses
(`minecart.rs:1595/:1605`), coin-merge witness, mode-−1 no-op witness.
f11-only: the table popcount sanity witness (`w4_f11_the_track_tables_transcribe_the_isolated_and_straight_frames`)
— keep, re-pointed at `TRACK_FRAMES`.

## 4. e21 census (`12fb1a2e`, branch `task/W4-e21-wiring-device-arms-3`) — NO overlap

e21's three commits (`8ebc25dd`, `fdbf37d9`, `12fb1a2e`) vs the e20 slice
(`git diff a725acb0..HEAD` hunks checked one by one):

| e21 change | Location | e20 touches there? |
| --- | --- | --- |
| Pump `XferWater` drops the invented `active()` gate (`Wiring.cs:481` reads liquid directly) | core `xfer_water` (old :1844 region) + `teleport_tests` witness (old :3912) | NO — e20's core hunks are :122/:297/:362/:774/:896-954/:2322 + test modules `trap_tests`/`s1_l6_tests` |
| Monolith flip gated to the nine kinds (`WorldGen.cs:51037-51134`) | core `SwitchMonolith` port (old :2871 region) + two `s1_l6_tests` witnesses | NO semantic overlap — e20 adds no monolith code; both append tests to `s1_l6_tests` (textual adjacency only) |
| Grate 546/557 wake drops queued (`Wiring.cs:2543/:2548`) | core cases 546/557 (old :2551 region) + one `s1_l6_tests` witness | NO — e20's new cases (314/21/467/219/642) insert at old :2322, a different match-arm region |
| Boulder statue projectile 99 rides the device-shot seam (`Wiring.cs:2002-2010`) | app `StatueSpawn::Boulder` arm (old :229-240) | NO — e20's nearest app hunk is the drain-extend block at old :247 (after `take_device_shots`), distinct lines |

**Verified: no semantic overlap.** Both lanes edit the same two files
(`tr-core/src/wiring.rs`, `tr-app/src/world_sim/wiring.rs`), so expect textual
merge adjacency in (a) the `update_wiring` drain block (e21's Boulder arm sits
~18 lines above e20's `switch_track_flips/hopper/extractinator` extends) and
(b) the `s1_l6_tests` tail (both append witnesses). Resolve by union — neither
side rewrites the other's lines. e21 does NOT touch the trap arms, the cap,
the minecart module, or the hopper/extractinator code, so the §3a fixes are
unaffected by it.

## 5. Merge checklist (for the merger)

1. Land f11's non-duplicated arms (torch/campfire/slab/doors/trapdoors/tall
   gates + `validate_tile_square` visibility) as-is.
2. Drop f11's `tr-core/src/minecart.rs` + `world_sim/wiring_devices.rs` + the
   core 314/21/467/219/642 duplicate arms; keep e20's.
3. Port §3b.1 (PlaySound(7)) and §3b.2 (OnceLock) into the kept e20 code;
   re-point the §3b.3 witnesses.
4. Keep every §3a item verbatim — f11's tip still carries the wall-trap
   `vy: 2.0` and the spawn-point/top-left cap; a merge that takes f11's
   `TrapRequest::Spear` shape or trap arms silently re-introduces both WRONGs.
5. e21 merges independently (union the two adjacency regions, §4).
6. PORT_MAP: the D8-residual row stays pointed at the e20 modules; delete any
   f11 row naming `tr-core::minecart`/`wiring_devices.rs`. ANALYSIS: keep the
   e20 "W4-e20 — the second device-arm slice" section; add the §3b.1 sound
   note when ported.

*All comparisons hand-verified against the decompiled source at the cited
lines; no cargo/tests/graphify run (code-only lane).*
