# 28 — Buffs & debuffs (ids, application rules, effects, UI, sync)

> Wave 3 · researched 2026-11-12 · sources: decompiled + crates + ROADMAP/ANALYSIS/PORT_MAP

## Scope

The buff system is the id table (`Terraria.ID.BuffID`, 401 ids), the **slot engine**
(`Player.buffType[44]` / `buffTime[44]`, the `AddBuff` family), the **per-buff effect
table** (`Player.UpdateBuffs`' 251-arm if/else chain), the **NPC twin**
(`NPC.buffType[20]`, `UpdateNPC_Buff*`, `DOTTally`), the **HUD** (`Main.DrawBuffIcon`)
and the **net/save** carriers. Modules 03/05 own the *ticking* of life/mana regen and
NPC AI; this file owns the buff system itself.

| File | Lines | Role |
| --- | --- | --- |
| `decompiled/Terraria/ID/BuffID.cs` | 999 | 401 named ids (1..400) + `Sets` (:40-171) + `Count` (:997) |
| `decompiled/Terraria/Player.cs` | 54 877 | slots :1508-1514 · `FindBuffIndex`/`AddBuff` family :5250-5480 · `UpdateBuffs` :9855-12069 · end-of-update immune purge :26012 |
| `decompiled/Terraria/NPC.cs` | 95 306 | `maxBuffs = 20` :95189 · `buffType/buffTime/buffImmune` :6193-6197 · `AddBuff` :89926-89983 · `DelBuff` :90002-90023 · `UpdateNPC_Buff*` :91594-92192 |
| `decompiled/Terraria/Main.cs` | 67 317 | `DrawBuffIcon` :43307-43368 · `TryRemovingBuff` :43370 · HUD call sites :41335, :43222 |
| `decompiled/Terraria/NetMessage.cs` | 2 977 | buff rows in the player state :1000 and the NPC state :1028 |
| `crates/tr-app/src/buffs.rs` | 599 | the player slot engine (`BuffSlots`) + the `Main.debuff`/pet/melee/pvp tables |
| `crates/tr-app/src/npc_buffs.rs` | 1 086 | the NPC slot engine, the DoT tally, the per-kind immunity table |
| `crates/tr-app/src/player.rs` | 10 098 | the per-slot effect scan (:2839-2930), `derive_dot_inputs` (:4260), `BUFF_IMMUNITY_ROWS` (:242) |
| `crates/tr-app/src/regen.rs` | 706 | `Regen` (life) + `ManaRegen` — the two regen engines, fed by the DoT/aura flags |
| `crates/tr-app/src/ui/*`, `render/*` | — | **no buff icon drawing anywhere** (only a `vanityPet`/`lightPet` comment in `ui/chrome.rs:238`) |

Responsibility: which buffs exist, how they are applied/refreshed/evicted, what each
one does, which debuffs cannot stack, what the player sees, and what travels over the
wire.

## Key entities

C# (source of truth):

- `Terraria/ID/BuffID.cs::BuffID.Count` (:997) — **401**; ids 1..400, named constants
  (`ObsidianSkin = 1` … `RedLightning = 400`, :173-989).
- `Terraria/ID/BuffID.cs::BuffID.Sets` (:40-171) — the set tables: `IsWellFed` (:43),
  `IsFedState` (:44), `SortingPriorityFoodBuffs` (:45), `IsAnNPCWhipDebuff` (:46),
  `TimeLeftDoesNotDecrease` (:47), `CanBeRemovedByNetMessage` (:48, **empty**),
  `IsAFlaskBuff` (:49), `BuffTimeIsExtendedWithGameDifficulty` (:50),
  `BuffTimeIsExtendedByDeadCellsPotionStationBuff` (:51), `NurseCannotRemoveDebuff`
  (:52), `AddBuffTimeAdditivelyToCap` (:53), `BuffTextHandlers` (:54-168, 28
  `CachedProjectileCounterBuffTextHandler` rows), `MountType` (:169).
- `Terraria/Player.cs::Player.AddBuff` (:5263) + helpers `AddBuff_ReduceTimeOfRelatedBuffs`
  (:5298), `AddBuff_ActuallyTryToAddTheBuff` (:5325), `AddBuff_RemoveOldMeleeBuffsOfMatchingType`
  (:5347), `AddBuff_RemoveOldPetBuffsOfMatchingType` (:5377),
  `AddBuff_TryUpdatingExistingBuffTime` (:5399), `AddBuff_DetermineBuffTimeToAdd` (:5424),
  `DelBuff` (:5451), `ClearBuff` (:5470), `FindBuffIndex` (:5250).
- `Terraria/Player.cs::Player.UpdateBuffs` (:9855) — the timer head (:9865-9874) then the
  effect chain: **251 `buffType[j] == N` arms over ~290 distinct ids**, ending before
  `UpdateBuffs_NebulaBuffs` (:12070).
- `Terraria/Main.cs::Main.DrawBuffIcon` (:43307) / `Main.TryRemovingBuff` (:43370).
- `Terraria/NPC.cs::NPC.AddBuff` (:89926) / `DelBuff` (:90002) / `FindBuffIndex` (:89910)
  / `UpdateNPC_BuffSetFlags` (:91965) / `UpdateNPC_BuffClearExpiredBuffs` (:91890) /
  `UpdateNPC_BuffApplyDOTs` (:91594) / `DOTTally` (:59-141) / `NPC.GetHurtByDebuff` (:91864).
- **`Player.HasBuff` does not exist in 1.4.5.8** — a grep over `decompiled/Terraria` finds
  only `HasBufferedChange`. The vanilla "has buff" query is `FindBuffIndex` +
  `buffType[i] == id`.

Rust (target):

- `crates/tr-app/src/buffs.rs::BuffSlots` (:125) — `buff_type[44]`, `buff_time[44]`,
  `buff_immune[401]`, `resist_cold`, `dead_cells_potion_station`, `expert_mode`.
- `crates/tr-app/src/buffs.rs::add_buff` (:154) and its five helpers (:175, :199, :221,
  :241, :258, :277) — the port of the whole `AddBuff` chain.
- `crates/tr-app/src/buffs.rs::del_buff` (:310) / `clear_buff` (:328) / `find_buff_index`
  (:337) / `update_buffs_tick` (:350) / `reset_buff_immune` (:365) / `purge_immune_buffs` (:372).
- `crates/tr-app/src/buffs.rs::DEBUFF_IDS` (:28, 76 entries), `LIGHT_PET_IDS` (:36),
  `VANITY_PET_IDS` (:39), `MELEE_BUFF_IDS` (:47), `PVP_BUFF_IDS` (:52).
- `crates/tr-app/src/npc_buffs.rs::NpcBuffs` (:124) / `NPC_MAX_BUFFS` (:29) /
  `NpcBuffFlags` (:36) / `DotTally` (:71) / `debuff_immune_ids` (:155) /
  `for_kind` (:377) / `add_buff` (:398) / `update_buffs_tick` (:477) / `apply_dots` (:542).
- `crates/tr-app/src/player.rs::BUFF_IMMUNITY_ROWS` (:242), `pub buffs: BuffSlots` (:638),
  `SUMMON_BUFF_ARMS` (:1022), `update_summon_buff_arms` (:2113),
  `derive_dot_inputs` (:4260), `remove_expired_buffs` (:4196), `clear_buff_type` (:4738),
  the inline effect `match` (:2839-2930).
- `crates/tr-app/src/regen.rs::Regen` / `ManaRegen` — the two regen engines; they are
  **life** and **mana** regen, not two buff engines; the buff input is the DoT/aura/fed
  flag set (`RegenInputs`, regen.rs:39-135).
- `crates/tr-app/src/mount.rs::mount_type_of_buff` (:1458) / `check_mount_buff` (:1861),
  `crates/tr-app/src/banners.rs::banner_buff_for` (:74),
  `crates/tr-app/src/projectile.rs::whip_debuffs` (:10223).

## Dependencies

- Calls into: `crates/tr-app/src/player.rs` (the tick, the stat adds), `regen.rs`
  (life/mana), `npc.rs`/`npc_buffs.rs` (the NPC twin), `mount.rs`, `banners.rs`,
  `luck.rs` (buff 257/383/120 → luck), `armor_sets.rs::set_buff_immunity` (accessory
  immunity rows), `app/mod.rs` (the Nurse heal clears debuffs, :1251-1260).
- Called by: `world_sim/player.rs` (the fixed step), `world_sim/npcs.rs` → `npc.rs::update_rng`
  (the buff tick runs **before** the AI, npc.rs:4107), `projectile.rs` (the whip debuffs),
  `world_sim/editing.rs` (the consumable/potion grants), `app/mod.rs` (nurse heal),
  `app/netglue.rs` (would carry them — see Deviations).
- Shared data structures: `BuffSlots` (44 slots + the 401-entry `buff_immune` table) is
  embedded in the Rust `Player` (player.rs:638); `NpcBuffs` (20 slots) in the Rust `Npc`.
  Both mirror the C# arrays one-to-one.
- C# callers of the buff engine: `Player.AddBuff` is called from ~40 sites (items,
  potions, auras, projectiles, the Nurse, the mount); `Main.TryRemovingBuff` is the
  player-facing removal path.

