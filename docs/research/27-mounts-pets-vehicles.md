# 27 — Mounts / pets / minecart / grapple

> Wave 3 · researched 2026-11-12 · sources: decompiled + crates + ROADMAP/ANALYSIS/PORT_MAP

## Scope
The player-mount framework, the mount data table, the mount activation/deactivation
rules, the minecart ride, the grappling-hook family, and the pet/minion budget.

C# sources read (ranges only): `decompiled/Terraria/ID/MountID.cs` (191 lines — the id
list, `Count = 66` and the 13 `MountID.Sets` bool sets), `decompiled/Terraria/Mount.cs`
(6407 lines — `MountData` :129-266, the `Mount` instance :325-383, `Reset` :675-694,
`Initialize` :696-2567, the accessors :449-652, `JumpHeight`/`JumpSpeed` :2810-2858,
`CanFly`/`CanHover` :2860-2896, `ResetFlightTime`/`CheckMountBuff` :5262-5281,
`SetMount`/`Dismount` :6142-6305, `CanMount`/`CanDismountWithResult` :6581-6624,
`SetAsMinecart` :2747-2799, `SetAsRollerSkate` :2569-2610, `SetAsChillet` :2544-2566,
`UpdateFrame` :3581-4506, `UpdateDrill` :3059, `TryPettingMount` :6073-6125),
`decompiled/Terraria/Player.cs` (54877 lines — the mount merge block :26623-26687,
`ItemCheck_HandleMount` :52922-52950, `QuickMount` :6045-6100 +
`QuickMount_GetItemToUse` :6179-6198, the item-use mount path :42768-42771 and its
`CanMount` gate :52312-52315, `GrappleMovement` :22613-22740,
`GetGrapplingForces` :22757-22856, `QuickGrapple` :6296-6478,
`QuickGrapple_GetItemToUse` :6481-6500, the minion counters :1260-1264, `maxBuffs`
:57106), `decompiled/Terraria/Minecart.cs` (1583 lines — the 36-frame decode tables
:103-547, `TrackCollision` :562-941, `GetOnTrack`/`OnTrack`/`TrackRotation` :1200-1300,
`WheelSparks` :1623), `decompiled/Terraria/Item.cs` (the `mountType` assignment sites
:20779-47407 and `DefaultToMinecart` :48395).

Rust sources read: `crates/tr-app/src/mount.rs` (2466), `crates/tr-app/src/minecart.rs`
(2638), `crates/tr-app/src/grapple.rs` (1088), `crates/tr-app/src/player.rs` (the mount
slices), `crates/tr-app/src/world_sim/player.rs` (the fixed-step wiring),
`crates/tr-app/src/projectile.rs` (the minion budget + hook pool),
`crates/tr-app/src/buffs.rs` (the pet buff sets).

**Path correction for module 03**: it lists `crates/tr-app/src/world_sim/player/mount.rs`,
`grapple.rs`, `minecart.rs`. The real files are at the **crate root**
(`crates/tr-app/src/mount.rs` etc.); `world_sim/player.rs` only *calls* them.

## Key entities

### C#
- `Terraria/ID/MountID.cs::MountID` (:3-190) — 66 ids (`None = -1` … `TrustyChilletIgnis = 65`),
  `Count = 66` (:185-190); `MountID.Sets` (:5-49): `CanUseHooks {54,57,58,59,60}`,
  `CanDash {56-65}`, `DoesNotOverride*Frames {57-60}`, `IsRollerSkates {57-60}`,
  `Cart {6,13,11,15,16,18-36,38,39,51,53}` (27), `IsTransformationMount`/`PlayerIsHidden`
  `{52,54,55,56,61}`, `DontHoldItems`/`DontDismountWhenCCed` `{55,56,61}`.
- `Terraria/Mount.cs::MountData` (:129-266) — the per-mount stat row (buff, heightBoost,
  flightTimeMax, usesHover, run/dash/swim speed, acceleration, jump height/speed,
  fallDamage, extraFall, fatigueMax, constantJump, blockExtraJumps, frame sets, offsets).
- `Terraria/Mount.cs::Mount` (:325-383) — the per-player instance state (`_active`, `_type`,
  `_frame*`, `_flyTime`, `_fatigue*`, `_ability*`, `_shouldSuperCart`).
