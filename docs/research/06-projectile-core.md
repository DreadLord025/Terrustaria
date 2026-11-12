# 06 — Projectile core

> Wave 1 · researched 2026-11-12 · sources: decompiled + crates + ROADMAP/ANALYSIS/PORT_MAP

## Scope

The C# side is one god file: `decompiled/Terraria/Projectile.cs` (~80 000 lines) — the
`Projectile` entity, `SetDefaults`, `NewProjectile`/`NewProjectileSetup`, `Update`, `AI()`
dispatch, `Damage()` and `Kill()`. This module covers the *core* only: the defaults
protocol, the update pass order, the aiStyle dispatch table, the spawn protocol and the
damage pass. Per-family aiStyle behaviour is Wave 3 (`26-projectile-ai-audit.md`).

The Rust side is two files:

- `crates/tr-app/src/projectile.rs` — 17 941 lines: `proj_data` (the `SetDefaults` slice,
  ~212 kinds), `Projectile` (the slot), `ProjectilePool` (`update`, `spawn*`, `kill`, the
  ~45 ported `ai_*` arms, `damage_pve`), the draw helpers, and 193 unit tests.
- `crates/tr-app/src/world_sim/projectiles.rs` — 318 lines: `update_projectiles` (the
  fixed-step driver inside `App`'s 60 Hz pass) + 2 summon-fixture tests.

`crates/tr-core/src/whip.rs` (13 tests) carries the whip geometry the PVE sweep reuses.

## Key entities

- `Terraria/Projectile.cs::Projectile.SetDefaults` (697-~10580) — the reset block
  (697-806) plus the if/else per-type chain: 1475 `else if (this.type == N)` branches in
  the file, tail helpers `DefaultToFlail` (1085), `DefaultToSpear` (1300),
  `DefaultToDrillOrChainsaw` (1398), `UseGroupStaticImmunity` (1585),
  `ApplyGemStaffStats` (2112), `DefaultToYoyo` (6018), `DefaultToKite` (7891),
  `DefaultToShortsword` (7923), `DefaultToWhip` (8143), `DefaultToSpray` (9635).
- `Terraria/Projectile.cs::Projectile.NewProjectile` (10803 / 10862) and
  `NewProjectileSetup` (10823-10860) — first-free slot, `FindOldestProjectile` (10808),
  `slotGenerations` / `ProjectileKey` (10852-10858).
- `Terraria/Projectile.cs::Projectile.Update(int i)` (16668-17245) — the pass order.
- `Terraria/Projectile.cs::Projectile.AI()` (23199-36815) — the aiStyle if/else chain,
  arms for aiStyle 1..210 (23201, 23690, 24099, 24228, 25307, 36783, 36803).
- `Terraria/Projectile.cs::Projectile.Damage()` (12487-12515) → `Damage_PVE_Inner`
  (12598), `Damage_PVP`, `Damage_EVP`; the helper grids `Damage_CanDealDamage` (12448).
- `Terraria/Projectile.cs::Projectile.Kill()` (70642-~72200) — `PrepareBombToBlow`
  (46770), `Kill_ExplodeTiles` (77265), `Resize` (22353).
- `crates/tr-app/src/projectile.rs::proj_data` (1046) + `ProjData` (646-689) — the
  hand-ported `SetDefaults` table; `RESET_STATE` (699) is the reset triple.
- `::ProjectilePool::spawn` (7808), `::spawn_with_data_ai_owned` (8128),
  `::set_immunity` (7899), `::spawn_hook` (7727), `::spawn_falling` (7921).
- `::ProjectilePool::update` (8505-9102) — the whole pass; the `match data.ai_style`
  dispatch is a single site (8617).
- `::ProjectilePool::kill` (7716), `::kill_remote` (7632), `::update_max_turrets` (7574),
  `::summon_tick_block` (the `:16704-16740` block), `::damage_pve` (9460),
  `::damage_pve_boxed` (9658), `::damage_player` (9730),
  `::find_target_with_line_of_sight` (7176).
- `crates/tr-app/src/projectile.rs::MAX_PROJECTILES` (566) = the 1000-slot cap.
- `crates/tr-app/src/world_sim/projectiles.rs::update_projectiles` (13-204).

## Dependencies

- Calls into: `tr_core::falling` (tile-fall), `tr_core::whip`,
  `tr_core::edit` (the Ice Rod tile writes), `crate::grapple`, `crate::registry`
  (item rows), `crate::player::Player`, `crate::npc::Npc` (`hurt_from_projectile`),
  `tr_core::loot::DropRng` (the shared item stream), `tr_core::loot`-based `LootRng`.
- Called by: `world_sim/projectiles.rs::update_projectiles` ← `App`'s fixed step; the
  shoot path (`world_sim/editing.rs`, `Player::ItemCheck_Shoot` analogue); `wiring.rs`
  (`DeviceShot` drains, M7-WIRE-2); `npcs.rs` hostile-shot requests; `render/entities.rs`
  + `frame.rs` (`Projectile::draw`).
- Shared data structures: `Projectile` slot (2426), `local_immunity: Vec<i16>` (256 per
  slot), the ID-static immunity map (`update_count` keyed), `YoyoInputs`/`FlailInputs`
  snapshots, `hook_reg: [i16; 10]`/`hook_grap_count`, `WhipSettings`.

## Port status

| C# subsystem | Status | Rust location | Evidence / notes |
| --- | --- | --- | --- |
| `SetDefaults` reset block (697-806) | PARTIAL | `projectile.rs::RESET_STATE` (699), `ProjData` (646) | The reset is *table-driven*, not a per-slot mutation: `spawn_with_data_ai_owned` (8128-8192) writes the row into a fresh slot. Only 13 of the ~35 reset fields exist (`light`, `hide`, `drawLayer`, `glowMask`, `ownerHitCheckDistance`, `hostileDamageScaling`, `armorPenetration`, `bonusCritChance`, `netImportant` bookkeeping, `wet`/`honeyWet`/`shimmerWet`, `gfxOffY`, `stepSpeed`, `minionPos` partial, `numHits`/`restrikeDelay`/`soundDelay`) are absent — no `light` field anywhere in the slot struct (2426-2540). |
| `SetDefaults` per-type chain | PARTIAL | `proj_data` (1046-~2400) | ~212 rows hand-ported with citations vs the C#'s 1475 typed branches; `test_dw_1_1_every_row_carries_the_set_defaults_reset` (14069) pins the reset contract. Rowless kinds spawn nothing (`test_dw_1_2_an_unported_kind_spawns_nothing`, 14190) — a documented, deliberate gap, not a silent fallback. |
| `NewProjectile` / `NewProjectileSetup` (10803-10960) | PARTIAL | `::spawn` (7808), `::spawn_with_data_ai_owned` (8128) | First-free matches (`spawn_takes_the_first_free_slot_and_recycles_the_dead`, 10004). **Divergence:** the data path declines (`slot?`, 8149) when the pool is full, while C# falls back to `FindOldestProjectile` (10808) — only the *plain* `spawn` recycles (`full_pool_recycles_the_lowest_time_left`, 10018), and it ignores `netImportant`, so it can evict a net-important shot. `DebugOptions.Shared_RandomizeProjectileSlots`, the golf-ball 10-cap (10934-10955) and `CopiesOwnerAttackCDToLocalImmunityOnSpawn` (10956-10968) are absent. |
| Stable ids / free list | PARTIAL | `generation` (2434), `::kill_remote` (7632) | `generation` is bumped per spawn into a slot (the `ProjectileKey.generation` analogue, `DataStructures/ProjectileKey.cs:16-31`), but there is no `keyToIndex` map, no spawner field, and no `slotGenerations` array — a wire id cannot be resolved back to a slot. |
| `Update` pass order (16668-17245) | PARTIAL | `::update` (8505-9102) | `DecrementLocalImmuneTimeCounters` (8536-8548, positive-only decay kept), out-of-world guard (8551-8578, aiStyle-3 skip + minion rescue), minion/sentry block (8582-8584), the wind tail (9028-9034). Missing: `UpdateEnchantmentVisuals` (16700), the `gfxOffY` step (16740-16764), `soundDelay`, the water/lava `wet` recompute (16790+), `CutTiles` (12502), `Collision.SwitchTiles` (17087), `RecheckSectionsForSkippedUpdates` (17240), the `netUpdate`/`netSpam` block (17210-17238). |
| AI sub-step count `1 + extraUpdates` | PASS | `sub_steps` (8549), test 10224 | `extra_updates_runs_the_ai_twice_per_tick` (10224) pins the 2× 22 px = 44 px step. |
| `AI()` dispatch (23201-36803) | PARTIAL | `match data.ai_style` (8617) | 32 unique arms — 1,3,7,8,9,14,15,16,18,22,24,28,29,40,47,48,50,53,62,82,83,84,85,99,102,103,115,122,129,165,197,210 of the C#'s 210 (`:23201`..`:36803`); `_ => {}` (9026) silences the rest. **Defect:** aiStyles 1, 3, 14, 99 and 115 each appear **twice** (8624/8728, 8631/8729, 8695/8730, 8777/8983, 8732/8925) — the second copies are unreachable dead arms (the 99/15 arms are byte-duplicated bodies). |
| `timeLeft` / `penetrate` bookkeeping (17199-17208) | PARTIAL | 9079-9090 | `timeLeft--` and `penetrate == 0` → deactivate match, but both run **once per `update`** while C# runs them **inside the sub-step loop** (17201/17206 are within `while (numUpdates >= 0)`, 16676). A kind with `extraUpdates = 1` therefore burns its `timeLeft` at half the vanilla rate. Pinned by `time_left_expires_the_slot` (10198) with the ported behaviour. |
| `Damage()` → PVE walk | PARTIAL | `::damage_pve` (9460) | Order, immunity systems (12448/12562-12624/14087-14119) and the tag/yoyo/whip on-hit arms are ported. The hit test is `damage_hitbox` (`Damage_GetHitbox()`, `:14959-15023`) ∩ `npc.getRect()` (`:12654`) — a **rectangle** overlap, not a centre point (S1-C-8); `damage_pve_boxed` still serves the resized gem/bomb bursts. The full `Colliding` per-type overlay arms (gem-staff `oldPos` sweep, types 698/973/985/…, `Collision.CanHit`) remain unported. |
| `Damage_PVP` / hostile contact | PARTIAL | `::damage_player` (9730) | Box overlap → `Player.hurt` + the shot dies (M4-PROJ-1a). No `Damage_EVP`, no PVP player-vs-player, no net-mode branch. |
| `Kill()` | PARTIAL | `::kill` (7716) | Deactivate + `time_left = 0` only. The bomb half rides `::explode_bomb` (9069) and the Inferno/gem death arms ride 9094-9100. Not ported: `TryCancelChannel` / `DontCancelChannelOnKill` (70652-70659), `SelfHurtPlayers` (70669), the non-bomb `Kill_*` arms, gore/dust/sound, `Damage()`'s recursive call at 70796+ for the remaining types. |
| Owner / minion ownership | PASS | `owner: u8` (2430), the sim gate (8531-8535) | `owner == local_owner \|\| (owner == 255 && hostile_local)` mirrors `Update`'s `this.owner != Main.myPlayer` skip (17210) and `NewProjectile`'s 255 default (10864-10866); the minion rescue (8568-8573) mirrors 16684-16695. |
| `oldPos` / `oldRot` / `oldSpriteDirection` | MISSING | — | `SetDefaults` sizes them from `ProjectileID.Sets.TrailCacheLength` (718-735) and `Update` shifts them (17170-17195). No field exists in the slot; no trailing-mode-5 arm. |
| Homing / target acquisition | PARTIAL | `::find_target_with_line_of_sight` (7176), `::host_by_kind` (4362), `::sentry_npc_targeting` (3319) | The gem/magnet/sentry families have their own retarget arms; a shared toolbelt does not exist and `Damage()`'s per-style homing (12662, 13236) is not factored out. |
| `Projectile.light` emission | MISSING | — | The vanilla's per-shot light (`SetDefaults`' `light = 0.35f` etc., 853/856/944) never reaches the renderer; `draw` takes `&LightMap` for tinting only (6839). |
| Net sync (`netUpdate`, `netSpam`, 27/29) | MISSING | — | Out of scope per the module doc; `kill_remote` (7632) is the only wire-shaped entry point. |

