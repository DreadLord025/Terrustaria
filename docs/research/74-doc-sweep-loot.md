# 74 — DOC-SWEEP lane 3: NPC loot / death path (W4-f32)

Scope: the full C# `NPC.NPCLoot` call order (`NPC.cs:78830-78899`) plus the
`NPC.checkDead` death head/tail that wraps it (`:78040-78276`), censused
against the port's batched death pass in `world_sim/npcs.rs::update_npcs`.
The lane re-orders the port's death path to the C# sequence (stream-critical),
ports the real gaps it finds, and re-pins the RNG witnesses. Code-only round:
no compiler invoked, witnesses WRITTEN NOT RUN, verified by reading.

The luck-consumer sites themselves (the `RollLuck` gates in
`NPCLoot_DropCommonLifeAndMana` / `NPCLoot_DropHeals` /
`DoDeathEvents_SummonDungeonSpirit`) landed in the prior W4-f32 commits
(`f7e048e0`); this lane is the ORDER + the non-luck gaps around them.

## 1. The C# call order (census)

`NPC.checkDead` (`NPC.cs:78040`) runs inside `NPC.UpdateNPC` under the
`UpdateNPCs` swap (`Main.cs:17767-17769`). For a normal (non-carrier) death:

| # | C# call | C# lines | Draws `Main.rand`? | Port seam |
| --- | --- | --- | --- | --- |
| 1 | `DeathSound` play (case-4 pitch) | `:78186-78194` | one `Next(-10,11)` if audible | `play_npc_death_sound` |
| 2 | EoC `HitEffect` roar ×2 (type 4) | `:87275-87297` | none (int overload) | `play_npc_sound_request` |
| 3 | `CountKillForAchievements` + bestiary kill + `CountKillForBannersAndDropThem` | `:78839-78847` | banner milestone only | `register_kill` / `add_npc_kill_by` |
| 4 | statue rarity gate (early return) | `:78849-78852` | `NextFloat` if statue+rarity | **unported** (see §4) |
| 5 | `DoDeathEvents_BeforeLoot` (Twins demotion) | `:78854` → `:78916-78923` | none | **unported** (see §4) |
| 6 | `NPCLoot_DropItems` (rule engine) | `:78855` → `:78925-78937` | per-rule rolls + spawn triplet | `spawn_loot` |
| 7 | `DoDeathEvents` | `:78856` → `:79197-79514` | see the sub-calls below | `advance_slime_rain` → `summon_dungeon_spirit` → `do_death_events_switch` → boss tail |
| 7a | `DoDeathEvents_AdvanceSlimeRain` | `:79199` → `:79080-79096` | none (the King spawn is `SpawnOnPlayer`) | `advance_slime_rain` |
| 7b | `DoDeathEvents_SummonDungeonSpirit` | `:79200` → `:79064-79078` | `RollLuck(13/9)` when gates pass | `summon_dungeon_spirit` |
| 7c | remix King-Slime roll (`Next(200)`) | `:79201-79204` | one `Next(200)` when gates pass | **unported** (remix, see §4) |
| 7d | type switch (boss flags / DD2 / towers / …) | `:79205-79504` | per-case (King stop, orb meteor, cultist apocalypse) | `do_death_events_switch` |
| 7e | boss tail: `DropBossPotionsAndHearts` + `CelebrateBossDeath` | `:79505-79513` → `:78945-79034` | `Next(5,16)` + `Next(5)` + per-item spawns | `boss_potions_and_hearts` + the broadcast |
| 8 | Mechdusa progression (remix) | `:78857-78869` | none | **unported** (remix) |
| 9 | all-mechs-downed broadcast | `:78870-78873` | none | **unported** (see §4) |
| 10 | skyblock altar/dungeon/hellforge/lihzahrd/trap drops | `:78874-78896` | per-drop | **unported** (skyblock) |
| 11 | `NPCLoot_DropMoney` | `:78897` → `:79619-79750` | luck double-roll + bonus chains + split | `spawn_coins` |
| 12 | `NPCLoot_DropHeals` | `:78898` → `:79098-79195` | common life/mana + per-type hearts | `spawn_heals` |
| 13 | `checkDead` tail: `DD2Event.CheckProgress` → frost/pumpkin moon → invasion group | `:78230-78275` | DD2 medal drop only | `report_invasion_kill` + the moon walks |

