# 03 — Player core (movement, life/mana, death)

> Wave 1 · researched 2026-11-12 · sources: decompiled + crates + ROADMAP/ANALYSIS/PORT_MAP

Covers ROADMAP §7 (movement & physics), the player-status halves of §8 (regen; buffs/debuffs
belong to module 04) and the death/respawn halves of §9 (damage math belongs to module 05).
Out of scope: item use, draw layers, buff UI, item/buff tables.

## Scope

| File | Lines | Role |
| --- | --- | --- |
| `decompiled/Terraria/Player.cs` | 54 877 | `Update`, `UpdateBuffs`, `UpdateLifeRegen`/`UpdateManaRegen`, `CheckDrowning`, `KillMe`, `UpdateDead`, `Spawn`, `JumpMovement`/`DashMovement`/`WingMovement`/`GrappleMovement` |
| `decompiled/Terraria/Collision.cs` | 4 501 | `DrownCollision` :1383-1431, `WetCollision` :1572-1651, apex/velocity mask :775-830 |
| `crates/tr-app/src/player.rs` | 10 098 | `update_with_grapple` :2413-3593 — one player tick: defaults, statics, liquid handling, jumps, dash, grapple, flight, collision, fall damage, regen, death, frames |
| `crates/tr-app/src/world_sim/player.rs` | 1 488 | fixed-step driver, input assembly, dead/respawn branch |
| `crates/tr-app/src/regen.rs` | 706 | `Regen`/`ManaRegen` engines (DoT lanes, ramp, mana delay/floor) |
| `crates/tr-app/src/death.rs` | 435 | death text, tombstone roll, coin drop |
| `crates/tr-app/src/mount.rs` | 2 466 | mount `Flight`, `Hover`, `JumpHeight`, cart `CanHover`/`CanFly` |
| `crates/tr-app/src/grapple.rs` | 1 088 | hook anchor averaging, cap 11, jump-clears-hooks |
| `crates/tr-app/src/minecart.rs` | 2 638 | ride state, track boost, wheel sparks |
| `crates/tr-app/src/collision.rs` | 751 | tile AABB, `TrackCollision`, half-bricks, ice friction |
| `crates/tr-app/src/buffs.rs` | 599 | slot engine, expiry, expert debuff extension |
| `crates/tr-app/src/luck.rs` | 1 026 | `RecalculateLuck`, coin luck, torch table |
| `crates/tr-app/src/world_sim/step.rs` | 78 | `FixedStepper` tick order |
| `crates/tr-app/src/app/frame.rs` | 1 791 | per-frame orchestration (fixed steps + draw + UI) |

`Player.Update(i)` (`Player.cs:24350-28311`) is a single ~4000-line tick that runs, in order:
per-tick defaults → wet/shimmer gravity branch → ~50 `Update*` sub-calls → stat/debuff
roll-up → mount cache → gravity/jump/flight → NPC collision → movement integration →
wet recompute → tile touching/fall damage → release flags. The Rust port mirrors this as
`update_with_grapple`, which is the module's hot path and the subject of the phase-order
comparison below.

## Key entities

- `crates/tr-app/src/player.rs::update_with_grapple` (:2413) — the tick entry point; owns the
  whole phase order. Called from `world_sim/player.rs::update_player` under the stepper.
- `crates/tr-app/src/player.rs::Player` (struct, ~600 fields) — the per-player state;
  `lose_health` :2362, `kill_me`-equivalent death path, `spawn`-equivalent reset :1812-1831.
- `crates/tr-app/src/player.rs::movement_speed_for` (:169) / `liquid_gravity` (:182) /
  `liquid_fall_cap` (:188) — the three liquid helpers (`Player.cs:28168-28192`, :27008-27017).
- `crates/tr-app/src/player.rs::ExtraJump` (:64-140) + `PRIORITY` + `hold_multiplier` — the
  C# `extraJump` priority chain (`Player.cs:20698-21022`, `RefreshDoubleJumps` :22826).
- `crates/tr-app/src/player.rs::wing_slot`/:4439, `wing_stats`/:4451, `wing_constants` —
  `WingStatsInitializer.cs:11-61` mirror; `wing_air_logic_tweaks` :3171.
