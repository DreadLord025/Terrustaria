# 64 — Expert/Master mode scaling census (W4-e25)

Scope: `NPC.ScaleStats` and its whole family (`NPC.cs:18146-18739`), the
difficulty curves (`DataStructures/GameDifficultyData.cs`), the
player-count scaling, the drop-rate expert/master bonuses, and the boss
HP scaling. Census of port vs C#, gaps filled verbatim in
`crates/tr-app/src/npc_scaling.rs` (this wave, code-only mode — no
compiler invoked, witnesses WRITTEN NOT RUN).

## 1. The C# family (source of truth)

| C# | Lines | Shape |
| --- | --- | --- |
| `NPC.ScaleStats(int?, float?)` | `NPC.cs:18146-18169` | the entry: the `NeedsExpertScaling[type] \|\| (lifeMax > 5 && damage != 0 && !friendly && !townNPC)` gate; `difficulty = strengthOverride ?? Main.Difficulty`; expert+hardmode → `ScaleStats_ForExpertHardmode`; then `ScaleStats_ByDifficulty`; expert → `ScaleStats_ByPlayerCount(activePlayersCount ?? GetActivePlayerCount())`; the non-projectile `lifeMax < 6 → 6` floor; `life/defDamage/defDefense/defLifeMax` snap |
| `NPC.ScaleStats_ByDifficulty` | `:18171-18181` | non-projectile: `lifeMax = (int)(lifeMax * EnemyMaxLifeMultiplier.Sample(difficulty))`, `value = (int)(value * EnemyMoneyDropMultiplier.Sample)`; ALL: `damage = GetAttackDamage_ScaledByDifficulty(damage)` (`:7036-7039`, `(int)` truncation), `knockBackResist = GetKnockbackMultiplier_ScaledByDifficulty(kb)` (`:7031-7034`); then `_Tweaks` |
| `NPC.ScaleStats_ByDifficulty_Tweaks` | `:18183-18475` | the per-type boss table (below); `num4 = 1f * Remap(d, Classic, Expert, 1, num2) * Remap(d, Expert, Master, 1, num3)` (f32), `lifeMax = (int)Math.Round((float)lifeMax * num4)` — **banker's**; `num5 = 1f * Remap(d, Classic, Expert, 1, num)`, `damage = (int)Math.Round((float)damage * num5)` — banker's |
| `NPC.ScaleStats_ForExpertHardmode` | `:18510-18558` | skips ProjectileNPC / `DontDoHardmodeScaling` / boss / `lifeMax >= 1000` / the getGood twin exceptions; `num = damage + defense + lifeMax/4` (0 → 1); `num2 = 80 (+20 post-Plantera)`; `num < num2` → `num3 = num2 / num` (**INT division** — the 41→1 quirk); `damage = (int)(damage * num3 * 0.9f)`; non-projectile: `defense = (int)(defense * num3)`, `lifeMax = (int)((double)(lifeMax * num3) * 1.1)`, `value = (int)((double)(value * num3) * 0.8)` |
| `NPC.ScaleStats_ByPlayerCount` | `:18560-18720` | `statsAreScaledForThisManyPlayers = numPlayers`; the per-type balance table (below); `lifeMax = (int)Math.Round((double)lifeMax * num)` — **f64 banker's**; the 266/267 kb loop and the 115/116 kb arms |
| `NPC.GetStatScalingFactors` | `:18722-18739` | `balance = 1f, boost = 0.35f`; per extra player `balance += boost; boost += (1f - boost) / 3f` (all f32); `balance > 8 → (balance*2+8)/3`; cap 1000 |
| `NPC.GetBalance` / `GetMyBalance` | `:18477-18491` | the live balance / the spawn-time balance (`statsAreScaledForThisManyPlayers <= 1 → 1f`) |
| `NPC.GetActivePlayerCount` | `:18493-18508` | `netMode == 0 → 1`; else the 255-slot active-&&!ghost scan |
| `NPC.GetNPCInvasionGroup` | `:78297-78412` | goblin 1, frost 2, pirate 3, martian 4, frost-moon −1, pumpkin-moon −2, OOA −3 |
| `NPC.CommonMasterBossLifeReduction` | `:6587`, `:95269` | `0.85` (double) |
| the call sites | `:8342` (`SetDefaults_ForNetId`), `:17856` (`SetDefaults` tail) | both AFTER the getGood/anniversary adjustments; `life = lifeMax` follows |