The port consolidates this into ONE per-NPC loop in `update_npcs` (the dead
clones are collected, the pool retained, then each death runs steps 1-13 in
order). The draw-free reports (step 13's moon walks + invasion group) are
byte-identical to the C#'s per-NPC interleave because they consume nothing.

### The pre-sweep port order (what was wrong)

Before this lane the death pass ran, per NPC: invasion report + DD2 crystal
drop FIRST, then a separate tower-bolt loop, then a boss-flag loop, then a
slime-rain loop, then bestiary/banner, then `spawn_coins` → `spawn_loot` →
`spawn_heals_and_death_events` (the W4-f32 luck tail appended last). That put
the coins BEFORE the loot and the heals/spirit AFTER everything — the C# runs
DropItems → DoDeathEvents → DropMoney → DropHeals. The re-order moves the
bestiary/banner to the head, the loot before the coins, and splits the luck
tail into its C# slots (`summon_dungeon_spirit` inside DoDeathEvents,
`spawn_heals` last).

## 2. Landed gaps (this lane)

### 2.1 Heart/star scatter — the real `Item.NewItem` velocity

`spawn_heals` now draws the C#'s default-velocity pair verbatim
(`Item.cs:49749-49750`): `Next(-30,31)*0.1` then `Next(-40,-15)*0.1`, TWO
integer draws, no phantom third float. Hearts/stars are NOT in
`ItemID.Sets.ItemNoGravity` (`ItemID.cs:1320`), so the gravity arm applies.
`Item.Prefix(-1)` runs first (`Item.cs:49730`) but draws nothing for hearts
(the `CanHavePrefixes` gate, `Item.cs:515-518`); the `phase` field is a port
render stand-in the C# does not draw — pinned 0. The old 3-float triplet
stand-in (the `spawn_loot` model) is RETIRED for the heals tail.

Witness: `f32_heart_scatter_uses_the_csharp_integer_draws` — the Horseman
(315) drops a guaranteed heart (`NPC.cs:79126-79128`); the player spawns at
full life/mana so the common life/mana roll drops nothing, leaving exactly
one heart on the integer lattice (`vx*10`, `vy*10` near-integers, phase 0,
prefix 0).

### 2.2 Soul of light/night — the full shared gate

`soul_of_whatever_can_drop` (`loot.rs`) was a depth-only stub
(`info.flags.soul_depth_ok`). It now ports
`Conditions.SoulOfWhateverConditionCanDrop` (`Conditions.cs:1579-1612`)
verbatim: not a boss (`info.npc.boss`), not in `DontDropDungeonKeysOrSouls`
(`NPCID.cs:10115` — type 23 alone), not the 1/13/14/15/121/535 exclusion list,
hardmode, `lifeMax > 1`, not friendly, `value >= 1`, and (outside the remix
seed) the NPC below the rock layer (`position.Y > rockLayer * 16`). The remix
world drops the depth term (`:1599-1605`).

`spawn_loot` feeds the two new `LootFlags`/`LootNpc` fields:
`zone_hallow` (the light soul's zone, `Conditions.cs:856-876`) and
`soul_depth_ok` (`npc.y > rock_layer * 16`), plus `dont_drop_keys`
(`net_id == 23`). The corrupt/crimson zones already fed the night soul
(`Conditions.cs:830-854`).

Witness: `soul_of_whatever_gate_matches_the_csharp` — a worth-dropping deep
hardmode hostile passes; each refusal term (boss / keys / exclusion list /
pre-hardmode / lifeMax 1 / friendly / value 0 / shallow) and the remix
depth-drop and the two zone gates are pinned.

### 2.3 The faithful boss roster (`Npc::is_boss_kind`)

The loot soul gate and the `LegacyHack_IsABoss` trophy rows read
`info.npc.boss`; the death tail gates on it. The old feed was a KingSlime-only
stand-in (`npc.kind == NpcKind::KingSlime`), which left every other boss's
trophy condition dead and let bosses pass the soul gate. The new
`Npc::is_boss_kind()` is the per-kind read of the C#'s `SetDefaults`
`boss = true` rows.

Census of the 23 `this.boss = true` sites: 21 are static `SetDefaults` kinds
— 4 (`:8716`), 35 (`:9180`), 50 (`:9389`), 113 (`:10358`), 125 (`:10535`),
126 (`:10552`), 127 (`:10570`), 134 (`:10676`), 222 (`:11868`), 245 (`:12176`),
262 (`:12393`), 266 (`:12452`), 370 (`:13716`), **395 (`:14061` — the Martian
Saucer Core IS a boss)**, 396 (`:14097`), 397 (`:14116`), 398 (`:14136`),
439 (`:14227`), 636 (`:17202`), 657 (`:17323`), 664 (`:17401`, the Torch God —
unported), 668 (`:17460`). The 22nd is the DYNAMIC `DropEoWLoot` latch
(`:78425-78428`): the last EoW segment sets `boss = true` before `NPCLoot`, so
the EoW head is NOT a static `SetDefaults` boss. The 23rd site is that same
latch counted once. The DD2 bosses (551/564/565/576/577) and the Dutchman
(491) do NOT set `boss` (their trophies are unconditional `Common` rows,
`ItemDropDatabase.cs:903-908`).

The port's roster is the 19 ported static kinds (the 21 minus the unported
Empress 636 / Queen Slime 657 / Torch God 664, plus the three Moon Lord parts
396/397/398). Two corrections vs the lane's initial 17-kind guess: the
Martian Saucer Core (395) and the Moon parts (396/397) were added; the EoW
head was removed (it is a dynamic latch, not a kind read — see §4).