- `Terraria/Mount.cs::Mount.Initialize` (:696-2567) — the 66 rows, plus the helpers
  `SetAsMinecart` (:2747-2799), `SetAsRollerSkate` (:2569-2610), `SetAsChillet` (:2544-2566).
- `Terraria/Mount.cs::Mount.SetMount` (:6196-6305) — activation (guards, swap clear,
  buff add, hitbox growth, per-type extras).
- `Terraria/Mount.cs::Mount.Dismount` (:6142-6194) — deactivation (buff clear, cart state
  reset, `fullRotation` reset, hitbox restore, net msg 13).
- `Terraria/Mount.cs::Mount.CanMount` (:6581-6604) — the three activation gates:
  grown-box fit, the bat-wet rule (`m == 56`), and the grapple rule
  (`!Cart[m] && !CanUseHooks[m] && grappling[0] >= 0`).
- `Terraria/Mount.cs::Mount.CanDismountWithResult` (:6611-6624) — CC gate + space probe.
- `Terraria/Mount.cs::Mount.{RunSpeed,DashSpeed,Acceleration}` (:449-508),
  `JumpHeight`/`JumpSpeed` (:2810-2858), `CanFly`/`CanHover` (:2860-2896),
  `ResetFlightTime` (:5262-5273), `CheckMountBuff` (:5275-5281), `Hover` (:3340-3543).
- `Terraria/Player.cs::Player.Update` mount block (:26623-26687) — the three-mode stat
  merge, the wings/rocket suppression, the Fishron halving, the cart/OnTrack branch.
- `Terraria/Player.cs::Player.QuickMount` (:6045-6100) + `QuickMount_GetItemToUse` (:6179-6198).
- `Terraria/Player.cs::Player.GrappleMovement` (:22613-22740) and
  `GetGrapplingForces` (:22757-22856) — the hook pull.
- `Terraria/Minecart.cs::Minecart` — the track decode tables (:103-547),
  `TrackCollision` (:562-941), `GetOnTrack`/`OnTrack`/`TrackRotation` (:1200-1300).

### Rust
- `crates/tr-app/src/mount.rs::MountData` (:46-100) + `BASE` (:104-185) — the ported row.
- `crates/tr-app/src/mount.rs::MOUNTS` (:1305-1356) — **50 rows** (12 ground, 11 flyer,
  27 cart); `MOUNT_ITEMS` (:1366-1429) — the 49 item→mount rows.
- `crates/tr-app/src/mount.rs::MountState` (:1504-1539) — the instance state;
  `set_mount` (:1812-1841), `dismount` (:1848-1856), `check_mount_buff` (:1861-1863),
  `run_speed` (:1869-1893), `jump_height`/`jump_speed` (:1915-1951), `can_fly` (:1955-1967),
  `can_hover` (:1998-2007), `hover_integrate` (:2032-2122).
- `crates/tr-app/src/player.rs::can_mount` (:4657-4669), `quick_mount` (:4680-4710),
  `update_mount` (:4718-4781), `set_mount` (:4595-4605), `dismount` (:4618-4642),
  `mount_cart` (:1644-1658), `can_dismount` (:1665-1680), `cart_dismount` (:1681-1702).
- `crates/tr-app/src/minecart.rs::TRACK_FRAMES` (:158), `TEXTURE_POSITION` (:238),
  `track_collision` (:325), `on_track` (:821), `get_on_track` (:834),
  `track_rotation` (:938), `wheel_sparks` (:1131).
- `crates/tr-app/src/grapple.rs::hook_spec` (:43-81), `can_latch` (:129),
  `fire_budget`/`fire_allowed`/`recycles_oldest` (:163-191), `grapple_pull` (:266),
  `ai_hook` (:346).
- `crates/tr-app/src/world_sim/player.rs` — the fixed-step wiring: the hook fire edge
  (:344-460), the mount key (:436-442), the cart pin (:567), the spark drain (:621-623).
- `crates/tr-app/src/projectile.rs` — the minion budget accumulation (:7453-7471).

## Dependencies
- Calls into: `world_sim/player.rs` → `mount.rs` (set/dismount/accessors),
  `minecart.rs` (the track machine), `grapple.rs` (the hook pool);
  `projectile.rs` → `grapple.rs::ai_hook`; `ui/chrome.rs` reads the pet buff sets;
  `player.rs` → `collision::try_changing_size_from_bottom_center` (the dismount probe).
