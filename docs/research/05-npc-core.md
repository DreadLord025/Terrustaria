# 05 — NPC core (defaults, AI dispatch, net sync)

> Wave 1 · researched 2026-11-12 · sources: decompiled + crates + ROADMAP/PORT_MAP

## Scope

The NPC module is the biggest single subsystem in the port: the C# side is
`Terraria/NPC.cs` (95,306 lines) plus its id tables in `Terraria/ID/NPCID.cs`
(12,584 lines) and `Terraria/Projectile.cs` (79,791 lines, for the PvE strike
path). Its responsibility is the whole enemy/NPC lifecycle: `SetDefaults` /
`SetDefaultsFromNetId` (the per-type stat table), the fixed-step `UpdateNPC`
phase order, the `AI()` `aiStyle` dispatch (0..127), `StrikeNPC` damage, the
spawn protocol (`NewNPC` / `GetAvailableNPCSlot`), `Transform`, and the
`realLife` boss-part chains.

On the Rust side the module is spread over five files:

| Rust file | Lines | Role |
| --- | --- | --- |
| `crates/tr-app/src/npc.rs` | 17,489 | `NpcKind` (182 variants), the per-kind stat tables (`max_hp`/`defense`/`value`/`size`/`knock_back_resist`), the `Npc` struct, `Npc::new` (the `SetDefaults` stand-in), `update_rng` (the `UpdateNPC`+`AI()` stand-in), `find_frame`, hurt, spawn helpers, the net-id mappings |
| `crates/tr-app/src/npc_ai.rs` | 769 | the shared AI archetype steps (`walker_step`, `hover_step`, `caster_cycle`/`find_teleport_spot`, `sphere_step`, `swim_step`, `flyer_step`) + `WalkerParams`/`HoverParams`/`CasterParams`/`SwimParams` |
| `crates/tr-app/src/npc_buffs.rs` | 1,141 | the buff/DoT engine: the 5-slot debuff pool, the per-kind immunity table, `apply_dots`, `set_flags_tick` |
| `crates/tr-app/src/world_sim/npcs.rs` | 9,340 | the 60 Hz world pass: part-chain pre-passes, the per-NPC step, the worm pass `update_worms_in`, town arrival, the death/`checkDead` bookkeeping, loot/coins, invasion+DD2 state, `SpawnNPC` |
| `crates/tr-app/src/spawner.rs` | 2,223 | `GetSpawnRate` + `SpawnAnNPC` selection ladders + `spawn_npc_call` |
| `crates/tr-core/src/damage.rs` | 297 | the `StrikeNPC` arithmetic: `calculate_damage_npcs_take`, `strike_npc_damage`, `check_armor_penetration`, `knock_back_npc`, `damage_var`, `crit_roll`, `iframes_for` |

## Key entities

- `crates/tr-app/src/npc.rs::NpcKind` (31–686) — 182-variant enum, the port's NPC
  roster. Not the C# id space: `to_u16` (850) / `from_u16` (1105) are a
  **port-local protocol id space**; the vanilla ids live in `net_id` (1512,
  routed through `crate::bestiary::entry_for_npc`) and `cs_id` (1789, the
  vanilla `NPC.type` used by the statue/wiring rosters).
- `crates/tr-app/src/npc.rs::Npc` (2446) — the struct. Fields carry C# cites;
  `immune: [u16; 256]` (the per-owner i-frames), `worm_id`/`seg_index`/
  `worm_left`/`worm_inited` (the chain identity that replaces the vanilla's
  `ai[1]`/`ai[3]` slot links), `wants_sphere`/`wants_minion`/`wants_limbs`/
  `wants_hostile_shot` (deferred spawn requests drained by the world pass —
  the borrow-split against `Main.npc` being written inside `AI()`).
- `crates/tr-app/src/npc.rs::Npc::new` (3099) — the `SetDefaults` stand-in.
  hp/max_hp from `kind.max_hp()` (1941), `sprite_dir = -1` (the `SetDefaults`
  default, `NPC.cs:6515`), `buffs = NpcBuffs::for_kind(kind)`, `friendly` from
  `is_town_resident()` (1318) / the Sleeping Angler.