The Moon parts are bosses in `SetDefaults` but their ported deaths are ALL
bypass kills (`moon_despawn_kill`): the strike death restores the death hover
(`checkDead` `:78060-78072`), and the orphan/despawn kills set `active = false`
WITHOUT `checkDead` (`:41579-41583`/`:41490-41514`), so `NPCLoot` never runs on
them. The death loop's boss tail refuses `moon_despawn_kill` corpses, so no
boss-tail draw ever fires for a part (a Moon Lord despawn is not a defeat).

### 2.4 Boss potions + hearts + the Badger's Hat

`boss_potions_and_hearts` ports `DoDeathEvents_DropBossPotionsAndHearts`
(`NPC.cs:78945-79016`): one potion stack `Next(5,16)` of the per-boss id
ladder (113/35/668→188, 222→1134, 657/636/245/262/370/395/439→499,
398→3544, 113<t<222→499, else 28), then `Next(5)+5` hearts (each its own
`Item.NewItem`), then the Badger's Hat 5004 when BOTH the EoC and the WoF died
today. The daily latches `EoCKilledToday`/`WoFKilledToday` (`NPC.cs:6611`/
`:6613`) are new `World` fields (`eoc_killed_today`/`wof_killed_today`,
runtime-only, not saved); they reset at dusk via `NPC.ResetBadgerHatTime`
(`Main.cs:65221` → `NPC.cs:78939-78943`) in `clock.rs::advance_time`'s dusk
branch. The hat drop resets both latches (`:79013`).

Witnesses: `f32_boss_potions_hearts_and_badger_hat` (EoC potion 28 + 5..9
hearts + latch; WoF potion 188 + the hat + latch reset; a second EoC drops no
hat) and `f32_badger_hat_latches_reset_at_dusk` (the STRICT `> 54000.0` dusk
gate — resting AT 54000 and adding one tick crosses; 53999+1 lands exactly on
the boundary and stays day).

### 2.5 Coin split — verified verbatim; midas + blood-moon wired

`tr_core::loot::drop_money` (`loot.rs:549`) was re-read against
`NPCLoot_DropMoney` (`NPC.cs:79619-79750`) and is VERBATIM: the luck
double-roll (`NextFloat() < |luck|`), the midas `Next(30,51)*0.01`, the
±20-75 % spread, the six bonus chains (1/2, 1/4, 1/8, 1/16, 1/32, 1/64), the
blood-moon `Next(101)*0.01`, the best/worst-luck keep, the `+= extraValue`,
and the platinum/gold/silver/copper split with the >50-count shrink rolls.

The two flags the caller hardcoded `false` are now wired: `midas` reads
`npc.buffs.has_buff(72)` (the C#'s `this.midas` is set by `UpdateNPC_BuffApply`
while buff 72 is live, `NPC.cs:92017-92020`, cleared at `:92165`; the port
reads the buff directly — the Flask of Gold / Golden Bullet on-hit arms land
it, `melee_on_hit.rs:127`), and `blood_moon` reads `app.world.blood_moon`
(`NPC.cs:79660`).

### 2.6 Soul of light/night physics — `ItemNoGravity`

Souls 520/521 are in `ItemID.Sets.ItemNoGravity` (`ItemID.cs:1320`).
`ItemDrop::move_in_world` (`item.rs`) now branches on the set: BOTH axes decay
×0.95 with the ±0.1 rest snap, NO gravity and NO fall cap
(`WorldItem.cs:774-786`), gated `!shimmered`. The port models the two soul ids
(this lane); the other set members (the nebula pickups 3453-3459, the
fragments 547/548/549/575, 3580/3581, 4143) land with their items.

## 3. RNG witnesses (re-pinned, WRITTEN NOT RUN)

- `test_dw_3_2_death_loot_rides_the_update_npcs_stream` — the twin replays the
  C# NPCLoot order: death pitch → `spawn_loot` → `advance_slime_rain` →
  `summon_dungeon_spirit` → `do_death_events_switch` → `spawn_coins` →
  `spawn_heals` (the death events are draw-free for a slime). Byte-for-byte
  stream parity.
- `the_king_death_stops_the_slime_rain_on_the_update_npcs_stream`
  (`weather.rs`) — re-pinned: the stop now rides INSIDE `DoDeathEvents`
  (`NPC.cs:79405-79410`), AFTER the King's `NPCLoot_DropItems` draws and BEFORE
  the money/heals/boss-potion draws. The twin replays the death pitch + the
  loot half through the same `spawn_loot` seam, then reads the stop's
  `Next(3024,6048)`. The King's banner (item 1638) needs 50 kills, so the first
  kill draws no banner spawn — the twin omits it correctly.