## Tests

`crates/tr-app/src/projectile.rs` — 193 `#[test]`s. Core-relevant names (module groups):

- Pool engine / M4-PROJ-1: `spawn_takes_the_first_free_slot_and_recycles_the_dead`
  (10004), `full_pool_recycles_the_lowest_time_left` (10018),
  `single_hit_shuriken_dies_without_touching_immunity` (10034),
  `local_immunity_gates_rehits_for_the_cooldown` (10074),
  `default_penetrating_hits_use_the_owner_immunity_of_10` (10113),
  `id_static_immunity_is_shared_across_slots_of_the_same_kind` (10151),
  `time_left_expires_the_slot` (10198), `extra_updates_runs_the_ai_twice_per_tick`
  (10224), `the_falling_block_*` (10280-10338), `the_summon_fixture_*`
  (`world_sim/projectiles.rs` 229/288).
- Defaults/data rows: `the_family_data_map_is_verbatim` (10407),
  `spawn_with_data_applies_the_family_slice` (10446), `the_arrow_data_is_verbatim`
  family (10461-10578), `test_dw_1_1_*` / `test_dw_1_2_*` / `test_dw_1_3_*`
  (14057-14209), `the_magic_family_data_map_is_verbatim` (14287),
  `test_dw_2_1_the_out_of_scope_kinds_stay_rowless` (14410).