- `crates/tr-app/src/player.rs::update_sitting`/`update_sleeping` (:2481) — `Player.cs`
  sit/sleep flags; the regen ramp reads them.
- `crates/tr-app/src/player.rs::update_survival` (:3568) — breath/lava tick; `CheckDrowning`
  linkage.
- `crates/tr-app/src/regen.rs::Regen::tick` — `UpdateLifeRegen` (`Player.cs:18942`).
- `crates/tr-app/src/regen.rs::ManaRegen` — `Player.cs:19398-19560`.
- `crates/tr-app/src/death.rs::drop_coins`/`tombstone_roll`/`death_text_key` — `DropCoins`
  (`Player.cs:54261`), tombstone RNG (`Player.cs:39091-39115`).
- `crates/tr-app/src/world_sim/npcs.rs::kill_npc`-adjacent entry (:3160-3183) — the C#
  `npc.StrikeNPC` → `Player.KillMe` seam; `expert` is hardcoded `false` at :3180.

Shared data: `tr-core` tile/light maps, `world_sim::World` (liquid flags), `app::App` (input),
`assets` (wing/def/buff tables).

## Dependencies

- Calls into: `collision.rs` (AABB/tile tests), `mount.rs`, `grapple.rs`, `minecart.rs`,
  `regen.rs`, `buffs.rs`, `luck.rs`, `death.rs`, `item.rs` (accessory grants), `world_sim`
  (tile/liquid queries, NPC list).
- Called by: `world_sim/player.rs` (fixed step) → `world_sim/step.rs::FixedStepper` →
  `app/frame.rs::run_frame`; render/entities reads the final pose fields.
- Shared data structures: `Player.wet`/`lava_wet`/`honey_wet`/`shimmer_wet` (derived from the
  centre tile, `Player.cs` derives these from `WetCollision`); `Player.grappling`
  (`grapple[0] >= 0`); `RegenInputs` (the regen-view struct that gates grapple/centre-tile
  inputs); `ImmunityCooldownID`-style per-source iframes.

## Port status