- `f32_death_tail_rides_the_update_npcs_stream` — the heals tail draws after
  the loot; luck 1.0 diverges from luck 0.
- The three new witnesses in §2 (heart scatter, boss potions/hat, dusk reset)
  and the soul-gate witness.

## 4. Remaining deviations / unported arms (documented, NOT ported)

- **The triplet stand-in elsewhere.** `spawn_loot`, `spawn_coins`, the banner
  drop and the boss potions/hearts still use the port's 3-float
  velocity/phase triplet + prefix stand-in (`spawn_item_drop_px`), not the
  C#'s per-item `Item.NewItem` velocity. Only the heals tail (§2.1) was
  replaced verbatim this lane. The `phase` field is a port render stand-in
  the C# does not draw.
- **`DoDeathEvents_BeforeLoot` (Twins demotion).** `NPC.cs:78916-78923`: a
  Twin whose other half is still active sets `value = 0` and `boss = false`
  before the loot — the first eye drops no money / no boss tail, the second
  runs the full boss path. The port's death clone is read-only and the
  demotion needs a per-NPC value/boss override threaded through `spawn_loot` /
  `spawn_coins` / the tail; deferred (the static Twins boss flag stands).
- **The EoW last-segment boss latch.** `DropEoWLoot` (`NPC.cs:78414-78430`)
  sets `boss = true` on the last surviving EoW segment before `NPCLoot`. The
  port's `is_boss_kind` is a static kind read, so the EoW head is not a boss
  (its trophy 1361 and boss tail are gated off). A dynamic "last segment"
  read is deferred.
- **The statue rarity gate.** `NPC.cs:78849-78852`: a statue-spawned NPC in
  `StatueSpawnedDropRarity` rolls `NextFloat() >= rarity` (or no interactions)
  and returns early. The port's `spawned_from_statue` field exists but the
  rarity gate is not wired.
- **Unported `DoDeathEvents` switch arms** (the kinds are unported or the arm
  is a documented skip): 216→`SpawnBoss(662)` (the Pirate Captain's parrot),
  the moon-boss downed flags (327/325/344/345/346), 614→the 281 projectile
  (the bound goblin's release), 109→`downedClown`, 50→`ViolentlySpawnNerdySlime`
  (670, `NPC.cs:79411-79415`), 661→`SpawnBoss(636)` (the Empress's enrage
  summon), 22→`SpawnWOF` on lava (the Voodoo Demon), the remix `Next(200)`
  King roll (`:79201`), and `DD2Event.AnnounceGoblinDeath` (`:79245`, the
  goblin death announcement — deferred).
- **The all-mechs-downed broadcast** (`NPC.cs:78870-78873`) and the Mechdusa
  progression (`:78857-78869`, remix) and the skyblock drops
  (`:78874-78896`) are unported.
- **The death-sound stream choice.** The C# runs `checkDead` inside
  `StrikeNPC` (the striking stage's stream); the port batches deaths into the
  NPC pass, so the pitch draw rides `UpdateNPCs` — the recorded S1-E-21c
  deviation (unchanged by this lane).

## 5. Expert money ×2.5 — ownership (e25, NOT this lane)

The expert/master money multiplier is `GameDifficultyData.EnemyMoneyDropMultiplier`
(`DataStructures/GameDifficultyData.cs:87`: Classic 1.0, Expert 2.5, Master
2.5, Legendary 3.5), applied at SPAWN in `NPC.ScaleStats_ByDifficulty`
(`NPC.cs:18176`: `this.value = (int)(this.value * …Sample(this.difficulty))`),
NOT in `NPCLoot_DropMoney`. It is owned by branch
`task/W4-e25-expert-master-scaling` (its `npc_scaling.rs` landed UNWIRED; its
census is `docs/research/64-expert-master-scaling-census.md` on that branch).
This lane documents the ownership and does NOT port it — the port's
single-difficulty classic mode keeps the 1.0 multiplier until e25 wires the
difficulty system.

## 6. Verification

Code-only round: no `cargo`/test/graphify invoked. Every witness was verified
by READING against the C# source (the draw order, the gate order, the integer
ranges, the stream choice). The borrow shapes (`app.rngs.stream` + the
disjoint `app.items`/`app.world`/`app.npcs` fields) match the proven
`dd2_crystal_drop` / angler-reward pattern. The dusk witness's STRICT-gate fix
(54000.0, not 53999.0) and the heart witness's full-life/mana player were
confirmed by reading `clock.rs::advance_time` and `Player::spawn`.