- Families (Wave 3 detail, listed for coverage): boomerang 10666-10795, flail
  10837-11388, bomb 11402-11595, yoyo 11595-12751, whip 12826-13192, magic 14525+,
  leaf blower 9781/9835, hostile/bug 9851/9913.

`crates/tr-core/src/whip.rs` — 13 tests (geometry, `fill_whip_control_points`,
`sweep_hits`, tag rows).

## Deviations & risks

| Deviation | Severity | Admitted? |
| --- | --- | --- |
| `timeLeft--` / `penetrate == 0` → kill run once per update, not once per sub-step (`projectile.rs:9079-9090` vs `Projectile.cs:17201/17206` inside the `while` at 16676). Every `extraUpdates > 0` kind (lasers 83/100, the Blowgun dart 645, the Ray Gunner bolt 438, the EyeFire 101, deathrays) lives ~2× longer than vanilla | behavioural | No |
| `damage_pve` now tests `Damage_GetHitbox()`'s rect (`:14959-15023`) against `npc.getRect()` (`:12654`), but `Damage_PVE_Inner` calls the full `Colliding`, whose per-type overlay arms (gem-staff `oldPos` sweep, 698/973/985/1115/1116/554/301/…, the `Collision.CanHit` line tests) are unported — those kinds get the plain overlap only (S1-C-8 closed the point→rect half) | behavioural | Yes — `damage_hitbox`'s doc + `docs/ANALYSIS.md` |
| Duplicate unreachable match arms for aiStyles 1/3/14/99/115 (8624+8728, 8631+8729, 8695+8730, 8777+8983, 8732+8925) — dead code, two copies of the yoyo/flail dispatch bodies to keep in sync; the second `99` arm could silently become the live one under a refactor | determinism (maintenance) | No |
| `spawn_with_data_ai_owned` declines on a full pool (`slot?`, 8149) while C# recycles the oldest non-netImportant slot (10823-10846) | behavioural | Implicitly — `the_summon_fixture_respects_a_full_pool` (world_sim 288) locks the decline in |
| Plain `spawn` (7808) recycles by lowest `timeLeft` without the `!netImportant` filter of `FindOldestProjectile` (10814) | behavioural | No |
| No `oldPos`/`oldRot`/`oldSpriteDirection` (718-735, 17170-17195): no trails, no `TrailingMode[5]`, so the render pass cannot draw any vanilla-style projectile trail | pixel | No |
| No `Projectile.light` propagation (SetDefaults' `light` column, e.g. 853/856/944) | pixel | No |
| `Kill()` truncates to `active = false` (`:7716`): no channel cancel, no self-damage, no non-bomb kill arms, no gore/dust/sound drops | behavioural | Partially (bombs ride `explode_bomb`) |
| `kill_remote`/`generation` has no `ProjectileKey` ↔ slot index map (10852-10858): the `27`/`29` wire cannot address a slot by key | determinism (MP) | Yes (MP out of scope) |
| `AI()` dispatch covers 32/210 aiStyles; the rest silently no-op (`_ => {}`) rather than asserting | behavioural | Yes (Wave 3 scope) |
| The `SetDefaults` chain is a `proj_data` table, not a per-slot mutation — a kind added to the C# chain exists in the port only if a row is added | behavioural | Yes (documented gate tests) |

## Task seeds

- **S** — Hoist the `timeLeft`/`penetrate` bookkeeping and the `Damage()` call back inside
  the sub-step loop (`projectile.rs:9046-9090`) and re-pin `time_left_expires_the_slot`;
  depends on nothing, but re-check every `extraUpdates > 0` family fixture.
- **S** — Delete the duplicate match arms (8728/8729/8730/8983/8925) and replace
  `_ => {}` with an `assert!`-on-unknown-aiStyle arm gated by an allow-list, so a new
  `proj_data` row cannot silently miss its AI.
- **M** — Add the `oldPos`/`oldRot`/`oldSpriteDirection` ring buffer sized by
  `ProjectileID.Sets.TrailCacheLength` (718-728) and the `Update` shift (17170-17195),
  then a trail draw pass (`TrailingMode` 5 included).
- **M** — Port `Projectile.light` into the slot (`proj_data` column + the draw's light
  emission) for the ~40 kinds whose `SetDefaults` sets it.
- ✅ **M** — DONE 2026-11-12 (`S1-C-8-projectile-pve-hitbox`): `Damage_GetHitbox()`'s
  rectangle (`projectile.rs::damage_hitbox`) replaces the point test in `damage_pve`,
  keeping `damage_pve_boxed` for the resized bursts. Follow-up: port `Colliding`'s
  per-type overlay arms (the gem-staff `oldPos` sweep, the `Collision.CanHit` line
  tests).
- **M** — Complete the spawn protocol: `FindOldestProjectile` with the `netImportant`
  filter on both spawn paths, the golf-ball 10-cap, and
  `CopiesOwnerAttackCDToLocalImmunityOnSpawn` (10934-10968).
- **L** — Port the `ProjectileKey` table (`slotGenerations` + `keyToIndex`,
  10852-10858) so the `27`/`29` wire can address slots; prerequisite for module 17.
- **L** — Port `Kill()`'s full arm set (channel cancel, self-damage, per-type kill
  arms, gore/dust/sound) and the `CutTiles`/`SwitchTiles` calls; the family modules
  (26) each need the shared half once.