- Called by: `world_sim/editing.rs` (the saddle item-use path → `can_mount`),
  `world_sim/player.rs` (the mount key, the hook edge, the cart ride),
  `app/frame.rs` (the spark pool tick/draw), `net.rs` (not yet — msg 13 is missing).
- Shared data structures: `Player.mount: MountState` (`player.rs:698`), the buff array
  (`buffs.rs::MAX_BUFFS = 44`), the projectile pool (hook slots), the World (tiles for the
  track/latch), `Player.max_minions`/`num_minions`/`slots_minions`.

## Port status

| feature | C# rule | Rust | verdict | severity |
| --- | --- | --- | --- | --- |
| Mount id set + `Count` | `MountID.cs:51-190` (`Count = 66`) | `mount.rs::MOUNTS` :1305 + the `m >= 66` guard :1814 | PARTIAL (50 of 66 rows) | behavioural (16 mounts un-equippable) |
| Mount stat table (per-mount constants) | `Mount.cs:696-2567` | `mount.rs` rows :191-1304 | PORTED for the 50 present rows (0,1,3,5,10 spot-checked 1:1) | cosmetic (no per-row constant test for all) |
| `SetAsMinecart` body | `Mount.cs:2747-2799` (13/13/0.04, jump 15/5.15, boost 10) | `mount.rs::cart` :1092-1152 + `cart_vo` :1160 | PORTED 1:1 | — |
| `SetMount` guards / swap | `Mount.cs:6196-6243` | `mount.rs::set_mount` :1812-1841 + `player.rs::set_mount` :4595-4605 | PARTIAL (no type-44 velocity/dash reset, no `fullRotationOrigin` for 7/8, no `_mountSpecificData` for 8/35/54) | behavioural (low) |
| `Dismount` | `Mount.cs:6142-6194` | `mount.rs::dismount` :1848 + `player.rs::dismount` :4618-4642 | PARTIAL (no `fullRotation` reset, no `noItems` clear for 55/56, no net msg 13) | behavioural (low) |
| `CanMount` (3 gates) | `Mount.cs:6581-6604` | `player.rs::can_mount` :4890-4923 + `wet_collision` :6086 (`Collision.cs:1572-1651`) + `mount.rs::is_cart` :1451/`can_use_hooks` :1460 (`MountID.cs:30,43`) | PORTED (the fit probe is the `collides_rect` stand-in; the PlayerSize delegate cannot fire on the 50 rows) | - |
| `CanDismountWithResult` CC gate | `Mount.cs:6615-6618` (`DontDismountWhenCCed {55,56,61}`) | `player.rs::can_dismount` :1665-1680 (size probe only) | MISSING | behavioural (low-medium) |
| `DismountsOnItemUse` / `CanVisuallyHoldItem` | `Mount.cs:642-652`, `:6567-6579` | `mount.rs::dismount_on_item_use` :2183; `world_sim/player.rs:509-517` (`55\|56\|61` = `MountID.cs:46`) | PORTED | — |
| Stat merge (additive / max-while-flying / replace) + wings suppression | `Player.cs:26625-26654` | `player.rs::update_mount` :4741-4775 | PORTED 1:1 | — |
| Fishron halving (`Type == 12`) | `Player.cs:26655-26659` + `MountFishronSpecial` `:4044-4056` + the counter writers `Mount.cs:4982-5008`, decay `Player.cs:25625-25628` | `player.rs::update_mount` :5009 + `mount_fishron_special` :4946 + `mount_fishron_special_counter` | PORTED 1:1 (60-tick hurt / 420-tick wet-or-rain window) | - |
| Roller-skate movement arm | `Player.cs:26660-26663` | `is_roller_skates` helper only (rows 57-60 absent) | MISSING (table gap) | behavioural (low) |
| Drill ability (`UpdateDrill`/`UseAbility`) | `Mount.cs:2898-3340`, `Player.cs:26683-26686` | absent | MISSING | behavioural (low) |
| `ItemCheck_HandleMount` | `Player.cs:52922-52950` | absent (no gravDir model) | MISSING | behavioural (low) |
| `JumpHeight`/`JumpSpeed` \|xVel\| scaling | `Mount.cs:2810-2858` | `mount.rs::jump_height` :1915, `jump_speed` :1935 | PORTED 1:1 | — |
| `CanFly`/`CanHover`/`Flight`/`ResetFlightTime` | `Mount.cs:2860-2896`, `:5262-5273`, `:2973-2981` | `mount.rs` :1955-1987, :1998-2007 | PORTED (type-54 arm N/A — row 54 absent) | — |
| Hover + fatigue pool | `Mount.cs:3340-3560` | `mount.rs::hover_integrate` :2032-2122, `fatigue_recovery` :2019, `does_hover_ignore_fatigue` :2013 | PORTED | — |
| `RunSpeed`/`DashSpeed`/`Acceleration` | `Mount.cs:449-508` | `mount.rs::run_speed` :1869, `dash_speed` :1896, `acceleration` :1905 | PORTED 1:1 | — |
| `QuickMount` | `Player.cs:6045-6100` | `player.rs::quick_mount` :4680-4710 | PARTIAL (no Cart skip, no `DismountsOnItemUse` gate, no 3-strike `FailedNoSpace` force) | behavioural (medium) |
| Item→mount table | `Item.cs` `mountType` sites (30) | `mount.rs::MOUNT_ITEMS` :1366-1429 (49 rows) | PARTIAL (16 ids missing) | behavioural (medium) |
| Mount buff lifecycle (3600 tick buff, expiry dismount) | `Player.cs:9881-9884`, `Mount.cs:5275-5281` | `player.rs:4602` (`add_buff(.., 3600)`), `player.rs:2550` (the expiry check) | PORTED | — |
| `MountID.Sets` coverage | `MountID.cs:36-47` (13 sets) | `mount.rs:1434-1447` (3 sets only) | PARTIAL | behavioural (medium — see D2) |
| Minecart 36-frame decode | `Minecart.cs:103-547` | `minecart.rs::TRACK_FRAMES` :158 + `TEXTURE_POSITION` :238 | PORTED | — |
| `TrackCollision` | `Minecart.cs:562-941` | `minecart.rs::track_collision` :325 | PORTED | — |
| `OnTrack`/`GetOnTrack`/`TrackRotation` | `Minecart.cs:1200-1300` | `minecart.rs` :821/:834/:938 | PORTED | — |
| Ride integration (boost, friction 0.04/0.2, ramp freeze, fall-damage zero, key dismount) | `Player.cs:26665-26682`, `:19818-19849`, `:27352-27359`, `:25311-25314` | `player.rs:3116-3500` (the cart path) | PORTED (structurally relocated out of `update_mount`) | determinism (low) |
| Wheel sparks + landing/bumper sounds | `Minecart.cs:1623-1630`, `:809-813` | `minecart.rs::wheel_sparks` :1131, `Sparks` pool :994 | PORTED (pool capped at 500 vs the 6000 dust slots) | behavioural (low, documented) |
| `FrameTrack` (placing track by item use) | `Minecart.cs` + `Player.cs:46058` | no symbol in `minecart.rs` | MISSING | behavioural (low) |
| Hook AI (aiStyle 7) | `Projectile.cs:52565-52866` | `grapple.rs::ai_hook` :346 + the pool arm | PORTED | — |
| Hook fire tables (`num6`/`num7` vs `num14`) | `Player.cs:6296-6478` | `grapple.rs:163-191` | PORTED (the disagreement kept verbatim) | — |
| Item-driven hook fire | `Player.cs:6481-6500` + `ItemCheck_TryStartUse` | `TR_HOOK` env pin instead (`world_sim/player.rs:234`) | MISSING | behavioural (medium) |
| Grapple pull (`GetGrapplingForces`) | `Player.cs:22613-22856` | `grapple.rs::grapple_pull` :266 + `player.rs` | PORTED | — |
| Mount dismount on grapple | `Player.cs:22620-22623` (`!CanUseHooks[type]`) | `world_sim/player.rs` (the fire edge) + `player.rs::grapple_movement` | PORTED (S1-L-10c, 2026-11-14) | — the set check `MountID.Sets.CanUseHooks` = {54, 57-60} (`mount::can_use_hooks`) now gates both edges; before it the port dismounted **every** mount when a hook went out |
| Latch blacklist / Queen-Slime teleport / TrackHook→cart transfer / net msg / chain render | `Player.cs:6205-6248`, `Projectile.cs` | absent | MISSING (admitted in ANALYSIS) | behavioural (medium) |
| Minion budget (`maxMinions`/`numMinions`/`slotsMinions`) | `Player.cs:1260-1264`, `:17775`, `:51786` | `player.rs:254-258`, `projectile.rs:7453-7471`, `world_sim/player.rs:118-160` | PORTED | — |
| Buff slots (`maxBuffs = 44`) | `Player.cs:57106` | `buffs.rs:21` `MAX_BUFFS = 44` | PORTED | — |
| Light-pet / vanity-pet buff sets | `Main.cs:9214`/`:9288`, `Player.cs:5379-5395` | `buffs.rs:36-69` (`LIGHT_PET_IDS` 12, `VANITY_PET_IDS` 74) | PORTED (cross-ref module 28) | — |
| Petting (`PetMount`/`PetAnimal`/`StopPettingAnimal`) | `Player.cs:33339-33481`, `Mount.cs:6073-6125` | absent (`player.rs:1585` "petting stop - not ported") | MISSING | behavioural (low) |
| Mount net sync (msg 13) | `Mount.cs:6174`, `:6224` | absent | MISSING (MP out of scope) | behavioural (low) |