| C# subsystem | Status | Rust location | Evidence / notes |
| --- | --- | --- | --- |
| Per-tick movement defaults | PORTED | `player.rs:2507-2522` | 1:1 with `Player.cs:24364-24371`: `maxRunSpeed 3.0`, `runAcceleration 0.08`, `runSlowdown 0.2`, `accRunSpeed = maxRunSpeed`, jump statics → `jump_boost` floor (`UpdateJumpHeight` :19589-19593), `moveSpeed 1.0` |
| Wet/shimmer/on-fire gravity branch | **PARTIAL** | `player.rs:2516-2517`, :3189-3198 | C# `Player.cs:24382-24432` sets water `gravity 0.2 / maxFallSpeed 5 / jumpHeight 30 / jumpSpeed 6.01`, honey `0.1/3`, merman `0.3/7`, trident `0.25/6 + 25/5.51` (+`controlUp` → `0.1/2`), shimmerWet `gravity 0.15 + 23/5.51`; the port resets `jump_height 15` / `jump_speed 5.01` with **no** wet arm and uses `GRAVITY/3` (0.133) + `MAX_FALL/3` (3.33) for **all** liquids |
| `maxFallSpeed += 0.01f` | MISSING | — | `Player.cs:24437`; `MAX_FALL` stays `10.0` (`player.rs:30`). Also `PortalPhysicsEnabled = 35` (:24380) |
| Vortex debuff `gravity = 0` | MISSING | — | `Player.cs:24433-24436` |
| Gravity / fall cap integration | PORTED | `player.rs:3189-3198`, :3327 | Body is 1:1 for the *dry* case; see the liquid deviation below |
| Jump (ground, hold, release) | PORTED | `player.rs:3211-3279` | `controlJump`/`releaseJump`/`jump`/`jumpHeight` semantics match `Player.cs:20671-20712` |
| Extra jumps (8 types + priority) | PORTED | `player.rs:3242-3271` | `ExtraJump::PRIORITY`, `hold_multiplier`, `refresh_double_jumps` :3232 |
| Dash (4 dashTypes, double-tap) | PORTED | `player.rs:3287` | 16.9/14.5/21.9/16 ± decay 0.992→×0.96 |
| Wings (hold/hover/cap/rocket merge) | PORTED | `player.rs:4439-4540`, `wing_air_logic_tweaks` :3171 | `rocketTimeMax 7`, +wings merge `wingTime += rocketTime*6` (`Player.cs:26825-26833`) |
| Grapple movement | PORTED | `grapple.rs` :3301 | velocity = avg anchor − center, cap 11, `Player.cs:22414-22440` |
| Minecart ride / track | PORTED | `minecart.rs`, `player.rs:3113-3170` | runSpeed 13 / acc 0.04, ±4 track boost, `Mount.cs:2747` |
| Floor/ice modifiers | PORTED | `player.rs:3063-3096`, `collision.rs` | `slippy` accel ×0.7 / slowdown ×0.1, iceSkate ×3.5 + maxSpeed ×1.25 (`Player.cs:26533-26554`); `water_halves_the_stride` test |
| Fall damage | PORTED | `player.rs:3478-3552` | `25 + extraFall`, `(tiles − thr)*10`, stoned `(tiles−2)*20` (`Player.cs:25296-25355`); `fall_damage_matches_the_vanilla_formula` test |
| Wet post-collision step scale | PORTED | `player.rs:3199-3204` | water/lava 0.5, honey 0.25, shimmer 0.375 — a **position** multiplier, not drag (`Player.cs:28168-28192`) |
| Life regen engine | PORTED | `regen.rs` | ramp tiers 300..3000, moving ×0.5, still ×1.25, sitting ×1.3, campfire ×1.1, `round(ramp*(maxLife/400*0.85+0.15))`, accumulator 120 (`Player.cs:18942-19262`) |
| Mana regen engine | PORTED | `regen.rs` | delay `((1−mana/max)*240+45)*0.7`, base `maxMana/3 + bonus`, still/grappling/buff ×2, accumulator 120 (`Player.cs:19398-19560`) |
| DoT debuffs | PORTED | `regen.rs` | poison −4, venom −30, OnFire −8, cursed inferno −24, frostburn −16, hellfire −60, suffocation −40, electrified −8 (−40 moving); drain caps 4 HP/120-chunk |
| Drowning / breath | **PARTIAL** | `player.rs:2484-2493`, `BREATH_MAX` :53, `BREATH_CD` :55 | Gate derives `in_water` from the **centre tile**; C# uses `CheckDrowning` `Player.cs:23191-23282` → `Collision.DrownCollision` :1383-1431 (10×12 head box, liquid-level aware, honey included, lava/shimmer excluded) |
| `breathCDMax` item scaling | MISSING | `player.rs:55` | C# `breathCDMax` is a property: 7, ×2 Breathing Reed, ×6 Diving Helm (`Player.cs:3855-3869`) |
| Death — drops, tombstone, text | PORTED | `death.rs` | softcore gems only, mediumcore inventory, coins `stack/2` (expert /4), tombstone RNG (`Player.cs:38863`, 54261, 39091-39115) |
| Death — `KillMe` entry from NPC | **PARTIAL** | `world_sim/npcs.rs:3160-3183` | `expert = false` hardcoded at :3180, so expert ×1.5 in `GetRespawnTime` (`Player.cs:39060-39082`) is unreachable |
| `UpdateDead` countdown + revive | PORTED | `player.rs` `update_dead`, tests `update_dead_counts_down_and_revives_at_zero` | respawn timer `3600` matches `Player.cs:57108` |
| `UpdateDead` state reset | **PARTIAL** | `player.rs` `update_dead` | C# `Player.cs:16995-17020` clears non-persistent buffs, `chest = -1`, `itemAnimation`; only the buff-clear family is absent |
| Respawn timer skip rules | **PARTIAL** | `player.rs:1901-1930` | omits `Main.snowMoon ‖ Main.pumpkinMoon ‖ DD2Event.Ongoing` refusal (`Player.cs:38984-38987`); 90-tick lockout, jump edge, `bossHinders` Manhattan 4000, 1920×1080 + 320 inflated NPC rect are present |
| Respawn — life/iframes | **PARTIAL** | `player.rs:1820-1821` | `hp = max_hp` vs C# `statLife = max(statLifeMax, statLifeMax2)/2` (`Player.cs:37596-37611`); `hurt_cd = 0` vs `immuneTime = 180` (60 expert-ish) (`Player.cs:37661-37668`) |
| Lava flat damage | **PARTIAL** | `player.rs` lava tick | 30 vs C# 80 (`Player.cs:27687`); separate immunity counter is correct (`ImmunityCooldownID.Lava = 4`) |
| `whoAmI` / player index | **PARTIAL** | `player.rs:2001-2003` | owner stands in as `0`; structural, rendered OK because it only feeds per-player drain/aura ids |
| Head/body/leg slot assignment | **PARTIAL** | `player.rs:504`, :507 | `leg_frame`/`body_frame` are **frame-row indices**, not C#'s `PlayerFrame` slot ids; `headPosition`/`headVelocity` death animation (`Player.cs:17029-17040`) is unfilled |
| `UpdateBuffs` slot engine | PORTED | `buffs.rs` | `maxBuffs 44` (`Player.cs:57106`), `TimeLeftDoesNotDecrease` set, expert ×2 (master ×2.5) debuff extension |
| Luck | PORTED | `luck.rs` | `RecalculateLuck` `Player.cs:29137-29164`, unlclamped |
| Phase order (whole tick) | PORTED | `player.rs:2413-3593` | see Deviations for the two moves |