- `crates/tr-app/src/npc.rs::update_rng` (4103) — the fixed step: buff tick
  (4107) → `hurt_cd` (4108) → `immune[]` decay (4110) → the `match self.kind`
  AI dispatch (4114–4413) → `find_frame` (4414) → `contact` (4415). `update`
  (4093) is the same step with a scratch RNG.
- `crates/tr-app/src/npc.rs::hurt` (4027) / `hurt_from_projectile` (4062) — the
  strike entry points; they invoke `knock_back_npc` (tr-core) but take `amount`
  already reduced.
- `crates/tr-app/src/world_sim/npcs.rs::update_npcs` (1922–3533) — the world
  pass: Moon Lord carrier pre-pass (1933), golem/skeletron/prime anchor
  write-throughs, tax, invasion inputs, the 60 Hz `npc_step` (2253), worm pass
  (2274), town look-for-home (2280), `update_time_spawn_town_npcs` (2308),
  the revenge cache (2373), death retain + kill credit + boss flags
  (2404–2593), loot/coins (2591–2592), deferred spawns (2599–3115),
  `spawn_npc_call` (3185–3448).
- `crates/tr-app/src/world_sim/npcs.rs::update_worms_in` (800) — the `AI_006`
  chain: init loop (chain assembles front-to-back, 812–888), follower trail
  (891–970), the middle-split (944–957 / 975–993), `worm_head_step` (1010).
- `crates/tr-app/src/spawner.rs::spawn_npc_call` (472) — one `NPC.SpawnNPC`
  call including the `noSpawnCycle` latch.
- `crates/tr-core/src/damage.rs::strike_npc_damage` (88) — the *only* faithful
  port of `StrikeNPC_Inner`'s damage line.

## Dependencies

- **Calls into:** `tr_core::damage` (knockback/defense arithmetic),
  `tr_core::tiledata::TILE_SOLID`, `tr_core::bestiary`,
  `crate::npc_ai` (archetype steps), `crate::npc_buffs`, `crate::spawner`,
  `crate::town` / `crate::town_arrival`, `crate::loot`, `crate::projectile`
  (the deferred hostile-shot pool), `crate::revenge`, `crate::banners`,
  `crate::achievements`, `crate::world_sim::lunar`, `crate::shimmer`.
- **Called by:** `crate::world_sim::step` (the fixed stepper) →
  `update_npcs`; `world_sim/editing.rs` (the melee hit, 1124–1132);
  `crate::projectile::damage_pve` (9599 / 9703); `app/netglue.rs` (the net
  strike, 379); `app/saveglue.rs` (the `.wld` NPC sections).
- **Shared data structures:** `Npc` is the shared record between the sim
  (`world_sim/npcs.rs`), the renderer (`render/entities.rs`), the net glue and
  the save glue. `NpcKind` is referenced by `bestiary`, `spawner`, `projectile`
  (`target_allowed`/`refuses_friendly`), `wiring`, `editing`, `loot`.

## Port status