## Tests
- `crates/tr-app/src/mount.rs`: `the_cart_rows_carry_the_setas_minecart_body` (:2280),
  `the_cart_items_summon_their_rows` (:2324), `the_flyer_rows_answer_the_flight_gates`
  (:2350), `the_hover_fuel_drains_and_the_fatigue_builds` (:2382),
  `the_hover_fatigue_recovers_on_rest` (:2412),
  `the_twelve_ground_rows_carry_the_exact_c_constants` (:2427),
  `set_mount_dismount_and_swap_follow_the_c_guards` (:2469),
  `jump_height_and_speed_scale_with_x_velocity_like_the_c` (:2527),
  `run_speed_queen_slime_air_boost_and_can_fly` (:2546),
  `player_offset_follows_the_frame_table` (:2562).
- `crates/tr-app/src/player.rs`: `can_mount_needs_the_grown_box_to_fit` (:9891),
  `quick_mount_toggles_and_guards` (:9869),
  `dismount_on_item_use_respects_the_same_mount_exemption` (:9848),
  `mount_cart_sets_the_ride_state_fields` (:8762),
  `dismount_resets_the_ride_state` (:9527),
  `dismount_tick_flips_the_facing_to_the_pressed_direction` (:9190),
  `the_mount_replaces_the_run_stats_and_suppresses_the_wings` (:9551).