## Port status

| C# subsystem | Status | Rust location | Evidence / notes |
| --- | --- | --- | --- |
| `BuffID` id table (401 ids, named) | PARTIAL | `buffs.rs::BUFF_ID_COUNT` (:23) | Only the *count* exists. The Rust uses raw `i32` literals (e.g. player.rs:2844 `25 =>`), so there is no `BuffId` enum and no name↔id mapping — every buff reference is an unreadable magic number (maintainability risk, not behavioural). |
| `BuffID.Sets.IsWellFed` / `IsFedState` | PORTED | `buffs.rs::is_fed_state` (:82) | `{26,206,207,332,333,334}` matches BuffID.cs:44 exactly; `IsWellFed` ⊂ it. |
| `BuffID.Sets.TimeLeftDoesNotDecrease` | PORTED | `buffs.rs::time_left_does_not_decrease` (:87) | `{28,334,29,159,150,93,348,366}` = BuffID.cs:47. |
| `BuffID.Sets.BuffTimeIsExtendedWithGameDifficulty` | PORTED | `buffs.rs::buff_time_is_extended_with_game_difficulty` (:92) | 19 ids = BuffID.cs:50; the ×2 (expert) / ×2.5 (master) scale is in `add_buff_determine_buff_time_to_add` (:175). |
| `BuffID.Sets.BuffTimeIsExtendedByDeadCellsPotionStationBuff` | PORTED | `buffs.rs::buff_time_is_extended_by_dead_cells_potion_station` (:101) | `1..18, 104..117, 121..124, 257, 343` = BuffID.cs:51; the ×1.2 truncation is reproduced. |
| `BuffID.Sets.AddBuffTimeAdditivelyToCap` | PORTED | `buffs.rs::add_buff_time_additively_to_cap` (:109) | `{0, 94→600, 383→43200}` = BuffID.cs:53. |
| `BuffID.Sets.IsAFlaskBuff` (9 ids) | PARTIAL | `buffs.rs::MELEE_BUFF_IDS` (:47) | Rust has 8 ids (`71,73..79`); the C# set also contains **72 (Midas)**. See Deviations D-2. |
| `BuffID.Sets.NurseCannotRemoveDebuff` (20 ids) | MISSING | — | The Rust Nurse heal clears *every* debuff (`app/mod.rs:1251-1260` filters `is_debuff`); the C# keeps the 20 protected debuffs. See D-3. |
| `BuffID.Sets.SortingPriorityFoodBuffs` | MISSING | — | Only used by `UI/ItemSorting.cs:820` (item sorting) — module 21 territory. |
| `BuffID.Sets.IsAnNPCWhipDebuff` `{310,362}` | MISSING | — | The whip rows exist in `projectile.rs::whip_debuffs` (:10223) as raw ids; the set itself is not modelled. |
| `BuffID.Sets.CanBeRemovedByNetMessage` (empty) | N/A | — | The C# set is empty (`CreateBoolSet()`), so no behaviour is lost. |
| `BuffID.Sets.BuffTextHandlers` (28 rows) | MISSING | — | Only used by `Main.DrawBuffIcon` (:43317) — no Rust icon UI to feed. |
| `BuffID.Sets.MountType` | PARTIAL | `mount.rs::mount_type_of_buff` (:1458) | The mount→buff mapping is ported; the *set* table (BuffID.cs:169) is not a table in Rust. |
| `Player` slot array (44) + `buffImmune[401]` | PORTED | `buffs.rs::BuffSlots` (:125) | `MAX_BUFFS = 44` (Player.cs:57106), `BUFF_ID_COUNT = 401` (BuffID.cs:997). |
| `Player.AddBuff` chain (all 7 helpers) | PORTED | `buffs.rs::add_buff` (:154) + helpers | The immune guard, fed-state wipe, time determination, additive caps, pet/melee exclusivity, Warmth reduction and the evict-or-reject scan are all present and 1:1 (including the `DelBuff` compaction quirk). |
| `Player.DelBuff` / `ClearBuff` / `FindBuffIndex` | PORTED | `buffs.rs::del_buff` (:310) / `clear_buff` (:328) / `find_buff_index` (:337) | The `i < maxBuffs - 1` hole-at-42 quirk is reproduced and tested. |
| `Player.UpdateBuffs` timer head (:9865-9874) | PORTED | `buffs.rs::update_buffs_tick` (:350) | The `TimeLeftDoesNotDecrease` gate and the empty-slot skip match. |
| End-of-update immune purge (:26012) + per-frame reset | PORTED | `buffs.rs::purge_immune_buffs` (:372) / `reset_buff_immune` (:365) | The "no rescan after `DelBuff`" quirk is reproduced (and is observable). |
| `Main.debuff` / `lightPet` / `vanityPet` / `meleeBuff` / `pvpBuff` tables | PORTED | `buffs.rs` :28, :36, :39, :47, :52 | 76 / 12 / 74 / 8 / 20 entries, extracted verbatim; `is_pvp_buff` is `#[allow(dead_code)]` (the MP branch is unreachable in SP). |
| **Per-buff effect table** (`UpdateBuffs`, 251 arms / ~290 ids) | **PARTIAL** | `player.rs` inline `match` (:2839-2930), `derive_dot_inputs` (:4260), `regen.rs`, `luck.rs`, `mount.rs`, `banners.rs`, `projectile.rs` | Only a slice of the chain is ported: the visible arms are 14 Thorns, 25 Tipsy, 26/206/207 food, 46/47 chilled/frozen, 120 Stinky, 146 Sunflower, 149 Webbed, 158 Star-in-a-Bottle, 257 Lucky, 383 Kite; the DoT family, the auras, the whip rows, the summon arms and the mount buffs live in their own modules. **≈290 C# arms vs ≈40 ported ids.** |
| DoT family (20/24/30/39/44/68/69/70/137/144/145/153/323/324/375/395/397/398) | PORTED | `player.rs::derive_dot_inputs` (:4260) + `regen.rs` | Poison −4, venom −30, OnFire −8, cursed inferno −24, frostburn −16, hellfire −60, suffocation −40, electrified −8/−40; the drain caps and the `DotDeath` reasons are ported. |
| Movement/status debuffs (32 Slow, 46, 47, 149 Webbed, 22 Darkness, 33 Weak) | PORTED | `player.rs:2594`, `:2932-2944`, `frame.rs:803` | `strongest_move_speed_debuff` 0.5/0.75, the webbed velocity zeroing, the Darkness `blind` → `GlobalBrightness` 1.2→1.0. |
| Aura buffs (87/89/146/158/157/350/215) | PORTED | `world_sim` scene scan + `player.rs` | M2-BUFF-5 (the `SceneMetrics` scan + the one-tick grant order). |
| Food buffs (26/206/207) | PORTED | `player.rs:2858-2899` | The per-tier crit/damage/speed/defense adds. |
| Hunger family (332/333/334) | PARTIAL | `buffs.rs::is_fed_state` only | The tier rows are unported (ROADMAP M2-BUFF-7 says "the Hungry debuff 333 lands with the negative-tier need"). |
| Luck inputs (257/383/120) | PORTED | `player.rs:2903-2928`, `luck.rs` | The 257 tier by remaining time, the stale 383 tier, the Stinky flag. |
| Banner buff (147) | PORTED | `banners.rs::banner_buff_for` (:74) | M2-BUFF-8 (the strength tiers; the placement waits for the banner tile). |
| Whip debuffs (310/362 + the whip rows) | PORTED | `projectile.rs::whip_debuffs` (:10223) | The per-whip rows are verbatim; the `IsAnNPCWhipDebuff` set is not modelled. |
| Accessory `buffImmune` rows | PORTED | `player.rs::BUFF_IMMUNITY_ROWS` (:242) | The Ankh Shield's 11 debuffs etc. |
| **Buff UI** (`Main.DrawBuffIcon` :43307) | **MISSING** | — | No icon, no `buffAlpha` fade, no duration text, no hover, no right-click removal anywhere in `crates/tr-app/src/ui` or `render`. See D-1. |
| `Main.TryRemovingBuff` (:43370) | MISSING | — | The debuff refusal (only 60/151 may be removed), the mount dismount and sound 12 are unreachable. |
| Net sync — buff rows in the player state | MISSING | — | C# `NetMessage.cs:1000` sends `Player.maxBuffs` rows inside the player-state message; the Rust `netglue.rs` has no buff payload. |
| Net sync — message 55 `PlayerBuff` | MISSING | — | The ROADMAP lists `55 PlayerBuff` in the message catalogue; the Rust `netglue.rs` implements no `case 55`. |
| NPC slot engine (20 slots, AddBuff/DelBuff/FindBuffIndex) | PORTED | `npc_buffs.rs` :398/:457/:442 | `NPC.maxBuffs = 20` (NPC.cs:95189) and the Rust `NPC_MAX_BUFFS = 20` (npc_buffs.rs:29) — **they agree**. The module-05 note about a "5-slot debuff pool" is not backed by the code; the likely source is the per-kind *immunity* table, whose rows are 1-2 ids wide. |
| NPC DoT engine (`DOTTally`, `ApplyDOTs`, `GetHurtByDebuff`) | PORTED | `npc_buffs.rs::apply_dots` (:542), `DotTally` (:71) | The branch order, the `drippingSlime` ×2, the `accelerate_poisons` support, the ±120 chunk drain. |
| NPC debuff-immunity table | PARTIAL | `npc_buffs.rs::debuff_immune_ids` (:155) | The rows for the *ported roster* only (182 kinds); the C# `NPCID.Sets.DebuffImmunitySets` covers every id. The derived rules (20 ⇒ 30/375, 69 ⇒ 36) are ported. |
| NPC buff VFX (spore particles) | MISSING | — | `NPC.cs:91758-91773` is visual-only and not ported. |

