# 26 — Projectile AI families (aiStyle dispatch)

> Wave 3 · researched 2026-11-12 · sources: decompiled + crates + ROADMAP/ANALYSIS/PORT_MAP

## Scope

The C# side is `decompiled/Terraria/Projectile.cs` (~81 000 lines). This module owns
the **family-behaviour layer** of the projectile AI: the `AI()` dispatch chain
(`Projectile.cs:23199-36815`) that maps an `aiStyle` to a behaviour, and the ~60
`AI_xxx` methods it calls (`AI_001` 54002, `AI_003_Boomerang` 38866, `AI_010` 57826,
`AI_026` 58102, `AI_067_FreakingPirates` 49121, `AI_099_1_Counterweights`/`AI_099_2_Yoyos`
67380/67733, `AI_120_StardustGuardian` 44634, `AI_158_BabyBird` 48757,
`AI_165_Whip` 47667, `AI_187_ShadowHand` 42502, `AI_189_Volcano` 42264,
`AI_210_YoyoShots` 36809). The core (defaults, spawn protocol, damage pass) belongs to
module 06; this module is the **behaviour** layer the core deferred.

The Rust side is two files: `crates/tr-app/src/projectile.rs` — the
`match data.ai_style` dispatch (`projectile.rs:8617`) plus the ~40 ported `ai_*`
family functions — and `crates/tr-app/src/world_sim/projectiles.rs` (318 lines), the
60 Hz driver that feeds the owner/aim/channel/death snapshots into the pool.

## Key entities

- `Terraria/Projectile.cs::Projectile.AI` (23199-36815) — the if/else chain over
  `aiStyle`: **210 arm bodies covering aiStyle 1..210 with no gaps**, plus one extra
  sub-arm (the aiStyle-99 split: `AI_099_1_Counterweights` at 33153 for types
  556-561/1079, then `AI_099_2_Yoyos` at 33157).
- `Terraria/Projectile.cs::Projectile.SetDefaults` (697-10800) — the `this.aiStyle = N`
  assignments: **183 distinct aiStyle values** are actually bound to a projectile type.
- `Terraria/Projectile.cs::Projectile.ShouldUseWindPhysics` (22101-22136) — the wind
  list (aiStyles 1,2,8,10,14,16,17,21,24,28,29,32,33,34,35,49,72,93,96,106) plus the
  `ProjectileID.Sets.WindPhysicsImmunity` per-type override.
- `crates/tr-app/src/projectile.rs::ProjectilePool::update` (8505-9102) — the pass; the
  `match data.ai_style` sits at 8617 with **32 unique arms** and `_ => {}` at 9026.
- `::Projectile::ai_style1_flight` (5277) + `::ai_style1_rider` (5313) +
  `::ai_book_of_skulls` (5389) — the aiStyle-1 family (generic arrow flight + the four
  M4-PROJ-6 riders + the Book of Skulls).
- `::ai_boomerang` (4452), `::ai_flail` (4643), `::ai_flail_bounce` (4950),
  `::ai_bomb` (5168), `::ai_yoyo` (3478), `::ai_yoyo_shot` (3833), `::ai_yoyo_beam`
  (3868), `::ai_whip` (4547), `::ai_flying_minion` (2740), `::minion_separation` (2689),
  `::ai_shuriken` (3446).
- `::ai_ball_of_fire` (5501), `::ai_demon_scythe` (5606), `::ai_ice_block` (5637),
  `::ai_crystal_storm` (5812), `::ai_frost_bolt` (5909), `::ai_inferno_fork` (5927),
  `::ai_magic_missile` (5970), `::ai_gem_staff` (6195), `::ai_magnet_sphere` (6463),
  `::ai_magnet_bolt` (6527), `::ai_spirit_flame` (6562), `::ai_leaf_blower` (5851).
- `::ai_phasmal_eye` (3906), `::ai_phasmal_sphere` (3973), `::ai_phasmal_deathray`
  (4012), `::ai_moon_leech` (4086), `::ai_nebula_eye` (4290), `::ai_stardust_mark`
  (4264), `::ai_tower_damage_bolt` (4319), `::ai_vortex_ritual` (4144),
  `::ai_vortex_bolt` (4214).
- `::store_sentry_aim` (2666) + `::sentry_npc_targeting` (3319) — the sentry aiming
  seam; the per-kind shot machine is **not** ported.
- `crates/tr-app/src/projectile.rs::proj_data` (1046-2257) — the `ai_style` column of
  the data table: **172 arm groups / 229 projectile kinds / 60 distinct aiStyles**.
- `crates/tr-app/src/world_sim/projectiles.rs::update_projectiles` (13-204) — the
  fixed-step driver; builds `YoyoInputs` (121) and `FlailInputs` (148) and drains
  `drain_mark_spawns` (195).

## Dependencies

- Calls into: `crate::npc::Npc` (`can_be_chased_by`, `hurt_from_projectile`),
  `crate::player::Player` (channel/death/melee_speed), `crate::grapple::hook_spec_id`,
  `tr_core::whip`, `tr_core::edit` (the Ice Rod tile writes), `tr_core::tile`.