## Tests

The module's coverage is broad: **204 named tests** across four files.

`crates/tr-app/src/player.rs` (165 `#[test]`, :5815-10436) — highlights covering this module:

- defaults/statics: `dye_cache_mirrors_update_dyes`, `stat_defense_sums_pieces_set_bonus_and_buffs`,
  `the_endurance_rows_cut_the_incoming_damage`
- respawn: `respawn_time_base_boss_and_expert`, `update_dead_counts_down_and_revives_at_zero`,
  `respawn_timer_clamps_at_3600`, `bug_06b_only_the_live_invasions_npcs_block_the_respawn_skip`,
  `the_skip_needs_the_90_tick_lockout_and_a_jump_edge`, `the_skip_is_blocked_while_a_boss_hinders`,
  `the_jump_edge_is_required_for_the_skip`
- movement/physics: `falls_and_lands_on_ground`, `ice_slides_without_skates`,
  `water_halves_the_stride`, `air_friction_is_half_the_ground_friction`,
  `half_bricks_only_block_their_bottom_half`, `double_tap_fires_the_dash`, `wings_lift_and_drain_the_budget`,
  `grapple_pull_replaces_the_velocity`, `fall_damage_matches_the_vanilla_formula`
- survival: `drowning_drains_breath_and_damages`
- the whole minecart/mount/jump/extra-jump catalog, plus the buff-id regression pairs
  `test_DW_2_2_buff_64_tracks_projectile_266`, `test_DW_2_2_buff_216_respawn_arm` (…).

`crates/tr-app/src/regen.rs` (18 `#[test]`, :321-709): `the_mana_spend_resets_the_regen_delay`,
`ramp_tiers_build_up_over_five_standing_seconds`, `banker_rounding_differs_from_half_up_on_midpoints`,
`moving_halves_and_still_boosts_the_ramp`, `accumulator_yields_one_hp_per_120_and_caps`,
`ramp_time_caps_at_3600_and_freezes_the_bonus`, `dot_drains_clamp_positive_regen_and_reset_the_ramp`,
`campfire_and_expert_modify_the_ramp`, `mana_delay_ramps_with_missing_mana_and_counts_down`,
`mana_regen_floor_and_refill_per_120`, `mana_refill_returns_true_on_the_last_point`,
`grapple_counts_as_still_for_mana`, `fast_lane_takes_exactly_five_per_600_chunk_and_returns_early`,
`generic_lane_chunks_are_at_most_four_per_120`, `generic_lane_death_reasons_follow_the_lane_split`,
`counter_minus_2000_yields_several_chunks_and_one_reason`, `drain_stops_when_hp_already_gone`,
`no_second_chunk_after_death`.