## Tests

- `crates/tr-app/src/buffs.rs` — **17** tests: `add_to_empty_occupies_first_slot`,
  `re_add_takes_max_not_additive`, `additive_caps_94_and_383`,
  `fed_state_wipes_previous_fed_buffs`, `light_and_vanity_pets_are_exclusive`,
  `melee_buffs_are_exclusive_with_rescan`, `warmth_reduces_chilled_and_frozen`,
  `resist_cold_shortens_chilled_and_frozen_on_add`,
  `dead_cells_station_extends_by_1_2_truncating`,
  `full_positive_bar_evicts_first_positive_into_the_quirk_hole`,
  `full_debuff_bar_rejects_everything`, `mixed_bar_debuff_evicts_a_positive_buff`,
  `del_buff_quirk_leaves_hole_at_42_when_full`,
  `tick_decrements_and_honors_time_left_does_not_decrease`,
  `immune_blocks_add_and_purge_removes_active`, `find_buff_index_and_clear_buff`,
  `pvp_and_melee_sets_are_consistent_with_the_c_sharp_tables`.
- `crates/tr-app/src/npc_buffs.rs` — **24** tests (module 05's count), incl.
  `clear_expired_quirk_keeps_a_shifted_in_expired_buff_one_tick`,
  `del_buff_shifts_the_tail_once_per_empty_without_rescanning`,
  `find_buff_index_matches_the_c_sharp`.
- `crates/tr-app/src/player.rs` — buff-relevant tests: `stat_defense_sums_pieces_set_bonus_and_buffs`
  (:6013), `the_immunity_rows_block_their_debuffs` (:6188),
  `the_potion_sickness_buff_gates_and_expires` (:6468),
  `the_fed_flag_clears_when_the_buff_expires` (:6505),
  `the_sunflower_buff_moves_and_the_star_buffs_mana_regen` (:6518),
  `the_aura_buffs_grant_from_the_scene_snapshot` (:6533),
  `buff_257_tiers_follow_the_remaining_time` (:8521),
  `buff_383_tier_is_stale_after_expiry_never_zeroed` (:8541),
  `buff_120_sets_stinky_and_the_reset_clears_it` (:8556),
  `water_cures_both_fire_buffs_over_two_ticks_despite_the_no_rescan_quirk` (:8710),
  `immune_purge_skips_the_compacted_fire_buff_for_a_frame` (:8782),
  `expired_buffs_are_swept_after_the_tick` (:9003),
  `quick_mount_equips_the_first_saddle_and_the_buff_persists` (:10147),
  `the_buff_expiring_dismounts` (:10174), `test_edge_the_arms_rescan_after_delbuff` (:10731),
  `test_edge_buff_161_tracks_projectile_533` (:10777).
- `crates/tr-app/src/npc.rs` — `npc_update_runs_the_buff_tick_before_the_ai` (:13679),
  `spawned_npcs_carry_their_debuff_immunity_table` (:13694).
- `crates/tr-app/src/projectile.rs` — `the_whip_debuff_rows_are_verbatim` (:14823),
  `the_cool_whip_grants_its_buff_on_every_hit` (:15099).
- **No test covers the buff id table, the `BuffID.Sets` completeness, the buff UI or
  the buff net message** — because those are the missing pieces.

## Deviations & risks

- **D-1 — the buff HUD is entirely missing (behavioural + pixel, high).**
  C#: `Main.DrawBuffIcon` (Main.cs:43307-43368) draws `TextureAssets.Buff[num]` with the
  per-slot `Main.buffAlpha` (hover +0.1 / −0.05, clamped 0.4..1.0), the remaining-time
  text (`Lang.LocalizedDuration`, or the `BuffTextHandlers` projectile counters), and
  the right-click removal → `Main.TryRemovingBuff` (:43370). Rust: **no icon, no alpha,
  no timer text, no removal** — the only "buff" mentions under `crates/tr-app/src/ui`
  and `render` are a `vanityPet`/`lightPet` comment (ui/chrome.rs:238). ROADMAP §8
  does not list a UI task for the buff bar; **the ROADMAP does not admit this gap**.
- **D-2 — `IsAFlaskBuff` / `Main.meleeBuff` id 72 (Midas) (cosmetic/low).**
  `BuffID.cs:49` lists 9 flask ids (`71,72,73..79`); the Rust `MELEE_BUFF_IDS`
  (buffs.rs:47) has 8 (`71,73..79`). If the Rust list is a port of `Main.meleeBuff`
  (`Main.cs:8961`) it may be faithful, and if it is a port of `BuffID.Sets.IsAFlaskBuff`
  it is missing 72 → the Midas buff would not be flushed when another flask is drunk.
  **Verify which table `AddBuff_RemoveOldMeleeBuffsOfMatchingType` (Player.cs:5347)
  consults before changing anything.** Not admitted in ROADMAP/ANALYSIS.
- **D-3 — the Nurse removes protected debuffs (behavioural/medium).**
  `BuffID.Sets.NurseCannotRemoveDebuff` (BuffID.cs:52, 20 ids: 21,28,34,43,48,86,87,89,
  146,147,157,158,165,199,215,321,332,333,334,350) is not modelled; the Rust nurse heal
  clears every debuff (`app/mod.rs:1251-1260`). The C# price formula also counts
  "removable debuffs" only — the Rust `nurse_heal_cost_for` (app/mod.rs:4293) counts
  *all* debuffs, so the **price is also wrong** for a player holding a protected debuff.
- **D-4 — the effect table is ~15 % ported (behavioural/high).**
  The C# `UpdateBuffs` chain is 251 arms over ~290 ids; the Rust covers ≈40 ids across
  five modules. Unported examples that change play: 3 Swiftness / 5 Ironskin / 7
  MagicPower / 16 Archery / 113 Lifeforce / 115 Rage / 117 Wrath / 159 Sharpened
  (armour penetration) / 165 Dryad's Ward / 194 WindPushed / 199 NoBuilding /
  215 Cat Bast / 332-334 hunger / 348 War Table / 353 Shimmer / 396 Mana Heat.
  ROADMAP M2-BUFF-3/4/5/7/8 claim "done" for the *families* they cover; the
  **remainder of the chain is not claimed anywhere** — the ROADMAP overstates coverage
  by reading "M2-BUFF-4 stat debuffs" as "the effect table".
- **D-5 — no buff net sync (behavioural/medium).**
  The C# carries the buff rows in the player state (`NetMessage.cs:1000`, `:1028`) and
  has a dedicated `55 PlayerBuff` message; the Rust `netglue.rs` implements neither.
  Buffs therefore never replicate to a peer. Consistent with the port's current MP
  scope, but it is an admitted-by-omission gap, not a verified DONE.
- **D-6 — the id space is magic numbers (maintainability/low).**
  `BUFF_ID_COUNT = 401` is the only representation of the id table; every use site
  writes a literal (`player.rs:2844`, `npc_buffs.rs:502-518`). A `BuffId` enum (or a
  generated const set) would make the coverage measurable and prevent silent id drift.
- **D-7 — out-of-range ids panic (robustness/low).**
  `BuffSlots::add_buff` (buffs.rs:155) indexes `buff_immune[buff_type as usize]`
  without a guard, so a negative or ≥401 id panics (the C# throws
  `IndexOutOfRangeException` — same class of failure, but the Rust `find_buff_index`
  *does* guard (:338), so the two Rust entry points disagree).
- **D-8 — the expert/master debuff extension is hard-wired to `expert_mode` (behavioural/low).**
  `add_buff_determine_buff_time_to_add` (buffs.rs:175) applies ×2 with
  `expert_mode = true` and never ×2.5 (master). The field has no writer in the port
  (no difficulty modes yet), so the ×2 branch is currently dead. ROADMAP's
  "expert ×2 (master ×2.5)" claim is therefore **not observable** in the port today.
- **D-9 — NPC `NPC.maxBuffs` is 20, not 10 (correction, informational).**
  C# `NPC.maxBuffs = 20` (NPC.cs:95189) and the Rust `NPC_MAX_BUFFS = 20`
  (npc_buffs.rs:29) agree; the "5-slot debuff pool" attributed to module 05 does not
  exist in the code.

## Task seeds

- **S — add the `BuffId` id module (no behaviour change).** Generate
  `crates/tr-app/src/buff_id.rs` with the 401 named constants from `BuffID.cs:173-989`
  and replace the literals in `player.rs`/`npc_buffs.rs`/`buffs.rs`. Unblocks D-6 and
  makes the coverage table checkable. Depends on nothing.
- **M — `BuffID.Sets` completion.** Port `IsAFlaskBuff` (with the 72 question of D-2
  settled first), `NurseCannotRemoveDebuff` (D-3), `IsAnNPCWhipDebuff`,
  `SortingPriorityFoodBuffs`, `CanBeRemovedByNetMessage`, `BuffTextHandlers`
  (needed by the UI). Depends on the `BuffId` seed for readability.
- **M — the buff HUD.** Port `Main.DrawBuffIcon` + `Main.TryRemovingBuff`: the
  `TextureAssets.Buff` atlas rows, the `buffAlpha` fade, the duration text
  (`Lang.LocalizedDuration` + the projectile-counter handlers), the hover and the
  right-click removal (debuff refusal, mount dismount, sound 12). **No Rust buff
  texture atlas exists yet** — the asset row is part of the task. Closes D-1.
- **M — the remaining `UpdateBuffs` arms, batch 1 (the potion/buff families).**
  1 ObsidianSkin, 2 Regeneration, 3 Swiftness, 4 Gills, 5 Ironskin, 6 ManaRegeneration,
  7 MagicPower, 8 Featherfall, 9 Spelunker, 10 Invisibility, 11 Shine, 12 NightOwl,
  13 Battle, 14 Thorns, 15 WaterWalking, 16 Archery, 17 Hunter, 18 Gravitation.
  One arm per id with a C# citation, one test per family. Closes part of D-4.
- **M — the remaining `UpdateBuffs` arms, batch 2 (the world/event buffs).**
  43 Paladin's Shield, 48 Honey, 60-63, 87-90, 93-99, 104-117 (Dangersense, Ammo
  Reservation, Lifeforce, Endurance, Rage, Inferno, Wrath), 121-124, 152, 156-158,
  163-165, 194-199, 215, 332-334, 343, 348, 353, 366, 396.