- Called by: `world_sim/projectiles.rs::update_projectiles` ← `App`'s fixed step;
  `world_sim/editing.rs` (the shoot path); `npcs.rs` (hostile shots); `wiring.rs`.
- Shared data structures: the `Projectile` slot (`local_ai` carries the sentry aim),
  `ProjData.ai_style`, `YoyoInputs`/`FlailInputs`, `hook_reg`, the `wants_ritual_bolt` /
  `wants_mark_spawn` intents.

## Port status

| C# subsystem | Status | Rust location | Evidence / notes |
| --- | --- | --- | --- |
| `AI()` dispatch chain (23199-36815) | PARTIAL | `projectile.rs:8617` | 32 unique arms for the C#'s 210 aiStyles; `_ => {}` (9026) silently no-ops the other 178. |
| aiStyle 1 — arrow/bullet family | PARTIAL | `::ai_arrow` (4387), `::ai_style1_flight` (5277) | The generic flight is faithful; C# `AI_001` has **142 distinct per-type arms** (197 `type ==` checks at 54002-57826) — the port covers only the 4 M4-PROJ-6 riders + the Book of Skulls. |
| aiStyle 3 — boomerang | PORTED | `::ai_boomerang` (4452) | out/return, the bounce, the 3000 px cut-off. |
| aiStyle 7 — grappling hooks | PORTED | `::ai_hook` (grapple.rs) + arm 8642 | M4-PROJ-5 done; the chain/sprite rendering is deferred (PORT_MAP 152). |
| aiStyle 8/9/18/22/24/28/29/47/48/50/129 — casters | PORTED | 5501/5970/5606/5637/5812/5909/6195/6463/6527/5927/6562 | M4-PROJ-6 done (PORT_MAP 154). |
| aiStyle 15 — flails | PARTIAL | `::ai_flail` (4643) | M4-PROJ-3b done for the family, but the **helper rows 928/248/405 are not**: there is no `ai_bubble` and no aiStyle-70 arm. |
| aiStyle 16 — bombs | PORTED | `::ai_bomb` (5168) | M4-PROJ-9 done. |
| aiStyle 40 — leaf blower | PORTED | `::ai_leaf_blower` (5851) | M4-PROJ-6b-leaf done. |
| aiStyle 53 / 197 — sentries | PARTIAL | `::sentry_npc_targeting` (3319) + `store_sentry_aim` (2666) | Only the aim is computed and parked in `local_ai`; **the per-kind shot machine never fires** (arm comments 8876-8901, 8904-8923). |
| aiStyle 62 — minions | PARTIAL | `::ai_flying_minion` (2740) + arm 8926 | The follow/attack/return machine is ported; the **attack-shot families are not** ("the spawn drops until the shot families land", 8933). |
| aiStyle 70 — Flairon bubble | MISSING | — | No arm and no `ai_bubble` function anywhere in `crates/` (PORT_MAP 157 lists one). |
| aiStyle 82/83/84/85/102/103/122 — lunar battery | PORTED | 3906/3973/4012/4086/4290/4264/4319 | M4-EVENT-16 done (PORT_MAP 233). |
| aiStyle 88 / 108 — Vortex ritual | MISSING (wired) | `::ai_vortex_ritual` (4144), `::ai_vortex_bolt` (4214) | The functions exist and are unit-tested, but **no dispatch arm calls them** — only the test module does (13858-13926). M4-EVENT-17 claims done; PORT_MAP 236 claims done. |
| aiStyle 99 — yoyos | PARTIAL | `::ai_yoyo` (3478) | `AI_099_2_Yoyos` ported; `AI_099_1_Counterweights` (types 556-561/1079) has no row and no arm. |
| aiStyle 115 / 210 — Terrarian beam, yoyo shots | PORTED | `::ai_yoyo_beam` (3868), `::ai_yoyo_shot` (3833) | M4-PROJ-4b done. |
| aiStyle 165 — whips | PORTED | `::ai_whip` (4547) | M4-PROJ-7 done; the render ribbon is deferred. |
| `ShouldUseWindPhysics` (22101) | PARTIAL | `::uses_wind_physics` (6923) | Rust matches only aiStyles 8/24/28/29; the C# list has 20 styles, so the ported arrow (1), shuriken (2) and bomb (16) arms get **no wind**. |
| `AI_001`'s per-type arms (142 types) | MISSING | — | Only the generic flight + 4 riders + Book of Skulls; the remaining ~137 type variants fall back to the generic flight. |
| The other 178 aiStyles | MISSING | — | No arm and no data row: the projectiles are not spawnable and behave as inert points if forced. |

## Tests