`crates/tr-app/src/death.rs` (8 `#[test]`, :313-444): `death_text_key_matches_the_by_other_table`,
`death_message_picks_the_variant_and_formats_the_name`, `tombstone_roll_keeps_the_vanilla_id_set`,
`tombstone_x_scatter_clears_two_pixels`, `drop_coins_halving_math_is_the_greedy_split`,
`value_to_coins_skips_the_zero_denominations`, `the_death_sets_the_lost_coins_state`,
`the_zero_purse_leaves_the_state_clean`.

`crates/tr-app/src/world_sim/player.rs` (14 `#[test]`, :700-1496) — the fixed-step driver,
dead/respawn branch and input assembly.

Note: `grapple_counts_as_still_for_mana` **passes** only because the test sets
`RegenInputs.grappling` itself — the live tick never does (see Deviations).

## Deviations & risks

**Critical (behavioural, unadmitted)** — in-liquid statics are the wrong formula port-wide.

`player.rs:3190-3198` applies `liquid_gravity() = GRAVITY/3 = 0.1333` and
`liquid_fall_cap() = MAX_FALL/3 = 3.333` to *every* liquid. The citation
`Player.cs:27008-27017` is real but is **not** the plain in-liquid path: `27008` sits inside
`else if (this.mount.CanFly(this) && this.controlJump && this.jump == 0)` → `!mount.Flight()`
(`Player.cs:26962-27020`), and the sibling `gravity/3` at `Player.cs:27339` is the wings
slow-descent arm (`wingsLogic > 0 && controlJump && velocity.Y > 0`). The plain wet/lava path
is `velocity.Y += this.gravity * this.gravDir` (`Player.cs:27354`) capped by `maxFallSpeed`
(:27365). C# already rebased `gravity`/`maxFallSpeed` at the top of `Update`:

| liquid | C# gravity | C# maxFallSpeed | C# jumpHeight / jumpSpeed | port |
| --- | --- | --- | --- | --- |
| water (plain, lavaWet) | 0.2 | 5.01 | 30 / 6.01 | 0.1333 / 3.33 / 15 / 5.01 |
| honey | 0.1 | 3.01 | (unchanged) | 0.1333 / 3.33 |
| merman (in water) | 0.3 | 7.01 | (unchanged) | 0.1333 / 3.33 |
| trident | 0.25 (0.1 with `controlUp`) | 6.01 (2.01) | 25 / 5.51 | 0.1333 / 3.33 |
| shimmerWet | 0.15 | 10.01 (unset → base) | 23 / 5.51 | 0.1333 / 3.33 |
| `isPerformingJump_DownDash` | ×0.85 both | ×0.85 | — | absent |

(`Player.cs:24382-24437`; `Player.jumpHeight`/`jumpSpeed` are the static shared fields the port
models as `self.jump_height`/`self.jump_speed`, reset each tick at `player.rs:2516-2517`
mirroring `Player.cs:24366-24367`.) The wet jump statics do survive C#'s own `UpdateJumpHeight`
(`Player.cs:19587-19641` only raises them via `Math.Max`/`+=`), so the water jump really is
`30 / 6.01` while the port jumps with `15 / 5.01` — a 2× jump-height error in liquids. Severity
**behavioural**; unadmitted by ROADMAP §7 / ANALYSIS. *Related:* the in-water swim ascent
`self.vy = -2.0` (`player.rs:3240-3241`) has no C# counterpart I could locate as written — the
C# wet ascent comes from the `jumpSpeed` 6.01 branch (`Player.cs:20752-20754`); the 2.0 is a
**behavioural approximation** (flagged for verification, not proven against source).

**High (behavioural)** — `MAX_FALL` misses `+0.01f`. `player.rs:30` `MAX_FALL = 10.0` vs
`Player.cs:24437` `this.maxFallSpeed += 0.01f` (and `PortalPhysicsEnabled` → 35 at :24380).
Small per-tick drift on every fall, and `MAX_FALL/3` inherits it. **Behavioural**, cosmetic in
isolation.

