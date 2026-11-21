# 26d — Projectile aiStyles 31-40 COVERAGE census (W4-d4)

> Researched 2026-11-21 against `a725acb0` (wave-3 complete). C# cites are
> `decompiled/Terraria/Projectile.cs` (stable); Rust cites are
> `crates/tr-app/src/projectile.rs` / `projectile/data.rs` at this revision.
> Port task: the MISSING styles in 31-40 are ported verbatim here (the d4
> decade claim). Companion to `26c-projectile-ai-1-105-census.md` (W4-f19),
> whose finding F3 assigns the style-37 arm+row to this task.

## Method

- Dispatch census: the `match data.ai_style` block in `ProjectilePool::update`
  (`projectile.rs:16731-19779`). One top-level numeric arm per style; the
  coverage pin `s1_d_4c_every_rowed_style_has_exactly_one_live_arm`
  (`projectile.rs:23770`) enforces one arm per rowed style.
- In-flight probe: `git branch -a` + `git log` — every sibling `proj-ai`
  branch (d1-d21, g1/g2/g3-proj-ai-missing) sits at base `a725acb0` with NO
  code commit, so the five ports below collide with nothing in flight.
- C# arm spans: `AI()`'s if/else chain (`:23199-36815`); kind lists parsed from
  `SetDefaults`' `else if (this.type == N)` chain (`:697-10800`).

## Coverage table (31-40)

P = ported (dispatch line in `projectile.rs`), M = missing (C# body span +
SetDefaults kinds). "arm" = the Rust fn that flies the style.

| style | state | Rust dispatch / C# body | kinds | arm |
| --- | --- | --- | --- | --- |
| 31 | P | :17296 (`:26786-26886` Clentaminator spray) | 145 (+146-149/1015-1017 unrowed) | `projectile_ai_misc::ai_spray` |
| 32 | P | :26887-27048 (timeLeft=10, 255-player/NPC/1000-proj bounce scans at ai0≥20) | 155 BeachBall | `beach_ball_step` (pool-side) |
| 33 | P | :17153 (`:27049-27135` flares) | 163/310/1008-1011 | `projectile_ai_light_items::flare_step` |
| 34 | P | :17522 (`:27136-27188` fireworks) | 167-170/415-418 | `projectile_ai_darts::firework_flight` |
| 35 | P | :27189-27291 (ai0>30 fall + ×0.985 drag, 14 clamp, rope-place walk) | 171/475/505/506 rope coils | `ai_rope_coil` |
| 36 | P | :17598 (`:27292-27471` bees/bat) | 181/566/307/316 | `projectile_ai_thrown::bee_step` + `ai_bat` |
| 37 | P | :27472-27499 (spear trap; row deviation — f19 F3, FIXED) | 186 SpearTrap | `ai_trap_spear` |
| 38 | P | :27500-27512 (flame jet trap, 6-tick sound cycle + 188 spawn) | 187 FlamethrowerTrap | `ai_flame_jet` |
| 39 | P | :27513-27779 (alpha −50 + owner 2000-px cut + piranha homing) | 190 MechanicalPiranha | `piranha_step` (pool-side) |
| 40 | P | :16973 (`:27780-27783` → `AI_040_LeafBlower`) | 206 Leaf | `ai_leaf_blower` |

**Verdict: 10 of 10 styles armed. The census found 5 MISSING (32/35/37/38/39);
this task ported all five, one style per commit (37 `0f92565b`, 38 `a336f4aa`,
35 `548a9190`, 32 `d9324e5f`, 39 `5771c1f0`). Code-only round — the witness
tests are WRITTEN NOT RUN (no cargo invoked, the task contract).**

## FINAL band table (RETIRE phase, 2026-11-21, d4 tip `cafec722` + retire fixes)