`crates/tr-app/src/projectile.rs` — 193 `#[test]`s. Family-relevant names:
`the_boomerang_data_is_verbatim` (10653), `the_boomerang_switches_to_the_return_at_30_ticks`
(10666), `the_outbound_boomerang_bounces_off_a_wall` (10704), `the_returning_boomerang_passes_tiles_and_is_caught`
(10751), `the_stray_boomerang_dies_past_3000_px` (10795), `DW_3_6_the_dual_flail_inverts_the_throw`
(10974), `DW_4_3_the_drippler_spawns_its_extra_ball_on_the_arm_end` (11186), `DW_5_2_the_dual_flail_discounts_the_damage`
(11335), `the_bomb_data_is_verbatim` (11402), `the_bomb_detonates_and_digs_the_crater`
(11419), `the_yoyo_data_is_verbatim` (11595), `a_yoyo_spawns_through_the_dispatch_arm` (11664),
`the_yoyo_shot_fades_out_and_dies_at_twenty` (11977), `the_whip_swing_flies_out_from_the_anchor_and_dies`
(12846), `the_whip_hits_each_npc_once_across_the_swing` (12891), `the_moon_lord_whip_spawns_its_arms`
(12962), `test_dw_3_1_style_8_gravity_starts_at_ai_1_20` (14525), `test_dw_3_1_style_18_the_scythe_accelerates_from_ai_0_30_and_clamps_at_100`
(14671), `test_dw_3_1_style_22_the_placed_block_drains_ai_0_and_is_mined_away_at_minus_900`
(14722), `test_dw_3_1_style_24_the_crystal_storm_fades_after_ai_1_130` (14882),
`test_dw_3_1_the_styles_without_a_bounce_arm_die_on_tile_contact` (15125),
`test_dw_3_1_the_rider_kinds_take_the_rider_arm_not_the_plain_arrow_arm` (15156),
`test_dw_3_3_the_style_8_arm_takes_the_wind_tail_at_a_known_vector` (15593),
`test_dw_3_3_the_ported_arrow_arm_stays_wind_free` (15666), `test_dw_4a_the_gem_staff_features_round_trip_through_ai0`
(15746), `test_dw_4b_the_magnet_sphere_fires_its_bolt_past_the_cooldown` (16192),
`test_dw_4b_the_spirit_flame_retargets_inside_400f_and_caps_the_life` (16262),
`the_portal_spawns_two_aliens_on_the_90th_tick` (13852), `test_edge_unported_ai_style_holds_position`
(16945), `the_040_leaf_blower_fades_flaps_and_drifts` (9781).
`crates/tr-app/src/world_sim/projectiles.rs` — `the_summon_fixture_spawns_one_pair_per_activation`
(229), `the_summon_fixture_respects_a_full_pool` (288).

## Deviations & risks