**High (behavioural)** — drowning gate is centre-tile, not a head box. `player.rs:2484-2493`
derives `in_water` from the tile under the player's centre; C# `CheckDrowning`
(`Player.cs:23191-23282`) walks `Collision.DrownCollision` (`Collision.cs:1383-1431`), a 10×12
head box that excludes lava/shimmer, includes honey, and is level-aware. Already admitted in
ANALYSIS:1220-1223. Consequence: shallow dipping and standing in a 1-tile-deep puddle behave
differently, and honey drowns in C# but not in the port's centre test.

**High (behavioural)** — drowning damage ignores iframes. `player.rs:2375` `lose_health` forces
`hurt_cd = max(…, 0.8)` (48 ticks); C# drowning is `statLife -= 2` on its own
`breathCD` clock (`Player.cs:23263`) with **no** `immuneTime` set — the 2 HP lands exactly once
per 7 frames. Combined with `breathCDMax = 7` hardcoded (`player.rs:55`) the port's DoT rate
is right by accident and the immunity interaction is wrong.

**High (behavioural)** — respawn life & spawn iframes. `player.rs:1820` `self.hp =
self.max_hp` vs C# `statLife = max(statLifeMax, statLifeMax2)/2` (`Player.cs:37596-37611`);
`player.rs:1821` `hurt_cd = 0.0` vs C# `immuneTime = 180` / 60 (`Player.cs:37661-37668`).
Players spawn at full health and can be hit on respawn frame 1.

**Medium (behavioural)** — expert death scaling is unreachable. `world_sim/npcs.rs:3180`
hardcodes `expert = false`, so `GetRespawnTime`'s ×1.5 (`Player.cs:39060-39082`) never runs;
`respawn_time_base_boss_and_expert` cannot cover it.