### The curves (`GameDifficultyData.cs:81-92`, `GameDifficultyLevel.cs:17-21`)

Levels (f32): Journey 0.5, Classic 1, Expert 2, Master 3, Legendary 4.
`LinearCurve.Sample` (`:35-55`) — the bracket scan, zero-span →
`key.output`, else `(value - key.input) * num2 / num + key.output`.

| Curve | Keys | Expert (2) | Master (3) |
| --- | --- | --- | --- |
| `EnemyMaxLifeMultiplier` (`:83`) | (0.5, 0.5), (4, 4) | 2.0 | 3.0 |
| `EnemyDamageMultiplier` (`:84`) | (0.5, 0.5), (3, 3), (4, 5.3333335) | 2.0 | 3.0 |
| `KnockbackToEnemiesMultiplier` (`:86`) | (1, 1), (3, 0.8) | **0.8999999761581421** (f32) | 0.800000011920929 |
| `EnemyMoneyDropMultiplier` (`:87`) | (1, 1), (2, 2.5), (3, 2.5), (4, 3.5) | 2.5 | 2.5 |

The expert kb 0.9 is NOT exact in f32 (`1 + (0.8−1)*0.5` =
0.8999999761581421) — the port keeps the curve maths, not a rounded 0.9.

### The Tweaks table (`:18218-18470`), `num` = damage mult, `num2` = expert life mult, `num3` = master life mult (×0.85 = `CommonMasterBossLifeReduction`)

| types | num | num2 | num3 | extras |
| --- | --- | --- | --- | --- |
| 210/211 (bees) | 0.6 | 0.6000000000000001 | 1 | `defense = (int)(defense * Remap(d, 1, 2, 1, 0.8f))` (`:18222`) |
| 5 (Servant of Cthulhu, `NPCID.cs:10687`) | 1 | 0.75 | ×0.85 | — |
| 4 (EoC) | 1 | 0.65 | ×0.85 | — |
| 13-15 (EoW) | 1.1 / 0.8 / 0.8 | 0.7 | ×0.85 | expert: `scale *= 1.2f`, `defense += 2` |
| 266/267 (Brain/Creeper) | 0.9 | 0.85 | ×0.85 | expert: `scale *= 1.05f` |
| 50 (King Slime) | 0.8 | 0.7 | ×0.85 | — |
| 471 | 1 | 0.85 | 1 | — |
| 472 | 0.8 | 0.85 | 1 | — |
| 492, 393/394/395 | 1 | 0.85 | 1 | — |
| 222 (Queen Bee) | 0.9 | 0.7 | ×0.85 | — |
| 35 (Skeletron) | 1.1 | 1 | ×0.85 | — |
| 36 (Skeletron hand) | 1.1 | 1.3 | ×0.85 | — |
| 668 (Deerclops) | 1 | 0.85 | ×0.85 | — |
| 113/114 (WoF/Eye) | 1.5 | 0.7 | ×0.85 | expert: `defense += 6` |
| 115 (Hungry) | 1 | 0.7 | 1 | expert: `defense += 6` |
| 116 (Leech) | 1 | 0.7 | 1 | — |
| 117/118/119 | 1 | 0.8 | 1 | — |
| 657 | 1 | 0.8 | ×0.85 | — |
| 658-660 | 1 | 0.75 | ×0.85 | — |
| 134-136 (Destroyer) | 2.0 / 0.85 / 0.85 | 0.75 | ×0.85 | expert: `scale *= 1.05f` |
| 139 (Probe) | 0.8 | 0.75 | 1 | expert: `scale *= 1.05f` |
| 127-131, 125-126 (Twins) | 0.85 | 0.75 | ×0.85 | — |
| 262 (Plantera) | 1.15 | 0.7 | ×0.85 | — |
| 264 (Tentacle) | 1.15 | 1 | ×0.85 | — |
| 636 (Empress) | 1 | 0.7 | ×0.85 | — |
| 245-249 (Golem) | 0.8 | 0.75 | ×0.85 | — |
| 370 (Duke Fishron) | 0.7 | 0.65 | ×0.85 | — |
| 371/372/373 | 0.75 | 1 (371) / 0.75 | 1 | — |
| 439/440/454-459/522/523 | 0.75 | 0.75 (not 522) | ×0.85 (not 522) | — |
| 396/397/398 (Moon Lord) | 0.75 | 0.75 | ×0.85 | — |
| 551 (Betsy) | 0.65 | 0.75 | ×0.85 | — |
| OOA set (`NPCID.cs:10432`) | 1 | 1 | ×0.85 | — |
| invasion −2/−1: 315/325/327/344/345/346 | 0.75 | 0.65 | ×0.85 | — |
| invasion −2/−1: default | 0.75 | 0.75 | 1 | — |