| Deviation | Severity | Admitted? |
| --- | --- | --- |
| The dispatch covers **32 of 210** aiStyles; the other 178 hit `_ => {}` (9026) and hold position | behavioural | Partly — module 06 admits "the rest silently no-op" |
| **6 duplicated match arms** (module 06 says 5): aiStyle 1 (8624 wins / 8728 dead), 3 (8631 / 8729), 14 (8695 / 8730), **16 (8696 / 8731 — the 6th, missed by module 06)**, 115 (8732 / 8925), 99 (8777 / 8983, byte-identical bodies). Rust matches the **first** arm, so the later copies are dead code | determinism (maintenance) | No |
| aiStyle 88 / 108: `ai_vortex_ritual`/`ai_vortex_bolt` exist and are tested but **no dispatch arm calls them** — the Vortex ritual projectiles never animate in the live sim | behavioural | No — M4-EVENT-17 and PORT_MAP 236 both claim "done" |
| aiStyle 70 (the Flairon's bubble 405): **no arm and no `ai_bubble` function**; PORT_MAP 157 lists `ai_bubble` as ported | behavioural | No |
| aiStyle 53 / 197 sentries compute an aim and park it in `local_ai`; the shot machine never fires | behavioural | Yes (ANALYSIS "Minions & sentries") |
| aiStyle 62 minions: the attack-shot families are not ported (the shot "drops") | behavioural | Yes (ANALYSIS) |
| `AI_001`'s ~137 remaining per-type arms are absent — one generic arrow flight covers 32 arm groups / 36 kinds | behavioural | No |
| `ShouldUseWindPhysics`: Rust matches 8/24/28/29, the C# lists 20 styles → the arrow/shuriken/bomb arms get no wind | behavioural | Partly (module 06) |
| aiStyle 99: `AI_099_1_Counterweights` (556-561/1079) has neither a row nor an arm | behavioural | No |
| The Rust `ai_shuriken` (3446) is a Rust-invented approximation (2 half-steps, 0.6 bounce) of the C# aiStyle-2 block (23205-23689), a 480-line per-type machine covering ~40 vanilla types | behavioural | No |
| The 28 spawnable styles with a data row but no arm — see the gap list below | behavioural | No |
| The shuriken (kind 3) has **no `proj_data` row**; it is spawned through the plain `spawn` (9607) and AI'd by an explicit kind check (8585), bypassing the aiStyle table entirely | cosmetic / structural | No |

## Task seeds

- **S** — Delete the six duplicated arms (8728/8729/8730/8731/8925/8983) and replace
  `_ => {}` with an assert-on-unknown-aiStyle arm gated by an allow-list, so a new
  `proj_data` row cannot silently miss its AI. (Independent of everything below.)
- **S** — Wire `ai_vortex_ritual`/`ai_vortex_bolt` into the dispatch (aiStyle 88/108)
  and fix the M4-EVENT-17 + PORT_MAP 236 "done" claims. The functions already exist.
- **S** — Add the aiStyle-70 arm and `ai_bubble` (the Flairon's bubble homing, C#
  30304-30352) or correct PORT_MAP 157.
- **M** — Port the aiStyle-53 / 197 sentry shot machines on top of the existing
  `store_sentry_aim` seam (the aim is already there; only the fire is missing).
- **M** — Port the aiStyle-62 minion attack-shot families (the six kinds' projectiles
  need `proj_data` rows + arms).
- **M** — Port `AI_001`'s remaining per-type arms (142 `type ==` checks) or split the
  style-1 arm into a per-kind table so the ~137 unported variants are explicit.
- **M** — Port the 28 gap-list styles, grouped by family: the sentry/turret group
  (120/123/130/134/137/138), the pillar group (88/108/133), the beam/weapon group
  (27/51/59/71/75/118/119), and the small ones (2/23/26/66/67/70/79/80/157/158/187/189).
- **L** — Port `AI_099_1_Counterweights` (67380) + the counterweight rows 556-561/1079.
- **L** — Port the remaining ~150 vanilla aiStyles (the 1.4 boss/event families 130-210
  and the pre-1.3 groups 30-99) as a batch, one family per ROADMAP row.

## Gap list — projectile kinds in the Rust data table with no faithful AI

The Rust `proj_data` table binds **60 distinct aiStyles** across 172 arm groups
(229 kinds). The **28 styles below have a data row but no dispatch arm**, so those
projectiles hold position and never animate, damage or expire:

| style | Rust kinds (ids) | C# behaviour | Severity |
| --- | --- | --- | --- |
| 2 | DUTCHMAN_SHELL (240) | aiStyle-2 machine: wind, per-type dust/rotation (23205-23689) | behavioural |
| 23 | EYE_FIRE (101) | 20-tick alpha fade-in + the expected-type switch (33165) | behavioural |
| 26 | SLIME_MINION (266) | `AI_026` (58102): the shared minion/sentry machine | behavioural |
| 27 | UNHOLY_TRIDENT (114), UNHOLY_TRIDENT_HOSTILE (115), SWORD_BEAM (116), TERRA_BEAM (132), LIGHT_BEAM (156), NIGHT_BEAM (157), ENCHANTED_BEAM (173) | the beam machine: the velocity 1.125 accel, the per-type tails (26420) | behavioural |
| 51 | LOST_SOUL_HOSTILE (293), LOST_SOUL (297) | the 5-shot volley after the 4-tick charge (28504) | behavioural |
| 59 | SPECTRE_WRATH (356) | the 60-tick charge then the friendly flip + the target hunt (28938) | behavioural |
| 66 | RETANIMINI (387), SPAZMAMINI (388) | the owner-centre aim + the death kill (29604) | behavioural |
| 67 | STORM_TIGER_1/2/3 (833/834/835) | `AI_067_FreakingPirates` (49121) | behavioural |
| 70 | FLAIRON_BUBBLE (405) | NPC-scan homing within 650 px (30304-30352) | behavioural |
| 71 | TYPHOON (409) | the 6-shot fan + the dust spray (30355) | behavioural |
| 75 | LAST_PRISM (633) | `AI_075` (30617) — the held-item machine | behavioural |
| 79 | SAUCER_DEATHRAY (447) | the saucer deathray, aimed at the mounted NPC (30791) | behavioural |
| 80 | SAUCER_MISSILE (448) | the saucer missile (30911) | behavioural |
| 88 | VORTEX_RITUAL_BOLT (580) | the 900-wrap latch + the tile stop (31937-32261) | behavioural |
| 108 | VORTEX_LIGHTNING (578), VORTEX_PORTAL (579) | the ritual timeline, the aim rotation, the 580/427 spawns (33679-33988) | behavioural |
| 118 | NEBULA_ARCANUM (617) | the alpha ramp + the velocity-4 gate (34638) | behavioural |
| 119 | NEBULA_ARCANUM_SUBSHOT (618) | the expected-type switch + the 420-speed spawn (34809) | behavioural |
| 120 | STARDUST_GUARDIAN (623) | `AI_120_StardustGuardian` (44634) | behavioural |
| 123 | MOONLORD_TURRET (641), RAINBOW_CRYSTAL (643) | `AI_123_FloatingTurrets` (34966) | behavioural |
| 130 | FLAME_BURST_TOWER_1/2/3 (663/…) | `AI_130_FlameBurstTower` (36310) | behavioural |
| 133 | DARK_MAGE_MARKER (673) | the marker fade/scale (36435) | behavioural |
| 134 | BALLISTRA_TOWER_1/2/3 (677/…) | `AI_134_Ballista` (36499) | behavioural |
| 137 | LIGHTNING_AURA_1/2/3 (688/…) | `AI_137_LightningAura` (36511) | behavioural |
| 138 | EXPLOSIVE_TRAP_1/2/3 (691/…) | `AI_138_ExplosiveTrap` (36515) | behavioural |
| 157 | DEERCLOPS_ICE_SPIKE (961) | `AI_157_SharpTears` (36591) | behavioural |
| 158 | BABY_BIRD (759) | `AI_158_BabyBird` (48757) | behavioural |
| 187 | SHADOW_HAND (965) | `AI_187_ShadowHand` (42502) | behavioural |
| 189 | CASCADE_EXPLOSION (1125) | `AI_189_Volcano` (42264) | behavioural |

Partial-AI kinds (an arm exists but the family is incomplete): the aiStyle-53 sentries
(FROST_HYDRA, QUEEN_SPIDER, HOUNDIUS_SHOOTIUS — aim only), aiStyle 197
(DEAD_CELLS_BARNACLE — aim only), the aiStyle-62 minions (HORNET, FLYING_IMP, TEMPEST,
UFO_MINION, STARDUST_CELL, ABIGAIL — no attack shots), the aiStyle-1 kinds other than
the 5 riders (32 arm groups / 36 kinds share one generic flight), the aiStyle-15 helper
rows 928 (DRIPPLER_EXTRA_BALL) / 248 (FLOWER_POW_PETAL) / 405 (FLAIRON_BUBBLE), and the
counterweights (556-561/1079, no rows at all).

## Coverage table — every C# aiStyle → its Rust arm

Verdict: **P** = ported · **Pa** = partial · **M** = missing. Severity is the blast
radius of the gap (behavioural = the projectile misbehaves; n/a = the style has no
Rust data row, so nothing spawns it).

| aiStyle | C# behaviour (arm line) | Rust arm | Verdict | Sev | Note |
| --- | --- | --- | --- | --- | --- |
| 1 | `AI_001` arrow/bullet flight, 142 per-type arms (54002) | `ai_arrow`/`ai_style1_flight` + riders | Pa | behavioural | one generic arm for 36 kinds; only 5 kinds special-cased |
| 2 | inline: wind + per-type dust/rotation (23205) | `ai_shuriken` (3446), kind-gated | Pa | behavioural | DutchmanShell has a row but no arm |
| 3 | `AI_003_Boomerang` (23690) | `ai_boomerang` | P | — | |
| 4 | inline: net-mode timers, per-type tails (23694) | — | M | n/a | |
| 5 | inline: localAI timers (23830) | — | M | n/a | |
| 6 | inline: velocity 0.95 damp (24100) | — | M | n/a | |
| 7 | `AI_007_GrapplingHooks` (24229) | `ai_hook` | P | — | |
| 8 | inline: fireball bounce / cursed flame (24233) | `ai_ball_of_fire` | P | — | |
| 9 | `AI_009_MagicMissiles` (24363) | `ai_magic_missile` | P | — | |
| 10 | `AI_010` falling-block flight (24367) | the `falling_tile` path | Pa | behavioural | no style-10 arm; the fall is a separate path |
| 11 | inline: spriteDirection flip (24371) | — | M | n/a | |
| 12 | inline: sound + fade (24501) | — | M | n/a | |
| 13 | inline: owner-death/distance kill (24636) | — | M | n/a | |
| 14 | inline: spear arm, gravity + tile kill (24830) | `ai_spear` | Pa | behavioural | generic arm; ~20 C# types |
| 15 | `AI_015_Flails` (25308) | `ai_flail` | Pa | behavioural | helper rows 928/248/405 unported |
| 16 | `AI_016_Bombs` (25312) | `ai_bomb` | P | — | |
| 17 | inline: 0.98 damp + 0.2 gravity (25316) | — | M | n/a | |
| 18 | inline: Demon Scythe (25367) | `ai_demon_scythe` | P | — | |
| 19 | `AI_019_Spears` (25477) | — | M | n/a | |
| 20 | inline: timeLeft 60 + 4-frame anim (25481) | — | M | n/a | |
| 21 | inline: velocity-clamped rotation (25585) | — | M | n/a | |
| 22 | inline: Ice Rod block place/mine (25607) | `ai_ice_block` | P | — | |
| 23 | inline: 20-tick alpha fade (25861) | — | M | behavioural | EYE_FIRE row |
| 24 | inline: Crystal Storm shard rotation (25936) | `ai_crystal_storm` | P | — | |
| 25 | inline: net timer + surface rain (26015) | — | M | n/a | |
| 26 | `AI_026` minion machine (26416) | — | M | behavioural | SLIME_MINION row |
| 27 | inline: the beam accel 1.125 + per-type tails (26420) | — | M | behavioural | 7 rows (the Trident + the beams) |
| 28 | inline: Frost Staff bolt (26555) | `ai_frost_bolt` | P | — | |
| 29 | `AI_029_GemStaffs` (26676) | `ai_gem_staff` | P | — | |
| 30 | inline: alpha/dust ramp (26680) | — | M | n/a | |
| 31 | inline: 4-frame counter (26786) | — | M | n/a | |
| 32 | inline: alpha ramp (26887) | — | M | n/a | |
| 33 | inline: light + alpha fade (27049) | — | M | n/a | |
| 34 | inline: rotation + ai[1] state (27136) | — | M | n/a | |
| 35 | inline: velocity 1.25 accelerate (27189) | — | M | n/a | |
| 36 | inline: wet-kill + alpha fade (27292) | — | M | n/a | |
| 37 | inline: boulder fall (27472) | — | M | n/a | |
| 38 | inline: flame-jet trap (27500) | — | M | n/a | |
| 39 | inline: alpha ramp + frame (27513) | — | M | n/a | |
| 40 | `AI_040_LeafBlower` (27780) | `ai_leaf_blower` | P | — | |
| 41 | inline: random frame (27784) | — | M | n/a | |
| 42 | inline: crystal-leaf attach (27793) | — | M | n/a | |
| 43 | inline: rotation + sound (27860) | — | M | n/a | |
| 44 | inline: 0.96 damp + alpha (27893) | — | M | n/a | |
| 45 | inline: ai[0]/ai[1] vector (27950) | — | M | n/a | |
| 46 | inline: tile check (28107) | — | M | n/a | |
| 47 | `AI_047_MagnetSphere` (28200) | `magnet_sphere_step` | P | — | |
| 48 | inline: magnet bolt travel (28204) | `ai_magnet_bolt` | P | — | |
| 49 | inline: hostile switch (28243) | — | M | n/a | |
| 50 | inline: Inferno Fork bolts (28323) | `ai_inferno_fork` | P | — | |
| 51 | inline: the 5-shot volley (28504) | — | M | behavioural | LOST_SOUL rows |
| 52 | inline: the player-relative aim (28560) | — | M | n/a | |
| 53 | `AI_053_FrostHydra` (28564) | `sentry_npc_targeting` | Pa | behavioural | aim only, no shot |
| 54 | inline: the raven flag (28773) | — | M | n/a | |
| 55 | inline: frame counter (28845) | — | M | n/a | |
| 56 | inline: rotation from ai[0] (28867) | — | M | n/a | |
| 57 | inline: ai[0] cap 30 + gravity (28906) | — | M | n/a | |
| 58 | inline: alpha fade (28938) | — | M | n/a | |
| 59 | inline: the 60-tick charge + friendly flip (28938 arm) | — | M | behavioural | SPECTRE_WRATH row |
| 60 | inline: scale shrink → velocity boost (28994) | — | M | n/a | |
| 61 | `AI_061_FishingBobber` (29310) | — | M | n/a | |
| 62 | `AI_062` minion machine (29314) | `ai_flying_minion` | Pa | behavioural | no attack shots |
| 63 | inline: owner-inactive kill (29318) | — | M | n/a | |
| 64 | inline: multi-shot (29419) | — | M | n/a | |
| 65 | inline: ai[1] counter (29534) | — | M | n/a | |
| 66 | inline: the owner-centre aim (29604) | — | M | behavioural | Retinamini/Spazmamini rows |
| 67 | `AI_067_FreakingPirates` (30162) | — | M | behavioural | Storm Tiger rows |
| 68 | inline: rotation + direction (30166) | — | M | n/a | |
| 69 | inline: faces the owner (30240) | — | M | n/a | |
| 70 | inline: NPC-scan homing (30304) | — | M | behavioural | Flairon bubble row |
| 71 | inline: the 6-shot fan (30355) | — | M | behavioural | TYPHOON row |
| 72 | inline: alpha fade (30455) | — | M | n/a | |
| 73 | inline: the tile-338 check (30480) | — | M | n/a | |
| 74 | inline: extraUpdates multiply (30605) | — | M | n/a | |
| 75 | `AI_075` the held item (30617) | — | M | behavioural | LAST_PRISM row |
| 76 | inline: heldProj + mount check (30621) | — | M | n/a | |
| 77 | inline: the active sound (30671) | — | M | n/a | |
| 78 | inline: alpha −30 (30762) | — | M | n/a | |
| 79 | inline: the saucer deathray (30791) | — | M | behavioural | SAUCER_DEATHRAY row |
| 80 | inline: the saucer missile (30911) | — | M | behavioural | SAUCER_MISSILE row |
| 81 | inline: penetrate-based (31001) | — | M | n/a | |
| 82 | inline: the Phantasmal Eye homing (31214) | `ai_phasmal_eye` | P | — | |
| 83 | inline: the Phantasmal Sphere (31322) | `ai_phasmal_sphere` | P | — | |
| 84 | inline: the Deathray (31377) | `ai_phasmal_deathray` | P | — | |
| 85 | inline: the Moon Leech (31758) | `ai_moon_leech` | P | — | |
| 86 | inline: sound + ai[0]++ (31827) | — | M | n/a | |
| 87 | inline: position.Y = ai[0] (31901) | — | M | n/a | |
| 88 | inline: the Vortex wrap latch (31937) | `ai_vortex_bolt` **not dispatched** | M | behavioural | VORTEX_RITUAL_BOLT row |
| 89 | inline: alpha ramp (32262) | — | M | n/a | |
| 90 | inline: kill on owner death (32387) | — | M | n/a | |
| 91 | inline: scale shrink (32515) | — | M | n/a | |
| 92 | inline: per-type flags (32558) | — | M | n/a | |
| 93 | inline: alpha fade (32821) | — | M | n/a | |
| 94 | inline: frame counter (32883) | — | M | n/a | |
| 95 | inline: alpha −20 (32964) | — | M | n/a | |
| 96 | inline: ai[0] += 0.6, kill at 500 (33004) | — | M | n/a | |
| 97 | inline: frame counter (33029) | — | M | n/a | |
| 98 | inline: the distance kill (33134) | — | M | n/a | |
| 99 | `AI_099_1_Counterweights` (33153) + `AI_099_2_Yoyos` (33157) | `ai_yoyo` | Pa | behavioural | the counterweights are absent |
| 100 | `AI_100_Medusa` (33161) | — | M | n/a | |
| 101 | inline: alpha lerp + expected type (33165) | — | M | n/a | |
| 102 | inline: the Nebula Eye (33222) | `ai_nebula_eye` | P | — | |
| 103 | inline: the Stardust Mark (33448) | `ai_stardust_mark` | P | — | |
| 104 | inline: scale decay (33478) | — | M | n/a | |
| 105 | `AI_105_SporeSac` (33512) | — | M | n/a | |
| 106 | inline: rotation + the etherian portal (33516) | — | M | n/a | |
| 107 | inline: the etherian spawn counter (33553) | — | M | n/a | |
| 108 | inline: the Vortex ritual timeline + spawns (33679) | `ai_vortex_ritual` **not dispatched** | M | behavioural | VORTEX_LIGHTNING/PORTAL rows |
| 109 | inline: the NPC-target lock (34026) | — | M | n/a | |
| 110 | inline: the town-NPC check (34066) | — | M | n/a | |
| 111 | `AI_111_DryadsWard` (34110) | — | M | n/a | |
| 112 | inline: the dust burst (34114) | — | M | n/a | |
| 113 | `AI_113_TargetSticker` (34364) | — | M | n/a | |
| 114 | inline: the portal sync + light (34368) | — | M | n/a | |
| 115 | inline: the Terrarian beam, light + 0.985 damp (34414) | `ai_yoyo_beam` | P | — | |
| 116 | inline: rotation from ai[1] (34438) | — | M | n/a | |
| 117 | inline: per-type flags (34459) | — | M | n/a | |
| 118 | inline: the alpha ramp + the velocity-4 gate (34638) | — | M | behavioural | NEBULA_ARCANUM row |
| 119 | inline: the expected-type switch (34809) | — | M | behavioural | NEBULA_ARCANUM_SUBSHOT row |
| 120 | `AI_120_StardustGuardian` (34888) | — | M | behavioural | STARDUST_GUARDIAN row |
| 121 | `AI_121_StardustDragon` (34892) | — | M | n/a | |
| 122 | inline: the tower damage bolt (34896) | `ai_tower_damage_bolt` | P | — | |
| 123 | `AI_123_FloatingTurrets` (34966) | — | M | behavioural | MOONLORD_TURRET row |
| 124 | inline: player-relative (34970) | — | M | n/a | |
| 125 | inline: the counter (35697) | — | M | n/a | |
| 126 | inline: the velocity sign (35834) | — | M | n/a | |
| 127 | inline: the countdown (35998) | — | M | n/a | |
| 128 | inline: the colour ramp (36086) | — | M | n/a | |
| 129 | inline: the Spirit Flame (36198) | `ai_spirit_flame` | P | — | |
| 130 | `AI_130_FlameBurstTower` (36310) | — | M | behavioural | the tower rows |
| 131 | `AI_131_FlameBurstShot` (36314) | — | M | n/a | |
| 132 | inline: the localAI gate (36318) | — | M | n/a | |
| 133 | inline: the marker fade (36435) | — | M | behavioural | DARK_MAGE_MARKER row |
| 134 | `AI_134_Ballista` (36499) | — | M | behavioural | the tower rows |
| 135 | `AI_135_OgreStomp` (36503) | — | M | n/a | |
| 136 | `AI_136_BetsyBreath` (36507) | — | M | n/a | |
| 137 | `AI_137_LightningAura` (36511) | — | M | behavioural | the aura rows |
| 138 | `AI_138_ExplosiveTrap` (36515) | — | M | behavioural | the trap rows |
| 139 | `AI_139_ExplosiveTrapExplosion` (36519) | — | M | n/a | |
| 140 | `AI_140_MonkStaffT1` (36523) | — | M | n/a | |
| 141 | `AI_141_MonkStaffT1Explosion` (36527) | — | M | n/a | |
| 142 | `AI_142_MonkStaffT2And3` (36531) | — | M | n/a | |
| 143 | `AI_143_MonkStaffT2Ghast` (36535) | — | M | n/a | |
| 144 | `AI_144_DD2Pet` (36539) | — | M | n/a | |
| 145 | `AI_145_BookStaffStorm` (36543) | — | M | n/a | |
| 146 | `AI_146_DD2Victory` (36547) | — | M | n/a | |
| 147 | `AI_147_Celeb2Rocket` (36551) | — | M | n/a | |
| 148 | `AI_148_StarSpawner` (36555) | — | M | n/a | |
| 149 | `AI_149_GolfBall` (36559) | — | M | n/a | |
| 150 | `AI_150_GolfClubHelper` (36563) | — | M | n/a | |
| 151 | `AI_151_SuperStar` (36567) | — | M | n/a | |
| 152 | `AI_152_SuperStarSlash` (36571) | — | M | n/a | |
| 153 | `AI_153_ToiletPop` (36575) | — | M | n/a | |
| 154 | `AI_154_VoidLens` (36579) | — | M | n/a | |
| 155 | `AI_155_MysticSnakeCoil` (36583) | — | M | n/a | |
| 156 | `AI_156_BatOfLight` (36587) | — | M | n/a | |
| 157 | `AI_157_SharpTears` (36591) | — | M | behavioural | DEERCLOPS_ICE_SPIKE row |
| 158 | `AI_158_BabyBird` (36595) | — | M | behavioural | BABY_BIRD row |
| 159 | `AI_159_PaperAirplanes` (36599) | — | M | n/a | |
| 160 | `AI_160_Kites` (36603) | — | M | n/a | |
| 161 | `AI_161_RapierStabs` (36607) | — | M | n/a | |
| 162 | `AI_162_TigerPounce` (36611) | — | M | n/a | |
| 163 | `AI_163_Chum` (36615) | — | M | n/a | |
| 164 | `AI_164_StormTigerGem` (36619) | — | M | n/a | |
| 165 | `AI_165_Whip` (36623) | `ai_whip` | P | — | |
| 166 | `AI_166_Dove` (36627) | — | M | n/a | |
| 167 | `AI_167_SparkleGuitar` (36631) | — | M | n/a | |
| 168 | `AI_168_FirstFractal` (36635) | — | M | n/a | |
| 169 | `AI_169_Smolstars` (36639) | — | M | n/a | |
| 170 | `AI_170_FairyGlowstick` (36643) | — | M | n/a | |
| 171 | `AI_171_HallowBossRainbowStreak` (36647) | — | M | n/a | |
| 172 | `AI_172_HallowBossRainbowPelletStorm` (36651) | — | M | n/a | |
| 173 | `AI_173_HallowBossRainbowTrail` (36655) | — | M | n/a | |
| 174 | `AI_174_MultisegmentPet` (36659) | — | M | n/a | |
| 175 | `AI_175_TitaniumStormShards` (36663) | — | M | n/a | |
| 176 | `AI_176_EdgyLightning` (36667) | — | M | n/a | |
| 177 | `AI_177_IceWhipSlicer` (36671) | — | M | n/a | |
| 178 | `AI_178_FireExplosion` (36675) | — | M | n/a | |
| 179 | `AI_179_FairyQueenLance` (36679) | — | M | n/a | |
| 180 | `AI_180_FairyQueenSunDance` (36683) | — | M | n/a | |
| 181 | `AI_181_FairyQueenRangedItemShot` (36687) | — | M | n/a | |
| 182 | `AI_182_FinalFractal` (36691) | — | M | n/a | |
| 183 | `AI_183_ZoologistStrike` (36695) | — | M | n/a | |
| 184 | `AI_184_BadTorchLuck` (36699) | — | M | n/a | |
| 185 | `AI_185_LifeDrain` (36703) | — | M | n/a | |
| 186 | `AI_186_PrincessWeapon` (36707) | — | M | n/a | |
| 187 | `AI_187_ShadowHand` (36711) | — | M | behavioural | SHADOW_HAND row |
| 188 | `AI_188_LightsBane` (36715) | — | M | n/a | |
| 189 | `AI_189_Volcano` (36719) | — | M | behavioural | CASCADE_EXPLOSION row |
| 190 | `AI_190_NightsEdge` (36723) | — | M | n/a | |
| 191 | `AI_191_TrueNightsEdge` (36727) | — | M | n/a | |
| 192 | `AI_192_JuminoAnimation` (36731) | — | M | n/a | |
| 193 | `AI_193_Flamethrower` (36735) | — | M | n/a | |
| 194 | `AI_194_HorsemanPumpkin` (36739) | — | M | n/a | |
| 195 | `AI_195_JimsDrone` (36743) | — | M | n/a | |
| 196 | `AI_196_Petal` (36747) | — | M | n/a | |
| 197 | `AI_197_CeilingAndHoverTurret` (36751) | `sentry_npc_targeting` | Pa | behavioural | aim only, no shot |
| 198 | `AI_198_Flint` (36755) | — | M | n/a | |
| 199 | `AI_199_MeteorOre` (36759) | — | M | n/a | |
| 200 | `AI_200_BirdDroppings` (36763) | — | M | n/a | |
| 201 | `AI_201_ThrownMelee` (36767) | — | M | n/a | |
| 202 | `AI_202_TorchGodHelper` (36771) | — | M | n/a | |
| 203 | `AI_203_StormLightning` (36775) | — | M | n/a | |
| 204 | `AI_204_Digtoise` (36779) | — | M | n/a | |
| 205 | `AI_205_RemoteControlCar` (36783) | — | M | n/a | |
| 206 | `AI_206_ForbiddenMinion` (36787) | — | M | n/a | |
| 207 | `AI_207_SnappingStone` (36791) | — | M | n/a | |
| 208 | `AI_208_GlacierFangShot` (36795) | — | M | n/a | |
| 209 | `AI_209_ChlorophyteClaymoreBladeSlam` (36799) | — | M | n/a | |
| 210 | `AI_210_YoyoShots` (36803) | `ai_yoyo_shot` | P | — | |