- `crates/tr-app/src/minecart.rs`: `frame_decode_matches_the_csharp_tables` (:1383),
  `magnet_point_is_the_bottom_center_for_the_defaults` (:1483),
  `x_is_never_moved_by_track_collision` (:1497),
  `boost_left_and_right_flags_fire_on_booster_tiles` (:1527),
  `riding_over_pressure_trips_the_switch_flag` (:1560),
  `on_track_checks_the_tile_under_the_magnet` (:1637),
  `get_on_track_snaps_onto_the_nearest_valid_sub_column` (:1675),
  `track_rotation_measures_the_rail_slope` (:1706), `straight_run_holds_the_rail` (:1747),
  `corner_switches_the_axis_of_travel` (:1769), `ramp_end_launches_at_45_degrees` (:1798),
  `bumper_ends_the_run_on_the_rail` (:1837), `bouncy_bumper_rewinds_and_flips_vx` (:1863),
  `open_end_releases_into_a_fall` (:1897), `boost_fires_once_per_booster_tile` (:1945),
  `y_snap_rounds_half_to_even` (:2245), `sparks_one_reproduces_the_dust_213_spawn_math`
  (:2311), `threshold_rolls_follow_the_c_sharp_counts` (:2476), the stream-order tests
  (:2636, :2686).
- `crates/tr-app/src/grapple.rs`: `dw_2_1_the_three_states_run_in_order` (:523),
  `dw_2_2_the_latch_scan_is_x_major_and_pins_the_hook` (:552),
  `dw_2_3_the_track_hook_latches_only_tracks` (:583),
  `dw_2_5_the_attach_revalidates_its_tile` (:622),
  `dw_2_6_the_range_cap_and_the_owner_death_kill_the_hook` (:643),
  `dw_3_1_the_web_slinger_gate_counts_in_flight_hooks` (:769),
  `dw_3_2_the_lunar_cycle_and_the_static_caps` (:796),
  `dw_3_3_the_fish_hooks_disagreeing_tables` (:836),
  `dw_4_3_the_pull_averages_them_and_caps_on_the_first` (:911),
  `dw_4_4_the_crack_roll_is_deterministic_and_local` (:1079),
  `dw_4_1_the_four_corrected_constants` (:1131).
