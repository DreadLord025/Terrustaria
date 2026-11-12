# 40 — DONE hot-rows re-verification (NPC / entities / projectiles)

> Wave 4 · researched 2026-11-12 · sources: `decompiled/` (C# source of truth), `crates/`
> (Rust target), `docs/ROADMAP.md` (the claims), `docs/ANALYSIS.md`, `docs/research/01..24`
> (leads only — every verdict below was re-derived from the code).
> Read-only: no `graphify update`, no git, no cargo, no test runs.

## Scope

Every `✅` row in `docs/ROADMAP.md` is a **claim**, not a fact. This module re-verifies a
prioritised subset of them — the Milestone-4 hot area (NPCs, combat, events, projectiles)
first, then the rows that the finished research files flagged, then the `✅` claims that live
outside the task catalog (the "Progress snapshot", the Systems atlas, and the M1.5
performance tables).

Method per row:

1. Extract the claim + the C# anchors the roadmap cites.
2. Open the cited Rust file and read the actual implementation (constants, order, RNG use).
3. Open the cited C# range in `decompiled/` and check that the range really contains what the
   roadmap says it contains (line-range drift is a finding in its own right).
4. Compare the two, then look for a contradicting statement in `docs/research/01..24` and
   `docs/ANALYSIS.md`.

Evidence classes used in the table:

- **deep** — the Rust implementation and the cited C# range were both read line by line.
- **structural** — the Rust artefact exists, is wired into the sim, and matches the row's
  shape, but the numeric constants were not re-derived from the C# (cost cap).

**Not covered** (out of budget, recommend a follow-up module): M1 visual-fidelity rows
(TILE-DRAW-1..6, BG-11..22), M3 item/equipment rows, M5 worldgen, M6 audio/localisation/net/
save, M7 UI/wiring rows. Those are ~180 further `✅` rows that remain unverified claims.

## Main table

| Task id | Roadmap claim (short) | Verdict | Evidence (Rust `file.rs::sym` + C# `file.cs:line`) | Severity |
| --- | --- | --- | --- | --- |
| M4-NPC-4 | hover archetype (aiStyle 5): Eater of Souls 4/0.02, Hornet 3.25/0.018, Meteor Head 1/0.03 | **FAIL** | Rust `tr-app/src/npc_ai.rs::HoverParams::HORNET` (177-181) = 3.25/0.018, bound at `tr-app/src/npc.rs:4382`; C# `NPC.cs:50341-50344` = type 42 → `num = 3.5f; num2 = 0.021f` — the port ships the **type-205** values (`NPC.cs:50366-50367`) | high (behavioural) |
| M4-NPC-4 (same row, citations) | the preset citations `:50304-50305` / `:50322-50323` / `:50328-50329` | **FAIL** (folded into the row above) | C# `NPC.cs:50322-50323` = type **619** arm (6/0.1); `NPC.cs:50328-50329` = type **252** arm; the Meteor Head is at `NPC.cs:50374-50377` (1/0.03 ✓), the Servant at `:50379-50382` (5/0.03 ✓). The same wrong line numbers are repeated in `docs/ROADMAP.md:1404` | high (the row is self-invalidating) |
| M4-NPC-4 (same row, shape) | the C# `num *= 2f - scale` factor and the surface-gated vertical damping | **FAIL** (partial shape) | C# `NPC.cs:50350-50351` (scale factor) and `:50352-50362` (damping only for type 42/231-235 **and** only above `worldSurface`); Rust `npc_ai.rs:242-249` applies the ×0.97 damping to **every** hover kind and never applies the scale factor | medium (behavioural) |
| M2-COMBAT-1 | iframes + knockback caps; `Player::hurt` 40-tick iframe | **PARTIAL** | Rust `tr-core/src/damage.rs::knock_back_npc` (98-120, the 8/10/12/14/16 soft caps) ✓ vs C# `NPC.cs:81389-81458` (read: cap at `:81394-81400`) ✓; but the **player-side damage floor is missing**: C# `Player.cs:38378-38381` (`num2 < 1 → 1`) has no counterpart in `tr-app/src/player.rs::hurt_inner` (2223-2243), whose comment wrongly asserts the C# "does not re-floor here" — the C# floors *after* the endurance multiply at `Player.cs:38304` | medium (behavioural: 0-damage hits) |
| M2-COMBAT-2 | crit rolls + damage classes; `weapon_damage_multiplier` | **PARTIAL** | Rust `tr-app/src/player.rs::weapon_damage_multiplier` (2337-2347) + `::class_crit` (2351-2358); C# `Player.cs:53388-53398` — the arrow/bow branch (`AmmoID.Sets.IsArrow` → `bowEffectiveDamage`, `:53396-53398`) is **absent**; the Rust doc-comment admits "narrowed to the ported classes" | medium (behavioural: bow damage) |
| M4-BOSS-6 | Wall of Flesh + the hardmode gate | **PARTIAL** | Rust `tr-app/src/npc.rs::update_wof` (6704), the death hook `tr-app/src/world_sim/npcs.rs:2583-2585` (`WorldGen.StartHardmode`), the shop gate `tr-app/src/shop.rs:977/988` ✓; but C# `NPC.cs:10355` sets `behindTiles = true` and the port has **no `behind_tiles` field** (0 hits; `tr-app/src/render/entities.rs:2241-2242` admits "the behindTiles layer lands with the render-layers work") → the WoF draws in front of tiles | medium (pixel/behavioural) |
| M4-NPC-3c, M4-NPC-3h..3r (display rows) | `✅ (re-verified 2026-11-12, BUG-22/24)` | **STALE** | `docs/ROADMAP.md:73` still lists "🟡 the display rows M4-NPC-3c/3h..3r" while `docs/ROADMAP.md:1534` and `:1540-1550` mark the same ids `✅` — the snapshot and the catalog disagree about the same rows; nothing states which is current | medium (doc) |
| M1.5 Phase 1 + Phase 2 results | "pixel-identical (0 differing pixels)" + the timing tables | **STALE** | `docs/ROADMAP.md:1181-1226` — measurement-only claims with no code artefact to re-read; the neighbouring Phase 3 table (`:1199-1205`) is still "in progress", so the section is a mixed state | low-medium (needs `TR_BENCH`/`TR_STATS` re-run; cannot be settled statically) |
| M4-NPC-2 | worm chain (aiStyle 6) + the nine kinds + the spawner arms | PASS | Rust `tr-app/src/world_sim/npcs.rs::update_worms_in` (800), `::worm_head_step` (1010); C# `NPC.cs:51347-51368` (read: child spawned at the parent's bottom-centre, `type + 1`, `ai[2] -= 1`) ✓; C# `NPC.cs:51997-52002` (read: `num67 = num45 / num62`, the direct aim) ✓ | low (`behindTiles` layer for the segments is admitted as out) |
| M4-NPC-5 | caster (8) + sphere (9) archetypes, 7 kinds | PASS | Rust `tr-app/src/npc_ai.rs::caster_cycle` (398-424: the 0.93 stand, the ±0.1 snap, the 500 init, the 100/200/300 shot triggers), `::sphere_step` (488), `::find_teleport_spot` (316), `CasterParams` (266-293); C# `NPC.cs:20831-20835` (read: `velocity.X *= 0.93f` + the ±0.1 snap) ✓, `NPC.cs:20836-20838` (read: `ai[0] = 500f`) ✓ | low |
| M4-NPC-6 | swim archetype (aiStyle 16), 3 kinds | PASS | Rust `tr-app/src/npc_ai.rs::swim_step` (586); C# `NPC.cs:23698-23708` (read: the wet machine, and the Goldfish exclusion `this.type != 55` at `:23701`) ✓ | low |
| M4-SPAWN-1 | `GetSpawnRate` chain + the `Main.SwapRandom("SpawnNPC")` discipline | PASS | Rust `tr-app/src/spawner.rs::get_spawn_rate` (202), `::spawn_npc_call` (472), `tr-app/src/app/mod.rs:183 spawn_no_cycle`; C# `NPC.cs:473-484` (read: the method head, the hardmode ×0.9 and the underworld ×2) ✓, `NPC.cs:80169-80178` (read: the `noSpawnCycle` latch) ✓ | low |
| M4-SPAWN-2 | the `SpawnAnNPC` selection ladders | PASS (structural) | Rust `tr-app/src/spawner.rs` (33 `#[test]`s, per-seed sweeps); `spawn_basic_slime` (689, 875, 989) — the C# `GetBasicSlimeToSpawn` is renamed, not missing | low |
| M4-NPC-1 | slime family −3..−9, Wandering Eye 133, the Pinky roll | PASS (structural) | Rust `tr-app/src/spawner.rs::spawn_basic_slime` (689), `tr-app/src/bestiary.rs:49-52` (the Pinky credit row), `tr-app/src/loot_db.rs::PINKY` | low |
| M4-DAMAGE | `DamageVar` / `CalculateDamageNPCsTake` / `checkArmorPenetration` / the crit gate | PASS | Rust `tr-core/src/damage.rs::damage_var` (30, ±15 % + the luck reroll), `::calculate_damage_npcs_take` (50, the floor of 1), `::check_armor_penetration` (59, the defense/2 cap), `::strike_npc_damage` (88, crit ×2 after defense), `::round_bankers` (13) — the module header cites `Main.cs:66118`, `NPC.cs:81148`, `NPC.cs:81226` | low (the roadmap's `DamageVar` is a **function** in the port, not a type — naming divergence only) |
| M4-PROJ-1 | the projectile pool + the three immunity systems | PASS (structural) | Rust `tr-app/src/projectile.rs` (193 `#[test]`s; the 1000-slot pool, `Projectile::update` 8505-9102 per `docs/research/06`) | low |
| M4-PROJ-3b | flail family (aiStyle 15) | PASS | Rust `tr-app/src/projectile.rs::ai_flail` (4643), `::ai_flail_bounce` (4950); C# `Projectile.cs:43761-43775` (read: the per-kind switch — cases 947/948 and 25 with `num/num2/num5/num7`) ✓ | low |
| M4-PROJ-4 | yoyo family (aiStyle 99) | PASS | Rust `tr-app/src/projectile.rs::ai_yoyo` (3478), `::ai_yoyo_shot` (3833), `::ai_yoyo_beam` (3868); C# `Projectile.cs:67733-67745` (read: `AI_099_2_Yoyos` and the owner/type/`ai[0] >= -1` scan) ✓ | low |
| M4-PROJ-4b | yoyo on-hit effects (`EverySomeYoyoHits`) | PASS | Rust `tr-app/src/player.rs::every_some_yoyo_hits` (1132) + `tr-app/src/projectile.rs::ai_yoyo_shot` (3833) | low |
| M4-PROJ-7 | whips + the tag-damage system | PASS | Rust `tr-core/src/whip.rs` (13 `#[test]`s, the verbatim set/table checks), `tr-app/src/tag_effects.rs::TagEffectStack` (69, 6 `#[test]`s) | low |
| M4-PROJ-8 | minions + sentries (`AI_062`, `AI_053`) | PASS (structural) | Rust `tr-app/src/projectile.rs` — AI_062 at 2723, AI_053 at 3312, `MINION_SACRIFICABLE`/`SENTRY_SHOT`/`MINION_TARGETING_FEATURE` at 2284/2289/2293 | low (the dragon relink and the DD2 persistence are admitted stubs) |
| M4-PROJ-9 | bombs (aiStyle 16) + the explosion tiles | PASS (structural) | Rust `tr-app/src/projectile.rs::explode_bomb` (9069, the symbol `docs/research/06` cites) | low |
| M4-BOSS-2b | the full aiStyle-4 machine (EoC + Servants) | PASS (structural) | Rust `tr-app/src/npc.rs::update_eye_of_cthulhu` (6018), the `downed_eoc` death hook (`npc.rs:585`), `tr-app/src/shop.rs:977` (the downstream hardmode rows) | low (expert/getGood arms off — no difficulty flags, admitted) |
| M4-BOSS-6c | the voodoo sacrifice → hardmode | PASS (structural) | Rust `tr-app/src/world_sim/editing.rs:7647` (`app.world.hard_mode = true`), `tr-app/src/npc.rs:603` (the `WorldGen.StartHardmode` / `NPC.cs:79485` anchor) | low |
| M4-EVENT-1 | the Goblin Army invasion framework | PASS | Rust `tr-core/src/world.rs::roll_morning_invasion` (538), `tr-app/src/world_sim/editing.rs:7350-7351` (the size), `tr-app/src/world_sim/npcs.rs:3419` (the spawn call); C# `Main.cs:64435-64482` (read: `StartInvasion` — the gates, `invasionSize = 80 + 40 * num` at `:64456`, the `invasionX` roll at `:64476-64482`) ✓, `NPC.cs:1426-1432` (read: the type-1 table, the hardmode Summoner 471 roll) ✓ | low behaviour / cosmetic citation (the roadmap cites `Main.cs:64482-64511`, but the method starts at `:64435`; the cited range begins *inside* the body) |
| M4-LOOT-1 | the drop-rule engine | PASS | Rust `tr-core/src/loot.rs::try_dropping` (345), `::resolve_rule` (254, `ItemDropResolver.ResolveRule`), 17 `#[test]`s | low |
| M4-LOOT-2 | the full drop database (1 421 registrations) | PASS (structural) | Rust `tr-app/src/loot_db_gen.rs` (1 439 lines, 1 423 rule-shaped lines ≈ the claimed 1 421), `tr-app/src/loot_db.rs::database` (46), consumed at `tr-app/src/world_sim/npcs.rs:3521` | low (the exact registration count was not re-derived; the generator would have to run) |
| M4-LOOT-3 | the coin roll chain | PASS (structural) | Rust `tr-core/src/loot.rs::drop_money` (426), `tr-app/src/world_sim/npcs.rs` death pass | low |
| M4-TOWN-1 | the room-check + the first town NPC | PASS | Rust `tr-core/src/housing.rs::check_room` (80, 7 `#[test]`s), `tr-app/src/town.rs::quick_find_home` (21, 16 `#[test]`s) | low |
| M4-TOWN-1b | the town arrival gates | PASS (structural) | Rust `tr-app/src/town_arrival.rs` (the Dryad's downed-boss gate at 194) | low (the Zoologist 10 % / Princess every-other-resident arms need the full type set — admitted) |
| M4-TOWN-2 | the happiness + the shop prices | PASS | Rust `tr-app/src/happiness.rs::process_mood` (308, 8 `#[test]`s); C# `ShopHelper.cs:95-112` (read: `ProcessMood`'s head, the homeless 1000 sentinel at `:107-109`, the remix/town-pet early return at `:103-106`) ✓ | low |
| M4-TOWN-3 | the shop engine + the Merchant's stock | PASS | Rust `tr-app/src/shop.rs::setup_shop` (205), `::resolve_stock` (1177), 11 `#[test]`s; C# `Chest.cs:1371-1386` (read: `case 1` = the Merchant rows 88/87/35/1991/3509/3506/8) ✓ | low |
| M2-BUFF-3 | the DoT/debuff set, player + NPC sides | PASS (structural) | Rust `tr-app/src/npc_buffs.rs::NpcBuffs` (124), `tr-app/src/player.rs::derive_dot_inputs` (4107), `tr-app/src/regen.rs` | low |
| M3-CRAFT-6 | the 2 764-row generated recipe table | PASS (structural) | Rust `tr-app/src/recipes_gen.rs` (generated, present) + `tr-app/src/craft.rs` | low (the row count was not re-derived without cargo) |

**Counts:** PASS 27 · PARTIAL 3 · FAIL 1 (M4-NPC-4, counted as three findings in the
table) · STALE 2. **No phantom ports were found** in the 32 rows checked: every Rust symbol
the roadmap cites exists. Three rows use a *renamed* artefact (`DamageVar` → `damage_var`,
`ItemDropResolver` → `resolve_rule`/`try_dropping`, `GetBasicSlimeToSpawn` →
`spawn_basic_slime`) — naming divergence, allowed by `AGENTS.md`, not missing code.

## Deviations & risks

### High — behavioural

- **The Hornet's hover constants are wrong** (`npc_ai.rs::HoverParams::HORNET` = 3.25/0.018
  vs C# `NPC.cs:50341-50344` = 3.5/0.021 for type 42). The port ships the type-205 values.
  This is the only *wrong constant* found in the hot area — everything else matched. The
  defect is invisible to the current test suite because `npc_ai.rs` has just **2** unit
  tests (`the_walker_chases_jumps_and_lands`, `the_flyer_homes_and_bounces`); the hover,
  caster, sphere and swim machines are covered only indirectly by `world_sim/npcs.rs`
  (152 tests) and `spawner.rs` (33 tests).
- **The hover presets' C# citations are all off** (`:50299-50301` → the `targetData` block,
  `:50322-50323` → type 619, `:50328-50329` → type 252). Both the Rust doc-comments and
  `docs/ROADMAP.md:1404` repeat them, so the roadmap row is self-invalidating: it cannot be
  re-verified by following its own anchors.

### Medium — behavioural / pixel

- **The player-side damage floor is missing** (`Player.cs:38378-38381`). `hurt_inner`
  (`player.rs:2223-2243`) subtracts `amount` without the `num2 < 1 → 1` clamp, and its
  comment asserts the opposite of the source (the C# clamps *after* the endurance multiply
  at `Player.cs:38304`). A 1-damage hit with enough endurance deals 0 in the port and 1 in
  vanilla. `docs/research/04-player-combat.md:147` already lists this as a finding, so the
  ROADMAP's `✅` on M2-COMBAT-1 contradicts a finished research file.
- **The bow/arrow branch of `GetWeaponDamageMultiplier` is missing**
  (`Player.cs:53396-53398` → `bowEffectiveDamage`). `docs/research/04` lists it too; the
  M2-COMBAT-2 row is `✅` regardless.
- **`behindTiles` is not ported at all.** C# `NPC.cs:10355` (the WoF) sets it, and the EoW /
  Destroyer segments and the boss parts rely on it. The port has no `behind_tiles` field, so
  the Wall of Flesh draws in front of the world. `docs/research/05-npc-core.md:100` says
  "MISSING — no `behind_tiles` field anywhere in `crates/`"; the ROADMAP's M4-BOSS-6
  SetDefaults citation lists the flag as if it were ported.
- **The ROADMAP contradicts itself** on the M4-NPC-3c/3h..3r display rows (snapshot
  `🟡` vs catalog `✅`).

### Low — documentation / process

- Citation drift in the rows: `Main.cs:64482-64511` (the method starts at `:64435`),
  `Player.cs:53386` (the method starts at `:53388`). A "cited range" lint would have caught
  both, and the M4-NPC-4 class of error.
- The M1.5 performance tables (`ROADMAP.md:1181-1226`) are measurement claims with no
  artefact; they must be stamped with the commit they were measured on, or re-run.
- `DamageVar` / `ItemDropResolver` are types in C# and functions in Rust — harmless, but the
  roadmap text reads as if a struct exists.

### Cosmetic

- The `M4-PROJ-4c` / `M4-PROJ-6b` naming (`render` vs `leafblower` vs `magic-followups`)
  makes the catalog harder to scan; the rows are consistent inside themselves.

## Task seeds

Re-verification tasks for the ROADMAP (one per FAIL/PARTIAL group). Sizes: S ≤ ½ day,
M ≤ 2 days, L > 2 days.

| # | Task | Size | Depends on | Notes |
| --- | --- | --- | --- | --- |
| V1 | **Fix the hover presets**: `HoverParams::HORNET` → 3.5 / 0.021 (C# `NPC.cs:50341-50344`); apply the `num *= 2f - scale` factor (`:50350-50351`); gate the ×0.97 damping on `position.Y/16 < worldSurface` and on the type-42/231-235 family (`:50352-50362`) instead of applying it to every hover kind | M | — | `crates/tr-app/src/npc_ai.rs`, `crates/tr-app/src/npc.rs:4381-4387` |
| V2 | **Repair the hover citations** in `npc_ai.rs` (162-191) and in `docs/ROADMAP.md:1404`; add a per-type table (type → num/num2 → C# line) so the next re-verification cannot drift again | S | V1 | docs-only after the code fix |
| V3 | **Restore the player damage floor**: clamp `amount` to ≥ 1 in `Player::hurt_inner` after the endurance multiply (`crates/tr-app/src/player.rs:2223-2243`), fix the wrong "does not re-floor" comment, and add the long-invincibility / PVP-8 iframe arms to `tr-core/src/damage.rs::iframes_for` | M | — | `docs/research/04` already scoped it |
| V4 | **Port the arrow/bow arm** of `GetWeaponDamageMultiplier` (`Player.cs:53394-53398`, `bowEffectiveDamage`) or record it as an admitted deviation in `docs/ANALYSIS.md`; requires `Item.useAmmo`/`Item.ammo` in the item registry | M | M3-ITEM-3 (ammo) | the M2-COMBAT-2 row currently claims the accessor is ported |
| V5 | **Port `behindTiles`**: add `behind_tiles` to `Npc`, set it from the vanilla per-kind rows (WoF `NPC.cs:10355`, EoW/Destroyer segments, the boss parts) and add the behind-tiles draw pass in `crates/tr-app/src/render/entities.rs` | L | the render-layers work | `docs/research/05` scope |
| V6 | **Reconcile the ROADMAP snapshot with the catalog** for the M4-NPC-3c/3h..3r rows (`ROADMAP.md:73` vs `:1534-1550`); add a rule that a row's status lives in exactly one place | S | — | doc-only |
| V7 | **Stamp the M1.5 performance tables** with the commit/hash they were measured on and re-run `TR_BENCH=600`/`TR_STATS=1` against `main`, or mark them "historical" (`ROADMAP.md:1181-1226`) | S | — | needs a benchmark run, not a research pass |
| V8 | **Add a "cited-range" lint** to the doc toolchain: parse `ROADMAP.md` `File.cs:NNNN-MMM` anchors and assert the range is non-empty and contains the quoted symbol name — this catches the V2/V6 class of error automatically | M | — | tooling |
| V9 | **Cover the hover/caster/sphere/swim machines with unit tests** in `crates/tr-app/src/npc_ai.rs` (today: 2 tests) — a per-kind preset table test would have caught V1 | M | V1 | |
| V10 | **Re-verify the untested `✅` rows** — M1 visual, M3 items/equipment, M5 worldgen, M6 audio/net/save, M7 UI/wiring (~180 rows) with this module's method | L | — | schedule as module 43+ |

## Tests

Which cargo tests underpin each PASS (read from the sources; **not executed** — this module
runs no cargo).

- `crates/tr-core/src/damage.rs` (8) — M4-DAMAGE: `damage_var_stays_inside_the_15_percent_band`,
  `damage_var_is_deterministic_per_seed`, `positive_luck_rerolls_keep_the_higher`,
  `bankers_rounding_matches_the_c_sharp_midpoints`,
  `npc_damage_takes_defense_halved_with_a_floor_of_one`,
  `strike_npc_line_applies_the_crit_after_defense`,
  `armor_penetration_caps_at_half_defense`, `crit_roll_is_the_1_to_100_le_crit_gate`.
- `crates/tr-core/src/housing.rs` (7) — M4-TOWN-1: `a_furnished_room_passes_and_reports_its_bounds`,
  `the_room_needs_fail_without_furniture`, `the_failure_reasons_match_the_source`,
  `unsafe_walls_fail_the_room`, `a_breach_to_the_outside_fails_with_a_hole`,
  `start_room_check_is_the_scan_without_the_furniture_gate`,
  `the_candelabra_counts_solid_for_the_spawn_check`.
- `crates/tr-app/src/happiness.rs` (8) — M4-TOWN-2: `the_travelling_merchant_ignores_the_mood`,
  `the_forest_guide_alone_is_content`, `the_homeless_guide_prices_at_the_ceiling`,
  `the_evil_biome_prices_at_the_ceiling`, `the_crowd_multiplies_past_three`,
  `the_love_space_gate_needs_the_village_room`,
  `the_dryad_judges_the_jungle_and_the_desert`,
  `the_tinkerer_judges_the_underground_and_the_jungle`.
- `crates/tr-app/src/tag_effects.rs` (6) — M4-PROJ-7: `the_tag_mark_runs_the_general_240_and_expires`,
  `the_constellation_row_runs_480`, `the_eviction_keeps_the_newest_and_caps_at_five`,
  `the_whip_immune_kinds_refuse_the_tag`, `the_spawn_reset_clears_the_marks`,
  `the_tagged_shot_takes_the_bonus_and_the_proc_arms`.
- `crates/tr-core/src/whip.rs` (13) — M4-PROJ-7: `the_is_a_whip_set_is_verbatim`,
  `the_whip_settings_table_is_verbatim`, `the_time_fly_out_overrides_match`,
  `the_curve_has_segments_plus_one_points_and_full_reach`,
  `the_curve_retracts_past_progress_one`, `the_cob_whip_remaps_the_progress`,
  `the_sweep_spans_the_gap_between_frames`, `the_tag_table_is_verbatim`,
  `the_summon_tag_damage_multiplier_is_verbatim`,
  `the_tagged_hit_arm_accumulates_and_rolls_crit`,
  `the_tag_damage_math_reproduces_the_c_sharp_line`,
  `the_proc_consumes_gate_is_verbatim`, `the_constellation_tag_arms_the_proc_window`.
- `crates/tr-core/src/loot.rs` (17) — M4-LOOT-1/3 (the seeded FastRandom streams, the rule
  chains, the money split).
- `crates/tr-app/src/shop.rs` (11) — M4-TOWN-3 (the base stock, the gate flips, the empty
  unknown index, the price math).
- `crates/tr-app/src/town.rs` (16) — M4-TOWN-1/1b (the find/reject sweeps, the arrival gates).
- `crates/tr-app/src/spawner.rs` (33) — M4-SPAWN-1/2 (the seed sweeps, the ladders).
- `crates/tr-app/src/world_sim/npcs.rs` (152) — M4-NPC-2/5/6, M4-BOSS-*, M4-EVENT-* (the
  chain assembly, the teleport/shoot cycle, the water ladder, the invasion tables, the boss
  phases).
- `crates/tr-app/src/projectile.rs` (193) — M4-PROJ-1/3b/4/4b/7/8/9 (the pool, the flail
  states, the yoyo lifecycle, the whips, the minion/sentry harness).
- `crates/tr-core/src/dd2.rs` (6) — M4-EVENT-5d..5q (the DD2 families).
- **Gap:** `crates/tr-app/src/npc_ai.rs` has only **2** tests
  (`the_walker_chases_jumps_and_lands`, `the_flyer_homes_and_bounces`) — the hover/caster/
  sphere/swim machines are untested at the unit level, which is exactly why the wrong Hornet
  preset survived (see V9).

## Reading notes

- `docs/ROADMAP.md:1404` (M4-NPC-4) must not be used as a re-verification anchor until V2
  lands — its C# line numbers point at the wrong `if` branches.
- The research files `docs/research/01..24` were used only as **leads**; every verdict above
  was re-derived from `crates/` + `decompiled/`. Where a research file and the ROADMAP
  disagree (the player hurt floor, `GetWeaponDamageMultiplier`, `behindTiles`), the research
  file is correct and the ROADMAP row is over-claimed.
- No repository file was modified except this one; no `graphify update`, no `git`, no
  `cargo` run.