| C# subsystem | Status | Rust location | Evidence / notes |
| --- | --- | --- | --- |
| `SetDefaults` (8426–17858) | PARTIAL | `npc.rs::NpcKind` tables + `Npc::new` (3099) | No C#-style per-type if/else chain and no field-order guarantee. Stats come from per-kind lookup fns (`max_hp` 1941, `defense` 3490, `value` 3751, `size` 1533, `knock_back_resist` 3205). Verified 1:1 for the slime family (`slime_variants_match_the_setdefaults_lines`, npc.rs:13197, against `NPC.cs:8567-8580` and the `SetDefaultsFromNetId` cases −1..−9). |
| `SetDefaults` prologue (8437–8571) | MISSING | — | The field-clearing block is unnecessary in Rust (struct init); but `catchable → friendly` (`NPC.cs:17845`) and `ShimmerImmunity[type]` (`:17868`) have no counterpart — `spawn_shimmer_release` (world_sim/npcs.rs:1910) documents the gap. |
| `SetDefaults` derived debuff immunity (17818–17837) | PORTED | `npc_buffs.rs::debuff_immune_ids` (155) + `NpcBuffs::for_kind` | Passive 20 ⇒ +30/+375 and passive 69 ⇒ +36 are tested (`spawned_npcs_carry_their_debuff_immunity_table`, npc.rs:13181; `dw_3_5_immune_kind_rejects_buffs_and_derives_30_375`, npc_buffs.rs:957). |
| `ScaleStats` / `ScaleStats_ByDifficulty` / `_ForExpertHardmode` / `_ByPlayerCount` (18146–18205) | MISSING | — | No `difficulty`/expert/master flags exist in the sim. `NPCID.Sets.NeedsExpertScaling` (NPCID.cs:10447) is unimplemented; several machines document "the expert arm folds out" (npc.rs:6014, 6700, 6812, 7101, 7538, 11012). |
| `UpdateNPC` phase order (90274–90517) | PARTIAL | `npc.rs::update_rng` (4103) + `world_sim/npcs.rs::update_npcs` (1922) | Only three `UpdateNPC` phases survive as named steps: the buff block (`NPC.cs:90373-90377`) at npc.rs:4107, the `immune[256]--` at 4110, and the `find_frame`/contact tail. Missing: netOffset smoothing (90286, only in netglue 1643–1651), `UpdateAltTexture`, `CastLights`, `TeleportVisuals`, `CritterSounds`, `TrySyncingUniqueTownNPCData`, `UpdateGravity` (90464 — the generic gravity is re-implemented per archetype instead), the velocity.X 0.005 snap, `CheckDrowning` (94897), `CheckLifeRegen`, `GetHurtByOtherNPCs`, the bees, `UpdateCollision`, `SwitchTiles`, `CheckDialogue`, `UpdateTrails`, `UpdateNetworkCode`, `CheckActive`/`timeLeft` deactivation. |
| `AI()` `aiStyle` dispatch 0..127 (19819–41132) | PARTIAL | `npc.rs::update_rng` match (4114–4413) | **The central structural deviation.** C# dispatches on `aiStyle` (128 distinct values, 130 `aiStyle ==` tests in the `AI()` range); the Rust dispatches on `NpcKind` (182 variants, compiler-exhaustive). 54 of the 128 `aiStyle` values are named somewhere in the Rust tree (npc.rs + npc_ai.rs + world_sim/npcs.rs); **74 styles (10, 15, 17–25, 31, 37, 41–44, 49, 50, 52, 53, 55–73, 83, 84, 86–92, 98–104, 111–122, 124–127) are not referenced at all** — they belong to kinds the roster does not carry. |
| `StrikeNPC` damage (81189–81564) | PARTIAL | `tr-core/src/damage.rs::strike_npc_damage` (88) → `world_sim/editing.rs:1124` | The melee path is 1:1 (`max(1, CalculateDamageNPCsTake × (crit?2:1))`, with the Flymeal armor-penetration arm at 1102–1123). The projectile path is **not**: `projectile.rs::damage_pve` (9460) passes `p.damage` straight to `hurt_from_projectile` (9599) with no `checkArmorPenetration` and no defense reduction, while C# `Projectile.cs:13630-13649` adds `targetNPC.checkArmorPenetration(...)` then calls `targetNPC.StrikeNPC(dmg, ...)`. `netglue.rs:379` has the same gap. |
| `StrikeNPC` knockback (81389–81458) | PORTED | `damage.rs::knock_back_npc` (98) | The soft caps 8/10/12/14/16, crit ×1.4, the `damage*10 > lifeMax` (expert 15) gate and the Y pop. Called from `Npc::hurt` (4040). The `expert` argument is hard-coded `false` (4049). |
| `NewNPC` (80750) / `GetAvailableNPCSlot` (80797) | PARTIAL | `world_sim/npcs.rs` spawn blocks | `NewNPC`'s `spawnSlotProtected = 2`, `timeLeft = activeTime * 1.25`, the `wet` check, the `ai[0..3]`/`target` payload and `OnSpawn` have no single equivalent — every spawn site re-does a subset. `GetAvailableNPCSlot`'s slot protocol is **absent**: the Rust vec is a free list with a length cap (`app.npcs.len() < 200`, world_sim/npcs.rs:3196) and `Vec::push`/`retain`. `NPCID.Sets.SearchSpawnSlotsInReverse` (NPCID.cs:10451) and `CannotSpawnInSlot0` (10452) are **not ported** (no Rust symbol matches). |
| `SetDefaultsFromNetId` (7653–8058) | PARTIAL | `NpcKind` variant tables + `SlimeVariant::stats` | The negative-id cases −1..−38 (the slime colour family, blue slimes, the king's minion batch) are partially modelled by `SlimeVariant` (npc.rs:710) and covered by npc.rs:13197. The remaining net-id rows (the boss minions, the town-NPC variants, the ×scale post-maths) are not a generic facility. |
| `Transform` (81084) / `TransformVisuals` (80963) | PARTIAL | in-place kind swaps | No `transform` function exists. Each metamorphosis is a local `kind = Kind::X` assignment: the worm split (world_sim/npcs.rs:950 / 987), the headcrab→alien (test at world_sim/npcs.rs:8105), alien→hunter (8139), cell→big (8676), the drakomire rider (npc.rs:9329), the alien's `Transform(426)` (9872). The `Transform` life-ratio carry-over (`life = life * lifeMax / newLifeMax`, `NPC.cs:81129`), the ±height reposition (81110) and the town home switch (81136) are **not** ported. |
| `realLife` chains | PARTIAL | `worm_id`/`seg_index` + the pre-passes | The worm families are faithful (`update_worms_in`, tests `the_worm_chain_assembles_and_trails`, `the_destroyer_chain_assembles_same_kind`, `the_pillar_worms_round_trip_their_rows`, `the_crawltipede_assembles_its_thirty_segments`). The boss parts use a different mechanism — `carrier_x/y`/`carrier_timer`/`moon_parts`/`crew_cannons` plus kind-search pre-passes (world_sim/npcs.rs:1933–1994, 2229–2248, 2656–2686, 2709–2753) instead of C#'s `realLife` index links (51131–51392). |
| `GetAlpha` (93406) | PORTED | `render/entities.rs:2374` | The tint `(255 − alpha)/255` is ported; the `alpha` value itself is written by the per-kind machines. |
| `behindTiles` (6517, set at 8777–10673) | MISSING | — | No `behind_tiles` field anywhere in `crates/`. ROADMAP M4-NPC-2/3g admit it ("the segments draw in the NPC pass"). |
| `CheckDrowning` (94897) | MISSING | — | No Rust counterpart (no `breath`/`breathCounter` for NPCs). |
| Town/friendly life regen, `GetHurtByOtherNPCs` | MISSING | — | `UpdateNPC`'s friendly/town arm has no port. |
| `NPCID.Sets` tables | PARTIAL | `npc_buffs.rs::debuff_immune_ids`, `NpcKind::is_*` predicates | The immunity table is explicit and cited per kind. `NeedsExpertScaling`, `ProjectileNPC`, `DontDropDungeonKeysOrSouls`, `ZappingJellyfish`, `TakesDamageFromHostilesWithoutBeingFriendly`, `MPAllowedEnemies`, `NoMultiplayerSmoothing*`, `TrailCacheLength`, `DangerDetectRange`, `ExtraFramesCount`/`AttackFrameCount` (partially — `humanoid_frame_count`, 1848) are not ported as tables. |

## Tests

Static grep, no cargo run. `#[test]` counts: `npc.rs` 150, `world_sim/npcs.rs`
152 (31 inline `mod` blocks), `npcs.rs`'s helper `spawner.rs` 33,
`npc_buffs.rs` 24, `damage.rs` 8, `npc_ai.rs` 2. The core-relevant ones:

- **Defaults / stats:** `spawned_npcs_carry_their_debuff_immunity_table`
  (npc.rs:13181), `slime_variants_match_the_setdefaults_lines` (13197),
  `slime_hitboxes_scale_with_the_variant` (13229),
  `the_dye_trader_and_party_girl_kinds_stats_roundtrip_and_shops` (15011) and
  the 5 sibling round-trip tests (15117/15251/15326/15426) — these assert both
  `cs_id()` (the vanilla id) and the `to_u16`/`from_u16` protocol round-trip.
- **AI dispatch:** `npc_update_runs_the_buff_tick_before_the_ai` (13166) —
  the one test that pins the `UpdateNPC` phase order.
- **Archetypes:** `the_hover_kinds_spawn_fly_and_map`, `the_hover_chases_the_target`,
  `the_cave_walker_spawn_and_the_bestiary_map`, `the_cave_bat_spawn_flies_and_bites`,
  `the_caster_teleports_and_shoots`, `the_fire_imp_shoots_at_its_tick`, `the_sphere_launches_once_and_flies_straight`,
  `the_swimmer_bobs_idly_in_water`, `the_shark_chases_the_wet_player`,
  `the_beached_fish_flops` (npc.rs 12703–13648) plus the two in
  `npc_ai.rs`.
- **Chains:** `the_worm_chain_assembles_and_trails` (13675),
  `the_worm_chain_dies_with_the_head` (13719), `the_worm_head_digs_and_stumbles`
  (13739), `the_worms_spawn_and_map` (13781),
  `the_destroyer_chain_assembles_same_kind` (14508),
  `the_eow_chain_is_boss_wired` (14220),
  `the_pillar_worms_round_trip_their_rows` (world_sim/npcs.rs:7538),
  `the_crawltipede_assembles_its_thirty_segments` (7562),
  `the_eow_middle_split_lives_as_a_new_head` (4697),
  `the_moon_lord_family_round_trips_its_rows` (8219),
  `the_core_spawns_its_parts_on_the_60th_tick` (8244).
- **Strike/death:** `attacking_kills_a_slime` (13152), `dw_3_4_dot_damage_bypasses_hurt_cd_and_knockback`
  (npc_buffs.rs:909), `dw_3_4_drain_stops_at_zero_and_death_reaches_the_collect` (929),
  `calculate_damage_npcs_take` / `strike_npc_damage` / `armor_penetration_caps_at_half_defense`
  (damage.rs:248/259/266).
- **Boss flags:** `the_golem_death_chain` (world_sim/npcs.rs:5029),
  `the_cores_drama_ends_in_the_downed_flag` (8365),
  `the_tower_death_flags_the_shield_reset` (8550).
- **Spawn:** `the_rate_floor_and_cap_hold` (spawner.rs:1359),
  `nearby_cap_blocks_spawns` (1807), the seed-sweep tests at 1518/1544/1572/1680.

**Coverage gap:** no test covers the projectile-vs-NPC defense arithmetic
(there is nothing to test — it does not exist), no test covers
`GetAvailableNPCSlot`'s slot rules, and no test asserts `NpcKind::cs_id` for
the Skeleton / UndeadMiner / MeteorHead rows (see deviations).

## Deviations & risks

Everything below was verified against the sources on 2026-11-12; items in
rows ROADMAP marks DONE are included and flagged.

1. **`NpcKind::cs_id` has three wrong vanilla ids — `Skeleton => 77`,
   `UndeadMiner => 23`, `MeteorHead => 5`** (`npc.rs:1823/1824/1831`). The C#
   values are `Skeleton = 21` (NPCID.cs:10719), `UndeadMiner = 44` (10765),
   `MeteorHead = 23` (10723); 77 is `ArmoredSkeleton` and 5 is
   `ServantofCthulhu`. `cs_id` is the vanilla `NPC.type` — the same value the
   seven round-trip tests assert for the town NPCs — and its own doc comment
   (`npc.rs:1840-1847`) says "21 Skeleton / 44 UndeadMiner", contradicting the
   code. Severity: **behavioural** (the statue/wiring teleport rosters read
   `cs_id`, `world_sim/wiring.rs:54/112`) and the kinds are unverified by any
   test. Not admitted in ROADMAP.
2. **Projectile damage ignores NPC defense.** `projectile.rs::damage_pve`
   (9460) calls `hurt_from_projectile(dmg, …)` (9599) with the raw projectile
   damage; `strike_npc_damage` (tr-core) is never reached from that path and
   `checkArmorPenetration` is called only for the melee Flymeal arm
   (editing.rs:1115). C# `Projectile.cs:13630-13649` does both. Severity:
   **behavioural**, high — every ranged hit over-damages by roughly
   `defense/2`. Not admitted.
3. **Unreachable duplicate match arms in `NpcKind::to_u16` / `from_u16`,
   contradicting their own comment.** `npc.rs:881-886` claims "the duplicated
   martian/snowman arms above were the merge leftovers; the unreachable dead
   copies are gone", yet `to_u16` still carries 25 dead arms
   (`SolarCrawltipedeHead/Body/Tail`, `StardustWormHead`, `SolarSpearman`
   repeated, 953–977) and three more later arms that can never match
   (`VortexHunter => 427`, `VortexAlien => 427`-style vanilla ids 426/427/428
   at 987–989, `DD2LanePortal => 549` at 990); `from_u16` repeats the same 25
   (1207–1226). The earlier arms win, so the surviving values are the port-local
   ids (141/144/145, 127/128/129, 147) and behaviour is unaffected — but the
   code is unreachable and the invariant the comment asserts is false.
   Severity: **cosmetic** today, a trap for the next id merge.
4. **The goblin fighters' AI arm is shadowed.** `update_rng` has
   `GoblinPeon | GoblinThief | GoblinWarrior => update_goblin` at
   `npc.rs:4163-4165` *before* the intended `GoblinThief | GoblinWarrior |
   GoblinArcher | GoblinPeon => update_skeleton` at 4180-4183, so the second
   arm is dead. `update_goblin` (12155) walks at `speed: 1.0` while
   `update_skeleton` (6434) uses `0.8` — the goblin army walks at the zombie's
   speed instead of the skeleton's. `GoblinArcher` still reaches
   `update_archer` (4166). Severity: **behavioural**, low-moderate.
   (Two further duplicated arms — `MartianSaucer…` at 4218/4361 and
   `DD2Bartender` at 4119/4358 — are harmless because both arms call the same
   function.)
5. **`aiStyle` dispatch coverage is by-kind, not by-style.** `AI()`'s
   `aiStyle == N` chain (19819–41132) has 128 arms; the Rust `match self.kind`
   has no `aiStyle` concept and 74 of the 128 styles have no Rust reference.
   Any kind added later must be given a hand-written machine rather than
   inheriting its `aiStyle` default, which is where per-family drift enters.
   Severity: **structural / behavioural** for unported kinds. Documented
   implicitly by the per-kind `npc.rs` comments, not called out as a
   deviation.
6. **`GetAvailableNPCSlot`'s slot protocol is missing.** No
   `spawn_slot_protected`, no `SearchSpawnSlotsInReverse`, no
   `CannotSpawnInSlot0`, no `CanBeReplacedByOtherNPCs` second pass. The port
   uses `app.npcs` as a grow-only vec with `retain` for death, capped at 200
   entries (world_sim/npcs.rs:3196). Net sync therefore indexes by vec
   position (`net_npc_slots`, `app/netglue.rs:1705/1735`) rather than by the
   stable slot. Severity: **behavioural / determinism risk** on net sync
   ordering. Not admitted.
7. **`ScaleStats` (expert/master/hardmode/player-count scaling) is entirely
   missing.** No `difficulty` field on `Npc`; the machines hand-patch the
   missing arms ("the expert arm folds out", npc.rs:6014, 6700, 6812, 7101,
   7538, 11012). Combined with `NPCID.Sets.NeedsExpertScaling` (NPCID.cs:10447)
   being unimplemented, expert/master enemies have classic stats. Severity:
   **behavioural**, admitted in ROADMAP M5-SEED/M6-CREATIVE terms only.
8. **`Transform` is not a function.** The life-ratio carry-over
   (`NPC.cs:81129`), the ±height reposition (81110), the town-home switch
   (81136) and `TransformVisuals` (80963) are not ported; each metamorphosis is
   a bare `kind` assignment. A transform therefore loses the vanilla's hp
   scaling and the gore/dust. Severity: **behavioural**, partially admitted by
   the "in-place kind swap" comments.
9. **`behindTiles` is not modelled** (set in 130+ `SetDefaults` arms,
   `NPC.cs:8777-10673`). ROADMAP M4-NPC-2 and M4-NPC-3g admit the approximation
   for worms; it applies to every other kind too. Severity: **pixel**.
10. **`UpdateNPC` phases absent**, in cost order: `CheckActive`/
    `timeLeft` despawn (78000), `CheckDrowning` (94897), `CheckLifeRegen`,
    `GetHurtByOtherNPCs`, `UpdateCollision`/`SwitchTiles`, `UpdateNetworkCode`.
    The port compensates with ad-hoc `retain` rules (e.g. the Eye of Cthulhu's
    day despawn, world_sim/npcs.rs:2266-2270) and a revenge marker cache
    standing in for `CheckActive`'s `CacheEnemy` (2373-2399). Severity:
    **behavioural**.
11. **`NewNPC`'s spawn capsule is not a single code path.** `spawnSlotProtected`,
    `timeLeft`, the `wet` check, `ai[0..3]`, `target` and `OnSpawn` are
    re-derived (or dropped) at each of the ~20 spawn sites; the only shared
    shape is `Npc::new` + `hp = max_hp`. Severity: **behavioural / drift risk**.
12. **`Npc::hurt` uses `hurt_cd = 0.18`** as the melee cooldown stand-in
    (npc.rs:4037) instead of `StrikeNPC`'s 10-tick `NPC.immune` window, and
    `hurt_from_projectile` bypasses the cooldown entirely (4062) on the theory
    that the projectile immunity systems already gated it. The per-owner
    `immune[256]` array exists and decays (4110) but is read only by
    `projectile.rs` (9536). Severity: **behavioural**, minor.

## Task seeds

- **S — Fix `NpcKind::cs_id`.** Set `Skeleton => 21`, `UndeadMiner => 44`,
  `MeteorHead => 23` (`npc.rs:1823/1824/1831`, citing NPCID.cs:10719/10765/10723)
  and add the three rows to an existing `cs_id` round-trip test. Depends on
  nothing; blocks any wiring/statue feature.
- **S — Delete the dead `to_u16`/`from_u16` arms** (`npc.rs:953-990`,
  `1207-1226`) and correct the "the unreachable dead copies are gone" comment
  at 881-886. No behaviour change; unblocks the next id-space merge.
- **S — Remove the shadowed goblin arm** (`npc.rs:4163-4165`) so
  `GoblinThief/Warrior/Peon` reach `update_skeleton` (4180-4183) as intended,
  and pin the walk speed in a test. Also collapse the two harmless duplicate
  arms (4218/4361, 4119/4358).
- **M — Wire `strike_npc_damage` + `check_armor_penetration` into the
  projectile path.** `projectile.rs::damage_pve` should reduce `dmg` through
  `tr_core::damage` before `hurt_from_projectile`, mirroring
  `Projectile.cs:13630-13649` (armor penetration only when `!hostile`). Same
  for `netglue.rs:379`. Add a test that a def-20 NPC takes `dmg − 10`.
  Depends on the S-class cs_id work only for test hygiene.
- **M — Port the `GetAvailableNPCSlot` protocol** (`NPC.cs:80797-80837`):
  a 200-slot array with `spawnSlot_protected: [u8; 200]`,
  `SearchSpawnSlotsInReverse` (222, 245) and `CannotSpawnInSlot0` (NPCID.cs:10452)
  as data, plus the two-pass free/replaceable search. This is the prerequisite
  for stable net sync and for `CheckActive`/`timeLeft`.
- **M — Port `UpdateNPC`'s tail phases**: `CheckActive`+`timeLeft`
  (78000), `CheckDrowning` (94897), `CheckLifeRegen`, `GetHurtByOtherNPCs`,
  the velocity.X 0.005 snap, and `UpdateNetworkCode`. Depends on the slot
  protocol for `CheckActive`.
- **M — Port `Transform` as a function** (`NPC.cs:81084-81145`): the ±height
  reposition, `SetDefaults(newType)`, the life-ratio carry-over, the town-home
  switch list, the net ids 23/54 and `TransformVisuals` (80963); then convert
  the eight ad-hoc kind swaps (worm split, headcrab/alien/hunter, cell/big,
  drakomire rider) to call it.
- **L — Port `ScaleStats`** with a difficulty enum on `Npc` and
  `NPCID.Sets.NeedsExpertScaling`/`ProjectileNPC` tables (NPCID.cs:10447-10448),
  so the "expert arm folds out" comments can be retired.
- **L — Close the `aiStyle` gap by porting the missing archetypes** (the 74
  unnamed styles) as `npc_ai` steps rather than per-kind machines — starting
  with the styles that the ported kinds' neighbours already need.
- **L — `behindTiles` + the render layer** (`NPC.cs:6517` + the 130
  `SetDefaults` sites): add the flag to `Npc`, populate it from the kind table,
  and let `render/entities.rs` emit the behind-tiles pass.