- **L — the NPC buff completion.** The full `NPCID.Sets.DebuffImmunitySets` table for
  every id (not just the 182 roster kinds), the unported NPC flags (25 Tipsy, 69 Ichor,
  31 Confused, the whip marks), `UpdateNPC_SoulDrainDebuff` and the spore VFX.
- **M — buff net sync.** The buff rows in the player-state message
  (`NetMessage.cs:1000`) and the `55 PlayerBuff` message (`MessageBuffer.cs`), with
  the `CanBeRemovedByNetMessage` gate (currently empty in the C#). Closes D-5.
- **S — re-verify the DONE rows.** M2-BUFF-1..8: read each Rust claim against the C#
  cited in `docs/ROADMAP.md` §"Task catalog" (rows at the M2-BUFF block) and mark
  PASS/PARTIAL/FAIL per row — M2-BUFF-1 (PASS), M2-BUFF-2 (PASS for life/mana, but
  note the *buff* input is only the DoT/aura set), M2-BUFF-3 (PASS), M2-BUFF-4
  (PARTIAL — the six named debuffs only), M2-BUFF-5 (PASS), M2-BUFF-6 (PASS),
  M2-BUFF-7 (PARTIAL — the hunger tiers), M2-BUFF-8 (PARTIAL — the banner placement
  is still open).
- **S — fix the Nurse debuff rule (D-3)**: honour `NurseCannotRemoveDebuff` in
  `app/mod.rs:1251-1260` *and* in `nurse_heal_cost_for` (:4293), with a test for a
  protected (e.g. Potion Sickness 21) and an unprotected debuff.