getGood twin arms (`:18188-18216`):
6/32/24 with the partner alive → master ×0.85, expert life ×1.5, expert
defense +2/+6/+10.

### The ByPlayerCount table (`:18560-18718`)

`num *= balance` for: 4, 13-15, 266/267, 50, 222, 35, 36, 668, 113/114,
115, 116, 657, 658-660, 134-136, 127-131, 125-126, 262, 264, 636, 245-249,
370, 439/440/454-459/523 (**522 excluded here**, unlike Tweaks), 396/397/398,
551. Lerps (`Utils.Lerp` double, `:107-110`): 471 → `Lerp(1, balance, 2/3)`,
472 → `Lerp(1, balance, 0.5)`, 139 → `Lerp(1, balance, 2/3)`, OOA set →
`Lerp(1, balance, 0.8571428656578064)` (the f32 6/7 widened). Invasion
−2/−1 default → `num *= 1.0 + (numPlayers - 1) * 0.2` (the six boss-moon
ids 315/325/327/344/345/346 skip). kb side effects: 266/267 the
`for (num2 = 1f; num2 < balance; num2 += 0.34f)` ×0.8 loop, `< 0.1 → 0`
break (`:18576-18584`); 115/116 `numPlayers > 4 → kb = 0`, `> 1 →
kb *= 1f - boost` (`:18621-18640`).

## 2. The drop-rate expert/master bonuses (census)