- `crates/tr-app/src/world_sim/player.rs`: `update_player_drains_cart_sparks_into_the_pool`
  (:1057) and the minion-budget tests (:1391-1513).

Gap: the per-row constants of the 11 flyer and 27 cart rows beyond the
`SetAsMinecart` body are still untested, and the `CanDismountWithResult` CC gate
(D7 below) remains absent. `can_mount`'s three gates and the Fishron halving are
now covered by `s1_l_8_can_mount_applies_the_bat_and_grapple_sets` and
`s1_l_8_fishron_halves_run_stats_outside_the_special`.

## Deviations & risks
- **D1 — the "12 rows" story is stale (documentation, cosmetic).** `docs/ANALYSIS.md`
  "Mounts framework (M2-MOUNT-1)" still says *"a 12-row ground table"* and *"Partial table
  (12 of `MountID.Count` 66)"*; `docs/ROADMAP.md:1357` (M2-MOUNT-1) says *"the 12-row
  `MountData` table"*; `docs/PORT_MAP.md:96` says *"the 12-row ground table"*. The code
  now carries **50 rows** (`mount.rs:1305-1356` = 12 ground + 11 flyer + 27 cart) after
  M2-MOUNT-2/-3 — the deviation list is right in substance but its count is wrong.
- **D2 — `can_mount` drops the Cart / CanUseHooks grapple exemption (behavioural,
  medium) — RESOLVED (S1-L-8).** C# `Mount.cs:6599-6602`: `if (!MountID.Sets.Cart[m] && !MountID.Sets.CanUseHooks[m]
  && mountingPlayer.grappling[0] >= 0) return false;`. Rust `player.rs::can_mount` returned
  false for **any** grappling mount. The Rust comment (*"no row in this table is in the
  CanUseHooks set"*) is true for `CanUseHooks {54,57,58,59,60}` but **false for
  `MountID.Sets.Cart`** — 27 cart rows are present, so vanilla allows mounting a minecart
  while a hook is out and the port refused it. Now gated by `mount::is_cart`
  (`mount.rs:1451`) / `mount::can_use_hooks` (`:1460`) in the C# order.
- **D3 — `can_mount` drops the bat-wet rule (behavioural, medium) — RESOLVED
  (S1-L-8).** C# `Mount.cs:6595-6598`:
  `m == 56 && (wet || dripping || WetCollision(...))` → cannot mount. Rust `player.rs::can_mount`
  has no such arm, and its comment (*"the PlayerSize-delegate and bat-wet arms cannot fire
  on this table"*) is **false** since row 56 (Bat) is in the table (`mount.rs:1022`).
  Now checks `wet`/`dripping` (buff 103) and the faithful `Collision.WetCollision`
  probe (`player.rs::wet_collision`).
- **D4 — the Fishron halving is missing (behavioural, medium) — RESOLVED
  (S1-L-8).** C# `Player.cs:26655-26659`
  halves `runAcceleration`/`maxRunSpeed` for `mount.Type == 12` unless `MountFishronSpecial`.
  Rust skipped it with the comment *"have no rows in this table yet"* —
  row 12 (`CUTE_FISHRON`) **is** in the table (`mount.rs:865`). The halving now runs with
  the `MountFishronSpecialCounter` (60 hurt / 420 wet-or-rain) and the
  `Player.cs:4044-4056` getter.
- **D5 — `SetMount` per-type extras missing (behavioural, low).** C# `Mount.cs:6228-6235`
  (type 44 Pirate Ship: `velocity *= 0.2f` + dash reset), `:6279-6282` (types 7/8:
  `fullRotationOrigin = centre`), `:6287-6298` (`_mountSpecificData` for 8/35/54),
  `:6283-6286` (the PalChillet sound for 62-65), `:6301-6304` (the type-38 achievement).
  Rust `mount.rs::set_mount` :1812-1841 covers only the guards, the fly-time reset, the
  fatigue max, `_flipDraw` and the 9/46 ability cooldowns. Rows 44, 7, 8, 35 exist, so
  the first three are reachable today.
- **D6 — `Dismount` housekeeping missing (behavioural, low).** C# `Mount.cs:6152-6155`
  clears `noItems` for types 55/56 and `:6161-6162` resets `fullRotation`/
  `fullRotationOrigin`. Rust `player.rs::dismount` :4618-4642 restores the hitbox and the
  buff only. Row 56 (Bat) exists.
- **D7 — `CanDismountWithResult`'s CC gate missing (behavioural, low-medium).** C#
  `Mount.cs:6615-6618` returns `FailedCCed` for `DontDismountWhenCCed {55,56,61}`; Rust
  `player.rs::can_dismount` :1665-1680 only probes the space. 55 is un-equippable but
  56 (Bat) and 61 (Pixie) are not.
- **D8 — `QuickMount` diverges (behavioural, medium).** C# `Player.cs:6190` skips
  `MountID.Sets.Cart` items and falls through to `QuickMinecart()` (:6097); the Rust
  `quick_mount` (`player.rs:4693-4695`) takes *any* inventory slot with a known
  `mount_type` — the 26 cart items are in `MOUNT_ITEMS`, so a minecart item is mounted
  through the normal path. C# also gates on `Mount.DismountsOnItemUse(item.mountType) &&
  TryingToUseItem()` (:6082-6085); the Rust does not. The Rust comment (*"the cart items
  are skipped by the table itself (no cart rows)"*) is stale.
- **D9 — 16 mount ids are absent (behavioural, medium).** Missing rows: 4 (Turtle),
  17 (Golf Cart), 37 (Flamingo), 45 (Spooky Wood), 46 (Santank), 52 (Wolf),
  54 (Velociraptor), 55 (Rat), 57-60 (roller skates), 62-65 (Chillet). `set_mount`
  returns `None` for them (`mount.rs:1814`), so the saddles do nothing.
  Partially admitted in ANALYSIS ("partial table"), with a stale count (see D1).
- **D10 — SuperCart statics are placeholders (behavioural, latent).** C# `Mount.cs:6660-6664`
  sets `SuperCartRunSpeed = 20f`, `SuperCartDashSpeed = 20f`, `SuperCartAcceleration = 0.1f`,
  `SuperCartJumpHeight = 15`, `SuperCartJumpSpeed = 5.15f`; Rust `mount.rs:1481-1485` has
  all five as `0`. The branches are dead (`should_super_cart` is only ever written as
  `false` in `reset` :1617), so there is no behavioural effect today — but the constants
  are wrong and will bite when the SuperCart is implemented. Admitted in ANALYSIS.
- **D11 — the idle-pick RNG is a stand-in (determinism, low).** `Main.rand.Next(900, 1500)`
  (`Mount.cs:4165`) is replaced by a local splitmix64 stream (`mount.rs:1584-1596`).
  Admitted in ANALYSIS; the same tick sequence always yields the same delays.
- **D12 — `MountState::check_mount_buff` is dead code (cosmetic).** `mount.rs:1861-1863` has
  no caller; the equivalent check lives inline at `player.rs:2550`.
- **D13 — the minecart ride block is relocated (determinism, low).** C# puts the
  cart/`OnTrack` branch *inside* the mount merge block (`Player.cs:26665-26682`, before
  `HorizontalMovement`); the Rust runs it in the separate cart path (`player.rs:3116-3500`).
  The `runSlowdown = runAcceleration` assignment (`player.rs:3166` ↔ `Player.cs:26679`) is
  therefore ordered differently relative to the movement merge — worth an explicit A/B.
- **D14 — `FrameTrack` missing (behavioural, low).** C# places track tiles from the item
  use (`Player.cs:46058` → `Minecart.FrameTrack`); no `FrameTrack` symbol exists in
  `minecart.rs` (the smart-cursor `Step_MinecartTracks` is a different path).
- **D15 — the rail sprite does not render (pixel, low).** Admitted in ANALYSIS: the tile
  renderer treats `frame_x/frame_y` as pixel coordinates while tile 314's frames are track
  *indices* (`Minecart.cs:1632-1650`).
- **D16 — hook fire is env-pinned, not item-driven (behavioural, medium).** C#
  `Player.QuickGrapple_GetItemToUse` (`Player.cs:6481-6500`) picks the held/inventory hook;
  the Rust fires `TR_HOOK` (`world_sim/player.rs:234`). Admitted in ANALYSIS.
- **D17 — two premise corrections (not port bugs).** (a) `Player.GrappleLen` does **not**
  exist in 1.4.5.8 — a grep over `decompiled/Terraria` returns zero hits; the hook length
  lives in the projectile AI (`Projectile.cs:52565-52866`) and in `grapple.rs::hook_spec`
  :43-81. (b) There are no "5 buff slots for minions": `Player.maxBuffs = 44`
  (`Player.cs:57106`, mirrored by `buffs.rs:21`); the minion budget is the
  `maxMinions`/`numMinions`/`slotsMinions` triple (`Player.cs:1260-1264`), which the port
  does implement (`player.rs:254-258`, `projectile.rs:7453-7471`). The pet system in
  1.4.5.8 is the light/vanity-pet **buff** sets, ported at `buffs.rs:36-69`.

## Task seeds
- **S — mount-id sets as data.** Add the 13 `MountID.Sets` tables to `mount.rs`
  (`MountID.cs:36-47`) and consume them in `can_mount` (the Cart/CanUseHooks exemption,
  D2), `can_dismount` (the CC gate, D7) and `quick_mount` (the Cart skip, D8).
  Depends on nothing; unblocks D2/D3/D7/D8. **Partial (S1-L-8):** `Cart`
  (`mount.rs::is_cart`) and `CanUseHooks` (`can_use_hooks`) landed; the remaining sets
  stay for D7/D8.
- **S — restore the `CanMount` arms.** Add the `m == 56` wet rule (D3) and the
  Cart/CanUseHooks exemption (D2) to `player.rs::can_mount`, with the C# order
  (fit → wet → grapple). **DONE (S1-L-8).**
- **S — Fishron halving.** Port `Player.cs:26655-26659` into `player.rs::update_mount`
  (the `MountFishronSpecial` counter has no port model — decide between a constant-off
  arm and a new field). D4. **DONE (S1-L-8)**: the counter field and the
  60/420-tick writers landed with it.
- **S — `SetMount`/`Dismount` extras.** The type-44 velocity/dash reset, the 7/8
  `fullRotationOrigin`, the `fullRotation` reset on dismount, and the `noItems` clear for
  55/56 (D5, D6).
- **M — the missing 16 rows.** Land ids 4, 17, 37, 45, 46, 52, 54, 55, 57-60, 62-65 with
  their C# constants and their item→mount rows (M2-MOUNT-4 territory); the roller skates
  also need `CanRideMinecartTracks`/`CanUseWings`/`MovementStatsAreAdditive`
  (`Mount.cs:2569-2610`). Unblocks D9 and the dash mounts.
- **S — refresh the stale documentation.** Update ANALYSIS "Mounts framework (M2-MOUNT-1)",
  ROADMAP:1357 and PORT_MAP:96 from "12 rows" to the current 50 (+ the 16-row backlog),
  and delete the three stale in-code comments that claim no cart/row-56/bat arms exist
  (`player.rs:4645-4647`, `:4654-4656`, `:4674-4675`). D1-D4, D8.
- **S — SuperCart constants.** Replace the `0.0` placeholders with 20/20/0.1/15/5.15
  (`Mount.cs:6660-6664`) and add the writer for `should_super_cart`. D10.
- **S — `Minecart.FrameTrack`.** Port the track-tile placement used by the item use
  (`Player.cs:46058`) into `minecart.rs`. D14.
- **M — hook fire from the item.** Port `QuickGrapple_GetItemToUse` + `ItemCheck_TryStartUse`
  so the held/inventory hook drives the fire, keeping `TR_HOOK` as the test pin. D16.
- **M — petting.** `Player.petting` state machine (`Player.cs:33339-33481`) plus
  `Mount.TryPettingMount` (`Mount.cs:6073-6125`) for the Chillets (needs the 62-65 rows).
  Cross-ref module 28 for the pet buff display.
- **M — table-driven constant test.** Replace
  `the_twelve_ground_rows_carry_the_exact_c_constants` with a 50-row table test
  (C# values pasted from `Mount.cs:696-2567`) so every present row is pinned.
- **L — mount abilities.** `UseAbility`/`StartAbilityCharge`/`AimAbility`/`UpdateDrill`
  (`Mount.cs:2898-3340`) and the dash mounts 62-65 (`MountID.Sets.CanDash {56-65}`) —
  M2-MOUNT-4, depends on the missing-rows seed.
- **L — mount net sync.** msg 13 (`Mount.cs:6174`, `:6224`); blocked by the MP design.