Re-pinned at the retire pass — every dispatch line below is the CURRENT
`projectile.rs` line (the union commits shifted the census-era numbers).
**10/10 styles ARMED, every row live, every arm dispatch-reachable.** All
witnesses WRITTEN NOT RUN (the code-only contract; the deferred gate is the
merge pass's `cargo test --workspace -- --skip migrates_version_2_kinds_to_item_ids`).

| style | status | dispatch (current) | arm | kinds | witness coverage (fn names, current homes) |
| --- | --- | --- | --- | --- | --- |
| 31 | ARMED | `projectile.rs:17568` | `projectile_ai_misc::ai_spray` | 145 (+146-149/1015-1017 unrowed) | `s1_d_4_unarmed_the_spray_cleanses_and_caps` + `…the_four_rows_carry_their_styles` + `…the_dispatch_arms_the_four_styles` (misc) |
| 32 | ARMED | `:17825` | `beach_ball_step` (pool-side) | 155 | `the_beach_ball_row_and_flight_tail` / `…bounces_off_an_npc_at_20_ticks` / `…dies_at_zero_velocity` (projectile.rs) + the return pair `w4_d4_union_the_beach_ball_death_returns_the_item` (pool) / `…the_beach_ball_return_drains_on_the_items_stream` (world_sim) + `w4_d4_retire_the_refund_gates_and_the_dig_cue` (foreign-ball arm) + the item-use `w4_d4_sweep_the_beach_ball_item_throws_the_155_and_consumes` + the row pin `w4_d4_verify_the_beach_ball_and_piranha_gun_item_rows` (editing.rs); wind-32 in `uses_wind_physics` (`:13896`) |
| 33 | ARMED | `:17425` | `projectile_ai_light_items::flare_step` | 163/310/1008-1011 | `the_flare_rows_carry_the_csharp_defaults` / `the_flare_fades_alpha_by_50_and_snapshots_its_velocity` / `the_flare_sticks_on_a_blocked_move_and_zeroes_damage` (light_items) |
| 34 | ARMED | `:17794` | `projectile_ai_darts::firework_flight` + the `dart_electro_firework_kill` burst (`:76464-76674`) | 167-170 (+415-418 unrowed boxes) | rows: `s1_p_i16_the_dart_rows_match_set_defaults` (the 167-170 loop, darts); the use arm: `s1_p_i16_the_fireworks_launcher_fires_two_rockets` (editing.rs); wind tail runs after the dispatch (the arm-owned-integration convention) |
| 35 | ARMED — RETIRE-PENDING canonical flip | `:17818` (d4 `ai_rope_coil`; the merge keeps g82's `35 => coil_step`, research/26e FINAL-FINAL) | d4 `ai_rope_coil` `:11413` / g82 `coil_step` (canonical) | 171/475/505/506 | rows `the_rope_coil_rows_carry_style_35`; fall `the_rope_coil_falls_after_30_ticks`; the four-kind placement loop `the_rope_coil_places_its_rope_and_dies` (the g82 salvage note); owner gate `the_rope_coil_placement_is_owner_gated`; refund queue `w4_d4_union_the_coil_death_queues_the_refund_ladder`; gates+cue `w4_d4_retire_the_refund_gates_and_the_dig_cue`; module ladder/reverse-walk/parity/NaN pins (projectile_rope_coil.rs); drain contract `w4_d4_union_the_coil_expiry_drains_the_refund_as_a_world_drop` (world_sim); usability ×3 (editing.rs); wind-35 in the list |
| 36 | ARMED | `:17913` | `projectile_ai_thrown::bee_step` + pool-side `ai_bat` (316, the i21 type-branch) | 181/566/307/316 | bee: `s1_p_i28_bee_steers_toward_target` + the beenade draw-order pair (thrown); bat: `s1_p_i21_the_bat_fades_and_flips` / `s1_p_i21_the_bat_homes_after_thirty_ticks` (projectile.rs) |
| 37 | ARMED | `:17804` | `ai_trap_spear` (the 185 stand-in RETIRED — F3) | 186 | `the_spear_reverses_on_solid_contact_and_kills_at_the_anchor` / `the_spear_flies_straight_and_points_off_its_anchor` (projectile.rs) |
| 38 | ARMED | `:17811` | `ai_flame_jet` | 187 (+188 child) | `the_flame_jet_cycles_every_six_ticks_and_never_moves` / `the_flame_jet_child_spawn_is_owner_gated` (projectile.rs) + the child ramp gate `w4_d4_sweep_the_flames_trap_launch_ramp_deals_no_damage` (the DOC-SWEEP `Damage_CanDealDamage` port, `:12448`) |
| 39 | ARMED | `:17833` | `piranha_step` (pool-side) | 190 | `the_piranha_row_and_alpha_fade` / `the_piranha_dies_with_its_owner_and_returns_on_release` (projectile.rs) + the row census (data.rs:621 + :2352-2363, `94a51fb7`) + the item-use trio/channel `w4_d4_sweep_the_piranha_gun_fires_a_trio_and_latches_the_channel` + the draw-chain determinism `…draw_chain_is_seed_deterministic` + the row pin (editing.rs) + `w4_d4_sweep_change_dir_flips_facing_and_negates_the_rotations` (player.rs, the ChangeDir folds census) |
| 40 | ARMED | `:17245` | `ai_leaf_blower` | 206 | `ai_040_leaf_blower_fades_flaps_and_drifts` / `leaf_blower_row_and_frames` / `test_dw_3_1_style_40_the_leaf_spin_sign_latches_on_the_first_tick` (projectile.rs) |

The one-arm-per-rowed-style coverage pin
(`s1_d_4c_every_rowed_style_has_exactly_one_live_arm`, `projectile.rs:24715`)
guards the whole band — a second `35 =>` at the merge would trip the
`unreachable_patterns` deny (S1-D-4c), which is why the keep-one verdict
matters (research/26e FINAL-FINAL).

## Finding F3 carried from W4-f19 — style 37's row was a DELIBERATE stand-in

The Spear Trap's spear (186) is C# `aiStyle 37`, 10×14, tileCollide false,
ignoreWater, friendly AND hostile, penetrate −1, timeLeft 300, trap
(`Projectile.cs:2728-2740`). The port's row WAS the 185 spiky-ball stand-in
(14×14, aiStyle **14**, tileCollide true, timeLeft 900 —
`projectile/data.rs:2264-2273`), a RECORDED deviation (ANALYSIS "The projectile
data-row deviations (S1-D-5)"). Correcting the row to 37 before the arm exists
would park the spear in `_ => {}` forever, so this task lands the arm AND the
row correction TOGETHER and retires the ANALYSIS stand-in note. The stale doc
comments at `projectile.rs:301-306` called 186 "the boulder (aiStyle 37)" — the
boulder is kind 99 / aiStyle 25; 186 is the spear trap. Fixed here.

## Port notes (per style)

- **32 BeachBall (155):** `SetDefaults` `:2426-2433` (44×44, aiStyle 32,
  friendly, netImportant). The AI pins timeLeft=10, counts ai[0], and at
  ai[0]≥20 runs THREE bounce scans (255 players / maxNPCs NPCs / 1000
  projectiles) that reset ai[0], kick velocity off the hit entity's
  direction/velocity and bump ai[1]. The tail is the wet/dry drag + gravity.
  Movement is `HandleMovement`'s type-155 arm (`:18603-18618`: the ai[1]>10
  `Game.BallBounceResult` chat, ai[1]=0, the ×−0.6 axis bounces) + the wet-exit
  Y×0.5 (`:16920-16923`). Pool-side step (the projectile scan reads the slot
  array); the chat rides a pool intent drained to the chat monitor.
- **35 rope coils (171/475/505/506):** `SetDefaults` `:2570-2578` (171/505/506)
  / `:5496-5504` (475) — 14×14, aiStyle 35, penetrate −1, tileCollide false,
  timeLeft 400. The AI counts ai[0], at >30 adds 0.2 gravity + ×0.985 X drag +
  14 Y clamp, spins by (|vx|+|vy|)·direction·0.02, then (owner-gated) runs
  `Collision.TileCollision` + the bottomSlope snap and the 10-cell rope-place
  walk (`WorldGen.IsRope` skip, the 314/platform toggle, the tileCut/165 clear,
  `PlaceTile` 213/353/365/366 per kind), then Kill. Takes `&mut World`.
- **37 spear trap (186):** `SetDefaults` `:2728-2740`. The AI latches the
  anchor (Center − velocity·1.5) on the first tick, points rotation at it, and
  reverses velocity on SolidCollision or the 300-px range cap (outbound) /
  kills on SolidCollision or reaching the anchor (return). tileCollide false →
  free integrate. Row corrected from the 185 stand-in (see F3).
- **38 flame jet (187):** `SetDefaults` `:2741-2751` (6×6, aiStyle 38,
  alpha 255, tileCollide false, ignoreWater, timeLeft 60, trap). The AI counts
  ai[0]; at ≥6 resets, plays Item34 and (owner-gated) spawns the 188 FlamesTrap
  with the jet's velocity/damage/knockBack. aiStyle 38 is on `UpdatePosition`'s
  skip list (`:20435`) → NO integrate. The 188 spawn rides `wants_dart_spawns`,
  the cue rides `wants_sound`. The wiring drain's 187 drop is lifted.
- **39 MechanicalPiranha (190):** `SetDefaults` `:2779-2790` (22×22, aiStyle 39,
  friendly, penetrate −1, alpha 255, ranged, usesLocalNPCImmunity cooldown 14).
  The AI fades alpha −50, kills on owner-inactive/dead/2000-px, then the
  channel-release / target-lock / return-to-owner state machine (ai[0]/ai[1]),
  the 16-px target snap + the 190 separation push (8-px sibling band), the
  4-tick frame cycle and the `AI_AdjustPlayerItemRotationToFaceProjectile`
  item-rotation write. Pool-side step (the sibling scan reads the slot array);
  the player writes ride `&mut Player` + the dummy-item-time intent.

## Deliberate skips (recorded in ANALYSIS)

Dust/gore bodies, `Lighting.AddLight`, `NetMessage.SendData` (the netUpdate
flag is set; the MP relay is the net task's), the beach-ball chat's
`NetworkText` localization folds to `lang::text_args`. The piranha's
`SetDummyItemTime(5)` rides the pool's player-write intent (the App-level item
timers are not reachable from the slot loop — the golf `PlayerWrites`
precedent).

## DOC-SWEEP deviation census (2026-11-21, item 5)

The d4 ANALYSIS section records two `Player.ChangeDir` folds (the piranha's
`flag18` caller, `Player.cs:42091-42116`). Re-censused against source:

- **The pulley snap (`:42097-42110`) — NOT ported, correctly deferred.** The
  `pulley`/`pulley_dir` fields exist (`player.rs:1210-1211`) but no sim
  source sets them: the only C# writers are the rope-climb machine
  (`Player.cs:19690-19746`), unported. The `pulley && pulleyDir == 2` arm is
  unreachable in the port today — porting the snap would be dead code. It
  lands with the rope-movement task (its `Main.cameraX` shift is draw-time
  in the port, `world_sim/camera.rs`). NOT small-portable in isolation.
- **The `itemLocation.X` mirror (`:42115`) — observationally folded.**
  `itemLocation` is not a Player field here; the draw recomputes it per
  frame from `facing` (`melee_swing::swing_item_placement`,
  `render/player_layers.rs::item_location`), so the facing flip already
  mirrors every subsequent frame's item position. No state to port.

Verdict: both folds stand (no code port needed); the `change_dir` doc
comment now cites this census and a witness pins the flip + the three
rotation negates + the same-direction early return.

## DOC-SWEEP follow-ups (2026-11-21, same branch)

- **Rope-coil duplicate vs W4-g82 (`de797f45`)**: the arm-by-arm keep-one
  comparison + the union recommendation (keep d4's machine; keep g82's
  chain preview / Kill-refund ladder / usability walk / framing arm; drop
  g82's duplicate `coil_step` half) lives in
  `26e-rope-coil-keep-one.md`. The one behavioural gap found: the coil's
  `Kill()` refund (`Projectile.cs:77572-77615`) is unported on this branch
  — g82's `coil_death_drop` is the ladder to wire at the death drain.
  **RETIRED (2026-11-21)**: the union landed (refund `bb2f8d2c`, usability
  `1cae02fd`, preview geometry `bb2f8d2c`/`ee950e51`); the FINAL-FINAL
  re-adjudication against g82 tip `187557f8` REVERSED the machine verdict
  (g82 `coil_step` canonical — see `26e-rope-coil-keep-one.md` "FINAL-FINAL").