| C# site | Lines | Port status |
| --- | --- | --- |
| `NPCLoot_DropItems` → `DropAttemptInfo.IsExpertMode/IsMasterMode = Main.expertMode/masterMode` | `NPC.cs:78925-78937` | the engine is READY (`tr_core::loot::DropAttemptInfo.is_expert/is_master`, `DropBasedOnExpertMode/MasterMode`, `normalvs_expert*`, `expert_gets_rerolls`, `master_mode_*` factories, the moon gates' expert +5/−1/−2 arms `Conditions.cs:55-122` → `loot.rs::moon_wave_gate`, the trophy `Next(3)` `Conditions.cs:161/213` → `moon_trophy_gate`); the LIVE call sites pass `is_expert: false` (`world_sim/npcs.rs:6805/8043`, `npc.rs:22312`, `loot_db.rs:111/308`) — constants-off until the difficulty flip |
| `HalloweenWeapons` value/damage caps × difficulty | `Conditions.cs:809-810` | `loot.rs:149-151` + `Cond::HalloweenWeapons` (`:1762-1765`) — the caps ride `LootFlags`, caller-supplied |
| the money drop ×2.5 | `ScaleStats_ByDifficulty` `:18176` (the `value` scale), `NPCLoot_DropMoney` `:79619-79698` (no expert term of its own) | `Npc::value()` reads the SetDefaults copper — UNSCALED (this wave's module ports the scale; wiring deferred) |
| boss hearts 116-119: `!expertMode \|\| Next(5) == 0` | `NPC.cs:79183` | the heart-drop block is unported (no `DoDeathEvents_DropBossPotionsAndHearts` port) — RNG census: the expert arm adds ONE `Next(5)` draw per heart-eligible kill; recorded for the loot wave |
| `GetMoonEventPointScalar` master 2.5 / expert 2 | `NPC.cs:78432-78443` | `tr_core::world::MOON_POINT_SCALAR = 1.0` constants-off (`world.rs:882-885`) |
| `MasterModeDropOnAllPlayers` (the 1/4 relics/pets) | `ItemDropDatabase.cs:15/317/333/359` | `loot.rs::master_mode_drop_on_all_players` (`:2331-2343`) — engine ready, `is_master: false` at the call sites |

**There is no generic "expert drop-rate multiplier" in the C#** — the
bonus is entirely the per-rule `DropBasedOnExpertMode` trees + the moon
wave gates + the money `value` scale. The census pins that shape; no
invented flat multiplier.

## 3. The i11 kb deviation (the task's seed)

ANALYSIS "The player→NPC strike mechanics (S1-P-i11)" (the deviations
tail, ~line 15631): *"The expert/master knockback-resist scaling is
unmodelled: the C# multiplies `knockBackResist` by
`GameDifficultyData.KnockbackToEnemiesMultiplier` at spawn
(`NPC.cs:18179` → `:7031-7034`), and the multiplayer
`ScaleStats_ByPlayerCount` (`:18560`) re-scales it for the Brain/Creeper
pair (`:18573-18584`, ×0.8 per 0.34 balance step, floored to 0 below
0.1) — the port's `knock_back_resist` reads the normal-mode SetDefaults
value (the `expert` flag threads only the `num4` gate's ×15)."*

This wave fills the MATHS verbatim (`npc_scaling.rs`): the curve sample,
the spawn kb scale, the 266/267 loop and the 115/116 arms, with the exact
f32 0.8999999761581421 expert factor pinned. The strike-time `expert`
flag (`damage.rs::knock_back_npc`'s ×15 gate) is UNCHANGED — it is a
different mechanism (the strong/weak-hit gate, `NPC.cs:81418-81422`),
already landed by i11.

## 4. Port census (what exists today)

- **No `ScaleStats` port anywhere.** `npc.rs`'s only mentions are
  comments (`:3780` the 210/211 ×0.8 remap note, `npc_ai_style27.rs:18`,
  `npc_ai_style70.rs:117`, `npc_ai_style71.rs:103`). The live NPC stats
  are the raw SetDefaults values (`npc/data.rs::entry_stats`,
  `Npc::knock_back_resist` `npc.rs:3168`, `Npc::value` `:3919`,
  `damage_override`/`contact_damage`).
- **No difficulty state**: `Main.Difficulty`/`expertMode`/`masterMode`
  have no world carrier ("the World carries no game mode" — the
  recurring constants-off note: nurse `Main.cs:40086`, spawner
  `NPC.cs:853`, dd2 `DD2Event.cs:1123`, death coins, buff times
  `buffs.rs:220-224`). The world-creation draft carries a difficulty
  byte (`app/flow.rs:632/820`) but it is the CHARACTER difficulty
  (softcore/mediumcore/hardcore/journey), not the world game mode.
- **Partial curve ports exist**: `npc_ai_style74.rs:283-318`
  (`linear_curve_sample` + the damage/knockback curves, private),
  `npc_ai_style39.rs` (the same family, W-10), `buffs.rs:220-224` (the
  debuff-time 2× expert step). This wave's module carries the FULL
  curve set incl. `EnemyMaxLifeMultiplier`/`EnemyMoneyDropMultiplier`
  (previously unported).
- **`round_bankers`** (`tr_core::damage.rs:13`) — the shared
  `Math.Round` seam, reused.
- **`GetNPCInvasionGroup`**: no standalone port (the dd2/moon lanes key
  on kind tables); ported here verbatim.
- **Witnesses**: none for scaling (the i11 witnesses pin the strike
  knockback, not the spawn scale). This wave adds the scaling pins.

## 5. What this wave lands

`crates/tr-app/src/npc_scaling.rs` — the whole family as pure fns over
a `ScalingStats` seam (the `npc_ai_style53.rs` unwired-module
precedent): `scale_stats`, `scale_stats_by_difficulty`,
`scale_stats_by_difficulty_tweaks`, `scale_stats_for_expert_hardmode`,
`scale_stats_by_player_count`, `get_stat_scaling_factors`,
`get_active_player_count`, `get_balance`/`get_my_balance`,
`get_npc_invasion_group`, the four curves + `linear_curve_sample`,
`Utils.Remap`/`GetLerpValue`/`Lerp` (f32 + f64), the three ID sets
(`NeedsExpertScaling` `NPCID.cs:10447`, `DontDoHardmodeScaling` `:10116`,
`ProjectileNPC` `:10448`, `BelongsToInvasionOldOnesArmy` `:10432`).
UNWIRED: the live spawns keep the classic SetDefaults values
byte-identical (the world carries no game mode — the wiring rides the
difficulty-flip task, M6-CREATIVE-1 / R-03-6). RNG census: the WHOLE
family draws ZERO `Main.rand` — pure arithmetic; the witnesses pin
that by construction (no rng parameter).

Deviations/skips (recorded in ANALYSIS): the getGood twin arms ported
behind the `get_good_world` flag (constants-off today); the
`statsAreScaledForThisManyPlayers` carrier is a seam field (the live
`Npc` struct does not grow it until wiring); the 255-slot
`GetActivePlayerCount` scan takes the active/ghost flags as a slice
(the port is single-player → 1).