**Medium (behavioural)** — live grappling never reaches the regen engine.
`RegenInputs.grappling` is left at its neutral default everywhere outside
`regen.rs`'s own tests (the only writer is the test at `regen.rs:559`). C# reads `grappling[0]
>= 0` at `Player.cs:19241` (×1.25 vs ×0.5 life ramp) and `:19426`/`:19449` (mana ×2). So
grappling is live in movement but dead in regen. Admitted in ANALYSIS:4131-4135.

**Medium (behavioural)** — `UpdateDead` reset is partial. No non-persistent buff purge vs
`Player.cs:16995-17002`, and no `chest = -1` (:17020); `headPosition`/`headVelocity`
(`Player.cs:17029-17040`) are not modelled, so the death animation has no head debris.
Partly admitted (ANALYSIS:7220-7222).

**Medium (behavioural)** — respawn-skip refusal list. `allows_respawn_timer_skip`
(`player.rs:1901-1930`) omits the `Main.snowMoon ‖ Main.pumpkinMoon ‖ DD2Event.Ongoing`
refusal at `Player.cs:38984-38987`: the skip is granted during invasions/events where C#
forbids it.

**Low (determinism / pixel)** — `whoAmI` stand-in. `player.rs:2001-2003` uses `0` for the
owner index; multi-player/lan-secondary aura and drain ids would collide, single-player is
unaffected. `leg_frame`/`body_frame` (`player.rs:504`, :507) are frame-row indices rather than
C#'s `PlayerFrame` slot ids — the mapping must be re-derived per set before any head/body/leg
slot switching is claimed.

**Doc (ROADMAP is stale, not the code)** — ROADMAP §8 header reads "**Rust** ⬜ (none yet) ·
⬜" (`ROADMAP.md:401`) while `regen.rs`, `buffs.rs` and `luck.rs` exist with 18 + ~40 tests.
The task rows below it (`✅ M2-BUFF-2 regen engines`) contradict the header. Correct the header
and the module 04 (buffs) claim before anyone trusts the file as a status map.

**Risk — phase order is 1:1 today but two moves are load-bearing and easy to break in a
refactor.** In Rust the wet-flag derivation (`player.rs:2484-2493`) happens *before* the
movement defaults (:2507), matching C# where `base.wet`/`honeyWet` are recomputed at
`Player.cs:27728-27850`, i.e. **after** the 24382-24432 branch consumed the *previous* tick's
flags. So Rust's wet statics are one tick fresher than C#'s; the port gets away with it only
because its liquid handling is not phase-sensitive in the dry→wet frame. Any fix to the
statics above must keep this ordering property in mind. Similarly, the jump section
(`player.rs:3206-3279`) runs *before* `clear_stale_jumps` (:3282) and `dash_movement` (:3287),
matching `Player.cs:26745-27600`; `StickyMovement`/`WallslideMovement` (`Player.cs:26764`)
sit between dash and the wing arm and must stay in that slot.

## Task seeds

| # | Task | Size | Depends on |
| --- | --- | --- | --- |
| S1 | Move the wet/shimmer statics into `update_with_grapple` before the movement defaults: add `wet_gravity_and_cap()` returning the `(gravity, maxFallSpeed, jumpHeight, jumpSpeed)` tuple from `Player.cs:24382-24437`, consume at `player.rs:3189-3198` and `player.rs:2516-2517`, and replace the `liquid_gravity`/`liquid_fall_cap` call sites. Keep the mount/wings `÷3` arms (`Player.cs:27008`, :27339) as separate, correctly-gated branches. | M | — |
| S2 | `maxFallSpeed += 0.01f` + `PortalPhysicsEnabled = 35` (`Player.cs:24437`, :24380) and `vortexDebuff` gravity = 0 (:24433-24436). | S | — |
| S3 | Port `breathCDMax` as a property: 7 base, ×2 Breathing Reed, ×6 Diving Helm (`Player.cs:3855-3869`); feeds `player.rs:55`. | S | item registry access |
| S4 | Port `Collision.DrownCollision` (`Collision.cs:1383-1431`) + `CheckDrowning` (`Player.cs:23191-23282`) as the drowning gate, and stop `lose_health` forcing `hurt_cd = 0.8` on the `-2` path (`player.rs:2375`). | M | — |
| S5 | Respawn fidelity: `statLife = max(statLifeMax, statLifeMax2)/2` and `immuneTime = 180`/`60` (`Player.cs:37596-37611`, :37661-37668) at `player.rs:1820-1821`; add the snowMoon/pumpkinMoon/DD2Event refusal to `allows_respawn_timer_skip` (`Player.cs:38984-38987`). | M | — |
| S6 | Remove the `expert = false` hardcode at `world_sim/npcs.rs:3180` and thread expert through the respawn path, so `GetRespawnTime` ×1.5 (`Player.cs:39060-39082`) is reachable. | M | world-gen/difficulty plumbing |
| S7 | Feed `RegenInputs.grappling` from the live hook state (`grapple.rs` → `player.rs` regen call), so `Player.cs:19241`, :19426, :19449 apply. Update `grapple_counts_as_still_for_mana` to construct its input the same way the tick does. | M | — |
| S8 | Finish `UpdateDead`: non-persistent buff purge (`Player.cs:16995-17002`), `chest = -1` (:17020), `headPosition`/`headVelocity` death animation (:17029-17040). | M | buffs module boundary |
| S9 | Verify the in-water ascent: decide whether C# supports `self.vy = -2.0` (`player.rs:3240-3241`) or whether the wet jump should reuse `jumpSpeed` 6.01; the paragraph above is marked **UNVERIFIED** deliberately. | S | S1 |
| S10 | Replace the `whoAmI = 0` stand-in (`player.rs:2001-2003`) with the real index once multi-player netsession ids land. | M | net |
| S11 | Correct ROADMAP §8's stale "Rust ⬜ (none yet)" header (`ROADMAP.md:401`) and reconcile its task rows with the 2026-11-12 reality. | S | — |
| S12 | Derive the `leg_frame`/`body_frame` → C# `PlayerFrame` slot-id table before any head/body/leg slot work (module 06 territory; the table itself belongs to this module). | M | framing module |
